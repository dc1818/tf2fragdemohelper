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
        self.assertIn('startmovie \\"', self.batch)
        self.assertIn('jpeg_quality ', self.batch)
        self.assertIn('? "endmovie"', self.batch)

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
        self.assertIn('startmovie \\"', self.batch)

    def test_ffmpeg_is_discovered_or_selected_before_hlae_launch(self):
        self.assertIn('AddPathRow(layout, 0, "FFmpeg.exe"', self.batch)
        self.assertIn('FindFfmpegNearHlae', self.batch)
        self.assertIn('Select ffmpeg.exe at the top of the setup window.', self.batch)


if __name__ == "__main__":
    unittest.main()
