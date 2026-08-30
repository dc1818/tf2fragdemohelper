# TF2 MIRV Director

TF2 MIRV Director is a separate companion executable for the manual HLAE workflow. The helper writes a versioned `director_session.json` beside the candidate's `camera_path.xml`, launches the Director, and closes it when the TF2 session ends.

The implemented Option C layout uses two operating-system windows: a thin click-through timeline strip docked to the full top edge of the active monitor and an interactive cue card docked beneath it at the right edge. Both use the monitor's current pixel size and DPI scale, redock when that geometry changes, and are repeatedly reasserted as topmost over TF2's borderless window. The rest of TF2 remains directly usable, while the card can be hidden or restored with the saved overlay-panel shortcut (`S` by default).

The timeline is live without adding another injected DLL. The helper writes a temporary VDM action for every demo tick in the selected clip. TF2's demo player emits each namespaced marker only when it reaches that exact start tick; Director tails the existing TF2 console log and moves the playhead to that real engine tick. Pausing naturally freezes it, and seeking or restarting lets the VDM clock resynchronize it. The displayed tick is exact, although the separate window can trail TF2 by the small console-file delivery latency. HLAE backbuffer capture does not include these operating-system windows.

The timeline is driven by an HLAE `mirv_cmd addCurves tick` command installed by the staged VDM after the safe seek. HLAE evaluates that command from its Source 1 demo-playback tick on every tick change, and the Director tails the resulting marker in TF2's temporary console log. The library also owns the typed optional bridge protocol for future camera-pose/campath-key editing without changing the recording pipeline.

Run it directly with:

```text
tf2-mirv-director path/to/director_session.json
```
