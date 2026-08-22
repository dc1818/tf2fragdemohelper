use crate::models::{DemoJob, WorkerSample};
use serde::{Deserialize, Serialize};
use anyhow::{bail, Result};
use std::{
    cmp::Reverse,
    collections::BTreeMap,
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use sysinfo::System;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourcePlan {
    pub logical_processors: usize,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub reserved_memory_bytes: u64,
    pub parse_workers: usize,
    pub analysis_workers: usize,
    pub parse_worker_ceiling: usize,
    pub analysis_worker_ceiling: usize,
    pub estimated_parse_worker_bytes: u64,
    pub estimated_analysis_worker_bytes: u64,
    pub reason: Vec<String>,
}

impl ResourcePlan {
    pub fn detect(jobs: &[DemoJob], history: &[WorkerSample]) -> Self {
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
        // Parsing is an external streaming process.  The old multiplier made
        // large demos look like they needed several GiB each and left CPU idle.
        let parse_memory = (256 * MIB + source_p75 / 3).clamp(384 * MIB, 2 * GIB);
        // Analysis reads structured NDJSON and normally uses a much smaller
        // working set than the complete export.  Keep a conservative floor,
        // then let the live gate reduce admission if free memory drops.
        let analysis_memory = if parsed_p75 > 0 {
            (256 * MIB + parsed_p75 / 32).clamp(384 * MIB, 1536 * MIB)
        } else {
            512 * MIB
        };

        let cpu_reserve = if logical >= 12 { 2 } else if logical >= 4 { 1 } else { 0 };
        let cpu_budget = logical.saturating_sub(cpu_reserve).max(1);
        let parse_memory_limit = (usable / parse_memory).max(1) as usize;
        let analysis_memory_limit = (usable / analysis_memory).max(1) as usize;
        let count = jobs.len().max(1);
        let parse_ceiling = cpu_budget.min(parse_memory_limit).min(count).max(1);
        let analysis_ceiling = cpu_budget.min(analysis_memory_limit).min(count).max(1);

        // Historical results are retained for benchmark reporting, but they do
        // not cap a fresh run.  That avoids one slow/old configuration keeping
        // a faster computer underutilized.  Runtime admission responds to the
        // currently available RAM before every parser/analyzer job starts.
        let _ = history;

        Self {
            logical_processors: logical,
            total_memory_bytes: total,
            available_memory_bytes: available,
            reserved_memory_bytes: reserve,
            parse_workers: parse_ceiling,
            analysis_workers: analysis_ceiling,
            parse_worker_ceiling: parse_ceiling,
            analysis_worker_ceiling: analysis_ceiling,
            estimated_parse_worker_bytes: parse_memory,
            estimated_analysis_worker_bytes: analysis_memory,
            reason: vec![
                format!("{} logical processors; {} reserved for the OS/UI", logical, cpu_reserve),
                format!("analysis sized from parsed exports; p75={} MiB", parsed_p75 / MIB),
                "worker admission is rechecked from live available memory before every job starts".into(),
                "benchmark history is recorded for comparison but cannot throttle a new hardware plan".into(),
            ],
        }
    }

    pub fn parser_gate(&self, cancelled: Arc<AtomicBool>) -> AdaptiveWorkerGate {
        AdaptiveWorkerGate::new(
            self.parse_worker_ceiling,
            self.estimated_parse_worker_bytes,
            self.reserved_memory_bytes,
            cancelled,
        )
    }

    pub fn analyzer_gate(&self, cancelled: Arc<AtomicBool>) -> AdaptiveWorkerGate {
        AdaptiveWorkerGate::new(
            self.analysis_worker_ceiling,
            self.estimated_analysis_worker_bytes,
            self.reserved_memory_bytes,
            cancelled,
        )
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
    active: Arc<AtomicUsize>,
    current_limit: Arc<AtomicUsize>,
    cancelled: Arc<AtomicBool>,
}

pub struct WorkerPermit {
    active: Arc<AtomicUsize>,
}

impl Drop for WorkerPermit {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

impl AdaptiveWorkerGate {
    fn new(ceiling: usize, worker_memory_bytes: u64, reserved_memory_bytes: u64, cancelled: Arc<AtomicBool>) -> Self {
        Self {
            ceiling: ceiling.max(1),
            worker_memory_bytes: worker_memory_bytes.max(1),
            reserved_memory_bytes,
            active: Arc::new(AtomicUsize::new(0)),
            current_limit: Arc::new(AtomicUsize::new(1)),
            cancelled,
        }
    }

    pub fn acquire(&self) -> Result<WorkerPermit> {
        loop {
            if self.cancelled.load(Ordering::Relaxed) {
                bail!("cancelled");
            }
            let limit = self.live_limit();
            self.current_limit.store(limit, Ordering::Relaxed);
            let active = self.active.load(Ordering::Relaxed);
            if active < limit
                && self
                    .active
                    .compare_exchange(active, active + 1, Ordering::SeqCst, Ordering::Relaxed)
                    .is_ok()
            {
                return Ok(WorkerPermit { active: self.active.clone() });
            }
            thread::sleep(Duration::from_millis(125));
        }
    }

    pub fn active(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }

    pub fn limit(&self) -> usize {
        self.current_limit.load(Ordering::Relaxed)
    }

    fn live_limit(&self) -> usize {
        let mut system = System::new_all();
        system.refresh_memory();
        let available = system.available_memory();
        let reserve = self.reserved_memory_bytes.min(available.saturating_div(2));
        let usable = available.saturating_sub(reserve).max(self.worker_memory_bytes);
        ((usable / self.worker_memory_bytes) as usize).clamp(1, self.ceiling)
    }
}

pub fn largest_first(mut jobs: Vec<DemoJob>, analysis: bool) -> Vec<DemoJob> {
    jobs.sort_by_key(|job| Reverse(if analysis { job.parsed_bytes } else { job.source_bytes }));
    jobs
}

fn guarded_history_choice(kind: &str, hardware_maximum: usize, history: &[WorkerSample], machine: &str) -> usize {
    let mut groups: BTreeMap<usize, Vec<f64>> = BTreeMap::new();
    for sample in history.iter().filter(|sample| {
        sample.kind == kind
            && sample.machine_id == machine
            && sample.succeeded
            && sample.workers > 0
            && sample.workers <= hardware_maximum
            && sample.throughput_mib_s > 0.0
    }) {
        groups.entry(sample.workers).or_default().push(sample.throughput_mib_s);
    }
    let eligible: Vec<_> = groups
        .into_iter()
        .filter(|(_, samples)| samples.len() >= 2)
        .collect();
    if eligible.len() < 2 {
        return hardware_maximum;
    }
    let (best_workers, _) = eligible
        .into_iter()
        .map(|(workers, mut samples)| {
            samples.sort_by(f64::total_cmp);
            let median = samples[samples.len() / 2];
            (workers, median)
        })
        .max_by(|left, right| left.1.total_cmp(&right.1))
        .unwrap();
    best_workers.max((hardware_maximum + 1) / 2).min(hardware_maximum)
}

fn percentile(mut values: Vec<u64>, percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let index = ((values.len() - 1) as f64 * percentile).ceil() as usize;
    values[index.min(values.len() - 1)]
}

pub fn machine_id(logical: usize, total_memory: u64) -> String {
    format!("{}-{}-{}", std::env::consts::OS, logical, total_memory / GIB)
}

pub fn history_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TF2FragDemoHelper")
        .join("rust_benchmark_history.ndjson")
}

pub fn load_history() -> Vec<WorkerSample> {
    fs::read_to_string(history_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_repeated_worker_count_cannot_collapse_plan() {
        let history = (0..5)
            .map(|_| WorkerSample {
                kind: "analysis".into(),
                machine_id: "test".into(),
                workers: 1,
                throughput_mib_s: 130.0,
                succeeded: true,
                ..WorkerSample::default()
            })
            .chain(std::iter::once(WorkerSample {
                kind: "analysis".into(),
                machine_id: "test".into(),
                workers: 8,
                throughput_mib_s: 614.0,
                succeeded: true,
                ..WorkerSample::default()
            }))
            .collect::<Vec<_>>();
        assert_eq!(guarded_history_choice("analysis", 14, &history, "test"), 14);
    }
}
