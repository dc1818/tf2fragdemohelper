# Benchmark, disk-estimation, and ETA data

Every parse run now records machine/resource/test data so batch performance can be analyzed instead of guessed.

## Where the files are written

For every export, the program creates:

```text
<export root>/benchmark/
```

A multi-demo batch therefore looks like:

```text
tf2_demo_batch_export_YYYYMMDD_HHMMSS/
├─ 001_..._export/
├─ 002_..._export/
├─ ...
├─ benchmark/
│  ├─ benchmark_summary.json
│  ├─ parse_metrics.csv
│  ├─ analysis_metrics.csv
│  ├─ resource_samples.csv
│  ├─ eta_samples.csv
│  ├─ failures.csv
│  ├─ batch_run.log
│  ├─ benchmark_history_before_run.ndjson   (when prior history exists)
│  └─ benchmark_history_after_run.ndjson
└─ frag_candidates.ndjson
```

The persistent calibration history is also kept at:

```text
%LOCALAPPDATA%\TF2FragDemoHelper\benchmark_history.ndjson
```

That file lets future runs learn from earlier runs on the computer. It is not required for parsing; deleting it simply resets the estimates to conservative cold-start defaults.

## Preflight disk estimate

Before a multi-demo batch starts, the GUI shows:

- total source-demo size;
- estimated full parse-output size;
- estimated candidate-analysis additions;
- safety headroom;
- recommended free disk space;
- current free disk space;
- automatic parser-worker count;
- automatic analyzer-worker count;
- historical ETA when enough prior benchmark samples exist.

The first runs use conservative defaults. Once successful runs exist, the parser uses the 90th percentile of observed output expansion with extra safety margin. During the current batch it can also raise the per-demo reservation immediately if completed demos are expanding more than the original estimate.

A live disk gate checks the output drive before each parser/analyzer is allowed to start. It reserves expected output space for workers already running and keeps additional headroom available. If the drive can no longer safely fit another job, the batch stops before launching it rather than intentionally running the drive to zero.

## ETA

ETA is weighted by data size rather than treating every demo as equal.

- Phase 1 weights work by `.dem` input bytes.
- Phase 2 weights work by the actual parsed-export bytes produced in phase 1.
- On a new machine, ETA says `calibrating` until enough work completes.
- On later runs, median historical seconds/GiB provides an initial estimate before the first job completes.
- The estimate is continuously replaced by live wall-clock throughput as jobs finish.

The status line shows both time remaining and an approximate local completion clock time.

## `parse_metrics.csv`

One row per successfully parsed demo.

Columns:

- `timestamp_utc`
- `order`
- `demo`
- `input_bytes`
- `parse_output_bytes`
- `output_ratio` — parsed bytes / `.dem` bytes
- `wall_seconds`
- `process_cpu_seconds`
- `peak_working_set_bytes`
- `input_mib_per_sec`
- `output_mib_per_sec`
- `worker_limit`

This is the primary file for comparing parser performance, output expansion, and parser RAM usage.

## `analysis_metrics.csv`

One row per successfully analyzed demo.

Columns:

- `timestamp_utc`
- `order`
- `demo`
- `analysis_input_bytes` — size of that demo's parsed export before analysis
- `analysis_added_output_bytes`
- `added_output_ratio` — analyzer-added bytes / source `.dem` bytes
- `wall_seconds`
- `process_cpu_seconds`
- `peak_working_set_bytes`
- `input_mib_per_sec`
- `candidate_count`
- `worker_limit`

This file is useful when deciding how much candidate analysis costs separately from parsing.

## `resource_samples.csv`

A roughly one-second system sample while a batch is running.

Columns:

- `timestamp_utc`
- `elapsed_seconds`
- `phase`
- `completed`
- `total`
- `worker_limit`
- `active_processes`
- `cpu_percent`
- `available_ram_bytes`
- `free_disk_bytes`

Use this file to answer questions such as:

- Was CPU saturated?
- Did free RAM collapse as worker count increased?
- Was the disk filling faster than predicted?
- Was the worker ceiling too low or too aggressive?

## `eta_samples.csv`

Every ETA update produced when a demo completes.

Columns:

- `timestamp_utc`
- `phase`
- `completed`
- `total`
- `fraction` — byte-weighted phase fraction
- `elapsed_seconds`
- `remaining_seconds`
- `estimated_completion_local`

This makes it possible to measure how accurate the ETA becomes over the course of a batch.

## `failures.csv`

Contains parser/analyzer failures even if the batch aborts.

Columns:

- `timestamp_utc`
- `phase`
- `order`
- `demo`
- `message`

Partial benchmark files remain available after cancellation or failure.

## `batch_run.log`

A plain-text copy of the parser GUI log, including prefixed concurrent parser/analyzer output. This is useful for diagnosing a failed demo alongside `failures.csv`.

## `benchmark_summary.json`

Machine and whole-run summary, including:

- operating system;
- 32/64-bit process information;
- processor identifier when Windows exposes it;
- logical CPU count;
- total and initially available RAM;
- parser/analyzer worker limits;
- estimated per-worker RAM;
- initial/free/final/minimum disk space;
- disk preflight ratios and historical sample counts;
- phase wall times;
- total child-process CPU time;
- max observed child-process working set;
- average sampled system CPU usage;
- minimum available RAM;
- aggregate parser source/output MiB/s;
- aggregate analyzer input MiB/s;
- candidate count;
- success/cancel/failure state.

This is usually the easiest single file to compare between computers or between scheduler versions.

## Calibration history

`benchmark_history.ndjson` contains compact per-demo and per-batch measurements. Future resource planning currently learns from it in three ways:

1. Full-export disk expansion ratio.
2. Candidate-analysis output expansion ratio.
3. Observed parser/analyzer peak working-set memory.
4. Historical seconds/GiB for initial ETA.
5. Same-machine worker-count throughput. Once there are enough successful batches at at least two different concurrency levels, Auto can prefer the measured throughput winner while still obeying current CPU/RAM/I/O limits.

The existing CPU/RAM-aware scheduler still enforces a live CPU ceiling and live free-RAM reserve. The benchmark history makes the memory, storage, ETA, and eventually concurrency assumptions increasingly specific to the real workload instead of relying only on generic defaults.

## Comparing computers or settings

For repeatable testing, use the same representative set of demos and compare `benchmark_summary.json` plus the two metrics CSV files. Useful values are:

- `phase1_parse_wall_seconds`
- `phase2_analysis_wall_seconds`
- `parse_source_mib_per_sec_wall`
- `parse_output_mib_per_sec_wall`
- `analysis_input_mib_per_sec_wall`
- `average_sampled_system_cpu_percent`
- `minimum_available_ram_bytes`
- `minimum_free_disk_bytes`
- `max_parse_peak_working_set_bytes`
- `max_analysis_peak_working_set_bytes`
- `parse_workers`
- `analysis_workers`

A faster machine should normally achieve higher aggregate throughput, but the resource samples reveal whether CPU, RAM, or the output drive became the actual bottleneck.

## v32.3 candidate-analysis profiling

Each analyzed export now also contains `analysis_profile.json`. The same fields are copied into the batch-level `benchmark/analysis_metrics.csv` when available.

Important fields:

- `capture_type` and `analysis_scope` (`pov_player_only` vs `all_players`)
- `total_player_death_events`
- `accepted_live_scope_kills`
- `death_rejections.outside_live_round`
- `death_rejections.not_pov_attacker`
- `state_lookup_count`
- `unindexed_state_lookup_count`
- `projectile_tracks_total`
- `projectile_tracks_examined`
- `stage_seconds.read_and_index_state`
- `stage_seconds.early_death_gating`
- `stage_seconds.state_enrichment`
- `stage_seconds.candidate_grouping_and_scoring`
- `total_seconds`

Normal GUI batch analysis no longer enables verbose `--debug` output. The analyzer still supports `--debug` when run manually for a specific export, but batch runs avoid printing one log line for every intentionally rejected warmup/post-round/non-POV death.

### Reference vs optimized benchmark

To benchmark real parsed exports against the retained v32.2 analyzer:

```text
python tools\benchmark_candidate_analyzer.py "C:\path\to\001_demo_export" "C:\path\to\002_demo_export"
```

Optional item schema:

```text
python tools\benchmark_candidate_analyzer.py --item-schema "C:\...\tf\scripts\items\items_game.txt" "C:\path\to\001_demo_export"
```

Results are written under `benchmark_results` as CSV and JSON. The benchmark records old/new wall time, speedup, POV/STV scope, live-round/scope rejection counts, state/projectile lookup counts, and whether the final `frag_candidates.ndjson` output is byte-identical.
