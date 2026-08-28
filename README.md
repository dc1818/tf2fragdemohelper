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
- Batch work keeps the original two-phase barrier (parse every demo, then analyze every export), schedules larger demos first, and offers Low, Medium, and High computer-specific performance ceilings. A live whole-system CPU/RAM governor can throttle below the chosen ceiling when a game, video, or other foreground workload starts; Windows batch work also runs below normal process priority.
- Before parsing starts, the GUI shows estimated parser, analyzer, and total time plus conservative parsed-export size, 20% safety headroom, destination volume, and currently available space. The run is blocked before an export folder is created when the destination lacks the required free space. Successful same-machine timings are retained locally to calibrate later estimates for each performance profile.
- Phase-two state scanning stores compact change-only histories and targeted frag snapshots instead of repeated full state copies. Rayon thread count is sized independently from the number of demos allowed in RAM, so one remaining demo can still use the safe CPU budget.
- Candidate scoring includes the legacy frag identifiers plus bookmark identifiers. The analyzer merges embedded bookmark commands with TF2 Demo Support's same-name `.json` sidecar `events`, so every `ds_mark` creates a candidate, adds its bookmark score, and inherits nearby frag identifiers when present.
- Airshot scoring requires sustained airborne motion before projectile impact. Stationary elevation alone is rejected, and Loose Cannon kills use the victim state before the first cannon collision so the cannon cannot create the airborne state used to score its own double donk or ordinary cannon kill as an airshot.
- Demo context reports POV or STV and classifies RGL/competitive 6v6, Highlander, Valve public, community public, or uncertain modes from config signatures and sustained roster evidence.
- Candidate filtering retains the legacy same-field OR / different-field AND syntax, quoted terms, negation, score threshold, drag selection, Select/Deselect All, details, and recorded/bookmark/mode/type fields.
- HLAE recording is Windows-only. Parsing and candidate browsing are intended to work on Windows, Linux, and macOS.
- Recording launches TF2 with `-insecure` and `+sv_lan 1` for offline demo playback.
- Each recording capture closes the console/GameUI and active selection panels, and suppresses normal, server, and STV chat so menus and messages are not burned into the clip.
- Preview stages a temporary demo and VDM seek script. Automatic recording uses the original POV view, or the established attacker in-eye spectator focus for STV candidates.
- The experimental automatic cinematic-camera selector and planner have been removed. Select exactly one candidate and use `Launch TF2 with HLAE` for a manual MIRV camera session instead.
- Manual HLAE launch stages the selected demo, starts at tick 0, seeks to the current `Before first tick` time in forward jumps capped at 15,000 ticks, automatically pauses there, retains the `After last tick` boundary for naming/reference, and installs temporary number-row (`1` through `=`, including `0` and `-`) camera, kill-tick, campath, and recording hotkeys. Key `3` uses the same staged restart instead of one large backward `demo_gototick`; Right Arrow advances demo time by 0.25 seconds, Up Arrow toggles the HUD, and `4` cycles through every distinct kill tick, including multi-tick candidates.
- Manual captures are written under `Manual HLAE/` in the selected recording directory. `9` and `0` start and stop recording with the selected format, FPS, encoder, and image quality. `=` saves the current campath as `camera_path.xml` in that capture folder.
- Manual sessions use the same isolated recording profile and recovery marker as automatic recording. When TF2 closes, the helper restores the complete original `tf/cfg` folder (including binds and overrides), custom content, HUD, hitsounds, `config.cfg`, `video.txt`, and DX setting. Closing the helper while TF2 is open also closes that launched TF2 process and restores the profile.
- See [MANUAL_HLAE_CAMERA_GUIDE.md](MANUAL_HLAE_CAMERA_GUIDE.md) for the controls, multi-kill framing workflow, keyframes, and recording steps.
- The Candidates page reports recording recovery, the active candidate and batch count, finalization, interrupted-batch consolidation, log archival, and TF2-file restoration in the top header. Record with HLAE remains disabled until that background work finishes.
- Candidates whose recording windows overlap are assigned to separate playback passes of the same demo. Their exact lead-in, kill ticks, outro, identity, and individual output are preserved; the scheduler never shifts a later clip past its frag to force it into one forward-only VDM pass.
- The recorder temporarily installs selected bundled resources and restores the original `tf/custom` content, custom hitsounds, complete `tf/cfg` folder (including `cfg/overrides`), `config.cfg`, `video.txt`, and DX settings after TF2 exits.
- Recording completion uses explicit per-clip and final-batch markers. Process monitoring tolerates transient Windows query failures, confirms TF2 exit across repeated polls, waits for HLAE to flush, then finalizes and indexes every usable capture even when TF2 was closed before the batch finished.
- If recording is interrupted, the manifest marks the batch and unfinished clips as interrupted, completed outputs remain indexed, and the next launch uses the saved recovery marker to restore the original TF2 files.
- Closing the helper while recording, muxing, moving outputs, or recovering an interrupted session requires an additional confirmation. Confirmed shutdowns preserve the session manifest and raw HLAE artifacts for the next launch.
- At startup, retained session manifests are scanned clip-by-clip. Existing finalized outputs are indexed, recoverable HLAE video/audio is muxed and moved into `Videos/`, and recoverable image captures are moved into `Image Sequences/`; unresolved sessions remain available through the Logs page instead of being discarded.
- `MP4 - Standard` is the default recording format and uses H.264 High, 8-bit `yuv420p`, and AAC for DaVinci Resolve and broad editor compatibility. The collapsed Advanced Encoding Options panel is format-aware: MP4 exposes compatibility, chroma/profile, CRF, x264 preset, and AAC controls; AVI preserves the original raw preset while offering FFV1/HuffYUV lossless codecs and valid pixel formats; MOV DNxHR exposes LB/SQ/HQ/HQX/444 profiles with FFmpeg-enforced bit depth/chroma; JPG quality appears only for JPG output. Fixed lossless MP4 and TGA modes keep their existing pipelines.
- Parser/analyzer and HLAE/finalizer logs are kept separately for troubleshooting without opening extra terminal windows.
- A hidden Rust panic is appended to `%LOCALAPPDATA%\TF2FragDemoHelper\crash.log` on Windows with its location and backtrace; adaptive CPU/RAM statistics are included in each export's benchmark summary and batch log.
- Final videos are written to `Videos/`. Native TGA/JPG captures are written to `Image Sequences/<clip>/Frames/` with WAV audio under `Audio/`. Recording manifests, queues, working captures, and finalizer logs stay in `%LOCALAPPDATA%\TF2FragDemoHelper\Recording Sessions\tf2fragdemohelper_batch_<timestamp>/`; a fully successful session is removed automatically after output finalization and TF2 restoration, while failed or interrupted sessions remain available for diagnostics and recovery.
- Format-specific output folders are created only when that format produces a finalized output; unused video or image-sequence folders are not created.
- HLAE pre-flight estimates use the selected candidates' exact clip windows, FPS, resolution, output method, and JPG quality. They include finalized output, the largest simultaneous encoded working capture, audio, metadata, and safety headroom; recording is blocked when the selected output volume cannot safely hold the batch.
- Completed recordings use a demo-content SHA-256 candidate key, a full output fingerprint, and candidate/tick filename fallback. The recorded state therefore survives ordinary video renames and moves after the output has been indexed.
- Choosing to re-record an already completed candidate keeps its original output until the replacement has finalized and been indexed. The old video or image-sequence folder is then removed, leaving the new output as the recorded candidate.

## Figma-to-Slint interface

The Slint interface mirrors the approved TF2-themed Figma frames for Parse Demos,
Candidates, Recording Settings, Logs, and Candidate Details. It uses one shared
component system for the muted gunmetal palette, warm borders, TF2 Build control
labels, hover/pressed/disabled states, fields, checkboxes, tabs, panels, and class
filters.

- The full desktop composition is used above 1050 logical pixels wide.
- At 1050 pixels and below, every page switches to its dedicated compact frame.
- The supported minimum window size is 900x520; compact screens reflow instead of
  scaling the desktop canvas.
- The compact header's Menu control exposes every page without consuming the
  limited-height layout with a permanent tab row.
- Parse accepts individual demo selection, recursive folder selection, and native
  operating-system file/folder drops.
- The Candidates page retains advanced query syntax while exposing direct map,
  class, server-type, recorded-state, and minimum-score controls. Official TF2
  leaderboard class icons use the canonical Scout, Soldier, Pyro, Demoman, Heavy,
  Engineer, Medic, Sniper, and Spy order.
- Select All and Deselect All are two states of one control. RED identifies record
  and parse actions, BLU identifies preview/estimate actions, and gold identifies
  selection or file-location utilities.

The responsive visual source is the project Figma file:
https://www.figma.com/design/Yr10mYuw4jcnQCMKuTPCH3

## Source layout

- `app/` — Slint UI plus Rust analysis, filtering, scheduling, recording, and settings code.
- `app/ui/tf2-theme.slint` — shared Figma-derived palette and interactive TF2 controls.
- `app/ui/assets/` — bundled typography and canonical TF2 class-filter images.
- `parser/` — Rust demo-parser library and the single `export_all` helper binary.
- `recording_resources_archive/` — split resource archive retained beside the built GUI.
- `.github/workflows/build.yml` — Windows, Linux, and macOS checks plus the Windows release artifact.

The parser library is derived from `demostf/parser` and remains MIT OR Apache-2.0 licensed. Bundled recording assets retain their upstream terms; see `THIRD_PARTY_NOTICES.md`.
