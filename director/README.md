# TF2 MIRV Director

TF2 MIRV Director is a separate companion executable for the manual HLAE workflow. The helper writes a versioned `director_session.json` beside the candidate's `camera_path.xml`, launches the Director, and closes it when the TF2 session ends.

The implemented Option C layout uses two operating-system windows: a thin timeline strip docked to the full top edge of the active monitor and an interactive cue card docked beneath it at the right edge. Both use the monitor's current pixel size and DPI scale, redock when that geometry changes, stay topmost over TF2's borderless window, and use Windows' non-activating-window style so clicking them does not take keyboard focus from TF2. The rest of TF2 remains directly usable, while the card can be hidden or restored with the saved overlay-panel shortcut (`C` by default).

The timeline and direct controls are live without adding another injected DLL. A temporary HLAE `mirv_cmd addCurves tick` command writes authoritative moving demo ticks. For each Director click, the companion atomically writes the current private CFG slot; TF2 polls that slot through its own guarded `wait` loop, so actions execute in the engine command buffer even while demo playback is paused. Director reports completion only after TF2 echoes the unique action acknowledgement, then verifies campath state through `mirv_campath print`. If TF2 reports that `wait` is unavailable, the non-activating overlay preserves TF2 focus and uses a scan-code `SendInput` press of the dedicated emergency bind. HLAE backbuffer capture does not include these operating-system windows.

Run it directly with:

```text
tf2-mirv-director path/to/director_session.json
```
