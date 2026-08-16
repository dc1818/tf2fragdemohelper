import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


class GuiBatchRecordingContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.program = (ROOT / "gui" / "Program.cs").read_text(encoding="utf-8")
        cls.batch = (ROOT / "gui" / "BatchSupport.cs").read_text(encoding="utf-8")

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

    def test_lawena_style_output_choices_are_exposed(self):
        self.assertIn('TGA image sequence', self.program)
        self.assertIn('JPG image sequence', self.program)
        self.assertIn('MP4 - standard', self.program)
        self.assertIn('MP4 - lossless', self.program)
        self.assertIn('AVI - raw', self.program)
        self.assertIn('afxClassic', self.batch)
        self.assertIn('afxFfmpegLosslessBest', self.batch)
        self.assertIn('startmovie " + captureBaseName + " raw', self.batch)

    def test_ffmpeg_is_discovered_or_selected_before_hlae_launch(self):
        self.assertIn('AddPathRow(layout, 0, "FFmpeg.exe"', self.batch)
        self.assertIn('FindFfmpegNearHlae', self.batch)
        self.assertIn('Select ffmpeg.exe at the top of the setup window.', self.batch)


if __name__ == "__main__":
    unittest.main()
