use crate::models::DemoJob;
use serde::{Deserialize, Serialize};
use anyhow::{bail, Result};
use std::{
    cell::RefCell,
    cmp::Reverse,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use sysinfo::System;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

/// A user preference layered on top of the machine-specific resource plan.
/// The underlying CPU and RAM calculation remains dynamic; this only scales
/// its safe maximum before the live-memory gate admits a new job.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PerformanceProfile {
    Low,
    Medium,
    High,
}

impl PerformanceProfile {
    pub fn from_setting(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "low" => Self::Low,
            "medium" => Self::Medium,
            _ => Self::High,
        }
    }

    pub fn label(self) -> &'static str {
        match self { Self::Low => "Low", Self::Medium => "Medium", Self::High => "High" }
    }

    fn scale(self, maximum: usize) -> usize {
        let multiplier = match self { Self::Low => 0.45, Self::Medium => 0.70, Self::High => 1.0 };
        ((maximum as f64 * multiplier).round() as usize).clamp(1, maximum.max(1))
    }

    /// Maximum whole-system CPU load the adaptive governor aims for. The
    /// worker ceiling above still sizes the initial pool for this computer;
    /// this target lets foreground applications force the helper lower while
    /// a batch is already running.
    pub fn target_cpu_percent(self) -> f32 {
        match self { Self::Low => 55.0, Self::Medium => 75.0, Self::High => 92.0 }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourcePlan {
    pub performance_profile: PerformanceProfile,
    pub logical_processors: usize,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub reserved_memory_bytes: u64,
    pub parse_workers: usize,
    pub analysis_workers: usize,
    pub parse_worker_ceiling: usize,
    pub analysis_worker_ceiling: usize,
    pub analysis_job_ceiling: usize,
    pub estimated_parse_worker_bytes: u64,
    pub estimated_analysis_worker_bytes: u64,
    pub reason: Vec<String>,
}

impl ResourcePlan {
    pub fn detect(jobs: &[DemoJob], performance_profile: PerformanceProfile) -> Self {
        let mut system = System::new_all();
        system.refresh_memory();
        let logical = thread::available_parallelism().map(|value| value.get()).unwrap_or(1);
        let total = system.total_memory();
        let available = system.available_memory();
        // Leave enough room for TF2, the UI, and the operating system, but do
        // not reserve so much that a capable machine is artificially held to
        // one or two workers.  The live gate below rechecks this before every
        // new job, so this is a starting point rather than a fixed allocation.
        let reserve = (total / 8).max(2 * GIB).min(available.saturating_div(3));
        let usable = available.saturating_sub(reserve).max(GIB);

        let source_p75 = percentile(jobs.iter().map(|job| job.source_bytes).collect(), 0.75).max(64 * MIB);
        let parsed_p75 = percentile(
            jobs.iter().filter_map(|job| (job.parsed_bytes > 0).then_some(job.parsed_bytes)).collect(),
            0.75,
        );
        // JSON output is streamed, but the decoder currently retains the
        // complete source .dem bytes. Include those bytes plus parser state so
        // High performance remains aggressive without overcommitting RAM.
        let parse_memory = (256 * MIB + source_p75).clamp(512 * MIB, 4 * GIB);
        // Analysis retains compact state histories plus death snapshots while
        // scanning the NDJSON. Account for allocator/serde overhead instead of
        // assuming the working set is only 1/32 of the parsed export. The live
        // gate and runtime low-memory guard provide two additional safeguards.
        let analysis_memory = Self::analysis_job_memory_bytes(parsed_p75);

        let cpu_reserve = if logical >= 12 { 2 } else if logical >= 4 { 1 } else { 0 };
        let cpu_budget = logical.saturating_sub(cpu_reserve).max(1);
        let parse_memory_limit = (usable / parse_memory).max(1) as usize;
        let analysis_memory_limit = (usable / analysis_memory).max(1) as usize;
        let count = jobs.len().max(1);
        let parse_hardware_ceiling = cpu_budget.min(parse_memory_limit).min(count).max(1);
        // Rayon threads and simultaneously resident demo analyses are separate
        // limits. A single large demo should still use all safe CPU threads;
        // RAM controls how many demos may be resident at once.
        let analysis_thread_hardware_ceiling = cpu_budget;
        let parse_ceiling = performance_profile.scale(parse_hardware_ceiling);
        let analysis_ceiling = performance_profile.scale(analysis_thread_hardware_ceiling);
        let analysis_job_ceiling = analysis_memory_limit.min(count).min(analysis_ceiling).max(1);

        Self {
            performance_profile,
            logical_processors: logical,
            total_memory_bytes: total,
            available_memory_bytes: available,
            reserved_memory_bytes: reserve,
            parse_workers: parse_ceiling,
            analysis_workers: analysis_ceiling,
            parse_worker_ceiling: parse_ceiling,
            analysis_worker_ceiling: analysis_ceiling,
            analysis_job_ceiling,
            estimated_parse_worker_bytes: parse_memory,
            estimated_analysis_worker_bytes: analysis_memory,
            reason: vec![
                format!("{} performance profile: {}% of this computer's safe dynamic worker maximum", performance_profile.label(), match performance_profile { PerformanceProfile::Low => 45, PerformanceProfile::Medium => 70, PerformanceProfile::High => 100 }),
                format!("{} logical processors; {} reserved for the OS/UI", logical, cpu_reserve),
                format!("analysis sized from parsed exports; p75={} MiB", parsed_p75 / MIB),
                format!("live CPU governor target: at most {:.0}% whole-system load; foreground work can force it lower", performance_profile.target_cpu_percent()),
                "worker admission is rechecked from live CPU pressure and available memory before every job starts".into(),
                "sustained critically low memory stops analysis cleanly before the operating system terminates it".into(),
                "per-run benchmark logs and summaries are written beside the export".into(),
            ],
        }
    }

    pub fn parser_gate(&self, cancelled: Arc<AtomicBool>, governor: Arc<RuntimeGovernor>) -> AdaptiveWorkerGate {
        AdaptiveWorkerGate::new(
            self.parse_worker_ceiling,
            self.estimated_parse_worker_bytes,
            self.reserved_memory_bytes,
            cancelled,
            governor,
        )
    }

    pub fn analyzer_gate(&self, cancelled: Arc<AtomicBool>, governor: Arc<RuntimeGovernor>) -> AdaptiveWorkerGate {
        AdaptiveWorkerGate::new(
            self.analysis_job_ceiling,
            self.estimated_analysis_worker_bytes,
            self.reserved_memory_bytes,
            cancelled,
            governor,
        )
    }

    pub fn analysis_job_memory_bytes(parsed_bytes: u64) -> u64 {
        if parsed_bytes > 0 {
            (768 * MIB + parsed_bytes.saturating_mul(3) / 4).clamp(GIB, 8 * GIB)
        } else {
            GIB
        }
    }

}

/// A lightweight admission controller for the Rayon pools.  Rayon owns enough
/// threads to use the machine, while this gate admits only as many processes or
/// analyses as current free memory can safely support.  It therefore expands
/// and contracts between jobs instead of baking a single worker count into the
/// executable.
pub struct AdaptiveWorkerGate {
    ceiling: usize,
    worker_memory_bytes: u64,
    reserved_memory_bytes: u64,
    state: Arc<parking_lot::Mutex<GateState>>,
    current_limit: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
    governor: Arc<RuntimeGovernor>,
}

#[derive(Default)]
struct GateState {
    active: usize,
    reserved_bytes: u64,
}

pub struct WorkerPermit {
    state: Arc<parking_lot::Mutex<GateState>>,
    reserved_bytes: u64,
}

impl Drop for WorkerPermit {
    fn drop(&mut self) {
        let mut state = self.state.lock();
        state.active = state.active.saturating_sub(1);
        state.reserved_bytes = state.reserved_bytes.saturating_sub(self.reserved_bytes);
    }
}

impl AdaptiveWorkerGate {
    fn new(
        ceiling: usize,
        worker_memory_bytes: u64,
        reserved_memory_bytes: u64,
        cancelled: Arc<AtomicBool>,
        governor: Arc<RuntimeGovernor>,
    ) -> Self {
        Self {
            ceiling: ceiling.max(1),
            worker_memory_bytes: worker_memory_bytes.max(1),
            reserved_memory_bytes,
            state: Arc::new(parking_lot::Mutex::new(GateState::default())),
            current_limit: Arc::new(AtomicUsize::new(1)),
            cancelled,
            governor,
        }
    }

    pub fn acquire(&self) -> Result<WorkerPermit> {
        self.acquire_for(self.worker_memory_bytes)
    }

    pub fn acquire_for(&self, requested_memory_bytes: u64) -> Result<WorkerPermit> {
        let requested_memory_bytes = requested_memory_bytes.max(1);
        loop {
            if self.cancelled.load(Ordering::Relaxed) {
                bail!("cancelled");
            }
            self.governor.checkpoint()?;
            let limit = self.live_limit();
            self.current_limit.store(limit, Ordering::Relaxed);
            let available = self.governor.available_memory_bytes();
            let reserve = self.reserved_memory_bytes.min(available.saturating_div(2));
            let usable = available.saturating_sub(reserve);
            let mut state = self.state.lock();
            let fits_memory = state.reserved_bytes.saturating_add(requested_memory_bytes) <= usable
                || state.active == 0;
            if state.active < limit && fits_memory {
                state.active += 1;
                state.reserved_bytes = state.reserved_bytes.saturating_add(requested_memory_bytes);
                return Ok(WorkerPermit { state: self.state.clone(), reserved_bytes: requested_memory_bytes });
            }
            drop(state);
            thread::sleep(Duration::from_millis(125));
        }
    }

    pub fn active(&self) -> usize {
        self.state.lock().active
    }

    pub fn limit(&self) -> usize {
        self.current_limit.load(Ordering::Relaxed)
    }

    fn live_limit(&self) -> usize {
        let available = self.governor.available_memory_bytes();
        let reserve = self.reserved_memory_bytes.min(available.saturating_div(2));
        let usable = available.saturating_sub(reserve).max(self.worker_memory_bytes);
        let memory_limit = ((usable / self.worker_memory_bytes) as usize).clamp(1, self.ceiling);
        self.governor.admission_limit(memory_limit)
    }
}

thread_local! {
    static LAST_GOVERNOR_SLEEP: RefCell<Option<Instant>> = const { RefCell::new(None) };
}

struct GovernorSample {
    system: System,
    last_refresh: Instant,
    low_memory_samples: u8,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct RuntimeGovernorStats {
    pub target_cpu_percent: f32,
    pub peak_cpu_percent: f32,
    pub throttle_count: u64,
    pub throttle_sleep_ms: u64,
    pub minimum_available_memory_bytes: u64,
}

/// Cooperative live-load governor shared by both phases. It samples total
/// system CPU/RAM at a safe interval, lowers new-job admission, and applies a
/// small duty-cycle delay inside the Rust analysis pool. Windows additionally
/// runs the batch below normal priority (see batch.rs), so a game or video gets
/// scheduling preference immediately rather than waiting for the next sample.
pub struct RuntimeGovernor {
    target_cpu_percent: f32,
    critical_memory_bytes: u64,
    sample: parking_lot::Mutex<GovernorSample>,
    observed_cpu_bits: AtomicU32,
    delay_ms: AtomicU64,
    peak_cpu_bits: AtomicU32,
    throttle_count: AtomicU64,
    throttle_sleep_ms: AtomicU64,
    minimum_available_memory: AtomicU64,
    available_memory: AtomicU64,
    cancelled: Arc<AtomicBool>,
}

impl RuntimeGovernor {
    pub fn new(profile: PerformanceProfile, reserved_memory_bytes: u64, cancelled: Arc<AtomicBool>) -> Self {
        let mut system = System::new_all();
        system.refresh_memory();
        let total = system.total_memory();
        let available = system.available_memory();
        // Roughly 3% of physical RAM, never below 512 MiB. Requiring several
        // consecutive low readings avoids aborting for a brief allocation.
        let critical_memory = (total / 32).max(512 * MIB).min(reserved_memory_bytes.max(512 * MIB));
        Self {
            target_cpu_percent: profile.target_cpu_percent(),
            critical_memory_bytes: critical_memory,
            sample: parking_lot::Mutex::new(GovernorSample {
                system,
                last_refresh: Instant::now(),
                low_memory_samples: 0,
            }),
            observed_cpu_bits: AtomicU32::new(0.0f32.to_bits()),
            delay_ms: AtomicU64::new(0),
            peak_cpu_bits: AtomicU32::new(0.0f32.to_bits()),
            throttle_count: AtomicU64::new(0),
            throttle_sleep_ms: AtomicU64::new(0),
            minimum_available_memory: AtomicU64::new(available),
            available_memory: AtomicU64::new(available),
            cancelled,
        }
    }

    pub fn checkpoint(&self) -> Result<()> {
        if self.cancelled.load(Ordering::Relaxed) {
            bail!("cancelled");
        }
        self.refresh_if_due()?;
        let delay = self.delay_ms.load(Ordering::Relaxed);
        if delay == 0 {
            return Ok(());
        }
        let should_sleep = LAST_GOVERNOR_SLEEP.with(|last| {
            let now = Instant::now();
            let mut last = last.borrow_mut();
            let due = last.is_none_or(|previous| now.duration_since(previous) >= Duration::from_millis(250));
            if due { *last = Some(now); }
            due
        });
        if should_sleep {
            thread::sleep(Duration::from_millis(delay));
            self.throttle_count.fetch_add(1, Ordering::Relaxed);
            self.throttle_sleep_ms.fetch_add(delay, Ordering::Relaxed);
        }
        Ok(())
    }

    pub fn admission_limit(&self, memory_limit: usize) -> usize {
        let observed = f32::from_bits(self.observed_cpu_bits.load(Ordering::Relaxed));
        if observed <= self.target_cpu_percent || observed <= 0.0 {
            return memory_limit.max(1);
        }
        let scale = (self.target_cpu_percent / observed).clamp(0.25, 1.0);
        ((memory_limit as f32 * scale).floor() as usize).clamp(1, memory_limit.max(1))
    }

    pub fn stats(&self) -> RuntimeGovernorStats {
        RuntimeGovernorStats {
            target_cpu_percent: self.target_cpu_percent,
            peak_cpu_percent: f32::from_bits(self.peak_cpu_bits.load(Ordering::Relaxed)),
            throttle_count: self.throttle_count.load(Ordering::Relaxed),
            throttle_sleep_ms: self.throttle_sleep_ms.load(Ordering::Relaxed),
            minimum_available_memory_bytes: self.minimum_available_memory.load(Ordering::Relaxed),
        }
    }

    pub fn available_memory_bytes(&self) -> u64 {
        self.available_memory.load(Ordering::Relaxed)
    }

    fn refresh_if_due(&self) -> Result<()> {
        let Some(mut sample) = self.sample.try_lock() else { return Ok(()) };
        if sample.last_refresh.elapsed() < Duration::from_millis(750) {
            return Ok(());
        }
        sample.system.refresh_cpu_usage();
        sample.system.refresh_memory();
        sample.last_refresh = Instant::now();
        let cpu = sample.system.global_cpu_usage().clamp(0.0, 100.0);
        let available = sample.system.available_memory();
        self.available_memory.store(available, Ordering::Relaxed);
        self.observed_cpu_bits.store(cpu.to_bits(), Ordering::Relaxed);
        update_peak_f32(&self.peak_cpu_bits, cpu);
        update_min_u64(&self.minimum_available_memory, available);

        let overload = (cpu - self.target_cpu_percent).max(0.0);
        let delay = if overload < 1.0 {
            0
        } else {
            (10.0 + overload * 3.0).round().clamp(10.0, 140.0) as u64
        };
        self.delay_ms.store(delay, Ordering::Relaxed);

        if available < self.critical_memory_bytes {
            sample.low_memory_samples = sample.low_memory_samples.saturating_add(1);
        } else {
            sample.low_memory_samples = 0;
        }
        if sample.low_memory_samples >= 3 {
            bail!(
                "batch stopped before system memory exhaustion (only {:.2} GiB available; safety floor {:.2} GiB). Close memory-heavy applications or use a lower performance profile, then retry",
                available as f64 / GIB as f64,
                self.critical_memory_bytes as f64 / GIB as f64,
            );
        }
        Ok(())
    }
}

fn update_peak_f32(target: &AtomicU32, value: f32) {
    let mut current = target.load(Ordering::Relaxed);
    while value > f32::from_bits(current) {
        match target.compare_exchange_weak(current, value.to_bits(), Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(actual) => current = actual,
        }
    }
}

fn update_min_u64(target: &AtomicU64, value: u64) {
    let mut current = target.load(Ordering::Relaxed);
    while value < current {
        match target.compare_exchange_weak(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(actual) => current = actual,
        }
    }
}

pub fn largest_first(mut jobs: Vec<DemoJob>, analysis: bool) -> Vec<DemoJob> {
    jobs.sort_by_key(|job| Reverse(if analysis { job.parsed_bytes } else { job.source_bytes }));
    jobs
}

fn percentile(mut values: Vec<u64>, percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let index = ((values.len() - 1) as f64 * percentile).ceil() as usize;
    values[index.min(values.len() - 1)]
}
