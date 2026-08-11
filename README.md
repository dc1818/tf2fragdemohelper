# TF2 Frag Demo Helper

TF2 Frag Demo Helper turns a Team Fortress 2 SourceTV (`.dem`) file into searchable JSON and a ranked list of potential frag-movie clips.

It is built for reviewing long competitive or public STV demos without manually scrubbing every round. The project keeps the complete decoded packet stream for detailed analysis while also writing compact event and highlight files for practical clip selection.

## Current capabilities

- Parses TF2 STV demos with the `demostf/parser` codebase.
- Exports the original decoded packet stream as newline-delimited JSON.
- Writes a compact, named game-event stream for deaths, damage, round transitions, class changes, objectives, and other TF2 events.
- Excludes setup, waiting, and post-round deaths from highlight candidates.
- Groups a player's rapid kills into one clip candidate instead of treating each kill as unrelated.
- Reconstructs delta-compressed player and projectile state while parsing, then ranks live-round candidates using confirmed airshots, Über drops, wipes, disadvantage swings, multi-kills, key picks, objective conversions, and random-crit penalties.
- Provides a Windows GUI with drag-and-drop demo selection, export-location selection, progress logging, cancellation, and result-folder opening.
- Includes a candidate browser with score and text filters plus a per-kill view of classes, teams, weapons, tags, clip ticks, and round-state evidence.
- The candidate browser can launch the original demo in TF2 at a selectable lead-in before the first event; double-click a candidate or use **Open selected in TF2**.
- Streams candidate-debug decisions into the embedded parser terminal, including rejected deaths, POV filtering, grouping windows, building events, and score outcomes.

## Packet-state analysis

The exporter applies Source entity baselines and deltas in sequence and writes only changed reconstructed values to `state_samples.ndjson`. The scorer carries those values forward and joins them to exact `player_death` ticks.

The state-backed pass currently adds:

- confirmed airshots only when the victim is off the ground and a matching projectile type, owner weapon handle, removal time, and impact proximity are present;
- direct-proximity, long-flight, and multiple-airshot bonuses based on the reconstructed projectile path;
- confirmed Über drops using `medic_death`, reconstructed charge, and recent deploy events;
- last-player kills/team wipes from reconstructed alive counts, with a minimum four-player state roster to avoid treating every duel as a wipe;
- sequences that erase a two-player-or-greater disadvantage;
- reconstructed health, position, velocity, ground flags, class/team, weapon handles, and projectile paths retained as evidence.

Class-specific direct-versus-splash confidence, reflects, headshot chains, gardens, and trickstabs remain future refinements because they need additional weapon-specific state rules.

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
| `state_samples.ndjson` | Delta-compressed reconstructed player/projectile state with demo and server ticks. |
| `frag_candidates.ndjson` | Ranked live-round clip candidates, their tick ranges, tags, kills, and scoring evidence. |
| `frag_summary.json` | Candidate counts, live-round counts, and analysis limitations. |
| `manifest.json` | Export format and file inventory. |

## Candidate selection rules

The current scorer creates candidates only inside closed live-round intervals. Tournament ready-up events (`teamplay_team_ready`, `teamplay_ready_restart`, and the restart countdown) are retained as evidence but never start an interval. TF2 can emit `teamplay_round_active` during map/warmup initialization, so the scorer accepts that event only after a real round-transition event (`teamplay_round_start`, restart, or ready-restart). The interval then moves to `teamplay_setup_finished` when setup exists and ends at round win, stalemate, game over, or waiting. Each candidate records that start/end evidence under `round_state`.

Candidates are grouped by attacker within the same round only when the entire first-to-last kill span is no more than four seconds. Measuring the total window prevents a chain of individually close kills from becoming one overly long multikill. Every candidate includes a five-second lead-in and three-second outro, clipped to the active round.

The score is intentionally explainable. `frag_candidates.ndjson` records `score_breakdown` plus the raw metrics that produced it. Assists are recorded on kills when present, but do not count as kills and currently add no score.

| Signal | Score |
|---|---:|
| Candidate base | +10 |
| Each kill after the first | +18 |
| Three-kill sequence | +15 |
| Four-or-more-kill sequence | +25 |
| Two or more kills within two seconds | +12 |
| At least one projectile kill | +8 |
| Each Medic killed | +18 |
| Each Demoman killed | +10 |
| Each rocket-jumping victim | +10 |
| Killstreak total of 10+ on a kill | +5 |
| Final kill within eight seconds of round end | +8 |
| Team wins within three seconds after the final kill | +12 |
| Point capture within eight seconds after the final kill | +24 |
| Capture block by the fragging player within two seconds | +20 |
| Payload progress within eight seconds after the final kill | +12 |
| Payload progress pushed by the attacker | +16 |
| Confirmed airshot | +20 |
| Direct-proximity confirmed airshot | +6 |
| Confirmed airshot with at least 0.5 seconds of flight | +5 |
| Two or more confirmed airshots in one sequence | +15 |
| Airborne projectile kill without a matched projectile | +8 |
| Confirmed Über drop, in addition to the Medic pick | +20 |
| Sequence finishes the remaining enemy players | +18 |
| Sequence erases a two-player-or-greater disadvantage | +16 |
| Each random full-crit kill | -12 |

The final score is floored at zero. `metrics.score_before_floor` preserves the pre-floor result so the displayed total can be audited against `score_breakdown`.

Building/object destruction events are not kills and do not create important standalone candidates. A destruction can add a small contextual bonus only when the same attacker produces a real player-kill sequence within two seconds. In a resolved POV demo, the recorded player's own deaths are rejected, and a death where that player appears only as an assister is never counted as a POV kill.

Objective follow-ups are kept as raw event evidence on the candidate. A `teamplay_point_captured` event by the attacker's team within eight seconds of the final kill is the confirmed conversion signal. A `teamplay_capture_blocked` event scores only when its recorded blocker is the fragging player and it occurs within two seconds of the sequence. `payload_pushed` is weaker progress evidence: it scores only when neither a capture nor a confirmed block follows the sequence, and only its first matching event adds score, so repeated cart-progress events cannot inflate a candidate. A team-matched round win within three seconds is separately scored as a clinch. The selected-candidate view shows every matching objective event and the score breakdown records which single outcome was rated.

Every kill records its exact original `player_death` event tick in `event_tick` and `point_of_kill_ticks`. Candidate `point_of_kill_ticks` and clip boundaries use the demo/playback tick you can seek to in TF2; `point_of_kill_server_ticks` and `event_tick` preserve the authoritative server tick used for analysis. The exporter preserves both namespaces because they are not interchangeable. Event records also preserve their source packet sequence and position within that packet, so two legitimate same-tick deaths remain distinguishable without inventing a sub-tick timestamp. Two same-tick deaths are still a valid multikill when they have different victims and event indexes. The exporter classifies the demo as STV, POV, or unknown using the header and `dem_usercmd` packet evidence. STV and unknown demos keep all players' candidates. A POV demo is narrowed to the recorded player when the header nickname matches a decoded player event or the parser's `players.json` userinfo roster; the roster fallback handles POV demos that omit usable `player_connect` events. If neither source resolves the nickname, the result is marked and candidates are not silently labeled POV-only.

After an export completes, use **View candidates** in the GUI. The parser log remains visible in the embedded terminal and can be used to trace every accepted/rejected event. The top filter matches player IDs, classes, teams, weapons, and tags; the selected candidate shows each kill and its round-state evidence. Team fields are populated from decoded `player_team` events when that information is present in the demo. The GUI invokes `analyze_frags.py --debug` automatically.

The selected-candidate view also shows packet-state evidence for each kill: airborne status, projectile match and distance, Über-drop confirmation, and the reconstructed alive-player counts. Older exports without `state_samples.ndjson` remain compatible and simply skip state-backed tags and points.

The candidate viewer reads the original `.dem` path from `manifest.json`. Set **Seconds before first event** (8 seconds by default), then double-click a candidate or press **Open selected in TF2**. The first use asks for `tf.exe`; TF2 is launched with the demo playback tick calculated from the candidate's first event. This uses demo/playback ticks, not the separate server-analysis tick.

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

Pull requests also check the Rust `export_all` binary, run the complete Python test suite, and compile the Windows Forms GUI on a Windows runner.

## Upstream parser

The bundled Rust parser is derived from [demostf/parser](https://github.com/demostf/parser), which is licensed under MIT OR Apache-2.0. See `parser/Cargo.toml` for its upstream attribution and dependency metadata.
