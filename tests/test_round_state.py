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

    def test_public_server_fallback_requires_state_confirmed_opposing_team_combat(self):
        timeline = ANALYZER.StateTimeline()
        timeline.sample_count = 2
        timeline.players[1].append((90, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((90, {"team": "red", "class": "soldier", "life_state": "alive", "health": 200}))
        events = [
            {"tick": 100, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2}},
            {"tick": 200, "event_type": "player_hurt", "event": {"attacker": 1, "user_id": 2}},
        ]

        rounds = ANALYZER.build_rounds(events, timeline)

        self.assertEqual(len(rounds), 1)
        self.assertEqual(rounds[0]["live_start_tick"], 100)
        self.assertEqual(rounds[0]["live_start_event"], "in_progress_public_server")
        self.assertEqual(rounds[0]["end_reason"], "demo_end_while_public_play_active")

    def test_public_server_fallback_does_not_open_for_unresolved_or_same_team_death(self):
        timeline = ANALYZER.StateTimeline()
        timeline.sample_count = 2
        timeline.players[1].append((90, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((90, {"team": "blu", "class": "soldier", "life_state": "alive", "health": 200}))
        events = [{"tick": 100, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2}}]

        self.assertEqual(ANALYZER.build_rounds(events, timeline), [])

    def test_casual_waiting_end_can_activate_a_round(self):
        events = [
            {"tick": 50, "event_type": "teamplay_waiting_begins", "event": {}},
            {"tick": 90, "event_type": "teamplay_waiting_ends", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2}},
            {"tick": 500, "event_type": "teamplay_round_win", "event": {"team": 3}},
        ]

        rounds = ANALYZER.build_rounds(events)

        self.assertEqual(len(rounds), 1)
        self.assertEqual(rounds[0]["live_start_tick"], 100)
        self.assertEqual(rounds[0]["activation_trigger"], {"event": "teamplay_waiting_ends", "tick": 90})

    def test_casual_setup_still_delays_live_play_after_waiting_ends(self):
        events = [
            {"tick": 50, "event_type": "teamplay_waiting_begins", "event": {}},
            {"tick": 90, "event_type": "teamplay_waiting_ends", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 300, "event_type": "teamplay_setup_finished", "event": {}},
            {"tick": 350, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2}},
            {"tick": 500, "event_type": "teamplay_round_win", "event": {"team": 3}},
        ]

        rounds = ANALYZER.build_rounds(events)

        self.assertEqual(rounds[0]["live_start_tick"], 300)
        self.assertEqual(rounds[0]["live_start_event"], "teamplay_setup_finished")

    def test_map_rollover_closes_event_confirmed_live_round_at_demo_end(self):
        events = [
            {"tick": 90, "event_type": "teamplay_waiting_ends", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2}},
            {"tick": 400, "event_type": "player_hurt", "event": {"attacker": 1, "user_id": 2}},
        ]

        rounds = ANALYZER.build_rounds(iter(events))

        self.assertEqual(len(rounds), 1)
        self.assertEqual(rounds[0]["live_start_tick"], 100)
        self.assertEqual(rounds[0]["end_tick"], 401)
        self.assertEqual(rounds[0]["end_reason"], "demo_end_while_event_confirmed_round_active")

    def test_split_demo_round_is_finalized_at_eof_and_keeps_its_kills(self):
        # Mirrors steelpub_2.dem: an earlier round closes at the next round
        # start, while the actual map-change continuation ends mid-round.
        events = [
            {"tick": 247, "event_type": "teamplay_round_start", "event": {}},
            {"tick": 577, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 2270, "event_type": "teamplay_round_start", "event": {}},
            {"tick": 2601, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 19416, "event_type": "player_death", "event": {"attacker": 47, "user_id": 52, "weapon": "claidheamohmor"}},
            {"tick": 19544, "event_type": "player_death", "event": {"attacker": 47, "user_id": 57, "weapon": "claidheamohmor"}},
            {"tick": 19796, "event_type": "player_death", "event": {"attacker": 47, "user_id": 28, "weapon": "claidheamohmor"}},
            {"tick": 19867, "event_type": "player_death", "event": {"attacker": 47, "user_id": 40, "weapon": "world"}},
        ]

        rounds = ANALYZER.build_rounds(events)
        deaths = ANALYZER.normalized_deaths(events, rounds, {}, {}, {}, {"analysis_scope": "all_players"})

        self.assertEqual(len(rounds), 2)
        self.assertEqual(rounds[1]["live_start_tick"], 2601)
        self.assertEqual(rounds[1]["end_tick"], 19868)
        self.assertEqual(rounds[1]["end_reason"], "demo_end_while_event_confirmed_round_active")
        self.assertEqual([death["event_tick"] for death in deaths], [19416, 19544, 19796, 19867])

    def test_split_demo_out_of_range_fallback_tick_is_ignored_and_histories_sort(self):
        with tempfile.TemporaryDirectory() as temp:
            state_path = Path(temp) / "state_samples.ndjson"
            records = [
                # Bootstrap value from the prior stream: no server tick means
                # it must not become future state in this 100-tick demo.
                {"demo_tick": 188669, "players": [{"entity_id": 1, "user_id": 7, "team": "Red"}], "projectiles": [], "removed_projectiles": []},
                {"demo_tick": 60, "players": [{"entity_id": 1, "user_id": 7, "team": "Blue"}], "projectiles": [], "removed_projectiles": []},
                {"demo_tick": 50, "players": [{"entity_id": 1, "user_id": 7, "team": "Red"}], "projectiles": [], "removed_projectiles": []},
            ]
            state_path.write_text("".join(json.dumps(record) + "\n" for record in records), encoding="utf-8")
            timeline = ANALYZER.read_state_timeline(state_path, max_demo_tick=100)

        self.assertEqual(timeline.sample_count, 2)
        self.assertEqual(timeline.ignored_out_of_range_samples, 1)
        self.assertEqual([tick for tick, _ in timeline.players[7]], [50, 60])
        self.assertEqual(timeline.player_at(7, 55)["team"], "Red")

    def test_bare_map_initialization_active_event_remains_rejected(self):
        events = [
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2}},
        ]

        self.assertEqual(ANALYZER.build_rounds(events), [])

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

    def test_taunt_kill_is_an_independently_ranked_candidate(self):
        taunt = dict(self.kill(100, 2, "fists"), custom_kill=9)
        score, tags, metrics, breakdown = ANALYZER.score_candidate([taunt], {"end_tick": 1000})

        self.assertEqual(score, 35.0)
        self.assertIn("taunt_kill", tags)
        self.assertEqual(metrics["taunt_kills"], 1)
        self.assertEqual(
            [item for item in breakdown if item["reason"] == "confirmed_taunt_kill"],
            [{"reason": "confirmed_taunt_kill", "points": 25.0, "event_tick": 100, "custom_kill": 9, "taunt": "high_noon"}],
        )

    def test_backstab_is_separate_from_melee_kills(self):
        # The killfeed can mark a backstab as a full crit. Its authoritative
        # custom-kill value still proves that it is not a random crit.
        backstab = dict(self.kill(100, 2, "knife"), custom_kill=2, crit_type=2)
        score, tags, metrics, breakdown = ANALYZER.score_candidate([backstab], {"end_tick": 1000})

        self.assertEqual(score, 30.0)
        self.assertNotIn("taunt_kill", tags)
        self.assertIn("backstab", tags)
        self.assertNotIn("melee_kill", tags)
        self.assertNotIn("random_full_crit", tags)
        self.assertEqual(metrics["taunt_kills"], 0)
        self.assertEqual(metrics["backstab_kills"], 1)
        self.assertEqual(metrics["melee_kills"], 0)
        self.assertEqual(
            [item for item in breakdown if item["reason"] == "confirmed_spy_backstab"],
            [{"reason": "confirmed_spy_backstab", "points": 20.0, "event_tick": 100, "custom_kill": 2, "weapon": "knife"}],
        )

    def test_spy_butter_knife_remains_an_ordinary_melee_kill(self):
        butter_knife = dict(self.kill(100, 2, "knife"), attacker_class="spy", custom_kill=0)
        score, tags, metrics, _ = ANALYZER.score_candidate([butter_knife], {"end_tick": 1000})

        self.assertEqual(score, 25.0)
        self.assertIn("melee_kill", tags)
        self.assertNotIn("backstab", tags)
        self.assertEqual(metrics["melee_kills"], 1)
        self.assertEqual(metrics["backstab_kills"], 0)

    def test_ordinary_melee_kill_is_a_standalone_candidate_signal(self):
        melee = dict(self.kill(100, 2, "market_gardener"), custom_kill=0)
        score, tags, metrics, breakdown = ANALYZER.score_candidate([melee], {"end_tick": 1000})

        self.assertEqual(score, 25.0)
        self.assertIn("melee_kill", tags)
        self.assertIn("market_gardener", tags)
        self.assertEqual(metrics["melee_kills"], 1)
        self.assertIn("player_melee_kill", {item["reason"] for item in breakdown})

    def test_weapon_id_keeps_unknown_melee_name_as_a_candidate(self):
        melee = dict(
            self.kill(100, 2, "claidheamohmor"),
            weapon_id=64,
            weapon_def_index=327,
            attacker_class="demoman",
            round_index=1,
            attacker_team="blu",
        )
        rounds = [{
            "round_index": 1,
            "live_start_tick": 1,
            "live_start_event": "in_progress_public_server",
            "round_active_tick": None,
            "setup_finished_tick": None,
            "activation_trigger": {"event": "opposing_team_player_death", "tick": 1},
            "ready_up": {},
            "end_tick": 1000,
            "end_reason": "demo_end_while_public_play_active",
        }]

        candidate = ANALYZER.build_candidates([melee], rounds, {"analysis_scope": "pov_player_only"})[0]

        self.assertEqual(candidate["overall_score"], 25.0)
        self.assertIn("melee_kill", candidate["tags"])
        self.assertEqual(candidate["metrics"]["melee_kills"], 1)
        melee_score = next(item for item in candidate["score_breakdown"] if item["reason"] == "player_melee_kill")
        self.assertEqual(melee_score["classification_source"], "weapon_id")
        self.assertEqual(melee_score["weapon_def_index"], 327)

    def test_item_schema_resolves_inherited_melee_slot(self):
        schema_text = '''"items_game"
{
    "prefabs"
    {
        "weapon_base" { "item_slot" "primary" }
        "weapon_sword" { "prefab" "weapon_base" "item_slot" "melee" }
    }
    "items"
    {
        "327" { "prefab" "weapon_sword" "name" "The Claidheamohmor" }
        "999" { "prefab" "weapon_base" }
    }
}
'''
        with tempfile.TemporaryDirectory() as temp:
            schema = Path(temp) / "items_game.txt"
            schema.write_text(schema_text, encoding="utf-8")
            slots = ANALYZER.item_schema_slots(schema)

        self.assertEqual(slots[327], "melee")
        self.assertEqual(slots[999], "primary")

    def test_schema_slot_is_authoritative_over_weapon_id_fallback(self):
        kill = dict(
            self.kill(100, 2, "unexpected_name"),
            weapon_id=64,
            weapon_def_index=999,
            weapon_slot="secondary",
            weapon_slot_source="item_schema",
        )

        self.assertFalse(ANALYZER.is_melee_kill(kill))

    def test_normalized_death_records_schema_slot_source(self):
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {
                "attacker": 1,
                "user_id": 2,
                "weapon": "unknown_schema_sword",
                "weapon_id": 0,
                "weapon_def_index": 327,
            }},
            {"tick": 500, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        deaths = ANALYZER.normalized_deaths(
            events, rounds, {}, {}, {}, {"analysis_scope": "all_players"}, item_slots={327: "melee"}
        )

        self.assertEqual(deaths[0]["weapon_slot"], "melee")
        self.assertEqual(deaths[0]["weapon_slot_source"], "item_schema")
        self.assertTrue(ANALYZER.is_melee_kill(deaths[0]))

    def test_item_schema_is_discovered_from_demo_tf_directory(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            tf_directory = root / "Team Fortress 2" / "tf"
            schema = tf_directory / "scripts" / "items" / "items_game.txt"
            schema.parent.mkdir(parents=True)
            schema.write_text('"items_game" { "prefabs" { } "items" { } }', encoding="utf-8")
            demo = tf_directory / "demos" / "match.dem"
            demo.parent.mkdir()
            demo.write_bytes(b"")
            export = root / "export"
            export.mkdir()
            (export / "manifest.json").write_text(json.dumps({"source_demo": str(demo)}), encoding="utf-8")

            discovered = ANALYZER.discover_item_schema(export)

        self.assertEqual(discovered, schema.resolve())

    def test_non_melee_weapon_id_does_not_bypass_single_kill_filter(self):
        ranged = dict(
            self.kill(100, 2, "unknown_future_weapon"),
            weapon_id=22,
            weapon_def_index=9999,
            round_index=1,
            attacker_team="blu",
        )
        rounds = [{
            "round_index": 1,
            "live_start_tick": 1,
            "live_start_event": "in_progress_public_server",
            "round_active_tick": None,
            "setup_finished_tick": None,
            "activation_trigger": {"event": "opposing_team_player_death", "tick": 1},
            "ready_up": {},
            "end_tick": 1000,
            "end_reason": "demo_end_while_public_play_active",
        }]

        self.assertEqual(ANALYZER.build_candidates([ranged], rounds, {"analysis_scope": "pov_player_only"}), [])

    def test_market_garden_requires_blast_jump_state_and_is_not_random_crit(self):
        garden = dict(self.kill(100, 2, "market_gardener"), crit_type=2, state_evidence={"attacker_blast_jumping": True})

        score, tags, metrics, breakdown = ANALYZER.score_candidate([garden], {"end_tick": 1000})

        self.assertEqual(score, 30.0)
        self.assertIn("market_garden", tags)
        self.assertNotIn("melee_kill", tags)
        self.assertNotIn("random_full_crit", tags)
        self.assertEqual(metrics["market_gardens"], 1)
        self.assertIn("confirmed_market_garden", {item["reason"] for item in breakdown})

    def test_market_gardener_crit_without_jump_is_not_random_crit_or_market_garden(self):
        grounded = dict(self.kill(100, 2, "market_gardener"), crit_type=2, state_evidence={"attacker_blast_jumping": False})

        score, tags, metrics, _ = ANALYZER.score_candidate([grounded], {"end_tick": 1000})

        self.assertEqual(score, 25.0)
        self.assertNotIn("market_garden", tags)
        self.assertNotIn("random_full_crit", tags)
        self.assertEqual(metrics["market_gardens"], 0)

    def test_loose_cannon_double_donk_requires_direct_then_minicrit_explosion(self):
        timeline = ANALYZER.StateTimeline()
        timeline.players[1].append((100, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((100, {"team": "red", "class": "soldier", "life_state": "alive", "health": 200}))
        # player_death reports the actual impact/explosion variant rather
        # than the weapon_log_class_name used by the launcher.
        kill = dict(self.kill(130, 2, "loose_cannon_explosion"), attacker_class="demoman", attacker_team="blu", victim_team="red", weapon_id=996)

        ANALYZER.enrich_state_evidence([kill], [
            {"tick": 100, "event_type": "player_hurt", "event": {"attacker": 1, "user_id": 2, "weapon_id": 996, "damage_amount": 50, "mini_crit": False}},
            {"tick": 130, "event_type": "player_hurt", "event": {"attacker": 1, "user_id": 2, "weapon_id": 996, "damage_amount": 81, "mini_crit": True}},
        ], timeline)
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kill], {"end_tick": 1000})

        self.assertTrue(kill["state_evidence"]["confirmed_double_donk"])
        self.assertEqual(score, 36.0)
        self.assertIn("double_donk", tags)
        self.assertEqual(metrics["double_donks"], 1)
        self.assertIn("confirmed_loose_cannon_double_donk", {item["reason"] for item in breakdown})

    def test_loose_cannon_minicrit_without_direct_impact_is_not_a_double_donk(self):
        timeline = ANALYZER.StateTimeline()
        timeline.players[1].append((100, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((100, {"team": "red", "class": "soldier", "life_state": "alive", "health": 200}))
        kill = dict(self.kill(130, 2, "loose_cannon_explosion"), attacker_class="demoman", attacker_team="blu", victim_team="red", weapon_id=996)

        ANALYZER.enrich_state_evidence([kill], [
            {"tick": 130, "event_type": "player_hurt", "event": {"attacker": 1, "user_id": 2, "weapon_id": 996, "damage_amount": 81, "mini_crit": True}},
        ], timeline)

        self.assertFalse(kill["state_evidence"]["confirmed_double_donk"])

    def test_reused_projectile_entity_does_not_inherit_old_flight_time(self):
        timeline = ANALYZER.StateTimeline()
        timeline.projectiles[99] = [
            (10, {"entity_id": 99, "launcher_handle": 42, "projectile_type": "loosecannon", "position": [0, 0, 0], "state_tick": 10}),
            (20, {"entity_id": 99, "launcher_handle": 42, "projectile_type": "loosecannon", "position": [100, 0, 0], "state_tick": 20}),
            (200, {"entity_id": 99, "launcher_handle": 42, "projectile_type": "loosecannon", "position": [0, 0, 0], "state_tick": 200}),
            (220, {"entity_id": 99, "launcher_handle": 42, "projectile_type": "loosecannon", "position": [40, 0, 0], "state_tick": 220}),
        ]
        timeline.projectile_removals[99] = [21, 221]
        kill = dict(self.kill(222, 2, "loose_cannon_explosion"))

        evidence = ANALYZER.matching_projectile(
            timeline,
            kill,
            {"weapon_handles": [42]},
            {"position": [40, 0, 0]},
        )

        self.assertIsNotNone(evidence)
        self.assertEqual(evidence["launch_tick"], 200)
        self.assertEqual(evidence["flight_ticks"], 21)
        self.assertLess(evidence["flight_seconds"], 0.5)

    def test_taunt_and_shield_bash_do_not_receive_ordinary_melee_points(self):
        taunt = dict(self.kill(100, 2, "fists"), custom_kill=9)
        bash = dict(self.kill(200, 3, "demoshield"), attacker_class="demoman", custom_kill=23)

        _, taunt_tags, taunt_metrics, _ = ANALYZER.score_candidate([taunt], {"end_tick": 1000})
        _, bash_tags, bash_metrics, _ = ANALYZER.score_candidate([bash], {"end_tick": 1000})
        self.assertNotIn("melee_kill", taunt_tags)
        self.assertEqual(taunt_metrics["melee_kills"], 0)
        self.assertNotIn("melee_kill", bash_tags)
        self.assertEqual(bash_metrics["melee_kills"], 0)

    def test_kritzkrieg_kill_replaces_random_crit_penalty(self):
        kritz = dict(self.kill(200, 2, "rocketlauncher"), crit_type=2, state_evidence={
            "confirmed_kritzkrieg_boost": True,
            "kritzkrieg_deployments": [{"medic_user_id": 7, "event_tick": 100, "seconds_before_kill": 1.5}],
        })
        score, tags, metrics, breakdown = ANALYZER.score_candidate([kritz], {"end_tick": 1000})

        self.assertEqual(score, 26.0)
        self.assertIn("kritzkrieg_kill", tags)
        self.assertNotIn("random_full_crit", tags)
        self.assertEqual(metrics["kritzkrieg_kills"], 1)
        self.assertIn("confirmed_kritzkrieg_boosted_kill", {item["reason"] for item in breakdown})

    def test_kritzkrieg_requires_targeted_deployment_and_active_crit_boost(self):
        timeline = ANALYZER.StateTimeline()
        timeline.players[1] = [(200, {"user_id": 1, "team": "blu", "class": "soldier", "health": 200, "life_state": "alive", "kritz_boosted": True})]
        timeline.players[2] = [(200, {"user_id": 2, "team": "red", "class": "soldier", "health": 200, "life_state": "alive"})]
        timeline.players[7] = [(99, {"user_id": 7, "team": "blu", "class": "medic", "health": 150, "life_state": "alive", "medigun": "kritzkrieg"})]
        kill = dict(self.kill(200, 2, "rocketlauncher"), attacker_team="blu", victim_team="red", crit_type=2)
        events = [{"tick": 100, "event_type": "player_chargedeployed", "event": {"userid": 7, "targetid": 1}}]

        ANALYZER.enrich_state_evidence([kill], events, timeline)
        self.assertTrue(kill["state_evidence"]["confirmed_kritzkrieg_boost"])
        self.assertEqual(kill["state_evidence"]["kritzkrieg_deployments"][0]["medic_user_id"], 7)

    def test_shield_bash_uses_authoritative_custom_kill(self):
        bash = dict(self.kill(100, 2, "demoshield"), attacker_class="demoman", custom_kill=23)
        score, tags, metrics, breakdown = ANALYZER.score_candidate([bash], {"end_tick": 1000})

        self.assertEqual(score, 32.0)
        self.assertIn("demoknight", tags)
        self.assertIn("shield_bash_kill", tags)
        self.assertEqual(metrics["shield_bash_kills"], 1)
        self.assertIn("confirmed_shield_bash_kill", {item["reason"] for item in breakdown})

    def test_recent_shield_charge_confirms_melee_kill_without_random_crit_penalty(self):
        charge_melee = dict(
            self.kill(200, 2, "eyelander"),
            attacker_class="demoman",
            custom_kill=0,
            crit_type=2,
            state_evidence={
                "attacker_recent_shield_charge_tick": 180,
                "attacker_seconds_since_shield_charge": 0.3,
            },
        )
        score, tags, metrics, breakdown = ANALYZER.score_candidate([charge_melee], {"end_tick": 1000})

        self.assertEqual(score, 26.0)
        self.assertIn("charge_melee_kill", tags)
        self.assertNotIn("random_full_crit", tags)
        self.assertEqual(metrics["charge_melee_kills"], 1)
        self.assertIn("shield_charge_followed_by_melee_kill", {item["reason"] for item in breakdown})

    def test_state_timeline_retains_recent_ended_shield_charge(self):
        timeline = ANALYZER.StateTimeline()
        timeline.players[1] = [
            (150, {"user_id": 1, "team": "blu", "class": "demoman", "health": 175, "life_state": "alive", "shield_charging": True}),
            (190, {"user_id": 1, "team": "blu", "class": "demoman", "health": 175, "life_state": "alive", "shield_charging": False}),
        ]
        timeline.players[2] = [
            (190, {"user_id": 2, "team": "red", "class": "soldier", "health": 200, "life_state": "alive"}),
        ]
        kill = dict(self.kill(200, 2, "eyelander"), attacker_class="demoman", attacker_team="blu", victim_team="red")

        ANALYZER.enrich_state_evidence([kill], [], timeline)

        evidence = kill["state_evidence"]
        self.assertFalse(evidence["attacker_shield_charging"])
        self.assertEqual(evidence["attacker_recent_shield_charge_tick"], 150)
        self.assertEqual(evidence["attacker_seconds_since_shield_charge"], 0.75)

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
        self.assertIn("kills_to_secure_cap", tags)
        self.assertNotIn("payload_progress_followup", tags)
        self.assertEqual(metrics["objective_conversion_kind"], "kills_to_secure_cap")
        self.assertEqual(score, 34.0)
        self.assertEqual(
            {item["reason"] for item in breakdown},
            {"candidate_base", "kills_to_secure_cap"},
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

    def test_grounded_pipe_cannot_confirm_an_airshot(self):
        with tempfile.TemporaryDirectory() as temp:
            state_path = Path(temp) / "state_samples.ndjson"
            records = [
                {
                    "demo_tick": 180, "server_tick": 180,
                    "players": [
                        {"entity_id": 1, "user_id": 1, "team": "blue", "class": "demoman", "position": [0, 0, 0], "flags": 1, "on_ground": True, "health": 175, "life_state": "alive", "weapon_handles": [55, 0, 0]},
                        {"entity_id": 2, "user_id": 2, "team": "red", "class": "soldier", "position": [100, 0, 80], "velocity": [0, 0, 250], "flags": 0, "on_ground": False, "health": 80, "life_state": "alive", "weapon_handles": [0, 0, 0]},
                    ],
                    "projectiles": [{"entity_id": 10, "team": "blue", "projectile_type": "pipe", "position": [100, 0, 0], "initial_velocity": [1200, 0, 200], "launcher_handle": 55, "critical": False}],
                    "removed_projectiles": [],
                },
                {
                    "demo_tick": 195, "server_tick": 195, "players": [],
                    "projectiles": [{"entity_id": 10, "team": "blue", "projectile_type": "pipe", "position": [100, 0, 0], "initial_velocity": [1200, 0, 200], "launcher_handle": 55, "critical": False}],
                    "removed_projectiles": [],
                },
                {"demo_tick": 200, "server_tick": 200, "players": [], "projectiles": [], "removed_projectiles": [10]},
            ]
            state_path.write_text("\n".join(json.dumps(record) for record in records) + "\n", encoding="utf-8")
            timeline = ANALYZER.read_state_timeline(state_path)
        events = [
            {"tick": 90, "event_type": "round_start", "event": {}},
            {"tick": 100, "event_type": "teamplay_round_active", "event": {}},
            {"tick": 200, "event_type": "player_death", "event": {"attacker": 1, "user_id": 2, "weapon": "iron_bomber"}},
            {"tick": 1000, "event_type": "teamplay_round_win", "event": {}},
        ]
        rounds = ANALYZER.build_rounds(events)
        deaths = ANALYZER.normalized_deaths(events, rounds, {}, {}, {}, {"analysis_scope": "all_players"})
        ANALYZER.enrich_state_evidence(deaths, events, timeline)
        evidence = deaths[0]["state_evidence"]
        self.assertTrue(evidence["victim_airborne"])
        self.assertFalse(evidence["confirmed_airshot"])
        self.assertEqual(evidence["projectile"]["flight_state"], "grounded_or_stationary")
        score, tags, metrics, _ = ANALYZER.score_candidate(deaths, rounds[0])
        self.assertNotIn("confirmed_airshot", tags)
        self.assertEqual(metrics["confirmed_airshots"], 0)

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

    def test_state_counts_rank_wipe_and_player_count_swing(self):
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
        self.assertIn("player_count_swing", tags)

    def test_medic_force_replaces_player_count_swing_when_deployment_follows(self):
        kills = [
            dict(self.kill(200, 2), state_evidence={
                "enemy_alive_before": 5,
                "friendly_alive_before": 3,
                "enemy_state_roster": 6,
                "victim_next_respawn_tick": 700,
                "enemy_medic_force_followups": [{"event_tick": 280, "medic_user_id": 7, "charge_before_sequence": 100}],
            }),
            dict(self.kill(250, 3), state_evidence={"victim_next_respawn_tick": 720}),
        ]
        score, tags, metrics, breakdown = ANALYZER.score_candidate(kills, {"end_tick": 1000})
        self.assertEqual(score, 56.0)
        self.assertIn("medic_force", tags)
        self.assertNotIn("player_count_swing", tags)
        self.assertTrue(metrics["medic_force"])
        self.assertIn("enemy_medic_forced_uber_after_sequence", [item["reason"] for item in breakdown])

    def test_medic_force_requires_an_enemy_medic_deployment(self):
        timeline = ANALYZER.StateTimeline()
        timeline.players[1].append((100, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((100, {"team": "red", "class": "engineer", "life_state": "alive", "health": 125}))
        # This is the POV player's Medic. Its deployment is context only and
        # must never label the POV player's kill as a Medic force.
        timeline.players[3].append((100, {"team": "blu", "class": "medic", "life_state": "alive", "health": 150, "medic_charge": 100}))
        # The actual enemy Medic is present too, making team attribution
        # unambiguous for the regression check.
        timeline.players[4].append((100, {"team": "red", "class": "medic", "life_state": "alive", "health": 150, "medic_charge": 100}))
        kill = dict(self.kill(200, 2), attacker_team="blu", victim_team="red")
        rounds = [{"round_index": 1, "live_start_tick": 100, "end_tick": 1000}]

        ANALYZER.enrich_state_evidence(
            [kill],
            [{"tick": 220, "event_type": "player_chargedeployed", "event": {"user_id": 3}}],
            timeline,
            rounds=rounds,
        )
        self.assertEqual(kill["state_evidence"]["enemy_medic_force_followups"], [])

        ANALYZER.enrich_state_evidence(
            [kill],
            [
                {"tick": 215, "event_type": "player_hurt", "event": {"attacker": 1, "user_id": 4}},
                {"tick": 220, "event_type": "player_chargedeployed", "event": {"user_id": 4, "target_id": 2}},
            ],
            timeline,
            rounds=rounds,
        )
        force = kill["state_evidence"]["enemy_medic_force_followups"]
        self.assertEqual(len(force), 1)
        self.assertEqual(force[0]["medic_user_id"], 4)
        self.assertEqual(force[0]["medic_team"], "red")
        self.assertEqual(force[0]["forced_by_team"], "blu")
        self.assertEqual(force[0]["target_user_id"], 2)
        self.assertEqual(force[0]["pressure_event_ticks"], [215])

    def test_medic_force_requires_candidate_pressure_on_enemy_medic_or_target(self):
        timeline = ANALYZER.StateTimeline()
        timeline.players[1].append((100, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((100, {"team": "red", "class": "engineer", "life_state": "alive", "health": 125}))
        timeline.players[3].append((100, {"team": "red", "class": "medic", "life_state": "alive", "health": 150, "medic_charge": 100}))
        timeline.players[4].append((100, {"team": "red", "class": "soldier", "life_state": "alive", "health": 200}))
        kill = dict(self.kill(200, 2), attacker_team="blu", victim_team="red")
        rounds = [{"round_index": 1, "live_start_tick": 100, "end_tick": 1000}]

        ANALYZER.enrich_state_evidence(
            [kill],
            [{"tick": 220, "event_type": "player_chargedeployed", "event": {"user_id": 3, "target_id": 4}}],
            timeline,
            rounds=rounds,
            teams={3: [(100, "red")]},
        )
        self.assertEqual(kill["state_evidence"]["enemy_medic_force_followups"], [])

    def test_pov_medic_force_uses_recorder_team_at_deployment_tick(self):
        timeline = ANALYZER.StateTimeline()
        # The candidate's original kill data says BLU, but the recorder is
        # RED when the Medic deploys. A RED Medic is therefore friendly in
        # this POV context and cannot be a force.
        timeline.players[1].append((100, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((100, {"team": "red", "class": "engineer", "life_state": "alive", "health": 125}))
        timeline.players[3].append((100, {"team": "red", "class": "medic", "life_state": "alive", "health": 150, "medic_charge": 100}))
        timeline.players[9].append((100, {"team": "red", "class": "demoman", "life_state": "alive", "health": 175}))
        kill = dict(self.kill(200, 2), attacker_team="blu", victim_team="red")
        rounds = [{"round_index": 1, "live_start_tick": 100, "end_tick": 1000}]

        ANALYZER.enrich_state_evidence(
            [kill],
            [
                {"tick": 215, "event_type": "player_hurt", "event": {"attacker": 1, "user_id": 3}},
                {"tick": 220, "event_type": "player_chargedeployed", "event": {"user_id": 3, "target_id": 2}},
            ],
            timeline,
            rounds=rounds,
            teams={3: [(100, "red")], 9: [(100, "red")]},
            context={"analysis_scope": "pov_player_only", "pov_player_user_id": 9},
        )
        self.assertEqual(kill["state_evidence"]["enemy_medic_force_followups"], [])

    def test_medic_force_rejects_conflicting_stale_state_team(self):
        timeline = ANALYZER.StateTimeline()
        timeline.players[1].append((100, {"team": "blu", "class": "demoman", "life_state": "alive", "health": 175}))
        timeline.players[2].append((100, {"team": "red", "class": "engineer", "life_state": "alive", "health": 125}))
        # The state snapshot is stale and still says RED, but the game-event
        # team history resolves this Medic as BLU. Do not create a force tag.
        timeline.players[3].append((100, {"team": "red", "class": "medic", "life_state": "alive", "health": 150, "medic_charge": 100}))
        kill = dict(self.kill(200, 2), attacker_team="blu", victim_team="red")
        rounds = [{"round_index": 1, "live_start_tick": 100, "end_tick": 1000}]

        ANALYZER.enrich_state_evidence(
            [kill],
            [{"tick": 220, "event_type": "player_chargedeployed", "event": {"user_id": 3}}],
            timeline,
            rounds=rounds,
            teams={3: [(100, "blu")]},
        )
        self.assertEqual(kill["state_evidence"]["enemy_medic_force_followups"], [])

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
