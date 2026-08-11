#!/usr/bin/env python3
"""Create ranked TF2 frag candidates from export_all's normalized events.ndjson.

This first pass deliberately uses only authoritative game events. It rejects
setup/post-round kills, groups live combat into compact sequences, and records
enough evidence for a later packet-state pass to confirm airshots, projectile
flight difficulty, health, positions, and objective proximity.
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


TICKS_PER_SECOND = 66.6666667
SEQUENCE_GAP_TICKS = round(TICKS_PER_SECOND * 4.0)
PRE_ROLL_TICKS = round(TICKS_PER_SECOND * 5.0)
POST_ROLL_TICKS = round(TICKS_PER_SECOND * 3.0)

ROUND_END_EVENTS = {
    "teamplay_round_win", "teamplay_round_stalemate", "teamplay_game_over",
    "tf_game_over", "game_end", "round_end",
}
ROUND_ACTIVATION_EVENTS = {
    "teamplay_round_start", "teamplay_restart_round", "teamplay_ready_restart",
    "teamplay_round_restart_seconds", "round_start",
}
ROUND_RESET_EVENTS = {
    "teamplay_waiting_begins",
}
BUILDING_DESTRUCTION_EVENTS = {
    "object_destroyed", "building_destroyed", "building_destruction",
}
READY_TEAM_NAMES = {2: "red", 3: "blu"}
PROJECTILE_WEAPONS = {
    "rocketlauncher", "directhit", "blackbox", "liberty_launcher", "airstrike",
    "grenadelauncher", "loch_n_load", "iron_bomber", "stickybomb_launcher",
    "quickiebomb_launcher", "flaregun", "detonator", "scorch_shot", "compound_bow",
    "crusaders_crossbow", "syringegun_medic", "rescue_ranger", "righteous_bison",
}
SPECIAL_WEAPON_TAGS = {
    "market_gardener": "market_garden",
    "axtinguisher": "axtinguisher",
    "backburner": "backburner",
    "ambassador": "ambassador",
    "kunai": "kunai",
    "eternal_reward": "eternal_reward",
    "tribalkukri": "tribalman's_shiv",
}


def scalar(value: Any) -> Any:
    """Unwrap common serde representations without guessing at unknown types."""
    if isinstance(value, dict) and len(value) == 1:
        return scalar(next(iter(value.values())))
    return value


def as_int(value: Any, default: int = 0) -> int:
    try:
        return int(scalar(value))
    except (TypeError, ValueError):
        return default


def as_text(value: Any) -> str:
    value = scalar(value)
    return value if isinstance(value, str) else ""


def as_bool(value: Any, default: bool = True) -> bool:
    """Decode the ready value without treating the string 'false' as true."""
    value = scalar(value)
    if isinstance(value, bool):
        return value
    if isinstance(value, (int, float)):
        return value != 0
    if isinstance(value, str):
        normalized = value.strip().lower()
        if normalized in {"true", "1", "yes"}:
            return True
        if normalized in {"false", "0", "no"}:
            return False
    return default


def event_fields(record: Dict[str, Any]) -> Dict[str, Any]:
    event = record.get("event", {})
    return event if isinstance(event, dict) else {}


def event_name(record: Dict[str, Any]) -> str:
    return as_text(record.get("event_type")).lower()


def read_events(path: Path) -> List[Dict[str, Any]]:
    events: List[Dict[str, Any]] = []
    with path.open("r", encoding="utf-8") as source:
        for line_number, line in enumerate(source, 1):
            line = line.strip()
            if not line:
                continue
            try:
                item = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValueError("Invalid JSON on events.ndjson line {}: {}".format(line_number, error))
            if not isinstance(item, dict):
                continue
            # Game events now carry both namespaces. `server_tick` is the
            # authoritative TF2 simulation tick; `demo_tick` is the packet
            # stream position and is retained for diagnostics only.
            item["demo_tick"] = as_int(item.get("demo_tick", item.get("tick")))
            item["server_tick"] = as_int(item.get("server_tick"), 0) if item.get("server_tick") is not None else 0
            item["tick"] = item["server_tick"] or item["demo_tick"]
            events.append(item)
    return sorted(
        events,
        key=lambda item: (
            item["tick"],
            as_int(item.get("packet_sequence")),
            as_int(item.get("event_index_in_packet")),
        ),
    )


def read_json_if_present(path: Path) -> Dict[str, Any]:
    if not path.is_file():
        return {}
    with path.open("r", encoding="utf-8") as source:
        value = json.load(source)
    return value if isinstance(value, dict) else {}


def build_rounds(events: Iterable[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Build closed playable intervals and retain tournament ready-up evidence.

    `teamplay_team_ready` only means a team has readied. It never opens a
    highlight window. A bare `teamplay_round_active` is also insufficient:
    TF2 emits one during map/warmup initialization. A window opens only when
    `teamplay_round_active` follows a real round-transition event, then moves
    to `teamplay_setup_finished` on maps where setup gates real combat.
    """
    rounds: List[Dict[str, Any]] = []
    current: Optional[Dict[str, Any]] = None
    index = 0
    ready_ticks: Dict[int, Optional[int]] = {team: None for team in READY_TEAM_NAMES}
    ready_restart_tick: Optional[int] = None
    countdown_tick: Optional[int] = None
    pending_activation: Optional[Dict[str, Any]] = None

    def clear_ready_up() -> None:
        nonlocal ready_ticks, ready_restart_tick, countdown_tick
        ready_ticks = {team: None for team in READY_TEAM_NAMES}
        ready_restart_tick = None
        countdown_tick = None

    def ready_up_evidence() -> Dict[str, Any]:
        red_tick = ready_ticks[2]
        blu_tick = ready_ticks[3]
        both_ready = red_tick is not None and blu_tick is not None
        return {
            "red_ready_tick": red_tick,
            "blu_ready_tick": blu_tick,
            "both_teams_ready": both_ready,
            "both_teams_ready_tick": max(red_tick, blu_tick) if both_ready else None,
            "ready_restart_tick": ready_restart_tick,
            "countdown_tick": countdown_tick,
        }

    for item in events:
        name = event_name(item)
        tick = item["tick"]
        fields = event_fields(item)
        if name == "teamplay_team_ready":
            team = as_int(fields.get("team"))
            if team in READY_TEAM_NAMES:
                ready_ticks[team] = tick if as_bool(fields.get("ready"), True) else None
        elif name in ROUND_ACTIVATION_EVENTS:
            if current is not None:
                current["end_tick"] = tick
                current["end_reason"] = name
                rounds.append(current)
                current = None
            if name == "teamplay_ready_restart":
                ready_restart_tick = tick
            elif name == "teamplay_round_restart_seconds":
                countdown_tick = tick
            pending_activation = {"event": name, "tick": tick}
        elif name == "teamplay_round_active":
            # The initial map/warmup activation has no preceding round
            # transition. Do not turn warmup deathmatch into frag candidates.
            if pending_activation is None:
                continue
            if current is not None:
                current["end_tick"] = tick
                current["end_reason"] = "superseded_by_round_active"
                rounds.append(current)
            index += 1
            current = {
                "round_index": index,
                "round_active_tick": tick,
                "live_start_tick": tick,
                "live_start_event": "teamplay_round_active",
                "setup_finished_tick": None,
                "ready_up": ready_up_evidence(),
                "activation_trigger": pending_activation,
                "end_tick": None,
                "end_reason": None,
            }
            pending_activation = None
            clear_ready_up()
        elif name == "teamplay_setup_finished" and current is not None:
            current["setup_finished_tick"] = tick
            current["live_start_tick"] = tick
            current["live_start_event"] = "teamplay_setup_finished"
        elif name in ROUND_END_EVENTS and current is not None:
            current["end_tick"] = tick
            current["end_reason"] = name
            if "team" in fields:
                current["winning_team"] = as_int(fields.get("team"))
            rounds.append(current)
            current = None
            clear_ready_up()
            pending_activation = None
        elif name in ROUND_RESET_EVENTS:
            if current is not None:
                current["end_tick"] = tick
                current["end_reason"] = name
                rounds.append(current)
                current = None
            clear_ready_up()
            pending_activation = None
    return [item for item in rounds if item["end_tick"] is not None and item["end_tick"] > item["live_start_tick"]]


def round_for_tick(rounds: Iterable[Dict[str, Any]], tick: int) -> Optional[Dict[str, Any]]:
    for round_data in rounds:
        if round_data["live_start_tick"] <= tick < round_data["end_tick"]:
            return round_data
    return None


def class_history(events: Iterable[Dict[str, Any]]) -> Dict[int, List[Tuple[int, str]]]:
    history: Dict[int, List[Tuple[int, str]]] = defaultdict(list)
    for item in events:
        if event_name(item) != "player_class":
            continue
        fields = event_fields(item)
        user_id = as_int(fields.get("user_id"))
        player_class = as_text(fields.get("class")).lower()
        if user_id and player_class:
            history[user_id].append((item["tick"], player_class))
    return history


def player_class_at(history: Dict[int, List[Tuple[int, str]]], user_id: int, tick: int) -> Optional[str]:
    found: Optional[str] = None
    for changed_tick, player_class in history.get(user_id, []):
        if changed_tick > tick:
            break
        found = player_class
    return found


def player_team_history(events: Iterable[Dict[str, Any]]) -> Dict[int, List[Tuple[int, Optional[str]]]]:
    """Keep the team value associated with each player_team game event."""
    history: Dict[int, List[Tuple[int, Optional[str]]]] = defaultdict(list)
    for item in events:
        if event_name(item) != "player_team":
            continue
        fields = event_fields(item)
        user_id = as_int(fields.get("user_id", fields.get("userid")))
        team = as_int(fields.get("team"))
        if user_id:
            history[user_id].append((item["tick"], READY_TEAM_NAMES.get(team)))
    return history


def player_team_at(history: Dict[int, List[Tuple[int, Optional[str]]]], user_id: int, tick: int) -> Optional[str]:
    found: Optional[str] = None
    for changed_tick, team in history.get(user_id, []):
        if changed_tick > tick:
            break
        found = team
    return found


def player_name_history(events: Iterable[Dict[str, Any]]) -> Dict[int, List[Tuple[int, str]]]:
    """Collect player names when the demo emits connection/name-change events."""
    history: Dict[int, List[Tuple[int, str]]] = defaultdict(list)
    for item in events:
        name = event_name(item)
        if name not in {"player_connect", "player_connect_client", "player_changename"}:
            continue
        fields = event_fields(item)
        user_id = as_int(fields.get("user_id", fields.get("userid")))
        player_name = as_text(fields.get("newname" if name == "player_changename" else "name"))
        if user_id and player_name:
            history[user_id].append((item["tick"], player_name))
    return history


def player_name_at(history: Dict[int, List[Tuple[int, str]]], user_id: int, tick: int) -> Optional[str]:
    found: Optional[str] = None
    for changed_tick, player_name in history.get(user_id, []):
        if changed_tick > tick:
            break
        found = player_name
    return found


def find_player_by_name(history: Dict[int, List[Tuple[int, str]]], name: str) -> Optional[int]:
    normalized = name.strip().casefold()
    if not normalized:
        return None
    matches = {
        user_id
        for user_id, changes in history.items()
        if any(player_name.casefold() == normalized for _, player_name in changes)
    }
    return next(iter(matches)) if len(matches) == 1 else None


def find_player_in_roster(roster: Dict[str, Any], name: str) -> Optional[int]:
    """Resolve header.nick from the parser's decoded userinfo string table."""
    normalized = name.strip().casefold()
    if not normalized or not isinstance(roster, dict):
        return None
    matches = set()
    for key, value in roster.items():
        if not isinstance(value, dict):
            continue
        if as_text(value.get("name")).casefold() != normalized:
            continue
        user_id = as_int(value.get("user_id", key), 0)
        if user_id > 0:
            matches.add(user_id)
    return next(iter(matches)) if len(matches) == 1 else None


def analysis_context(export_directory: Path, names: Dict[int, List[Tuple[int, str]]]) -> Dict[str, Any]:
    """Select all-player or POV-only analysis without guessing a POV identity."""
    manifest = read_json_if_present(export_directory / "manifest.json")
    header = read_json_if_present(export_directory / "header.json")
    roster = read_json_if_present(export_directory / "players.json")
    capture = manifest.get("demo_capture", {})
    capture = capture if isinstance(capture, dict) else {}
    capture_type = as_text(capture.get("classification")).lower() or "unknown"
    header_nick = as_text(header.get("nick"))
    pov_user_id: Optional[int] = None
    scope = "all_players"
    reason = "STV and unknown demos retain candidates from every player."
    if capture_type == "pov":
        pov_user_id = find_player_by_name(names, header_nick)
        if pov_user_id is None:
            pov_user_id = find_player_in_roster(roster, header_nick)
        if pov_user_id is not None:
            scope = "pov_player_only"
            reason = "POV recorder matched to player events or the decoded userinfo roster."
        else:
            reason = "POV recording detected, but the recorded player could not be matched safely; all players were retained."
    return {
        "capture_type": capture_type,
        "capture_confidence": as_text(capture.get("confidence")) or "unknown",
        "capture_evidence": capture.get("evidence", []),
        "header_nick": header_nick or None,
        "analysis_scope": scope,
        "pov_player_user_id": pov_user_id,
        "roster_match_available": bool(roster),
        "scope_reason": reason,
    }


def normalized_deaths(events: Iterable[Dict[str, Any]], rounds: List[Dict[str, Any]], classes: Dict[int, List[Tuple[int, str]]], teams: Dict[int, List[Tuple[int, Optional[str]]]], names: Dict[int, List[Tuple[int, str]]], context: Dict[str, Any], debug: bool = False) -> List[Dict[str, Any]]:
    deaths: List[Dict[str, Any]] = []
    pov_user_id = context.get("pov_player_user_id") if context.get("analysis_scope") == "pov_player_only" else None
    for item in events:
        if event_name(item) != "player_death":
            continue
        fields = event_fields(item)
        tick = item["tick"]
        round_data = round_for_tick(rounds, tick)
        if round_data is None:
            if debug:
                print("[candidate-debug] reject player_death tick={} reason=outside_live_round".format(tick))
            continue
        attacker = as_int(fields.get("attacker"))
        victim = as_int(fields.get("user_id"))
        if attacker <= 0 or victim <= 0 or attacker == victim:
            if debug:
                print("[candidate-debug] reject player_death tick={} attacker={} victim={} reason=invalid_or_world_damage".format(tick, attacker, victim))
            continue
        if pov_user_id is not None and victim == pov_user_id:
            if debug:
                print("[candidate-debug] reject player_death tick={} attacker={} victim={} reason=pov_recorder_death".format(tick, attacker, victim))
            continue
        if context["analysis_scope"] == "pov_player_only" and attacker != context["pov_player_user_id"]:
            if debug:
                print("[candidate-debug] reject player_death tick={} attacker={} reason=not_pov_attacker".format(tick, attacker))
            continue
        weapon = as_text(fields.get("weapon")).lower()
        deaths.append({
            # `tick` stays for compatibility. `event_tick` identifies the
            # precise game-event timestamp used for this kill.
            "tick": as_int(item.get("demo_tick", tick)),
            "event_tick": tick,
            "demo_tick": as_int(item.get("demo_tick", tick)),
            "server_tick": as_int(item.get("server_tick", tick)),
            "packet_sequence": as_int(item.get("packet_sequence")),
            "event_index_in_packet": as_int(item.get("event_index_in_packet")),
            "round_index": round_data["round_index"],
            "attacker_user_id": attacker,
            "victim_user_id": victim,
            "attacker_name": player_name_at(names, attacker, tick),
            "victim_name": player_name_at(names, victim, tick),
            "attacker_class": player_class_at(classes, attacker, tick),
            "victim_class": player_class_at(classes, victim, tick),
            "attacker_team": player_team_at(teams, attacker, tick),
            "victim_team": player_team_at(teams, victim, tick),
            "weapon": weapon,
            "weapon_id": as_int(fields.get("weapon_id")),
            "weapon_def_index": as_int(fields.get("weapon_def_index")),
            "custom_kill": as_int(fields.get("custom_kill")),
            "crit_type": as_int(fields.get("crit_type")),
            "rocket_jump_victim": bool(scalar(fields.get("rocket_jump", False))),
            "kill_streak_total": as_int(fields.get("kill_streak_total")),
            "assister_user_id": as_int(fields.get("assister")),
        })
        if debug:
            assister = as_int(fields.get("assister"))
            print("[candidate-debug] accept kill tick={} attacker={} victim={} assister={} weapon={}".format(tick, attacker, victim, assister or "none", weapon or "unknown"))
    return deaths


def normalized_building_destructions(events: Iterable[Dict[str, Any]], rounds: List[Dict[str, Any]], context: Dict[str, Any], debug: bool = False) -> List[Dict[str, Any]]:
    """Normalize building/object destruction events without treating them as kills."""
    destructions: List[Dict[str, Any]] = []
    pov_user_id = context.get("pov_player_user_id") if context.get("analysis_scope") == "pov_player_only" else None
    for item in events:
        if event_name(item) not in BUILDING_DESTRUCTION_EVENTS:
            continue
        tick = item["tick"]
        if round_for_tick(rounds, tick) is None:
            if debug:
                print("[candidate-debug] reject building tick={} reason=outside_live_round".format(tick))
            continue
        fields = event_fields(item)
        attacker = as_int(fields.get("attacker", fields.get("userid", fields.get("user_id"))))
        if context.get("analysis_scope") == "pov_player_only" and pov_user_id is not None and attacker != pov_user_id:
            if debug:
                print("[candidate-debug] reject building tick={} attacker={} reason=not_pov_attacker".format(tick, attacker))
            continue
        object_type = as_text(fields.get("objecttype", fields.get("object_type", fields.get("object")))).lower() or "building"
        destruction = {
            "event_tick": tick,
            "attacker_user_id": attacker,
            "object_type": object_type,
            "packet_sequence": as_int(item.get("packet_sequence")),
            "event_index_in_packet": as_int(item.get("event_index_in_packet")),
        }
        destructions.append(destruction)
        if debug:
            print("[candidate-debug] building destruction tick={} attacker={} type={} weight=low_until_followed_by_kills".format(tick, attacker, object_type))
    return destructions


def weapon_tags(weapon: str) -> List[str]:
    tags: List[str] = []
    if weapon in PROJECTILE_WEAPONS:
        tags.append("projectile_kill")
    if weapon in {"grenadelauncher", "loch_n_load", "iron_bomber"}:
        tags.append("pipe")
    if weapon in {"rocketlauncher", "directhit", "blackbox", "liberty_launcher", "airstrike"}:
        tags.append("rocket")
    if weapon in {"compound_bow", "huntsman"}:
        tags.append("huntsman")
    if weapon in {"crusaders_crossbow", "crossbow"}:
        tags.append("crossbow")
    if weapon in SPECIAL_WEAPON_TAGS:
        tags.append(SPECIAL_WEAPON_TAGS[weapon])
    return tags


def score_candidate(kills: List[Dict[str, Any]], round_data: Dict[str, Any], building_destructions: Optional[List[Dict[str, Any]]] = None) -> Tuple[float, List[str], Dict[str, Any], List[Dict[str, Any]]]:
    """Score one sequence and expose every contribution for auditing."""
    tags = set()
    score = 10.0
    breakdown: List[Dict[str, Any]] = [{"reason": "candidate_base", "points": 10.0}]
    for kill in kills:
        tags.update(weapon_tags(kill["weapon"]))
        if kill["victim_class"] == "medic":
            score += 18.0
            tags.add("medic_pick")
            breakdown.append({"reason": "medic_pick", "points": 18.0, "event_tick": kill["event_tick"]})
        if kill["rocket_jump_victim"]:
            score += 10.0
            tags.add("rocket_jump_victim")
            breakdown.append({"reason": "rocket_jump_victim", "points": 10.0, "event_tick": kill["event_tick"]})
        if kill["kill_streak_total"] >= 10:
            score += 5.0
            tags.add("streak_10_plus")
            breakdown.append({"reason": "streak_10_plus", "points": 5.0, "event_tick": kill["event_tick"]})
        if kill["crit_type"] == 2:
            score -= 12.0
            tags.add("random_full_crit")
            breakdown.append({"reason": "random_full_crit", "points": -12.0, "event_tick": kill["event_tick"]})
    if len(kills) > 1:
        multi_points = 18.0 * (len(kills) - 1)
        score += multi_points
        tags.add("multi_kill")
        breakdown.append({"reason": "additional_kills", "points": multi_points, "count": len(kills) - 1})
    if len(kills) >= 3:
        score += 15.0
        tags.add("three_kill")
        breakdown.append({"reason": "three_kill", "points": 15.0})
    if len(kills) >= 4:
        score += 25.0
        tags.add("four_kill_plus")
        breakdown.append({"reason": "four_kill_plus", "points": 25.0})
    duration_ticks = max(0, kills[-1]["event_tick"] - kills[0]["event_tick"])
    duration_seconds = duration_ticks / TICKS_PER_SECOND
    if len(kills) >= 2 and duration_seconds <= 2.0:
        score += 12.0
        tags.add("rapid_sequence")
        breakdown.append({"reason": "rapid_sequence", "points": 12.0})
    if any("projectile_kill" in weapon_tags(kill["weapon"]) for kill in kills):
        score += 8.0
        breakdown.append({"reason": "projectile_sequence", "points": 8.0})
    if round_data["end_tick"] - kills[-1]["event_tick"] <= round(TICKS_PER_SECOND * 8.0):
        score += 8.0
        tags.add("late_round")
        breakdown.append({"reason": "late_round", "points": 8.0})
    linked_buildings = [
        building for building in (building_destructions or [])
        if building["attacker_user_id"] == kills[0]["attacker_user_id"]
        and 0 <= kills[0]["event_tick"] - building["event_tick"] <= round(TICKS_PER_SECOND * 2.0)
    ]
    if linked_buildings:
        score += 6.0
        tags.add("building_to_kill_sequence")
        breakdown.append({"reason": "building_destruction_led_to_kills", "points": 6.0, "count": len(linked_buildings), "event_tick": linked_buildings[-1]["event_tick"]})
    raw_score = round(score, 2)
    metrics = {
        "kills": len(kills),
        "duration_seconds": round(duration_seconds, 3),
        "unique_weapons": sorted({kill["weapon"] for kill in kills if kill["weapon"]}),
        "projectile_kills": sum("projectile_kill" in weapon_tags(kill["weapon"]) for kill in kills),
        "medic_kills": sum(kill["victim_class"] == "medic" for kill in kills),
        "full_crit_kills": sum(kill["crit_type"] == 2 for kill in kills),
        "first_kill_tick": kills[0]["event_tick"],
        "last_kill_tick": kills[-1]["event_tick"],
        "score_before_floor": raw_score,
        "score_floor_applied": raw_score < 0.0,
        "linked_building_destructions": len(linked_buildings),
    }
    return round(max(0.0, raw_score), 2), sorted(tags), metrics, breakdown


def group_kills(kills: List[Dict[str, Any]]) -> List[List[Dict[str, Any]]]:
    """Group kills into non-overlapping first-to-last windows of at most four seconds.

    This intentionally measures from the first kill rather than repeatedly
    comparing adjacent kills. Otherwise a chain of kills can remain grouped
    indefinitely even though the first and last kills are far apart.
    """
    groups: List[List[Dict[str, Any]]] = []
    current: List[Dict[str, Any]] = []
    for kill in kills:
        if current and kill["event_tick"] - current[0]["event_tick"] > SEQUENCE_GAP_TICKS:
            groups.append(current)
            current = []
        current.append(kill)
    if current:
        groups.append(current)
    return groups


def build_candidates(deaths: List[Dict[str, Any]], rounds: List[Dict[str, Any]], context: Dict[str, Any], building_destructions: Optional[List[Dict[str, Any]]] = None, debug: bool = False) -> List[Dict[str, Any]]:
    by_attacker: Dict[Tuple[int, int], List[Dict[str, Any]]] = defaultdict(list)
    for death in deaths:
        by_attacker[(death["round_index"], death["attacker_user_id"])].append(death)
    round_lookup = {item["round_index"]: item for item in rounds}
    candidates: List[Dict[str, Any]] = []
    for (round_index, attacker), kills in by_attacker.items():
        kills.sort(key=lambda item: (item["event_tick"], item["packet_sequence"], item["event_index_in_packet"]))
        groups = group_kills(kills)
        if debug:
            print("[candidate-debug] grouping round={} attacker={} input_kills={} groups={}".format(round_index, attacker, len(kills), [[kill["event_tick"] for kill in group] for group in groups]))
        round_data = round_lookup[round_index]
        for group in groups:
            score, tags, metrics, score_breakdown = score_candidate(group, round_data, building_destructions)
            # Keep single kills only when they contain a meaningful known signal.
            if len(group) == 1 and score < 25.0:
                if debug:
                    print("[candidate-debug] discard group round={} attacker={} ticks={} kills={} score={} reason=single_kill_below_threshold".format(round_index, attacker, [kill["event_tick"] for kill in group], len(group), score))
                continue
            first_tick = group[0]["event_tick"]
            last_tick = group[-1]["event_tick"]
            first_demo_tick = group[0]["tick"]
            last_demo_tick = group[-1]["tick"]
            candidates.append({
                "candidate_id": "r{}-p{}-t{}".format(round_index, attacker, first_tick),
                "round_index": round_index,
                "live_round": True,
                "demo_context": context,
                "round_state": {
                    "classification": "live",
                    "start_tick": round_data["live_start_tick"],
                    "start_event": round_data["live_start_event"],
                    "round_active_tick": round_data["round_active_tick"],
                    "setup_finished_tick": round_data["setup_finished_tick"],
                    "activation_trigger": round_data["activation_trigger"],
                    "ready_up": round_data["ready_up"],
                    "end_tick": round_data["end_tick"],
                    "end_event": round_data["end_reason"],
                },
                "clip_start_tick": max(0, first_demo_tick - PRE_ROLL_TICKS),
                "clip_end_tick": last_demo_tick + POST_ROLL_TICKS,
                "start_tick": max(0, first_demo_tick - PRE_ROLL_TICKS),
                "first_kill_tick": first_demo_tick,
                "last_kill_tick": last_demo_tick,
                "first_kill_server_tick": first_tick,
                "last_kill_server_tick": last_tick,
                "point_of_kill_ticks": [kill["tick"] for kill in group],
                "point_of_kill_server_ticks": [kill["event_tick"] for kill in group],
                "point_of_kill_events": [
                    {
                        "tick": kill["tick"],
                        "demo_tick": kill.get("demo_tick"),
                        "server_tick": kill.get("server_tick"),
                        "packet_sequence": kill["packet_sequence"],
                        "event_index_in_packet": kill["event_index_in_packet"],
                    }
                    for kill in group
                ],
                "end_tick": last_demo_tick + POST_ROLL_TICKS,
                "attacker_user_id": attacker,
                "attacker_class": group[0]["attacker_class"],
                "attacker_team": group[0]["attacker_team"],
                "overall_score": score,
                "score_breakdown": score_breakdown,
                "tags": tags,
                "metrics": metrics,
                "kills": group,
                "building_destructions": [building for building in (building_destructions or []) if building.get("attacker_user_id") == attacker and first_tick - round(TICKS_PER_SECOND * 2.0) <= building["event_tick"] <= first_tick],
                "state_pass": {
                    "status": "pending",
                    "next": ["airshot confirmation", "projectile flight", "health", "position", "objective proximity"],
                },
            })
            if debug:
                print("[candidate-debug] candidate id={} round={} attacker={} kills={} exact_ticks={} score={} tags={}".format(candidates[-1]["candidate_id"], round_index, attacker, len(group), [kill["event_tick"] for kill in group], score, ",".join(tags) or "none"))
    return sorted(candidates, key=lambda item: (-item["overall_score"], item["start_tick"]))


def write_ndjson(path: Path, values: Iterable[Dict[str, Any]]) -> None:
    with path.open("w", encoding="utf-8", newline="\n") as output:
        for value in values:
            output.write(json.dumps(value, separators=(",", ":"), ensure_ascii=False))
            output.write("\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="Rank live-round TF2 frag candidates from events.ndjson.")
    parser.add_argument("export_directory", type=Path, help="Folder produced by export_all.exe")
    parser.add_argument("--debug", action="store_true", help="Print candidate acceptance, rejection, grouping, and scoring decisions")
    arguments = parser.parse_args()
    export_directory = arguments.export_directory.resolve()
    events_path = export_directory / "events.ndjson"
    if not events_path.is_file():
        raise FileNotFoundError("events.ndjson is missing. Rebuild and run the updated parser first: {}".format(events_path))
    events = read_events(events_path)
    rounds = build_rounds(events)
    classes = class_history(events)
    teams = player_team_history(events)
    names = player_name_history(events)
    context = analysis_context(export_directory, names)
    if arguments.debug:
        print("[candidate-debug] demo capture={} scope={} pov_user_id={} header_nick={}".format(context["capture_type"], context["analysis_scope"], context.get("pov_player_user_id") or "none", context.get("header_nick") or "unknown"))
        for round_data in rounds:
            print("[candidate-debug] live round #{} start={} ({}) end={} ({})".format(round_data["round_index"], round_data["live_start_tick"], round_data["live_start_event"], round_data["end_tick"], round_data["end_reason"]))
    deaths = normalized_deaths(events, rounds, classes, teams, names, context, arguments.debug)
    building_destructions = normalized_building_destructions(events, rounds, context, arguments.debug)
    candidates = build_candidates(deaths, rounds, context, building_destructions, arguments.debug)
    write_ndjson(export_directory / "frag_candidates.ndjson", candidates)
    summary = {
        "format": "tf2-frag-candidates",
        "format_version": 1,
        "source": "events.ndjson",
        "demo_context": context,
        "ticks_per_second_assumption": TICKS_PER_SECOND,
        "live_rounds": len(rounds),
        "live_round_kills": len(deaths),
        "live_round_building_destructions": len(building_destructions),
        "candidate_count": len(candidates),
        "limitations": [
            "This event-only pass does not confirm airshots.",
            "Airshot, position, health, projectile-flight, and objective-proximity scoring require the planned packet-state pass.",
            "Only kills inside closed playable intervals are candidates; team ready-up and countdown events are recorded as evidence but never open an interval.",
            "Kill ticks are the original player_death event ticks. Packet sequence and event-index fields preserve ordering when multiple events share one tick.",
            "POV-only filtering is enabled only when the demo is identified as POV and its header nickname resolves to exactly one player event; otherwise all players are retained.",
        ],
    }
    with (export_directory / "frag_summary.json").open("w", encoding="utf-8", newline="\n") as output:
        json.dump(summary, output, indent=2, ensure_ascii=False)
        output.write("\n")
    print("Analyzed {} events: {} live rounds, {} live-round kills, {} candidates.".format(len(events), len(rounds), len(deaths), len(candidates)))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print("ERROR: {}".format(error), file=sys.stderr)
        sys.exit(1)
