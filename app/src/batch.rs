use crate::{
    analyzer::analyze_export_in_current_pool,
    models::{Candidate, DemoJob},
    preflight::{disk_space_for, format_bytes, format_duration, require_disk_space},
    scheduler::{largest_first, PerformanceProfile, ResourcePlan, RuntimeGovernor, RuntimeGovernorStats},
};
use anyhow::{bail, Context, Result};
use chrono::Local;
use parking_lot::Mutex;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    any::Any,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    panic::{catch_unwind, AssertUnwindSafe},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use walkdir::WalkDir;

const GIB: f64 = 1_073_741_824.0;
const DEFAULT_PARSE_EXPANSION_RATIO: f64 = 32.584_887_884;
const DEFAULT_ANALYSIS_ADDITION_RATIO: f64 = 0.02;
const DEFAULT_PARSE_WORKER_SECONDS_PER_GIB: f64 = 261.1;
const DEFAULT_ANALYSIS_THREAD_SECONDS_PER_GIB: f64 = 13.4;

#[derive(Clone, Debug)]
pub struct BatchPreflight {
    pub demo_count: usize,
    pub input_bytes: u64,
    pub estimated_parse_bytes: u64,
    pub estimated_analysis_addition_bytes: u64,
    pub estimated_total_output_bytes: u64,
    pub safety_headroom_bytes: u64,
    pub required_free_bytes: u64,
    pub available_free_bytes: u64,
    pub output_volume: PathBuf,
    pub estimated_parse_seconds: u64,
    pub estimated_analysis_seconds: u64,
    pub history_samples: usize,
    pub plan: ResourcePlan,
}

impl BatchPreflight {
    pub fn has_enough_space(&self) -> bool {
        self.available_free_bytes >= self.required_free_bytes
    }

    pub fn ensure_space(&self, output: &Path) -> Result<()> {
        require_disk_space(output, self.required_free_bytes, "demo parsing and candidate analysis")?;
        Ok(())
    }

    pub fn summary(&self) -> String {
        let status = if self.has_enough_space() { "PASS" } else { "BLOCKED — insufficient free space" };
        format!(
            "Pre-flight estimate ({status})\n{} demo(s), {} source input\nEstimated time: {} parsing + {} analysis = {} total\nEstimated export: {} parsed data + {} analysis data = {}\nRequired free space with 20% safety headroom: {}\nAvailable on {}: {}\n{} performance: up to {} parser workers and {} analyzer threads\nTiming basis: {}",
            self.demo_count,
            format_bytes(self.input_bytes as f64),
            format_duration(self.estimated_parse_seconds),
            format_duration(self.estimated_analysis_seconds),
            format_duration(self.estimated_parse_seconds.saturating_add(self.estimated_analysis_seconds)),
            format_bytes(self.estimated_parse_bytes as f64),
            format_bytes(self.estimated_analysis_addition_bytes as f64),
            format_bytes(self.estimated_total_output_bytes as f64),
            format_bytes(self.required_free_bytes as f64),
            self.output_volume.display(),
            format_bytes(self.available_free_bytes as f64),
            self.plan.performance_profile.label(),
            self.plan.parse_worker_ceiling,
            self.plan.analysis_worker_ceiling,
            if self.history_samples > 0 {
                format!("{} successful same-machine run(s)", self.history_samples)
            } else {
                "supplied benchmark defaults; this computer will self-calibrate after successful runs".into()
            },
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TimingSample {
    machine: String,
    performance_profile: String,
    input_bytes: u64,
    parse_output_bytes: u64,
    parse_workers: usize,
    analysis_threads: usize,
    parse_wall_seconds: f64,
    analysis_wall_seconds: f64,
}

#[derive(Clone, Debug)]
pub enum ProgressEvent {
    Plan(ResourcePlan),
    Log(String),
    Phase {
        phase: u8,
        completed: usize,
        total: usize,
        fraction: f32,
        eta_seconds: Option<u64>,
        active_workers: usize,
        worker_limit: usize,
    },
    Complete { export_root: PathBuf, candidates: usize },
    Failed(String),
    Cancelled,
}

pub type ProgressSink = Arc<dyn Fn(ProgressEvent) + Send + Sync>;

#[derive(Clone)]
pub struct BatchController {
    cancelled: Arc<AtomicBool>,
    children: Arc<Mutex<Vec<u32>>>,
}

impl BatchController {
    pub fn new() -> Self {
        Self { cancelled: Arc::new(AtomicBool::new(false)), children: Arc::new(Mutex::new(Vec::new())) }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn cancellation_token(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }
}

#[cfg(target_os = "windows")]
struct ProcessPriorityGuard {
    previous: u32,
}

#[cfg(not(target_os = "windows"))]
struct ProcessPriorityGuard;

impl ProcessPriorityGuard {
    fn below_normal() -> Self {
        #[cfg(target_os = "windows")]
        unsafe {
            const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
            let process = GetCurrentProcess();
            let previous = GetPriorityClass(process);
            let _ = SetPriorityClass(process, BELOW_NORMAL_PRIORITY_CLASS);
            Self { previous }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for ProcessPriorityGuard {
    fn drop(&mut self) {
        if self.previous == 0 { return; }
        unsafe {
            let _ = SetPriorityClass(GetCurrentProcess(), self.previous);
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(non_snake_case)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn GetPriorityClass(process: *mut std::ffi::c_void) -> u32;
    fn SetPriorityClass(process: *mut std::ffi::c_void, priority_class: u32) -> i32;
}

fn crash_log_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("TF2FragDemoHelper")
        .join("crash.log")
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    payload.downcast_ref::<&str>().map(|value| (*value).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "non-text Rust panic".into())
}

pub fn estimate_batch_preflight(
    demos: &[PathBuf],
    output_parent: &Path,
    performance_profile: PerformanceProfile,
) -> Result<BatchPreflight> {
    if demos.is_empty() {
        bail!("choose at least one .dem file");
    }
    let mut jobs = Vec::with_capacity(demos.len());
    for (index, demo) in demos.iter().enumerate() {
        let metadata = fs::metadata(demo).with_context(|| format!("could not inspect {}", demo.display()))?;
        if !metadata.is_file() {
            bail!("demo is not a file: {}", demo.display());
        }
        jobs.push(DemoJob {
            order: index + 1,
            demo_path: demo.clone(),
            export_directory: PathBuf::new(),
            source_bytes: metadata.len(),
            parsed_bytes: 0,
        });
    }
    let plan = ResourcePlan::detect(&jobs, performance_profile);
    estimate_batch_preflight_from_jobs(&jobs, output_parent, plan)
}

fn estimate_batch_preflight_from_jobs(
    jobs: &[DemoJob],
    output_parent: &Path,
    plan: ResourcePlan,
) -> Result<BatchPreflight> {
    let input_bytes = jobs.iter().map(|job| job.source_bytes).sum::<u64>();
    if input_bytes == 0 {
        bail!("the selected demos are empty");
    }
    let mut matching_history = timing_history()
        .into_iter()
        .filter(|sample| {
            sample.machine == machine_signature(&plan)
                && sample.performance_profile == plan.performance_profile.label()
                && sample.input_bytes > 0
                && sample.parse_output_bytes > 0
        })
        .collect::<Vec<_>>();
    if matching_history.len() > 20 {
        let excess = matching_history.len() - 20;
        matching_history.drain(..excess);
    }
    let observed_ratios = matching_history
        .iter()
        .map(|sample| sample.parse_output_bytes as f64 / sample.input_bytes as f64)
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect::<Vec<_>>();
    let parse_ratio = percentile_f64(&observed_ratios, 0.90).unwrap_or(DEFAULT_PARSE_EXPANSION_RATIO).max(DEFAULT_PARSE_EXPANSION_RATIO);
    let estimated_parse_bytes = (input_bytes as f64 * parse_ratio).ceil() as u64;
    let estimated_analysis_addition_bytes = (input_bytes as f64 * DEFAULT_ANALYSIS_ADDITION_RATIO).ceil() as u64;
    let estimated_total_output_bytes = estimated_parse_bytes.saturating_add(estimated_analysis_addition_bytes);
    let safety_headroom_bytes = estimated_total_output_bytes / 5;
    let required_free_bytes = estimated_total_output_bytes.saturating_add(safety_headroom_bytes);
    let disk = disk_space_for(output_parent)?;

    let parse_rates = matching_history.iter().filter_map(|sample| {
        let gib = sample.input_bytes as f64 / GIB;
        (gib > 0.0 && sample.parse_wall_seconds > 0.0).then(|| {
            sample.parse_wall_seconds * effective_parallelism(sample.parse_workers, 0.82) / gib
        })
    }).collect::<Vec<_>>();
    let analysis_rates = matching_history.iter().filter_map(|sample| {
        let gib = sample.parse_output_bytes as f64 / GIB;
        (gib > 0.0 && sample.analysis_wall_seconds > 0.0).then(|| {
            sample.analysis_wall_seconds * effective_parallelism(sample.analysis_threads, 0.70) / gib
        })
    }).collect::<Vec<_>>();
    let parse_rate = median_f64(&parse_rates).unwrap_or(DEFAULT_PARSE_WORKER_SECONDS_PER_GIB);
    let analysis_rate = median_f64(&analysis_rates).unwrap_or(DEFAULT_ANALYSIS_THREAD_SECONDS_PER_GIB);
    let parse_seconds = input_bytes as f64 / GIB * parse_rate
        / effective_parallelism(plan.parse_worker_ceiling, 0.82);
    let analysis_seconds = estimated_parse_bytes as f64 / GIB * analysis_rate
        / effective_parallelism(plan.analysis_worker_ceiling, 0.70);

    Ok(BatchPreflight {
        demo_count: jobs.len(),
        input_bytes,
        estimated_parse_bytes,
        estimated_analysis_addition_bytes,
        estimated_total_output_bytes,
        safety_headroom_bytes,
        required_free_bytes,
        available_free_bytes: disk.available_bytes,
        output_volume: disk.mount_point,
        estimated_parse_seconds: (parse_seconds * 1.10).ceil().max(1.0) as u64,
        estimated_analysis_seconds: (analysis_seconds * 1.10).ceil().max(1.0) as u64,
        history_samples: matching_history.len(),
        plan,
    })
}

pub fn run_batch(
    demos: Vec<PathBuf>,
    output_parent: PathBuf,
    item_schema: Option<PathBuf>,
    performance_profile: PerformanceProfile,
    controller: BatchController,
    sink: ProgressSink,
) -> Result<PathBuf> {
    if demos.is_empty() {
        bail!("choose at least one .dem file");
    }
    let _priority_guard = ProcessPriorityGuard::below_normal();
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let root = if demos.len() == 1 {
        output_parent.join(format!("{}_export_{timestamp}", file_stem(&demos[0])))
    } else {
        output_parent.join(format!("tf2_demo_batch_export_{timestamp}"))
    };
    let single_demo = demos.len() == 1;
    let mut jobs = Vec::new();
    for (index, demo) in demos.into_iter().enumerate() {
        let export = if single_demo {
            root.clone()
        } else {
            root.join(format!("{:03}_{}_export", index + 1, sanitize(&file_stem(&demo))))
        };
        jobs.push(DemoJob {
            order: index + 1,
            source_bytes: fs::metadata(&demo).map(|metadata| metadata.len()).unwrap_or_default(),
            parsed_bytes: 0,
            demo_path: demo,
            export_directory: export,
        });
    }
    let plan = ResourcePlan::detect(&jobs, performance_profile);
    let preflight = estimate_batch_preflight_from_jobs(&jobs, &output_parent, plan.clone())?;
    preflight.ensure_space(&output_parent)?;
    fs::create_dir_all(&output_parent)?;
    fs::create_dir_all(&root)?;
    let benchmark = root.join("benchmark");
    fs::create_dir_all(&benchmark)?;
    let log = Arc::new(Mutex::new(File::create(benchmark.join("batch_run.log"))?));
    let emit = |message: String| {
        let _ = writeln!(log.lock(), "{message}");
        sink(ProgressEvent::Log(message));
    };
    let governor = Arc::new(RuntimeGovernor::new(
        performance_profile,
        plan.reserved_memory_bytes,
        controller.cancellation_token(),
    ));
    sink(ProgressEvent::Plan(plan.clone()));
    write_preflight(&root, &preflight)?;
    emit(preflight.summary().replace('\n', " | "));
    emit(format!(
        "Phase 1: parser starts with up to {} workers ({}) and adjusts admission from live free memory",
        plan.parse_worker_ceiling, plan.performance_profile.label()
    ));

    let parser = locate_exporter()?;
    let parse_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(plan.parse_worker_ceiling)
        .thread_name(|index| format!("tf2-parse-{index}"))
        .build()?;
    let parse_completed = AtomicUsize::new(0);
    let parse_completed_bytes = AtomicU64::new(0);
    let total = jobs.len();
    let total_parse_bytes: u64 = jobs.iter().map(|job| job.source_bytes).sum();
    let parse_started = Instant::now();
    let parse_gate = Arc::new(plan.parser_gate(controller.cancellation_token(), governor.clone()));
    // A shared queue keeps every Rayon thread supplied with work.  An indexed
    // par_iter can leave its final contiguous chunk assigned to one thread,
    // which was the cause of the one-worker tail observed in the benchmark.
    let parse_jobs = largest_first(jobs.clone(), false);
    let parse_next = AtomicUsize::new(0);
    let parse_error = Mutex::new(None::<String>);
    parse_pool.install(|| {
        (0..plan.parse_worker_ceiling).into_par_iter().for_each(|_| loop {
            if controller.is_cancelled() { break; }
            let index = parse_next.fetch_add(1, Ordering::SeqCst);
            let Some(job) = parse_jobs.get(index) else { break };
            let result = (|| -> Result<()> {
                let _permit = parse_gate.acquire()?;
                fs::create_dir_all(&job.export_directory)?;
                emit(format!("[PARSE {:03}] {} (active {}/{})", job.order, job.demo_path.display(), parse_gate.active(), parse_gate.limit()));
                let started = Instant::now();
                let mut command = Command::new(&parser);
                command.arg(&job.demo_path).arg(&job.export_directory);
                let output = run_command(command, &controller)?;
                if !output.status.success() {
                    bail!("parser failed for {}: {}", job.demo_path.display(), String::from_utf8_lossy(&output.stderr));
                }
                let elapsed = started.elapsed().as_secs_f64();
                let output_bytes = directory_size(&job.export_directory);
                let completed = parse_completed.fetch_add(1, Ordering::SeqCst) + 1;
                let completed_bytes = parse_completed_bytes.fetch_add(job.source_bytes, Ordering::SeqCst).saturating_add(job.source_bytes);
                sink(ProgressEvent::Phase { phase: 1, completed, total, fraction: completed as f32 / total as f32 * 0.5, eta_seconds: estimated_remaining_seconds(completed_bytes, total_parse_bytes, parse_started.elapsed()), active_workers: parse_gate.active(), worker_limit: parse_gate.limit() });
                emit(format!("[PARSE {:03}] complete in {:.1}s; {:.1} GiB", job.order, elapsed, output_bytes as f64 / 1_073_741_824.0));
                Ok(())
            })();
            if let Err(error) = result {
                if !controller.is_cancelled() { *parse_error.lock() = Some(error.to_string()); controller.cancel(); }
                break;
            }
        });
    });
    if let Some(error) = parse_error.lock().take() {
        emit(format!("Phase 1 failed: {error}"));
        let error = anyhow::Error::msg(error);
        sink(ProgressEvent::Failed(error.to_string()));
        return Err(error);
    }
    if controller.is_cancelled() { sink(ProgressEvent::Cancelled); bail!("cancelled"); }
    let parse_wall_seconds = parse_started.elapsed().as_secs_f64();

    for job in &mut jobs {
        job.parsed_bytes = directory_size(&job.export_directory);
    }
    // Re-plan Phase 2 from the parsed bytes now known.
    let analysis_plan = ResourcePlan::detect(&jobs, performance_profile);
    sink(ProgressEvent::Plan(analysis_plan.clone()));
    emit(format!(
        "Phase 2: Rust analysis starts with {} Rayon threads and up to {} resident demo jobs ({}), live CPU target {:.0}%, dynamic RAM admission; largest exports first",
        analysis_plan.analysis_worker_ceiling,
        analysis_plan.analysis_job_ceiling,
        analysis_plan.performance_profile.label(),
        analysis_plan.performance_profile.target_cpu_percent(),
    ));
    let analysis_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(analysis_plan.analysis_worker_ceiling)
        .thread_name(|index| format!("tf2-analysis-{index}"))
        .build()?;
    let analysis_completed = AtomicUsize::new(0);
    let analysis_completed_bytes = AtomicU64::new(0);
    let total_analysis_bytes: u64 = jobs.iter().map(|job| job.parsed_bytes).sum();
    let analysis_started = Instant::now();
    let analysis_gate = Arc::new(analysis_plan.analyzer_gate(controller.cancellation_token(), governor.clone()));
    let analysis_jobs = largest_first(jobs.clone(), true);
    let analysis_next = AtomicUsize::new(0);
    let analysis_error = Mutex::new(None::<String>);
    // Dispatcher threads wait on the live-memory gate outside Rayon. Blocking
    // them must not consume workers from the shared analysis pool, because all
    // admitted demos use that same pool for NDJSON parsing, state enrichment,
    // and scoring. As jobs finish, the remaining demo automatically inherits
    // the idle Rayon workers instead of collapsing to one core at the tail.
    thread::scope(|scope| {
        for _ in 0..analysis_plan.analysis_job_ceiling {
            let analysis_gate = analysis_gate.clone();
            let governor = governor.clone();
            let analysis_pool = &analysis_pool;
            let analysis_jobs = &analysis_jobs;
            let analysis_next = &analysis_next;
            let analysis_error = &analysis_error;
            let analysis_completed = &analysis_completed;
            let analysis_completed_bytes = &analysis_completed_bytes;
            let controller = &controller;
            let sink = &sink;
            let emit = &emit;
            let item_schema = item_schema.as_deref();
            scope.spawn(move || loop {
                if controller.is_cancelled() { break; }
                let index = analysis_next.fetch_add(1, Ordering::SeqCst);
                let Some(job) = analysis_jobs.get(index) else { break };
                let result = catch_unwind(AssertUnwindSafe(|| -> Result<()> {
                    let _permit = analysis_gate.acquire_for(ResourcePlan::analysis_job_memory_bytes(job.parsed_bytes))?;
                    emit(format!("[ANALYZE {:03}] {} ({:.1} GiB, active {}/{})", job.order, job.demo_path.display(), job.parsed_bytes as f64 / 1_073_741_824.0, analysis_gate.active(), analysis_gate.limit()));
                    let started = Instant::now();
                    let cancellation = controller.cancellation_token();
                    let candidates = analysis_pool.install(|| analyze_export_in_current_pool(
                        &job.export_directory,
                        item_schema,
                        cancellation.as_ref(),
                        governor.as_ref(),
                    ))?;
                    let elapsed = started.elapsed().as_secs_f64();
                    let completed = analysis_completed.fetch_add(1, Ordering::SeqCst) + 1;
                    let completed_bytes = analysis_completed_bytes.fetch_add(job.parsed_bytes, Ordering::SeqCst).saturating_add(job.parsed_bytes);
                    sink(ProgressEvent::Phase { phase: 2, completed, total, fraction: 0.5 + completed as f32 / total as f32 * 0.5, eta_seconds: estimated_remaining_seconds(completed_bytes, total_analysis_bytes, analysis_started.elapsed()), active_workers: analysis_gate.active(), worker_limit: analysis_gate.limit() });
                    emit(format!("[ANALYZE {:03}] complete in {:.1}s; {} candidates", job.order, elapsed, candidates.len()));
                    Ok(())
                }));
                let result = match result {
                    Ok(result) => result,
                    Err(payload) => Err(anyhow::anyhow!(
                        "analyzer panic for {}: {} (see {})",
                        job.demo_path.display(),
                        panic_payload_message(payload.as_ref()),
                        crash_log_path().display(),
                    )),
                };
                if let Err(error) = result {
                    if !controller.is_cancelled() { *analysis_error.lock() = Some(error.to_string()); controller.cancel(); }
                    break;
                }
            });
        }
    });
    if let Some(error) = analysis_error.lock().take() {
        emit(governor_summary_line(governor.stats()));
        emit(format!("Phase 2 failed: {error}"));
        let error = anyhow::Error::msg(error);
        sink(ProgressEvent::Failed(error.to_string()));
        return Err(error);
    }
    if controller.is_cancelled() { sink(ProgressEvent::Cancelled); bail!("cancelled"); }
    let analysis_wall_seconds = analysis_started.elapsed().as_secs_f64();

    let combined = combine_candidates(&root, &jobs)?;
    let governor_stats = governor.stats();
    emit(governor_summary_line(governor_stats));
    write_summary(
        &benchmark,
        &plan,
        &analysis_plan,
        &preflight,
        &jobs,
        combined.len(),
        parse_wall_seconds,
        analysis_wall_seconds,
        governor_stats,
    )?;
    if let Err(error) = append_timing_sample(TimingSample {
        machine: machine_signature(&plan),
        performance_profile: plan.performance_profile.label().into(),
        input_bytes: jobs.iter().map(|job| job.source_bytes).sum(),
        parse_output_bytes: jobs.iter().map(|job| job.parsed_bytes).sum(),
        parse_workers: plan.parse_worker_ceiling,
        analysis_threads: analysis_plan.analysis_worker_ceiling,
        parse_wall_seconds,
        analysis_wall_seconds,
    }) {
        emit(format!("WARNING: completed successfully, but timing calibration history could not be saved: {error}"));
    }
    sink(ProgressEvent::Complete { export_root: root.clone(), candidates: combined.len() });
    Ok(root)
}

fn run_command(mut command: Command, controller: &BatchController) -> Result<std::process::Output> {
    // export_all.exe is a command-line worker, but it is launched behind the
    // Slint GUI.  Prevent every concurrent parser worker from creating its own
    // Windows console window while retaining captured stdout/stderr.
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn()?;
    controller.children.lock().push(child.id());
    loop {
        if controller.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            bail!("cancelled");
        }
        if child.try_wait()?.is_some() {
            controller.children.lock().retain(|pid| *pid != child.id());
            return child.wait_with_output().map_err(Into::into);
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn locate_exporter() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let directory = executable.parent().context("application directory is unavailable")?;
    let name = if cfg!(windows) { "export_all.exe" } else { "export_all" };
    let candidates = [
        directory.join(name),
        directory.join("parser").join(name),
        directory.join("..").join(name),
        PathBuf::from("target").join("release").join(name),
        PathBuf::from("parser").join("target").join("release").join(name),
    ];
    candidates.into_iter().find(|path| path.is_file()).context("export_all Rust parser executable was not found beside the application")
}

fn combine_candidates(root: &Path, jobs: &[DemoJob]) -> Result<Vec<Candidate>> {
    let mut combined = Vec::new();
    for job in jobs {
        let path = job.export_directory.join("frag_candidates.ndjson");
        if !path.is_file() {
            continue;
        }
        for line in BufReader::new(File::open(path)?).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let mut candidate: Candidate = serde_json::from_str(&line)?;
            candidate.source_demo = job.demo_path.display().to_string();
            candidate.extra.insert("batch_context".into(), json!({
                "demo_order":job.order,
                "source_demo":job.demo_path,
                "export_directory":job.export_directory,
                "map_name":candidate.map_name,
                "mode":candidate.demo_context.mode,
                "mode_label":candidate.demo_context.mode_label,
            }));
            combined.push(candidate);
        }
    }
    combined.sort_by(|left, right| right.overall_score.total_cmp(&left.overall_score).then(left.clip_start_tick.cmp(&right.clip_start_tick)));
    let mut output = File::create(root.join("frag_candidates.ndjson"))?;
    for candidate in &combined {
        serde_json::to_writer(&mut output, candidate)?;
        output.write_all(b"\n")?;
    }
    fs::write(root.join("frag_summary.json"), serde_json::to_vec_pretty(&json!({"candidate_count":combined.len(),"demo_count":jobs.len(),"backend":"rust"}))?)?;
    Ok(combined)
}

fn write_preflight(root: &Path, estimate: &BatchPreflight) -> Result<()> {
    let plan = &estimate.plan;
    let text = format!(
        "RUST ADAPTIVE RESOURCE PLAN\nPerformance profile: {}\nWhole-system CPU target: {:.0}%\nLogical processors: {}\nAvailable RAM: {:.1} GB\nReserved RAM: {:.1} GB\nParser worker ceiling: {}\nAnalyzer Rayon thread ceiling: {}\nConcurrent analyzer demo ceiling: {}\nWorker admission: rechecked before each job from live CPU and RAM pressure\nIn-phase analysis throttling: enabled\nWindows batch priority: below normal (foreground applications take precedence)\n\nTIME PREFLIGHT\nEstimated parser time: {}\nEstimated analyzer time: {}\nEstimated total time: {}\nSame-machine timing samples: {}\n\nDISK PREFLIGHT\nOutput volume: {}\nInput demos: {}\nEstimated parse output: {}\nEstimated analysis additions: {}\nEstimated output total: {}\nSafety headroom (20%): {}\nRequired free space: {}\nCurrently free: {}\nStatus: {}\n\n{}\n",
        plan.performance_profile.label(),
        plan.performance_profile.target_cpu_percent(),
        plan.logical_processors,
        plan.available_memory_bytes as f64 / 1_073_741_824.0,
        plan.reserved_memory_bytes as f64 / 1_073_741_824.0,
        plan.parse_worker_ceiling,
        plan.analysis_worker_ceiling,
        plan.analysis_job_ceiling,
        format_duration(estimate.estimated_parse_seconds),
        format_duration(estimate.estimated_analysis_seconds),
        format_duration(estimate.estimated_parse_seconds.saturating_add(estimate.estimated_analysis_seconds)),
        estimate.history_samples,
        estimate.output_volume.display(),
        format_bytes(estimate.input_bytes as f64),
        format_bytes(estimate.estimated_parse_bytes as f64),
        format_bytes(estimate.estimated_analysis_addition_bytes as f64),
        format_bytes(estimate.estimated_total_output_bytes as f64),
        format_bytes(estimate.safety_headroom_bytes as f64),
        format_bytes(estimate.required_free_bytes as f64),
        format_bytes(estimate.available_free_bytes as f64),
        if estimate.has_enough_space() { "PASS" } else { "BLOCKED" },
        plan.reason.join("\n"),
    );
    fs::write(root.join("PRE_FLIGHT_ESTIMATE.txt"), text)?;
    Ok(())
}

#[derive(Serialize)]
struct Summary<'a> {
    format: &'static str,
    backend: &'static str,
    parse_plan: &'a ResourcePlan,
    analysis_plan: &'a ResourcePlan,
    demo_count: usize,
    candidate_count: usize,
    total_input_bytes: u64,
    total_parse_output_bytes: u64,
    estimated_parse_output_bytes: u64,
    estimated_required_free_bytes: u64,
    initial_free_disk_bytes: u64,
    estimated_parse_seconds: u64,
    estimated_analysis_seconds: u64,
    actual_parse_wall_seconds: f64,
    actual_analysis_wall_seconds: f64,
    runtime_governor: RuntimeGovernorStats,
}

fn write_summary(
    path: &Path,
    parse: &ResourcePlan,
    analysis: &ResourcePlan,
    preflight: &BatchPreflight,
    jobs: &[DemoJob],
    candidates: usize,
    parse_wall_seconds: f64,
    analysis_wall_seconds: f64,
    runtime_governor: RuntimeGovernorStats,
) -> Result<()> {
    let summary = Summary {
        format: "tf2-frag-helper-rust-benchmark",
        backend: "rust-streaming-analysis",
        parse_plan: parse,
        analysis_plan: analysis,
        demo_count: jobs.len(),
        candidate_count: candidates,
        total_input_bytes: jobs.iter().map(|job| job.source_bytes).sum(),
        total_parse_output_bytes: jobs.iter().map(|job| job.parsed_bytes).sum(),
        estimated_parse_output_bytes: preflight.estimated_parse_bytes,
        estimated_required_free_bytes: preflight.required_free_bytes,
        initial_free_disk_bytes: preflight.available_free_bytes,
        estimated_parse_seconds: preflight.estimated_parse_seconds,
        estimated_analysis_seconds: preflight.estimated_analysis_seconds,
        actual_parse_wall_seconds: parse_wall_seconds,
        actual_analysis_wall_seconds: analysis_wall_seconds,
        runtime_governor,
    };
    fs::write(path.join("benchmark_summary.json"), serde_json::to_vec_pretty(&summary)?)?;
    Ok(())
}

fn timing_history_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TF2FragDemoHelper")
        .join("preflight_timing_history.ndjson")
}

fn timing_history() -> Vec<TimingSample> {
    File::open(timing_history_path())
        .ok()
        .map(BufReader::new)
        .into_iter()
        .flat_map(|reader| reader.lines().map_while(|line| line.ok()))
        .filter_map(|line| serde_json::from_str::<TimingSample>(&line).ok())
        .collect()
}

fn append_timing_sample(sample: TimingSample) -> Result<()> {
    let path = timing_history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut output, &sample)?;
    output.write_all(b"\n")?;
    Ok(())
}

fn machine_signature(plan: &ResourcePlan) -> String {
    format!(
        "cpu{}-ram{}g",
        plan.logical_processors,
        (plan.total_memory_bytes as f64 / GIB).round() as u64,
    )
}

fn effective_parallelism(workers: usize, additional_worker_efficiency: f64) -> f64 {
    if workers <= 1 {
        1.0
    } else {
        1.0 + (workers - 1) as f64 * additional_worker_efficiency
    }
}

fn median_f64(values: &[f64]) -> Option<f64> {
    percentile_f64(values, 0.50)
}

fn percentile_f64(values: &[f64], percentile: f64) -> Option<f64> {
    let mut values = values.iter().copied().filter(|value| value.is_finite() && *value > 0.0).collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let index = ((values.len() - 1) as f64 * percentile.clamp(0.0, 1.0)).round() as usize;
    values.get(index).copied()
}

fn governor_summary_line(stats: RuntimeGovernorStats) -> String {
    format!(
        "Adaptive governor: target {:.0}% CPU, observed peak {:.1}%, {} throttle pauses ({} ms total), minimum available RAM {:.2} GiB",
        stats.target_cpu_percent,
        stats.peak_cpu_percent,
        stats.throttle_count,
        stats.throttle_sleep_ms,
        stats.minimum_available_memory_bytes as f64 / 1_073_741_824.0,
    )
}

fn directory_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum()
}

fn estimated_remaining_seconds(completed_bytes: u64, total_bytes: u64, elapsed: Duration) -> Option<u64> {
    if completed_bytes == 0 || total_bytes <= completed_bytes || elapsed.is_zero() {
        return None;
    }
    let bytes_per_second = completed_bytes as f64 / elapsed.as_secs_f64();
    (bytes_per_second.is_finite() && bytes_per_second > 0.0)
        .then(|| ((total_bytes - completed_bytes) as f64 / bytes_per_second).ceil() as u64)
}

fn file_stem(path: &Path) -> String {
    path.file_stem().and_then(|value| value.to_str()).unwrap_or("demo").into()
}

fn sanitize(value: &str) -> String {
    value.chars().map(|character| if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') { character } else { '_' }).collect()
}

#[cfg(test)]
mod preflight_tests {
    use super::*;

    #[test]
    fn parallelism_scaling_keeps_one_worker_exact_and_discounts_contention() {
        assert_eq!(effective_parallelism(1, 0.82), 1.0);
        assert!(effective_parallelism(8, 0.82) > 6.0);
        assert!(effective_parallelism(8, 0.82) < 8.0);
    }

    #[test]
    fn percentile_uses_sorted_finite_positive_samples() {
        let values = [30.0, 10.0, f64::NAN, 20.0, -1.0];
        assert_eq!(median_f64(&values), Some(20.0));
        assert_eq!(percentile_f64(&values, 0.90), Some(30.0));
    }
}
