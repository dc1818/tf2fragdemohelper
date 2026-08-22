# TF2 Frag Demo Helper

Rust desktop application for parsing TF2 `.dem` files, ranking clip candidates, and launching offline HLAE recording sessions. The GUI is built with Slint. Python, .NET, and Windows Forms are not required.

## Build

Install [Rust 1.88 or newer](https://rustup.rs/), then run:

- Windows: `BUILD_RUST_APP.bat`
- Linux/macOS: `./build_rust_app.sh`

The release package is created under `dist/` with:

- `TF2_Frag_Demo_Helper` — Slint GUI and Rust application logic.
- `export_all` — Rust TF2 demo decoder used by the GUI.
- `recording_resources_archive/` — required VPKs, recording HUDs, and skyboxes.

Keep those three items together. On Windows the binaries have `.exe` extensions.

## Runtime notes

- Parsing, analysis, filtering, candidate browsing, adaptive concurrency, and recording identification are implemented in Rust.
- HLAE recording is Windows-only. Parsing and candidate browsing are intended to work on Windows, Linux, and macOS.
- Recording launches TF2 with `-insecure` and `+sv_lan 1` for offline demo playback.
- The recorder temporarily installs selected bundled resources and restores the original `tf/custom` content and recording CFG after TF2 exits.
- If recording is interrupted, the next launch uses the saved recovery marker to restore the original TF2 files.
- Final videos are written to `Videos/`. Native TGA/JPG captures are written to `Image Sequences/<clip>/Frames/` with WAV audio under `Audio/`; manifests, queues, working captures, and finalizer logs remain under `Recording Metadata/tf2fragdemohelper_batch_<timestamp>/`.
- Completed recordings use a demo-content SHA-256 candidate key, a full output fingerprint, and candidate/tick filename fallback. The recorded state therefore survives ordinary video renames and moves after the output has been indexed.

## Source layout

- `app/` — Slint UI plus Rust analysis, filtering, scheduling, recording, and settings code.
- `parser/` — Rust demo-parser library and the single `export_all` helper binary.
- `recording_resources_archive/` — split resource archive retained beside the built GUI.
- `.github/workflows/build.yml` — Windows, Linux, and macOS checks plus the Windows release artifact.

The parser library is derived from `demostf/parser` and remains MIT OR Apache-2.0 licensed. Bundled recording assets retain their upstream terms; see `THIRD_PARTY_NOTICES.md`.
