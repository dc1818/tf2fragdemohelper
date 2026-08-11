# TF2 Frag Demo Helper

TF2 Frag Demo Helper turns a Team Fortress 2 SourceTV (`.dem`) file into searchable JSON and a ranked list of potential frag-movie clips.

It is built for reviewing long competitive or public STV demos without manually scrubbing every round. The project keeps the complete decoded packet stream for detailed analysis while also writing compact event and highlight files for practical clip selection.

## Current capabilities

- Parses TF2 STV demos with the `demostf/parser` codebase.
- Exports the original decoded packet stream as newline-delimited JSON.
- Writes a compact, named game-event stream for deaths, damage, round transitions, class changes, objectives, and other TF2 events.
- Excludes setup, waiting, and post-round deaths from highlight candidates.
- Groups a player's rapid kills into one clip candidate instead of treating each kill as unrelated.
- Ranks live-round candidates using multi-kills, rapid sequences, projectile kills, Medic picks, killstreaks, late-round timing, and random-crit penalties.
- Provides a Windows GUI with drag-and-drop demo selection, export-location selection, progress logging, cancellation, and result-folder opening.

## Planned analysis passes

The initial scorer uses authoritative game events only. It deliberately does not label ordinary projectile kills as airshots.

The next packet-state pass will reconstruct player and projectile state from `packets.ndjson` to add:

- confirmed airshots and double-airshots;
- projectile flight time, range, and direct-versus-splash confidence;
- target vertical/lateral motion and airtime;
- player health, conditions, weapon state, and local outnumbering;
- Medic charge drops, objective proximity, wipes, and advantage swings;
- class-specific scoring for pipes, directs, reflects, headshots, gardens, trickstabs, crossbow shots, and more.

## Build and run on Windows

1. Install [Rust](https://rustup.rs/) with Cargo.
2. Install [Python 3](https://www.python.org/downloads/) and enable the Python PATH option.
3. Run `Build_Parser_GUI.bat`.
4. Open `TF2_STV_Parser_GUI.exe`.
5. Select or drag in a `.dem` file, choose an export location, then select **Parse STV demo**.

The first build compiles `parser/src/bin/export_all.rs` into `parser/target/release/export_all.exe` and compiles the Windows Forms GUI.

## Export layout

Each run creates a timestamped folder beside the selected output location.

| File | Purpose |
|---|---|
| `header.json` | Demo header and match metadata. |
| `packets.ndjson` | Complete decoded top-level packet stream; retained as the source for future state reconstruction. |
| `packet_index.ndjson` | Packet sequence, demo tick, type, and original bit range. |
| `events.ndjson` | Compact decoded game-event records for analysis. |
| `frag_candidates.ndjson` | Ranked live-round clip candidates, their tick ranges, tags, kills, and scoring evidence. |
| `frag_summary.json` | Candidate counts, live-round counts, and analysis limitations. |
| `manifest.json` | Export format and file inventory. |

## Candidate selection rules

The current scorer creates candidates only inside closed live-round intervals. Tournament ready-up events (`teamplay_team_ready`, `teamplay_ready_restart`, and the restart countdown) are retained as evidence but never start an interval. The interval starts at `teamplay_round_active`, moves to `teamplay_setup_finished` when setup exists, and ends at round win, stalemate, game over, or restart. Each candidate records that start/end evidence under `round_state`.

Candidates are grouped by attacker within the same round when consecutive kills are no more than four seconds apart. Every candidate includes a five-second lead-in and three-second outro, clipped to the active round.

The score is intentionally explainable. `frag_candidates.ndjson` records the tags and raw metrics that produced it, including the kill count, duration, weapons, projectile kills, Medic kills, and full-crit count.

## Project structure

```text
parser/                 Rust TF2 demo parser and export_all binary source
gui/Program.cs          Windows parser and frag-analysis GUI
analyze_frags.py        Event-based live-round candidate scorer
Build_Parser_GUI.bat    Builds the parser and GUI
Build_Parser_Only.bat   Builds export_all without the GUI
```

## Validation

`tests/fixture_events.ndjson` is a small controlled event stream used to verify that the analyzer keeps a live-round multi-kill and excludes ready-up, countdown, pre-round, and post-round deaths. Run `python -m unittest tests/test_round_state.py` from the project root to verify the round-state rules.

## Upstream parser

The bundled Rust parser is derived from [demostf/parser](https://github.com/demostf/parser), which is licensed under MIT OR Apache-2.0. See `parser/Cargo.toml` for its upstream attribution and dependency metadata.
