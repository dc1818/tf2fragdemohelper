# Rust + Slint migration (v34)

The primary application is now a Rust workspace. `app/` contains the Slint desktop application and Rust analysis/recording code; `parser/` remains the Rust TF2 demo decoder. The older Python and Windows Forms files are retained temporarily as migration references, but the v34 executable does not launch Python or .NET.

## Benchmark-driven concurrency changes

The supplied 148-demo benchmark parsed 6.58 GiB into 195.6 GB. Parsing took 206.3 seconds with eight parser workers, while analysis took 1,405.8 seconds with one worker. Average sampled system CPU use was only 20.4%. Historical measurements in the same benchmark also showed 614.2 MiB/s with eight analyzers and 785.5 MiB/s with fourteen, compared with 132.7 MiB/s in the one-analyzer run.

The v34 scheduler therefore:

- reserves CPU and currently available RAM for the OS and Slint UI;
- caps parser concurrency separately because full packet/state export is write-heavy;
- orders both phases largest-first to reduce the long tail;
- replans phase 2 after the actual parsed-export sizes are known;
- streams `state_samples.ndjson`, retaining only death-tick and periodic roster snapshots;
- requires at least two different worker counts with at least two comparable successful samples each before history can override the hardware plan;
- refuses to cut a hardware-derived plan by more than 50% from historical tuning alone.

That last rule fixes the prior Auto mode bug: repeated one-worker samples were the only worker count meeting the sample threshold, so the old selector incorrectly treated one worker as a measured winner.

## Platform support

Parsing, Rust analysis, candidate filtering/viewing, benchmark history, and export management are designed for Windows, Linux, and macOS. TF2 preview depends on a locally installed native TF2 client. HLAE recording remains Windows-only because HLAE/AfxHookSource is Windows-specific.

## Preserved feature surface

- Two-phase multi-demo parsing and analysis with cancellation and combined candidates.
- Live-round gating, multi-kill grouping, state-backed scoring, bookmark candidates and bookmark score identifiers.
- POV/STV classification and tolerant RGL/6v6/Highlander/public mode labels.
- Candidate filtering, minimum score, multi-selection, recorded status, details, and non-sortable headers.
- Recorded-video reconciliation by content fingerprint, so renaming a finished video does not clear its status.
- Offline-only TF2 preview and HLAE launch (`-insecure`, `+sv_lan 1`, blocked `connect`/`retry`).
- Lead/outro, FPS, image/encoded output choices, JPG quality, graphics/HUD/viewmodel/sound settings, VDM sequencing, STV player focus, and final-demo `quit`.

The Slint layout is intentionally plain so it can be restyled without changing application logic.

## Build

Install stable Rust, then run `BUILD_RUST_APP.bat` on Windows or `./build_rust_app.sh` on Linux/macOS. Both binaries are copied to `dist/`; `export_all` must remain beside the GUI executable.
