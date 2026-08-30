# TF2 MIRV Director

TF2 MIRV Director is a separate companion executable for the manual HLAE workflow. The helper writes a versioned `director_session.json` beside the candidate's `camera_path.xml`, launches the Director, and closes it when the TF2 session ends.

The first implementation is a candidate-aware shot planner. It shows the exact clip window, a scaled cue timeline, tick-specific tags, available victim names, whole-candidate tags, and the recording destination while the existing in-game MIRV shortcuts remain authoritative.

The library in this crate also owns the typed local bridge protocol. A future Source 1 AfxHookSource extension can activate live pause, time, camera-pose, and campath-key editing without changing the session format or UI model.

Run it directly with:

```text
tf2-mirv-director path/to/director_session.json
```
