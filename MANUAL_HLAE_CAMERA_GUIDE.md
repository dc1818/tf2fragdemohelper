# Manual HLAE / MIRV Camera Guide

This workflow opens one selected candidate at the chosen lead-in and lets you place the camera yourself. It is designed for kills on multiple ticks and for simultaneous or nearby victims that are easier to frame by eye.

## Launch the candidate

1. Open the Candidates page and select exactly one candidate.
2. Set `Before first tick` to the amount of setup time you want. The launcher seeks to that point before the first kill.
3. Set `After last tick` for the end-of-clip reference. It is included in the manual capture folder name so the intended window stays visible.
4. Choose the recording format, FPS, resolution, encoder, HUD, and other recording settings as usual.
5. Click `Launch TF2 with HLAE`.
6. Wait for the demo to seek. It automatically pauses at the selected lead-in. The console prints `TF2FRAG_MANUAL_READY` and `TF2FRAG_MANUAL_PAUSED_AT_START` when the temporary controls are installed and the pause has been applied.

The launcher uses offline mode (`-insecure`, `sv_lan 1`) and the same temporary recording profile as automatic recording. Do not start a live-server session from this TF2 instance.

## Temporary hotkeys

These binds exist only in the isolated session. Your complete original `tf/cfg` folder is restored after TF2 closes.

| Key | Action |
|---|---|
| 1 | Print the complete temporary hotkey reminder in the console |
| 2 | Move demo time back one second with `mirv_skip time -1` |
| 3 | Return to the selected lead-in before the first kill |
| 4 | Cycle to each distinct kill tick in the candidate |
| 5 | Pause or resume the demo |
| 6 | Enter the manual MIRV camera |
| 7 | Add a campath keyframe at the current demo time and view |
| 8 | Exit manual input and enable campath playback |
| 9 | Start recording with the selected format and FPS |
| 0 | Stop recording and reset `host_framerate` |
| - | Print all campath keyframes in the console |
| = | Save the path as `camera_path.xml` in the capture folder |

While `mirv_input camera` is active, use W/A/S/D to move, R/F to move up/down, Page Up/Page Down to change FOV, Z/X to roll, `+`/`-` to change speed, and Home to reset. Press Escape to leave manual camera input. Manual input is suspended while the console is open.

## Make a keyframed shot

1. The demo is already paused at the selected lead-in.
2. Press 6 to enter the camera and compose the opening view.
3. Press 7 to add the first keyframe.
4. Advance demo time before each new keyframe. You can resume slowly, use 4 to visit the next kill tick, or use console commands such as `mirv_skip time 0.25`.
5. Reposition the camera and press 7 again. A normal HLAE campath needs at least four keyframes.
6. For a simultaneous kill, place one view that contains both victims. For kills a few ticks apart, add a key near each victim and leave enough demo time between keys for a readable transition.
7. Press - and inspect the keyframe list. To remove a bad key, use `mirv_campath remove <id>` in the console.
8. Press Escape, then 8. Manual camera input must be ended because it overrides campath playback.
9. Press 3 to rewind to the lead-in, close the console, and resume playback to preview the path.
10. Press = when you want to preserve the path.

Useful console commands:

```text
mirv_campath print
mirv_campath remove <id>
mirv_campath clear
mirv_campath enabled 0
mirv_campath enabled 1
mirv_campath load "C:/path/to/camera_path.xml"
```

## Record

1. Rewind with 3.
2. Make sure campath playback is enabled with 8 and the console is closed.
3. Press 9 just before the portion you want to keep. The helper sets both `host_framerate` and `mirv_streams record fps` for encoded formats.
4. Let the complete multi-kill sequence play.
5. Press 0 after the last kill/outro. Wait a moment for the encoder to flush before closing TF2.

The capture is stored in the configured recording directory under `Manual HLAE/<demo>__<candidate>__t<start>-<end>__<timestamp>/`. Encoded HLAE formats normally create their video inside that folder; image formats create numbered frames and audio there.

## Exit and restoration

Close TF2 normally when finished. The helper waits for TF2 to be fully gone, closes the HLAE launcher if necessary, and restores the original TF2 files and settings. Keep the helper open until its header returns to `READY`.

If you close the helper while this TF2 instance is still open, confirm the shutdown prompt. The helper closes the launched TF2 process and restores the profile before exiting. If Windows interrupts the helper or the PC loses power, reopening the helper triggers recovery from the saved backup marker before another session can start.

## Official HLAE references

- [mirv_input camera](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_input)
- [mirv_campath keyframes](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_campath)
- [mirv_streams recording](https://github.com/advancedfx/advancedfx/wiki/Source%3Amirv_streams)
