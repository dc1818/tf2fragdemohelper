import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("analyze_frags", ROOT / "analyze_frags.py")
ANALYZER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ANALYZER)


class RoundStateTests(unittest.TestCase):
    def test_ready_up_and_countdown_do_not_open_live_window(self):
        events = ANALYZER.read_events(ROOT / "tests" / "fixture_events.ndjson")
        rounds = ANALYZER.build_rounds(events)
        names = ANALYZER.player_name_history(events)
        context = {"analysis_scope": "all_players"}
        deaths = ANALYZER.normalized_deaths(
            events, rounds, ANALYZER.class_history(events), ANALYZER.player_team_history(events), names, context
        )

        self.assertEqual([death["tick"] for death in deaths], [200, 300])
        self.assertEqual(len(rounds), 1)
        self.assertEqual(rounds[0]["live_start_tick"], 100)
        self.assertEqual(rounds[0]["live_start_event"], "teamplay_round_active")
        self.assertEqual(rounds[0]["activation_trigger"], {"event": "teamplay_round_restart_seconds", "tick": 80})
        self.assertEqual(deaths[0]["attacker_team"], "blu")
        self.assertEqual(deaths[0]["victim_team"], "red")
        self.assertEqual(deaths[0]["event_tick"], 200)
        self.assertEqual(deaths[0]["packet_sequence"], 0)
        self.assertTrue(rounds[0]["ready_up"]["both_teams_ready"])
        self.assertEqual(rounds[0]["ready_up"]["ready_restart_tick"], 75)
        self.assertEqual(rounds[0]["ready_up"]["countdown_tick"], 80)

    def test_pov_scope_requires_a_resolved_recorder(self):
        events = ANALYZER.read_events(ROOT / "tests" / "fixture_events.ndjson")
        names = ANALYZER.player_name_history(events)
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            (directory / "header.json").write_text(json.dumps({"nick": "RecordedPlayer"}), encoding="utf-8")
            (directory / "manifest.json").write_text(json.dumps({"demo_capture": {"classification": "pov", "confidence": "medium", "evidence": ["dem_usercmd"]}}), encoding="utf-8")
            context = ANALYZER.analysis_context(directory, names)
        self.assertEqual(context["analysis_scope"], "pov_player_only")
        self.assertEqual(context["pov_player_user_id"], 1)


if __name__ == "__main__":
    unittest.main()
