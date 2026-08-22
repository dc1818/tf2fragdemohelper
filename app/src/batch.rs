use crate::{
    analyzer::analyze_export,
    models::{Candidate, DemoJob},
    scheduler::{largest_first, ResourcePlan},
};
use anyhow::{bail, Context, Result};
use chrono::Local;
use parking_lot::Mutex;
use rayon::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use walkdir::WalkDir;

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

pub fn run_batch(
    demos: Vec<PathBuf>,
    output_parent: PathBuf,
    item_schema: Option<PathBuf>,
    controller: BatchController,
    sink: ProgressSink,
) -> Result<PathBuf> {
    if demos.is_empty() {
        bail!("choose at least one .dem file");
    }
    fs::create_dir_all(&output_parent)?;
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let root = if demos.len() == 1 {
        output_parent.join(format!("{}_export_{timestamp}", file_stem(&demos[0])))
    } else {
        output_parent.join(format!("tf2_demo_batch_export_{timestamp}"))
    };
    fs::create_dir_all(&root)?;
    let benchmark = root.join("benchmark");
    fs::create_dir_all(&benchmark)?;
    let log = Arc::new(Mutex::new(File::create(benchmark.join("batch_run.log"))?));
    let emit = |message: String| {
        let _ = writeln!(log.lock(), "{message}");
        sink(ProgressEvent::Log(message));
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
    let plan = ResourcePlan::detect(&jobs);
    sink(ProgressEvent::Plan(plan.clone()));
    write_preflight(&root, &jobs, &plan)?;
    emit(format!(
        "Phase 1: parser starts with up to {} workers and adjusts admission from live free memory",
        plan.parse_worker_ceiling
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
    let parse_gate = Arc::new(plan.parser_gate(controller.cancellation_token()));
    let parse_result: Result<()> = parse_pool.install(|| {
        largest_first(jobs.clone(), false).par_iter().try_for_each(|job| {
            if controller.is_cancelled() {
                bail!("cancelled");
            }
            let _permit = parse_gate.acquire()?;
            fs::create_dir_all(&job.export_directory)?;
            emit(format!(
                "[PARSE {:03}] {} (active {}/{})",
                job.order,
                job.demo_path.display(),
                parse_gate.active(),
                parse_gate.limit()
            ));
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
            sink(ProgressEvent::Phase {
                phase: 1,
                completed,
                total,
                fraction: completed as f32 / total as f32 * 0.5,
                eta_seconds: estimated_remaining_seconds(completed_bytes, total_parse_bytes, parse_started.elapsed()),
                active_workers: parse_gate.active(),
                worker_limit: parse_gate.limit(),
            });
            emit(format!("[PARSE {:03}] complete in {:.1}s; {:.1} GiB", job.order, elapsed, output_bytes as f64 / 1_073_741_824.0));
            Ok(())
        })
    });
    if let Err(error) = parse_result {
        if controller.is_cancelled() {
            sink(ProgressEvent::Cancelled);
        } else {
            sink(ProgressEvent::Failed(error.to_string()));
        }
        return Err(error);
    }

    for job in &mut jobs {
        job.parsed_bytes = directory_size(&job.export_directory);
    }
    // Re-plan Phase 2 from the parsed bytes now known.
    let analysis_plan = ResourcePlan::detect(&jobs);
    emit(format!(
        "Phase 2: Rust analysis starts with up to {} workers and adjusts admission from live free memory; largest exports first",
        analysis_plan.analysis_worker_ceiling
    ));
    let analysis_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(analysis_plan.analysis_worker_ceiling)
        .thread_name(|index| format!("tf2-analysis-{index}"))
        .build()?;
    let analysis_completed = AtomicUsize::new(0);
    let analysis_completed_bytes = AtomicU64::new(0);
    let total_analysis_bytes: u64 = jobs.iter().map(|job| job.parsed_bytes).sum();
    let analysis_started = Instant::now();
    let candidates_total = AtomicUsize::new(0);
    let analysis_gate = Arc::new(analysis_plan.analyzer_gate(controller.cancellation_token()));
    let analysis_result: Result<()> = analysis_pool.install(|| {
        largest_first(jobs.clone(), true).par_iter().try_for_each(|job| {
            if controller.is_cancelled() {
                bail!("cancelled");
            }
            let _permit = analysis_gate.acquire()?;
            emit(format!(
                "[ANALYZE {:03}] {} ({:.1} GiB, active {}/{})",
                job.order,
                job.demo_path.display(),
                job.parsed_bytes as f64 / 1_073_741_824.0,
                analysis_gate.active(),
                analysis_gate.limit()
            ));
            let started = Instant::now();
            let candidates = analyze_export(&job.export_directory, item_schema.as_deref())?;
            let elapsed = started.elapsed().as_secs_f64();
            candidates_total.fetch_add(candidates.len(), Ordering::SeqCst);
            let completed = analysis_completed.fetch_add(1, Ordering::SeqCst) + 1;
            let completed_bytes = analysis_completed_bytes.fetch_add(job.parsed_bytes, Ordering::SeqCst).saturating_add(job.parsed_bytes);
            sink(ProgressEvent::Phase {
                phase: 2,
                completed,
                total,
                fraction: 0.5 + completed as f32 / total as f32 * 0.5,
                eta_seconds: estimated_remaining_seconds(completed_bytes, total_analysis_bytes, analysis_started.elapsed()),
                active_workers: analysis_gate.active(),
                worker_limit: analysis_gate.limit(),
            });
            emit(format!("[ANALYZE {:03}] complete in {:.1}s; {} candidates", job.order, elapsed, candidates.len()));
            Ok(())
        })
    });
    if let Err(error) = analysis_result {
        if controller.is_cancelled() {
            sink(ProgressEvent::Cancelled);
        } else {
            sink(ProgressEvent::Failed(error.to_string()));
        }
        return Err(error);
    }

    let combined = combine_candidates(&root, &jobs)?;
    write_summary(&benchmark, &plan, &analysis_plan, &jobs, combined.len())?;
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

fn write_preflight(root: &Path, jobs: &[DemoJob], plan: &ResourcePlan) -> Result<()> {
    let input: u64 = jobs.iter().map(|job| job.source_bytes).sum();
    // Supplied benchmark: p90 safety estimate was 32.58x and actual was 27.71x.
    let parse_estimate = (input as f64 * 32.584_887_884).ceil() as u64;
    let analysis_additions = (input as f64 * 0.02).ceil() as u64;
    let total = parse_estimate.saturating_add(analysis_additions);
    let headroom = total / 5;
    let text = format!(
        "RUST ADAPTIVE RESOURCE PLAN\nLogical processors: {}\nAvailable RAM: {:.1} GB\nReserved RAM: {:.1} GB\nParser worker ceiling: {}\nAnalyzer worker ceiling: {}\nWorker admission: rechecked before each job from live available RAM\n\nDISK PREFLIGHT\nInput demos: {:.2} GB\nEstimated parse output: {:.2} GB\nEstimated analysis additions: {:.2} GB\nSafety headroom: {:.2} GB\nRecommended free space: {:.2} GB\n\n{}\n",
        plan.logical_processors,
        plan.available_memory_bytes as f64 / 1_073_741_824.0,
        plan.reserved_memory_bytes as f64 / 1_073_741_824.0,
        plan.parse_worker_ceiling,
        plan.analysis_worker_ceiling,
        input as f64 / 1_073_741_824.0,
        parse_estimate as f64 / 1_073_741_824.0,
        analysis_additions as f64 / 1_073_741_824.0,
        headroom as f64 / 1_073_741_824.0,
        total.saturating_add(headroom) as f64 / 1_073_741_824.0,
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
}

fn write_summary(path: &Path, parse: &ResourcePlan, analysis: &ResourcePlan, jobs: &[DemoJob], candidates: usize) -> Result<()> {
    let summary = Summary {
        format: "tf2-frag-helper-rust-benchmark",
        backend: "rust-streaming-analysis",
        parse_plan: parse,
        analysis_plan: analysis,
        demo_count: jobs.len(),
        candidate_count: candidates,
        total_input_bytes: jobs.iter().map(|job| job.source_bytes).sum(),
        total_parse_output_bytes: jobs.iter().map(|job| job.parsed_bytes).sum(),
    };
    fs::write(path.join("benchmark_summary.json"), serde_json::to_vec_pretty(&summary)?)?;
    Ok(())
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
