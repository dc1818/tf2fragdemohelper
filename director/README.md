# TF2 MIRV Director

TF2 MIRV Director is a separate companion executable for the manual HLAE workflow. The helper writes a versioned `director_session.json` beside the candidate's `camera_path.xml`, launches the Director, and closes it when the TF2 session ends.

The first implementation is a compact, always-on-top shot-checklist overlay. It opens in the upper-right of the active display, passes mouse input through to TF2, and shows the exact clip window, scaled cue timeline, tick-specific tags, available victim names, whole-candidate tags, campath/output paths, keyframe reminders, record order, and the user's normalized in-game MIRV shortcuts. Arrow keys remain reserved for MIRV camera movement.

The overlay reports `PLANNED` because stock Source 1 HLAE does not provide reliable live tick or keyframe telemetry. HLAE backbuffer capture does not include this separate operating-system window. A future optional local bridge can turn the same UI into a live status display without changing the helper's recording pipeline.

The library in this crate also owns the typed local bridge protocol. A future Source 1 AfxHookSource extension can activate live pause, time, camera-pose, and campath-key editing without changing the session format or UI model.

Run it directly with:

```text
tf2-mirv-director path/to/director_session.json
```
