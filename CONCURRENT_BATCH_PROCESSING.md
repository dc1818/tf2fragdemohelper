# Concurrent two-phase batch processing

This build preserves the Rust parser outputs and Python candidate-scoring semantics while improving orchestration and indexed analysis. Candidate filtering and HLAE queue finalization also include correctness fixes described in the main README.

## Flow

1. Build every per-demo export job.
2. **Phase 1:** run multiple `export_all.exe` parser jobs concurrently.
3. Wait for **all** parser jobs to complete.
4. **Phase 2:** run multiple `analyze_frags.py` jobs concurrently.
5. Wait for **all** analyzers to complete.
6. Combine `frag_candidates.ndjson` files into the batch candidate list.

There is a hard barrier between phases. Candidate analysis never starts while another demo is still parsing.

## Automatic worker planning

`ResourcePlan` uses:

- `Environment.ProcessorCount` for logical CPU capacity.
- `GlobalMemoryStatusEx` for currently available physical RAM.
- the 75th-percentile size of the selected `.dem` files to estimate working-set pressure.
- a scalable parse I/O ceiling because full `packets.ndjson` / `state_samples.ndjson` exports are write-heavy.

The application reserves RAM for Windows and other applications. `AdaptiveResourceGate` rechecks free physical RAM and total system CPU usage before each job and can temporarily delay another parser/analyzer launch if memory pressure rises or CPU usage is already near 95% after the batch started. It always allows at least one job so low-memory readings cannot deadlock a batch.

## Cancellation

The previous GUI tracked one `activeProcess`. Concurrent execution now tracks a thread-safe set of all active parser/analyzer child processes. Cancel:

- cancels queued jobs,
- kills all currently active child workers, and
- leaves completed export directories on disk.

As before, a real parser/analyzer failure aborts the batch rather than silently combining a partial result.

## Logging / progress

Concurrent output is prefixed, for example:

```text
[PARSE 003] ...
[PARSE 007] ...
[ANALYZE 003] ...
```

Progress is split 50/50 between the parse and analysis phases.

## Next backend step

The Python analyzer is intentionally still present in this intermediate build. The next major optimization is to port `analyze_frags.py` to Rust and feed candidate evidence directly from the in-memory parser/game-state reconstruction while still writing every existing detailed export file.

## v32 disk preflight, ETA, and benchmark telemetry

Before a batch begins, the GUI estimates the full parsed-export footprint and the small amount of additional candidate-analysis output. It compares this with the free space on the selected output drive and includes a safety reserve. Historical expansion ratios are used once available; otherwise a deliberately conservative cold-start ratio is used.

A live `AdaptiveDiskGate` reserves estimated output for concurrently running workers. The estimate can increase during the current batch when completed demos demonstrate a higher expansion ratio than expected. This is designed to stop a new job before the drive is exhausted instead of allowing Windows/Rust `os error 112` to be the first warning.

Phase progress now includes a byte-weighted ETA. The first run calibrates from completed jobs, while later runs can use historical seconds/GiB immediately.

Each export contains a `benchmark` directory with CSV/JSON telemetry. See `BENCHMARK_DATA.md` for the complete schema. A compact history is persisted under `%LOCALAPPDATA%\TF2FragDemoHelper\benchmark_history.ndjson` so disk, RAM, and ETA assumptions become more accurate over subsequent runs.
