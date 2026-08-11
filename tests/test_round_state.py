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
    @staticmethod
    def kill(tick, victim, weapon="scattergun"):
        return {
            "tick": tick,
            "event_tick": tick,
            "packet_sequence": tick,
            "event_index_in_packet": 0,
            "attacker_user_id": 1,
            "victim_user_id": victim,
            "attacker_class": "scout",
            "victim_class": "soldier",
            "weapon": weapon,
            "rocket_jump_victim": False,
            "kill_streak_total": 0,
            "crit_type": 0,
        }

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

    def test_multikill_window_is_measured_first_to_last(self):
        # Each adjacent gap is under four seconds, but all three kills do not
        # belong to one four-second sequence.
        kills = [self.kill(100, 2), self.kill(350, 3), self.kill(600, 4)]
        groups = ANALYZER.group_kills(kills)
        self.assertEqual([[kill["event_tick"] for kill in group] for group in groups], [[100, 350], [600]])

    def test_multikill_tag_requires_multiple_death_events(self):
        round_data = {"end_tick": 5000}
        _, single_tags, single_metrics, single_breakdown = ANALYZER.score_candidate(
            [self.kill(100, 2, "grenadelauncher")], round_data
        )
        _, multi_tags, multi_metrics, multi_breakdown = ANALYZER.score_candidate(
            [self.kill(100, 2), self.kill(100, 3)], round_data
        )
        self.assertNotIn("multi_kill", single_tags)
        self.assertEqual(single_metrics["kills"], 1)
        self.assertIn("multi_kill", multi_tags)
        self.assertEqual(multi_metrics["kills"], 2)
        self.assertEqual([item["reason"] for item in multi_breakdown].count("additional_kills"), 1)
        self.assertEqual(sum(item["points"] for item in single_breakdown), single_metrics["score_before_floor"])

    def test_candidate_exposes_exact_kill_ticks_separately_from_clip_padding(self):
        kills = [dict(self.kill(1000, 2), round_index=1, attacker_team="blu")]
        rounds = [{
            "round_index": 1,
            "live_start_tick": 100,
            "live_start_event": "teamplay_round_active",
            "round_active_tick": 100,
            "setup_finished_tick": None,
            "activation_trigger": {"event": "round_start", "tick": 90},
            "ready_up": {},
            "end_tick": 2000,
            "end_reason": "teamplay_round_win",
        }]
        candidates = ANALYZER.build_candidates(kills, rounds, {"analysis_scope": "all_players"})
        self.assertEqual(len(candidates), 0)  # ordinary low-signal single kill is intentionally filtered

        kills[0]["victim_class"] = "medic"
        candidate = ANALYZER.build_candidates(kills, rounds, {"analysis_scope": "all_players"})[0]
        self.assertEqual(candidate["first_kill_tick"], 1000)
        self.assertEqual(candidate["last_kill_tick"], 1000)
        self.assertEqual(candidate["point_of_kill_ticks"], [1000])
        self.assertEqual(candidate["clip_start_tick"], 667)
        self.assertEqual(candidate["clip_end_tick"], 1200)


if __name__ == "__main__":
    unittest.main()
