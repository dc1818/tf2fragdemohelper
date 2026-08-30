# TF2 MIRV Director: implementation and recording game plan

## Decision

Build TF2 MIRV Director as a separate executable in this repository, launched by TF2 Frag Demo Helper for one selected candidate. Do not embed it in the main helper window and do not inject its UI into the recorded game frame.

This keeps the responsibilities clean:

| Component | Owns |
|---|---|
| TF2 Frag Demo Helper | Candidate selection, clip window, safe staged seeking, recording profile, capture output, recovery, and restoration |
| TF2 MIRV Director | Candidate timeline, shot cues, campath key list/editor, preview controls, and warnings |
| HLAE Source 1 bridge | Local camera/demo/campath telemetry and a small allowlist of commands |
| TF2 | Demo playback and final rendered camera |

## What was learned from HOT

[HLAE Observer Tools (HOT)](https://github.com/papesgit/hot) is a CS2 observer desk that uses a custom HLAE build, WebSocket/UDP connections, game-state integration, a viewport, and campath/sequencer tools. Its current releases show the useful interaction model: a separate control surface, a cue timeline, visual campath editing, and live game feedback.

HOT is not directly reusable for this project:

- HOT targets Source 2 / CS2, while TF2 uses Source 1 `AfxHookSource`.
- HOT explicitly requires its custom HLAE; official HLAE is not sufficient for its remote features.
- HOT versions from v0.2.3 use a source-available license that prohibits redistribution. No current HOT application code is copied into this project.
- The older GPL release would impose GPL terms on a derivative and is still tied to CS2 systems.

The custom HLAE fork is MIT-licensed. Its architectural ideas—a loopback transport, typed actions, state snapshots, and notifications—can be independently ported to Source 1 with attribution. The actual Source 2 entity, renderer, and camera code cannot simply be moved to TF2.

Official HLAE already confirms that Source 1 supports smooth campaths and paused free-camera movement, and that POV demos need third-person plus `mirv_input camera`: [AfxHookSource](https://github.com/advancedfx/advancedfx/wiki/AfxHookSource), [mirv_campath](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_campath), and [mirv_input](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_input).

## Recording workflow

1. Select one candidate and set the before/after window in TF2 Frag Demo Helper.
2. Launch the manual MIRV session.
3. The helper stages the demo, writes `director_session.json`, starts HLAE, safely seeks in steps no larger than 15,000 ticks, pauses, and opens Director.
4. Director opens in Option C: a click-through live timeline strip docked across the monitor's top edge plus an interactive cue card docked beneath it at the right edge. Temporary per-tick VDM markers drive the current-tick playhead. The saved `Y` shortcut hides or restores only the right card.
5. In TF2, enter the MIRV camera and compose an establishing frame. Add the first key.
6. Advance time in small increments or move to the next cue. Reframe and add a key for each intentional camera beat. Simultaneous victims remain one cue and can be framed together.
7. Use safe restart, enable campath playback, and preview the complete move. Director remains visible beside a smaller TF2 window as the shot checklist.
8. Revise keys. HLAE recommends roughly 1–2 seconds between keys and warns that crowded keys or sudden reversals can produce sharp curves.
9. Save `camera_path.xml`, safely restart, resume, start recording, and stop after the outro.
10. Close TF2. The helper finalizes the existing output pipeline, restores the original TF2 profile, and closes Director.

## Implemented in the first branch

- Separate `TF2_MIRV_Director.exe` built and packaged with the helper.
- Versioned JSON session contract shared by the helper and Director.
- Candidate-aware cue timeline with per-tick tags and optional victims.
- Whole-candidate tags and exact campath/output locations.
- Option C split overlay: a resolution- and DPI-aware full-width top timeline strip and a separate right-docked cue card.
- Real current-tick progress from an engine-scheduled VDM marker on every playback tick in the existing TF2 console log.
- Reasserted Windows `HWND_TOPMOST` state so clicking TF2 does not bury the overlay.
- Click-through strip plus an interactive card, preserving TF2 input outside the card.
- Configurable hide/show-card shortcut in Recording Settings (`Y` by default), available while TF2 has focus.
- Saved shortcut grid, arrow-key reservation notice, keyframe checklist, and custom-key recording order.
- Automatic launch and lifecycle tied to the manual TF2 session.
- Typed, allowlisted bridge messages for demo state, small skips, camera pose, keyframe add/replace/remove, campath enable/draw, and save/load.
- No arrow-key capture and no replacement of the existing temporary MIRV binds.

## Live bridge milestone

The VDM/log telemetry is sufficient for the live clip timeline and does not require another hook. It emits one engine-scheduled marker per demo tick inside the selected clip. Stock TF2 still does not offer the richer remote-control path used by the CS2 HOT setup, so camera-pose and keyframe editing remain a separate milestone.

The robust implementation is a small MIT-compatible extension in a separate `advancedfx` fork:

- Add a loopback-only WebSocket or named-pipe server to `AfxHookSource`.
- Default to `127.0.0.1`; require a random per-session token.
- Expose only the `BridgeRequest` allowlist defined by `tf2-mirv-director`.
- Publish demo tick/time, pause state, current camera pose/FOV/roll, campath enabled state, and keyframes.
- Marshal every engine mutation onto the game thread.
- Keep safe restart in the helper; never perform a large backward `demo_gototick` through the bridge.
- Ship the custom hook as a clearly versioned optional component with AdvancedFX MIT attribution.

Once connected, the same Director UI can add these live features:

- Click a cue to move forward safely to that tick.
- Add, replace, delete, and label keys without opening the console.
- Display the current camera pose and key spacing.
- Draw and preview the path from the Director.
- Warn about fewer than four keys, keys closer than one second, abrupt direction changes, first-person/viewmodel state, or a cue outside the path.
- Keep recording start/stop delegated to the helper so output and restoration behavior do not diverge.

## Later improvements

- A 2D map overview based on parsed map metadata and only positions known to be network-valid.
- Camera/key attachment presets after TF2 entity behavior is verified.
- Curve editing after the official Source 1 interpolation semantics are represented exactly.
- Undo/redo and alternate shot versions stored beside each candidate.

Multi-camera extensions should remain out of the initial bridge. HOT notes that its multi-camera/curve paths are incompatible with official HLAE; this project should first preserve portable `mirv_campath` XML files.
