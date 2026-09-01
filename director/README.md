# TF2 MIRV Director

TF2 MIRV Director is a separate companion executable for the manual HLAE workflow. The helper writes a versioned `director_session.json` beside the candidate's `camera_path.xml`, launches the Director, and closes it when the TF2 session ends.

The implemented Option C layout uses two operating-system windows: a thin click-through timeline strip docked to the full top edge of the active monitor and an interactive cue card docked beneath it at the right edge. Both use the monitor's current pixel size and DPI scale, redock when that geometry changes, and are repeatedly reasserted as topmost over TF2's borderless window. The rest of TF2 remains directly usable, while the card can be hidden or restored with the saved overlay-panel shortcut (`S` by default).

The timeline and direct controls are live without adding another injected DLL. A temporary HLAE `mirv_cmd addCurves tick` command writes authoritative moving demo ticks. For each Director click, the companion writes the current private CFG mailbox slot and posts its dedicated bind only to TF2's verified `Valve001` window, so actions still work while the demo is paused. Director tails the existing TF2 console log for tick updates and per-action sequence acknowledgements. HLAE backbuffer capture does not include these operating-system windows.

Run it directly with:

```text
tf2-mirv-director path/to/director_session.json
```
