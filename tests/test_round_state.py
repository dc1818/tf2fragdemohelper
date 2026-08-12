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

    def test_point_capture_is_scored_once_even_with_payload_progress(self):
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 120, "event_type": "player_team", "event": {"user_id": 1, "team": 3}},
            {"tick": 220, "event_type": "payload_pushed", "event": {"pusher": 1, "distance": 450}},
            {"tick": 240, "event_type": "teamplay_point_captured", "event": {"team": 3, "cp": 2, "cp_name": "second"}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        objectives = ANALYZER.normalized_objective_events(events, rounds, ANALYZER.player_team_history(events))
        kill = dict(self.kill(200, 2), attacker_team="blu")
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kill], rounds[0], objective_events=objectives)
        self.assertEqual(metrics["point_capture_followups"], 1)
        self.assertEqual(metrics["payload_progress_followups"], 1)
        self.assertIn("objective_capture_followup", tags)
        self.assertNotIn("payload_progress_followup", tags)
        self.assertEqual(metrics["objective_conversion_kind"], "point_capture")
        self.assertEqual(score, 34.0)
        self.assertEqual(
            {item["reason"] for item in breakdown},
            {"candidate_base", "kill_sequence_led_to_point_capture"},
        )

    def test_payload_progress_is_scored_once_without_a_capture(self):
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 120, "event_type": "player_team", "event": {"user_id": 1, "team": 3}},
            {"tick": 220, "event_type": "payload_pushed", "event": {"pusher": 1, "distance": 450}},
            {"tick": 230, "event_type": "payload_pushed", "event": {"pusher": 1, "distance": 500}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        objectives = ANALYZER.normalized_objective_events(events, rounds, ANALYZER.player_team_history(events))
        kill = dict(self.kill(200, 2), attacker_team="blu")
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kill], rounds[0], objective_events=objectives)
        self.assertEqual(metrics["payload_progress_followups"], 2)
        self.assertEqual(metrics["objective_conversion_kind"], "payload_progress")
        self.assertIn("payload_progress_followup", tags)
        self.assertIn("payload_pusher", tags)
        self.assertEqual(score, 26.0)
        self.assertEqual(
            [item["reason"] for item in breakdown],
            ["candidate_base", "kill_sequence_led_to_payload_progress"],
        )

    def test_demoman_pick_and_round_clinch_are_scored(self):
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 210, "event_type": "teamplay_round_win", "event": {"team": 3}},
        ]
        rounds = ANALYZER.build_rounds(events)
        kill = dict(self.kill(200, 2), attacker_team="blu", victim_class="demoman")
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kill], rounds[0])
        self.assertEqual(score, 40.0)
        self.assertEqual(metrics["demoman_kills"], 1)
        self.assertIn("demoman_pick", tags)
        self.assertIn("late_round", tags)
        self.assertIn("round_clinch", tags)
        self.assertEqual(
            {item["reason"] for item in breakdown},
            {"candidate_base", "demoman_pick", "late_round", "team_won_immediately_after_sequence"},
        )

    def test_capture_denial_requires_the_fragging_player_to_be_the_blocker(self):
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 120, "event_type": "player_team", "event": {"user_id": 1, "team": 3}},
            {"tick": 120, "event_type": "player_team", "event": {"user_id": 2, "team": 3}},
            {"tick": 220, "event_type": "teamplay_capture_blocked", "event": {"blocker": 1, "victim": 9, "cp": 3, "cp_name": "last"}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        objectives = ANALYZER.normalized_objective_events(events, rounds, ANALYZER.player_team_history(events))
        kill = dict(self.kill(200, 9), attacker_team="blu")
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kill], rounds[0], objective_events=objectives)
        self.assertEqual(score, 30.0)
        self.assertEqual(metrics["capture_denial_followups"], 1)
        self.assertEqual(metrics["objective_conversion_kind"], "capture_denial")
        self.assertIn("capture_denial_followup", tags)
        self.assertEqual([item["reason"] for item in breakdown], ["candidate_base", "kill_sequence_blocked_capture"])

        wrong_blocker = [dict(objectives[0], blocker_user_id=2)]
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kill], rounds[0], objective_events=wrong_blocker)
        self.assertEqual(score, 10.0)
        self.assertEqual(metrics["objective_conversion_kind"], "")
        self.assertNotIn("capture_denial_followup", tags)

    def test_state_timeline_confirms_airshot_with_matching_projectile(self):
        with tempfile.TemporaryDirectory() as temp:
            state_path = Path(temp) / "state_samples.ndjson"
            records = [
                {
                    "demo_tick": 190,
                    "server_tick": 190,
                    "players": [
                        {"entity_id": 1, "user_id": 1, "team": "blue", "class": "soldier", "position": [0, 0, 0], "velocity": [0, 0, 0], "flags": 1, "on_ground": True, "health": 200, "life_state": "alive", "weapon_handles": [55, 0, 0]},
                        {"entity_id": 2, "user_id": 2, "team": "red", "class": "soldier", "position": [100, 0, 100], "velocity": [0, 0, 250], "flags": 0, "on_ground": False, "health": 80, "life_state": "alive", "weapon_handles": [0, 0, 0]},
                    ],
                    "projectiles": [{"entity_id": 10, "team": "blue", "projectile_type": "rocket", "position": [100, 0, 100], "initial_velocity": [1100, 0, 0], "launcher_handle": 55, "critical": False}],
                    "removed_projectiles": [],
                },
                {"demo_tick": 200, "server_tick": 200, "players": [], "projectiles": [], "removed_projectiles": [10]},
            ]
            state_path.write_text("\n".join(json.dumps(record) for record in records) + "\n", encoding="utf-8")
            timeline = ANALYZER.read_state_timeline(state_path)
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2, "weapon": "rocketlauncher"}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        deaths = ANALYZER.normalized_deaths(events, rounds, {}, {}, {}, {"analysis_scope": "all_players"})
        ANALYZER.enrich_state_evidence(deaths, events, timeline)
        evidence = deaths[0]["state_evidence"]
        self.assertTrue(evidence["victim_airborne"])
        self.assertTrue(evidence["confirmed_airshot"])
        self.assertEqual(evidence["projectile"]["entity_id"], 10)
        score, tags, metrics, _ = ANALYZER.score_candidate(deaths, rounds[0])
        self.assertEqual(score, 44.0)
        self.assertIn("confirmed_airshot", tags)
        self.assertIn("direct_airshot", tags)
        self.assertEqual(metrics["confirmed_airshots"], 1)

    def test_medic_death_and_charge_state_confirm_uber_drop(self):
        with tempfile.TemporaryDirectory() as temp:
            state_path = Path(temp) / "state_samples.ndjson"
            state_path.write_text(json.dumps({
                "demo_tick": 190,
                "server_tick": 190,
                "players": [
                    {"entity_id": 1, "user_id": 1, "team": "blue", "class": "scout", "position": [0, 0, 0], "flags": 1, "on_ground": True, "health": 125, "life_state": "alive", "weapon_handles": [11, 0, 0]},
                    {"entity_id": 2, "user_id": 2, "team": "red", "class": "medic", "position": [50, 0, 0], "flags": 1, "on_ground": True, "health": 150, "life_state": "alive", "weapon_handles": [22, 0, 0], "medic_charge": 100, "medigun": "uber"},
                ],
                "projectiles": [],
                "removed_projectiles": [],
            }) + "\n", encoding="utf-8")
            timeline = ANALYZER.read_state_timeline(state_path)
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "medic_death", "event": {"user_id": 2, "attacker": 1, "charged": True}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2, "weapon": "scattergun"}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        deaths = ANALYZER.normalized_deaths(events, rounds, {}, {}, {}, {"analysis_scope": "all_players"})
        ANALYZER.enrich_state_evidence(deaths, events, timeline)
        score, tags, metrics, _ = ANALYZER.score_candidate(deaths, rounds[0])
        self.assertTrue(deaths[0]["state_evidence"]["confirmed_uber_drop"])
        self.assertEqual(score, 48.0)
        self.assertIn("uber_drop", tags)
        self.assertEqual(metrics["confirmed_uber_drops"], 1)

    def test_state_counts_rank_wipe_and_disadvantage_swing(self):
        wipe = dict(self.kill(200, 2), state_evidence={
            "enemy_alive_before": 1,
            "friendly_alive_before": 4,
            "enemy_state_roster": 6,
        })
        score, tags, _, _ = ANALYZER.score_candidate([wipe], {"end_tick": 1000})
        self.assertEqual(score, 28.0)
        self.assertIn("team_wipe", tags)
        self.assertIn("last_enemy_alive", tags)

        swing = [
            dict(self.kill(200, 2), state_evidence={"enemy_alive_before": 5, "friendly_alive_before": 3, "enemy_state_roster": 6}),
            self.kill(250, 3),
        ]
        score, tags, _, _ = ANALYZER.score_candidate(swing, {"end_tick": 1000})
        self.assertEqual(score, 56.0)
        self.assertIn("disadvantage_swing", tags)

    def test_sack_uber_recovery_requires_verified_advantage_and_medic_pick(self):
        with tempfile.TemporaryDirectory() as temp:
            state_path = Path(temp) / "state_samples.ndjson"
            state_path.write_text("\n".join(json.dumps(record) for record in [
                {
                    "demo_tick": 140,
                    "server_tick": 140,
                    "players": [
                        {"entity_id": 1, "user_id": 1, "team": "blue", "class": "scout", "health": 125, "life_state": "alive"},
                        {"entity_id": 3, "user_id": 3, "team": "blue", "class": "soldier", "health": 200, "life_state": "alive"},
                        {"entity_id": 4, "user_id": 4, "team": "blue", "class": "demoman", "health": 175, "life_state": "alive"},
                        {"entity_id": 2, "user_id": 2, "team": "red", "class": "medic", "health": 150, "life_state": "alive", "medic_charge": 100},
                        {"entity_id": 5, "user_id": 5, "team": "red", "class": "soldier", "health": 200, "life_state": "alive"},
                        {"entity_id": 6, "user_id": 6, "team": "red", "class": "scout", "health": 125, "life_state": "alive"},
                    ],
                    "projectiles": [], "removed_projectiles": [],
                },
                {
                    "demo_tick": 170,
                    "server_tick": 170,
                    "players": [
                        {"entity_id": 3, "user_id": 3, "team": "blue", "class": "soldier", "health": 0, "life_state": "dead"},
                        {"entity_id": 4, "user_id": 4, "team": "blue", "class": "demoman", "health": 0, "life_state": "dead"},
                    ],
                    "projectiles": [], "removed_projectiles": [],
                },
            ]) + "\n", encoding="utf-8")
            timeline = ANALYZER.read_state_timeline(state_path)
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 150, "event_type": "player_death", "event": {"attacker": 5, "user_id": 3}},
            {"tick": 160, "event_type": "player_death", "event": {"attacker": 5, "user_id": 4}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2, "weapon": "scattergun"}},
            {"tick": 220, "event_type": "player_death", "event": {"attacker": 1, "user_id": 5, "weapon": "scattergun"}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        deaths = ANALYZER.normalized_deaths(events, rounds, {}, {}, {}, {"analysis_scope": "all_players"})
        ANALYZER.enrich_state_evidence(deaths, events, timeline, rounds=rounds)
        response = [death for death in deaths if death["attacker_user_id"] == 1]
        score, tags, metrics, breakdown = ANALYZER.score_candidate(response, rounds[0])
        self.assertTrue(metrics["sack_uber_recovery"])
        self.assertTrue(metrics["sack_uber_medic_equalizer"])
        self.assertIn("sack_uber_recovery", tags)
        self.assertIn("sack_uber_medic_equalizer", tags)
        self.assertEqual(score, 122.0)
        self.assertEqual(
            {"sack_uber_recovery_after_losses", "sack_uber_medic_equalizer"},
            {item["reason"] for item in breakdown if item["reason"].startswith("sack_uber")},
        )

    def test_sack_uber_score_does_not_apply_without_verified_uber_advantage(self):
        kills = [
            dict(self.kill(200, 2), victim_class="medic", state_evidence={
                "recent_friendly_death_count": 2,
                "player_disadvantage_before": 2,
                "enemy_uber_advantage_before": False,
            }),
            self.kill(220, 3),
        ]
        _, tags, metrics, _ = ANALYZER.score_candidate(kills, {"end_tick": 1000})
        self.assertFalse(metrics["sack_uber_recovery"])
        self.assertNotIn("sack_uber_recovery", tags)

    def test_duplicate_victim_death_cannot_create_a_multikill(self):
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2, "weapon": "iron_bomber"}},
            {"tick": 235, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2, "weapon": "tf_projectile_pipe_remote"}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        deaths = ANALYZER.normalized_deaths(events, rounds, {}, {}, {}, {"analysis_scope": "all_players"})
        self.assertEqual(len(deaths), 1)
        score, tags, metrics, _ = ANALYZER.score_candidate(deaths, rounds[0])
        self.assertEqual(metrics["unique_victims"], 1)
        self.assertNotIn("multi_kill", tags)
        self.assertEqual(score, 18.0)

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
