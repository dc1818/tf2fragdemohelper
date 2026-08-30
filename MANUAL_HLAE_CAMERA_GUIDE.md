# Manual HLAE / MIRV Camera Guide

This workflow opens one selected candidate at the chosen lead-in and lets you build and record a manual MIRV campath. It supports candidates with kills on multiple ticks and simultaneous or nearby victims that are easier to frame by eye.

## Launch the candidate

1. Open the Candidates page and select exactly one candidate.
2. Set `Before first tick` to the amount of setup time you want.
3. Set `After last tick` for the end-of-clip reference.
4. Choose the recording format, FPS, resolution, encoder, HUD, and other recording settings.
5. Click `Launch TF2 with HLAE`.
6. The staged demo starts at tick 0 and advances to the selected lead-in in jumps no larger than 15,000 ticks.
7. The demo automatically pauses at the lead-in. Wait for `TF2FRAG_MANUAL_READY` and `TF2FRAG_MANUAL_PAUSED_AT_START` in the console.

The launcher uses offline mode (`-insecure`, `sv_lan 1`) and the same temporary recording-profile backup and restoration system as automatic recording. Do not join a live server from this TF2 instance.

## Temporary shortcuts

These binds exist only in the isolated manual session. The helper does not bind the arrow keys because MIRV camera control uses them. Your original `tf/cfg` folder and bindings are restored after TF2 closes.

| Key | Action |
|---|---|
| `[` | Advance demo time by approximately 0.25 seconds |
| `]` | Toggle the entire HUD off or on |
| `1` | Print the temporary shortcut reminder in the console |
| `2` | Move demo time back approximately one second |
| `3` | Safely restart at tick 0, advance in steps of at most 15,000 ticks, and pause at the selected lead-in |
| `4` | Cycle to each distinct kill tick in the candidate |
| `5` | Pause or resume the demo |
| `6` | Establish third-person/no-viewmodel state and enter the manual MIRV camera |
| `7` | Add a campath keyframe at the current demo time and view |
| `8` | End manual input, request third-person/no-viewmodel state, and enable campath playback |
| `9` | Start recording with the selected format and FPS |
| `0` | Stop recording and reset `host_framerate` |
| `-` | Print all campath keyframes in the console |
| `=` | Save the path as `camera_path.xml` in the capture folder |

While `mirv_input camera` is active, use the normal MIRV camera controls to position the shot. Manual input is suspended while the console is open.

## Fix POV weapons or a missing world model

In a POV demo, the campath can move correctly while TF2 still renders only the POV player's first-person weapon. Before placing keyframes, open the console and enter:

```text
sv_cheats 1
thirdperson
r_drawviewmodel 0
```

`thirdperson` makes TF2 render the POV player as a world model. `r_drawviewmodel 0` removes the first-person weapon.

There is one user-confirmed TF2 quirk to account for: leaving manual MIRV input can return a POV demo to first person even though shortcut `8` requests third person. After finishing the keyframes:

1. Press Escape if necessary.
2. Press `8`.
3. Open the console and explicitly enter:

```text
thirdperson
```

If the POV weapon is still visible, also enter:

```text
r_drawviewmodel 0
```

The desired playback state is:

```text
mirv_input end
thirdperson
r_drawviewmodel 0
mirv_campath enabled 1
```

Do not switch to `firstperson` while previewing or recording the cinematic path.

## Build and preview a keyframed shot

1. The demo begins paused at the selected lead-in.
2. Apply the POV fix above, then press `6` to enter the manual camera.
3. Position the opening view and press `7`.
4. Advance time with `[`, or enter one of the `mirv_skip` commands below.
5. Reposition the camera and press `7` again. A normal smooth campath should have at least four keyframes.
6. Repeat for every important moment. Use `4` to visit each distinct kill tick.
7. For a simultaneous kill, compose one view containing both victims. For kills a few ticks apart, place a key near each victim and leave enough time for a readable transition.
8. Press `-` to inspect the path. Remove a bad key with `mirv_campath remove <id>`.
9. Press Escape if necessary, then press `8`.
10. Open the console and explicitly enter `thirdperson`. Enter `r_drawviewmodel 0` too if the POV weapon remains visible.
11. Press `3` to safely reload from tick 0 and return to the lead-in. The in-memory campath is retained, the helper reapplies third-person/no-viewmodel state after the reload, and the demo pauses again.
12. Press `5` to preview the path.
13. Make corrections if necessary, then press `=` to save `camera_path.xml`.

## Time-navigation commands

Forward:

```text
mirv_skip time 0.25
mirv_skip time 0.5
mirv_skip time 1
mirv_skip time 1.5
```

Backward:

```text
mirv_skip time -0.25
mirv_skip time -0.5
mirv_skip time -1
```

Absolute time and tick navigation:

```text
mirv_skip time to <seconds>
mirv_skip tick <ticks>
mirv_skip tick to <tick>
```

Use shortcut `3` for a large backward reset. It avoids one large backward `demo_gototick` by restarting at tick 0 and using capped forward stages.

## Record

The recording order is important:

1. Press `3` and wait for the demo to return to the lead-in and pause.
2. Press `8` to ensure campath playback is enabled.
3. Open the console and enter `thirdperson`. If necessary, also enter `r_drawviewmodel 0`.
4. Optionally press `]` or enter `cl_drawhud 0` to hide the HUD.
5. Press `5` to resume demo playback.
6. Press `9` to start recording once playback is moving.
7. Let the full multi-kill sequence and outro play.
8. Press `0` to stop recording. Wait a moment for the encoder to flush before closing TF2.

The confirmed order is `5 → 9 → 0`: resume, start recording, then stop recording.

The helper keeps its existing output and finalization behavior. Captures use the configured recording directory under `Manual HLAE/<demo>__<candidate>__t<start>-<end>__<timestamp>/`.

If a capture cannot be found, check HLAE's current recording name:

```text
mirv_streams record name
```

If it reports a relative name such as `untitled`, check beneath the Team Fortress 2 installation directory. A typical fallback is:

```text
C:\Program Files (x86)\Steam\steamapps\common\Team Fortress 2\untitled\
```

For manual troubleshooting only, an absolute capture path can be set and verified with:

```text
mirv_streams record name "D:\TF2 Recordings\ManualCapture"
mirv_streams record name
```

## Campath commands and persistence

```text
mirv_campath add
mirv_campath print
mirv_campath remove <id>
mirv_campath clear
mirv_campath enabled 0
mirv_campath enabled 1
mirv_campath save "C:/path/to/camera_path.xml"
mirv_campath load "C:/path/to/camera_path.xml"
```

Saving a campath writes the keyframes to XML. It does not force a later demo launch to use that path. A future HLAE session must deliberately load and enable it:

```text
mirv_campath load "C:/path/to/camera_path.xml"
mirv_campath enabled 1
```

`mirv_campath enabled 0` disables playback without deleting the in-memory keys. `mirv_campath clear` deletes the in-memory keys but does not delete an XML file already saved to disk. Same-session shortcut `3` preserves the current in-memory path.

## HUD commands

```text
cl_drawhud 0
cl_drawhud 1
```

## Exit and restoration

Close TF2 normally when finished. The helper waits for TF2 to be fully gone, closes the HLAE launcher if necessary, and restores the original TF2 files and settings. Keep the helper open until its header returns to `READY`.

If you close the helper while this TF2 instance is still open, confirm the shutdown prompt. The helper closes the launched TF2 process and restores the profile before exiting. If Windows interrupts the helper or the PC loses power, reopening the helper triggers recovery from the saved backup marker.

## Official HLAE references

- [mirv_input camera](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_input)
- [mirv_campath keyframes](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_campath)
- [mirv_skip navigation](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_skip)
- [mirv_streams recording](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_streams)
