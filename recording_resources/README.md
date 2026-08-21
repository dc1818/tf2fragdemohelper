# Recording resources

This package includes optional sound-suppression VPKs in `custom/`:

- `no_announcer_voices.vpk`
- `no_applause_sounds.vpk`
- `no_domination_sounds.vpk`

It also includes the built-in **Kill notices only** and **Medic recording HUDs**
in `hud/`, plus the bundled skyboxes in `skybox/`.

The recording setup discovers these resources automatically; there is no
resources folder to select. The first time they are needed, the app expands its
packaged resource archive to its own Local AppData cache. The files are copied
only into the temporary offline recording profile and removed again when TF2
closes. If a packaged optional sound file is missing, recording still starts;
only that sound is left enabled.

Enhanced-particle installation is not implemented or exposed. The recorder uses
TF2/HLAE's normal particle rendering and never modifies particle archives.
