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
        self.assertIn('mirv_streams record screen settings afxFfmpegLosslessBest', self.batch)
        self.assertIn('mirv_streams record start', self.batch)
        self.assertIn('mirv_streams record end; host_framerate 0', self.batch)
        self.assertIn('playdemo ', self.batch)


if __name__ == "__main__":
    unittest.main()
