# TF2 MIRV Director

TF2 MIRV Director is a separate companion executable for the manual HLAE workflow. The helper writes a versioned `director_session.json` beside the candidate's `camera_path.xml`, launches the Director, and closes it when the TF2 session ends.

The implemented Option C layout uses two operating-system windows: a thin click-through timeline strip across the top and an interactive cue card at the right. Both are repeatedly reasserted as topmost over TF2's borderless window. The rest of TF2 remains directly usable, while the card can be hidden or restored with the saved overlay-panel shortcut (`HOME` by default).

The timeline is live without adding another injected DLL. The helper writes temporary VDM actions that echo a namespaced tick marker every five demo ticks (or a bounded adaptive interval for unusually long clips). Director tails the existing TF2 console log and moves the current-tick playhead from those real playback markers. Pausing naturally freezes it; restarting the demo reloads the same markers. HLAE backbuffer capture does not include these separate operating-system windows.

The library also owns the typed optional bridge protocol. A future Source 1 AfxHookSource extension can improve telemetry to every rendered frame and activate camera-pose/campath-key editing without changing the recording pipeline.

Run it directly with:

```text
tf2-mirv-director path/to/director_session.json
```
