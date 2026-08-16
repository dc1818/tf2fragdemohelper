import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class GuiBatchRecordingContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.program = (ROOT / "gui" / "Program.cs").read_text(encoding="utf-8")
        cls.batch = (ROOT / "gui" / "BatchSupport.cs").read_text(encoding="utf-8")
        cls.profile = (ROOT / "gui" / "RecordingProfile.cs").read_text(encoding="utf-8")

    def test_hlae_launcher_is_offline_only(self):
        self.assertIn('-steam -insecure +sv_lan 1', self.batch)
        self.assertIn('alias connect', self.batch)
        self.assertIn('alias retry', self.batch)
        self.assertNotIn('custom launch arguments', self.program.lower())

    def test_current_and_legacy_tf2_hooks_are_selected(self):
        self.assertIn('tf_win64.exe', self.batch)
        self.assertIn('Path.Combine(hlaeDirectory, "x64", "AfxHookSource.dll")', self.batch)
        self.assertIn('Path.Combine(hlaeDirectory, "AfxHookSource.dll")', self.batch)
        self.assertIn('-force32bit', self.batch)

    def test_particle_selector_only_lists_packaged_particles(self):
        self.assertNotIn('particles/scary_ghost.pcf', self.profile)

    def test_recording_dialog_saves_preferences_when_cancelled(self):
        self.assertIn('dialog.FormClosing += delegate { SaveSettings(dialog.Settings); };', self.batch)

    def test_batch_candidates_keep_source_demo_context(self):
        self.assertIn('candidate["batch_context"]', self.batch)
        self.assertIn('batchContext["source_demo"]', self.batch)
        self.assertIn('batchContext["demo_order"]', self.batch)
        self.assertIn('grid.MultiSelect = true', self.program)
        self.assertIn('Select all visible', self.program)

    def test_candidate_actions_require_an_explicit_selection(self):
        self.assertNotIn('grid.Rows[0].Selected = true', self.program)
        self.assertIn('ClearCandidateSelection();', self.program)
        self.assertIn('Shown += delegate', self.program)
        self.assertIn('grid.CurrentCell = null;', self.program)
        self.assertIn('recordButton.Enabled = hasSelection;', self.program)

    def test_preview_button_appears_only_for_one_selected_candidate(self):
        self.assertIn('int selectedCount = grid.SelectedRows.Count;', self.program)
        self.assertIn('private readonly Button inlinePreviewButton', self.program)
        self.assertIn('inlinePreviewButton.Visible = selectedCount == 1;', self.program)
        self.assertIn('Preview Selected in TF2', self.program)
        self.assertIn('PositionPreviewButton();', self.program)
        self.assertIn('inlinePreviewButton.Height = Math.Max(1, rowBounds.Height);', self.program)

    def test_candidate_rows_toggle_off_and_double_click_does_not_preview(self):
        self.assertIn('grid.CellMouseDown += RememberSelectedRowClick;', self.program)
        self.assertIn('grid.CellClick += ToggleClickedSelectedRow;', self.program)
        self.assertIn('grid.Rows[e.RowIndex].Selected = false;', self.program)
        self.assertNotIn('grid.CellDoubleClick += delegate { LaunchSelectedCandidate(); };', self.program)
        self.assertNotIn('private readonly Button launchButton', self.program)

    def test_details_pane_scrolls_after_its_handle_exists(self):
        self.assertIn('details.HandleCreated += delegate { ScrollDetailsToBottom(); };', self.program)

    def test_vdm_queue_records_and_stops_each_clip(self):
        self.assertIn(r'factory \"SkipAhead\"', self.batch)
        self.assertIn('mirv_streams record screen settings ', self.batch)
        self.assertIn('mirv_streams record start', self.batch)
        self.assertIn('mirv_streams record end', self.batch)
        self.assertIn('playdemo ', self.batch)
        self.assertIn('recorderFlushTicks', self.batch)
        self.assertIn('TF2FRAG_RECORD_START', self.batch)
        self.assertIn('TF2FRAG_RECORD_END', self.batch)

    def test_recorder_is_initialized_and_logged_before_demo_playback(self):
        self.assertIn('con_logfile tf2fragdemohelper_recording.log', self.batch)
        self.assertIn('TF2FRAG_RECORDER_INIT', self.batch)
        self.assertIn('TF2FRAG_RECORDER_READY', self.batch)

    def test_tf2_image_sequences_use_native_startmovie(self):
        self.assertIn('startmovie " + captureBaseName + " raw', self.batch)
        self.assertIn('jpeg_quality ', self.batch)
        self.assertIn('? "endmovie"', self.batch)
        self.assertIn('clip.CaptureBaseName', self.batch)
        self.assertIn('TransferNativeMovieFiles', self.batch)
        self.assertIn('WaitForTf2ToExit', self.batch)

    def test_vdm_executes_per_clip_cfg_files_without_nested_quotes(self):
        self.assertIn('WriteRecordingConfigs(demos, gameDirectory, sessionId', self.batch)
        self.assertIn('"exec " + clip.StartConfigRelative', self.batch)
        self.assertIn('"exec " + clip.StopConfigRelative', self.batch)
        self.assertNotIn('startmovie \\"', self.batch)

    def test_capture_fps_choices_are_fixed_temporal_sample_rates(self):
        for fps in ('60', '120', '240', '480'):
            self.assertIn(f'recordingFps.Items.Add("{fps}");', self.program)
        self.assertIn('int captureFps = Convert.ToInt32(recordingFps.SelectedItem);', self.program)
        self.assertIn('host_framerate " + fps + "; mirv_streams record fps " + fps', self.batch)
        self.assertIn('manifest["fps_semantics"] = "captured_frames_per_demo_second";', self.batch)

    def test_grid_starts_with_rank_column_and_no_row_header_column(self):
        self.assertIn('grid.RowHeadersVisible = false;', self.program)
        self.assertIn('AddColumn("#", 42);', self.program)

    def test_hlae_launch_matches_tf2_custom_loader_guidance(self):
        self.assertIn('-customLoader -autoStart -noGui', self.batch)
        self.assertIn('-afxGame tf', self.batch)
        self.assertIn('-no_texture_stream', self.batch)
        self.assertNotIn('-noConfig', self.batch)

    def test_movie_output_choices_are_exposed(self):
        self.assertIn('TGA image sequence', self.program)
        self.assertIn('JPG image sequence', self.program)
        self.assertIn('MP4 - standard', self.program)
        self.assertIn('MP4 - lossless', self.program)
        self.assertIn('AVI - raw', self.program)
        self.assertIn('afxClassic', self.batch)
        self.assertIn('afxFfmpegLosslessBest', self.batch)
        self.assertIn('startmovie " + captureBaseName + " raw', self.batch)

    def test_ffmpeg_is_discovered_or_selected_before_hlae_launch(self):
        self.assertIn('AddPathRow(layout, 0, "FFmpeg.exe"', self.profile)
        self.assertIn('FindFfmpegNearHlae', self.batch)
        self.assertIn('Select ffmpeg.exe at the top of the setup window.', self.batch)

    def test_item_schema_field_is_filled_only_for_tf_demos(self):
        self.assertIn('UpdateAutoDetectedItemSchema();', self.program)
        self.assertIn('DetectItemSchemaForSelectedDemos', self.program)
        self.assertIn('ItemSchemaBesideTfDemos', self.program)
        self.assertIn('directory.Name, "demos"', self.program)
        self.assertIn('directory.Parent.Name, "tf"', self.program)
        self.assertIn('Item schema (optional)', self.program)

    def test_movie_settings_are_exposed(self):
        for label in (
            '"Resolution"', '"DX level"', '"Skybox"', '"HUD"', '"Viewmodels"',
            '"Viewmodel FOV"', '"Maximum-quality graphics profile"',
            '"Enable motion blur"', '"Disable hit sounds"', '"Disable voice chat"',
            '"Minimal HUD"', '"Disable combat text"', '"Disable crosshair"',
            '"Disable crosshair switching"', '"3D player model in HUD"',
        ):
            self.assertIn(label, self.profile)
        self.assertIn('"98 (highest)"', self.profile)
        self.assertIn('"Kill notices only"', self.profile)
        self.assertIn('new RowStyle(SizeType.Absolute, 110)', self.profile)
        self.assertIn('q.AutoScroll = false;', self.profile)
        self.assertIn('new RowStyle(SizeType.Absolute, 170)', self.profile)
        self.assertIn('for (int row = 0; row < 4; row++) checks.RowStyles.Add', self.profile)

    def test_custom_resources_particles_and_skyboxes_are_supported(self):
        self.assertIn('Temporarily isolate custom resources', self.profile)
        self.assertIn('Disable announcer voices', self.profile)
        self.assertIn('Disable applause sounds', self.profile)
        self.assertIn('Disable domination/revenge sounds', self.profile)
        self.assertIn('Enable enhanced particles', self.profile)
        self.assertIn('particles/blood_impact.pcf', self.profile)
        self.assertIn('particles/default.pcf', self.profile)
        self.assertIn('particles/particles_manifest.txt', self.profile)
        self.assertIn('ExtractParticleFiles', self.profile)
        self.assertIn('Directory.CreateDirectory(Path.Combine(destination, "particles"))', self.profile)
        self.assertIn('CopyPackagedParticleFiles(root, profileRoot, selected)', self.profile)
        self.assertIn('InstallSkybox', self.profile)
        self.assertIn('CopyHud', self.profile)

    def test_maximum_graphics_profile_overrides_low_quality_configs(self):
        for command in (
            'mat_picmip -1', 'mat_antialias 8', 'mat_forceaniso 16',
            'mat_hdr_level 2', 'r_lod 0', 'r_rootlod 0',
            'r_shadowrendertotexture 1', 'r_waterforceexpensive 1',
        ):
            self.assertIn(command, self.profile)
        self.assertIn('+exec tf2fragdemohelper_recording_profile.cfg', self.batch)
        self.assertIn('"exec tf2fragdemohelper_recording_profile"', self.batch)

    def test_recording_profile_is_reversible_and_crash_recoverable(self):
        self.assertIn('active_recording_profile.json', self.profile)
        self.assertIn('Directory.Move(session.TemporaryCustomDirectory, session.OriginalCustomDirectory)', self.profile)
        self.assertIn('Directory.Move(session.OriginalCustomDirectory, session.TemporaryCustomDirectory)', self.profile)
        self.assertIn('RecoverInterruptedSession(null);', self.program)
        self.assertIn('HlaeBatchRecorder.ShutdownActiveRecording();', self.program)
        self.assertIn('HlaeBatchRecorder.CleanupTemporaryFiles();', self.program)
        self.assertIn('RecordingProfileManager.Restore(profile, false);', self.batch)
        self.assertIn('BackupVideoConfigPath', self.profile)
        self.assertIn('BackupConfigPath', self.profile)
        self.assertIn('config.cfg', self.profile)
        self.assertIn('RestoreDxLevel(session);', self.profile)
        self.assertIn('VerifyRestoredFiles(session);', self.profile)
        self.assertIn('FilesMatch(session.ConfigPath, session.BackupConfigPath)', self.profile)
        self.assertIn('temporary recording resources, including enhanced particles, could not be removed', self.profile)

    def test_recording_resources_are_optional_sidecar_assets(self):
        self.assertIn('recording_resources', self.profile)
        self.assertIn('pldx_particles.vpk', self.profile)
        self.assertIn('no_announcer_voices.vpk', self.profile)
        self.assertIn('ExtractParticleFiles(settings.Tf2Executable', self.profile)
        self.assertIn('Path.Combine(root, "bin", "vpk.exe")', self.profile)

    def test_parser_close_removes_only_helper_owned_recording_temporary_files(self):
        self.assertIn('demos", "tf2fragdemohelper_batch', self.batch)
        self.assertIn('demos", "tf2fragdemohelper', self.batch)
        self.assertIn('cfg", "tf2fragdemohelper_batch', self.batch)
        self.assertIn('tf2fragdemohelper_offline.cfg', self.batch)
        self.assertIn('tf2fragdemohelper_recording.log', self.batch)
        self.assertIn('recording_queue.json', self.batch)
        self.assertIn('exports, source demos, and recorded video/frame folders are never', self.batch)
        self.assertIn('RecordingProfileManager.IsRestoreComplete(out restoreReason)', self.batch)
        self.assertIn('Some helper-owned temporary files could not be removed', self.batch)


if __name__ == "__main__":
    unittest.main()
