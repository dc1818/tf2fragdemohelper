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

    def test_pov_scope_falls_back_to_userinfo_roster(self):
        events = ANALYZER.read_events(ROOT / "tests" / "fixture_events.ndjson")
        names = ANALYZER.player_name_history(events)
        with tempfile.TemporaryDirectory() as temp:
            directory = Path(temp)
            (directory / "header.json").write_text(json.dumps({"nick": "FocusFromRoster"}), encoding="utf-8")
            (directory / "players.json").write_text(json.dumps({"7": {"user_id": 7, "name": "FocusFromRoster"}}), encoding="utf-8")
            (directory / "manifest.json").write_text(json.dumps({"demo_capture": {"classification": "pov", "confidence": "medium", "evidence": ["dem_usercmd"]}}), encoding="utf-8")
            context = ANALYZER.analysis_context(directory, names)
        self.assertEqual(context["analysis_scope"], "pov_player_only")
        self.assertEqual(context["pov_player_user_id"], 7)

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

    def test_pov_recorder_death_and_assist_are_not_frag_kills(self):
        events = [
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {"user_id": 1, "attacker": 2, "assister": 3}},
            {"tick": 210, "event_type": "player_death", "event": {"user_id": 4, "attacker": 1, "assister": 3}},
            {"tick": 500, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(sorted(events + [{"tick": 90, "event_type": "round_start", "event": {}}], key=lambda item: item["tick"]))
        context = {"analysis_scope": "pov_player_only", "pov_player_user_id": 1}
        deaths = ANALYZER.normalized_deaths(events, rounds, {}, {}, {}, context)
        self.assertEqual([death["event_tick"] for death in deaths], [210])
        self.assertEqual(deaths[0]["assister_user_id"], 3)

    def test_building_destruction_only_adds_weight_when_followed_by_kills(self):
        kill = self.kill(200, 2)
        building = {"event_tick": 190, "attacker_user_id": 1, "object_type": "sentrygun"}
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kill], {"end_tick": 1000}, [building])
        self.assertIn("building_to_kill_sequence", tags)
        self.assertEqual(metrics["linked_building_destructions"], 1)
        self.assertEqual([item["reason"] for item in breakdown if item["reason"] == "building_destruction_led_to_kills"], ["building_destruction_led_to_kills"])

    def test_server_tick_is_authoritative_over_demo_packet_tick(self):
        with tempfile.TemporaryDirectory() as temp:
            events_path = Path(temp) / "events.ndjson"
            events_path.write_text(json.dumps({
                "tick": 105192,
                "demo_tick": 105192,
                "server_tick": 104745,
                "event_type": "player_death",
                "event": {"attacker": 1, "user_id": 2},
            }) + "\n", encoding="utf-8")
            events = ANALYZER.read_events(events_path)
        self.assertEqual(events[0]["demo_tick"], 105192)
        self.assertEqual(events[0]["server_tick"], 104745)
        self.assertEqual(events[0]["tick"], 104745)

    def test_candidate_keeps_demo_seek_ticks_separate_from_server_ticks(self):
        first = dict(self.kill(70386, 42), event_tick=71302, demo_tick=70386, server_tick=71302, round_index=1, attacker_team="blu")
        second = dict(self.kill(70386, 31), event_tick=71302, demo_tick=70386, server_tick=71302, round_index=1, attacker_team="blu")
        rounds = [{
            "round_index": 1,
            "live_start_tick": 1,
            "live_start_event": "teamplay_round_active",
            "round_active_tick": 1,
            "setup_finished_tick": None,
            "activation_trigger": {"event": "round_start", "tick": 0},
            "ready_up": {},
            "end_tick": 90000,
            "end_reason": "teamplay_round_win",
        }]
        candidate = ANALYZER.build_candidates([first, second], rounds, {"analysis_scope": "all_players"})[0]
        self.assertIn("multi_kill", candidate["tags"])
        self.assertEqual(candidate["point_of_kill_ticks"], [70386, 70386])
        self.assertEqual(candidate["point_of_kill_server_ticks"], [71302, 71302])


if __name__ == "__main__":
    unittest.main()
