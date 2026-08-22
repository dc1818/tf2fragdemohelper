use crate::models::{DemoJob, WorkerSample};
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::BTreeMap, fs, path::PathBuf, thread};
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
        let reserve = (total / 10).max(2 * GIB).min(available.saturating_div(2));
        let usable = available.saturating_sub(reserve).max(GIB);

        let source_p75 = percentile(jobs.iter().map(|job| job.source_bytes).collect(), 0.75).max(64 * MIB);
        let parsed_p75 = percentile(
            jobs.iter().filter_map(|job| (job.parsed_bytes > 0).then_some(job.parsed_bytes)).collect(),
            0.75,
        );
        let parse_memory = (512 * MIB + source_p75.saturating_mul(2)).clamp(768 * MIB, 8 * GIB);
        // Rust streams event/state NDJSON. Keep enough room for indexes while
        // sizing from the data Phase 2 actually reads rather than the .dem.
        let analysis_memory = if parsed_p75 > 0 {
            (384 * MIB + parsed_p75 / 6).clamp(512 * MIB, 4 * GIB)
        } else {
            GIB
        };

        let cpu_reserve = if logical >= 12 { 2 } else if logical >= 4 { 1 } else { 0 };
        let cpu_budget = logical.saturating_sub(cpu_reserve).max(1);
        let parse_memory_limit = (usable / parse_memory).max(1) as usize;
        let analysis_memory_limit = (usable / analysis_memory).max(1) as usize;
        let parse_io_limit = ((logical as f64).sqrt() * 2.0).ceil() as usize;
        let mut parse_workers = cpu_budget.min(parse_memory_limit).min(parse_io_limit.max(2));
        let mut analysis_workers = cpu_budget.min(analysis_memory_limit);

        let machine = machine_id(logical, total);
        parse_workers = guarded_history_choice("parse", parse_workers, history, &machine);
        analysis_workers = guarded_history_choice("analysis", analysis_workers, history, &machine);
        let count = jobs.len().max(1);
        parse_workers = parse_workers.min(count).max(1);
        analysis_workers = analysis_workers.min(count).max(1);

        Self {
            logical_processors: logical,
            total_memory_bytes: total,
            available_memory_bytes: available,
            reserved_memory_bytes: reserve,
            parse_workers,
            analysis_workers,
            estimated_parse_worker_bytes: parse_memory,
            estimated_analysis_worker_bytes: analysis_memory,
            reason: vec![
                format!("{} logical processors; {} reserved for the OS/UI", logical, cpu_reserve),
                format!("analysis sized from parsed exports; p75={} MiB", parsed_p75 / MIB),
                "history can tune only when at least two worker counts each have two successful comparable runs".into(),
                "a historical recommendation may not cut the hardware plan by more than 50% without repeated evidence".into(),
            ],
        }
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

