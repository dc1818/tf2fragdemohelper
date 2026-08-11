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
ROUND_RESET_EVENTS = {
    "teamplay_round_start", "teamplay_restart_round", "teamplay_waiting_begins",
    "teamplay_round_restart_seconds", "round_start",
}
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
            item["tick"] = as_int(item.get("tick"))
            events.append(item)
    return sorted(events, key=lambda item: item["tick"])


def build_rounds(events: Iterable[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Build closed live intervals; setup_finished overrides provisional active start."""
    rounds: List[Dict[str, Any]] = []
    current: Optional[Dict[str, Any]] = None
    index = 0
    for item in events:
        name = event_name(item)
        tick = item["tick"]
        if name == "teamplay_round_active":
            if current is not None:
                current["end_tick"] = tick
                current["end_reason"] = "superseded_by_round_active"
                rounds.append(current)
            index += 1
            current = {
                "round_index": index,
                "provisional_start_tick": tick,
                "live_start_tick": tick,
                "setup_finished_tick": None,
                "end_tick": None,
                "end_reason": None,
            }
        elif name == "teamplay_setup_finished" and current is not None:
            current["setup_finished_tick"] = tick
            current["live_start_tick"] = tick
        elif name in ROUND_END_EVENTS and current is not None:
            current["end_tick"] = tick
            current["end_reason"] = name
            fields = event_fields(item)
            if "team" in fields:
                current["winning_team"] = as_int(fields.get("team"))
            rounds.append(current)
            current = None
        elif name in ROUND_RESET_EVENTS and current is not None:
            current["end_tick"] = tick
            current["end_reason"] = name
            rounds.append(current)
            current = None
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


def normalized_deaths(events: Iterable[Dict[str, Any]], rounds: List[Dict[str, Any]], classes: Dict[int, List[Tuple[int, str]]]) -> List[Dict[str, Any]]:
    deaths: List[Dict[str, Any]] = []
    for item in events:
        if event_name(item) != "player_death":
            continue
        fields = event_fields(item)
        tick = item["tick"]
        round_data = round_for_tick(rounds, tick)
        if round_data is None:
            continue
        attacker = as_int(fields.get("attacker"))
        victim = as_int(fields.get("user_id"))
        if attacker <= 0 or victim <= 0 or attacker == victim:
            continue
        weapon = as_text(fields.get("weapon")).lower()
        deaths.append({
            "tick": tick,
            "round_index": round_data["round_index"],
            "attacker_user_id": attacker,
            "victim_user_id": victim,
            "attacker_class": player_class_at(classes, attacker, tick),
            "victim_class": player_class_at(classes, victim, tick),
            "weapon": weapon,
            "weapon_id": as_int(fields.get("weapon_id")),
            "weapon_def_index": as_int(fields.get("weapon_def_index")),
            "custom_kill": as_int(fields.get("custom_kill")),
            "crit_type": as_int(fields.get("crit_type")),
            "rocket_jump_victim": bool(scalar(fields.get("rocket_jump", False))),
            "kill_streak_total": as_int(fields.get("kill_streak_total")),
            "assister_user_id": as_int(fields.get("assister")),
        })
    return deaths


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


def score_candidate(kills: List[Dict[str, Any]], round_data: Dict[str, Any]) -> Tuple[float, List[str], Dict[str, Any]]:
    tags = set()
    score = 10.0
    for kill in kills:
        tags.update(weapon_tags(kill["weapon"]))
        if kill["victim_class"] == "medic":
            score += 18.0
            tags.add("medic_pick")
        if kill["rocket_jump_victim"]:
            score += 10.0
            tags.add("rocket_jump_victim")
        if kill["kill_streak_total"] >= 10:
            score += 5.0
            tags.add("streak_10_plus")
        if kill["crit_type"] == 2:
            score -= 12.0
            tags.add("random_full_crit")
    if len(kills) > 1:
        score += 18.0 * (len(kills) - 1)
        tags.add("multi_kill")
    if len(kills) >= 3:
        score += 15.0
        tags.add("three_kill")
    if len(kills) >= 4:
        score += 25.0
        tags.add("four_kill_plus")
    duration_ticks = max(0, kills[-1]["tick"] - kills[0]["tick"])
    duration_seconds = duration_ticks / TICKS_PER_SECOND
    if len(kills) >= 2 and duration_seconds <= 2.0:
        score += 12.0
        tags.add("rapid_sequence")
    if any("projectile_kill" in weapon_tags(kill["weapon"]) for kill in kills):
        score += 8.0
    if round_data["end_tick"] - kills[-1]["tick"] <= round(TICKS_PER_SECOND * 8.0):
        score += 8.0
        tags.add("late_round")
    metrics = {
        "kills": len(kills),
        "duration_seconds": round(duration_seconds, 3),
        "unique_weapons": sorted({kill["weapon"] for kill in kills if kill["weapon"]}),
        "projectile_kills": sum("projectile_kill" in weapon_tags(kill["weapon"]) for kill in kills),
        "medic_kills": sum(kill["victim_class"] == "medic" for kill in kills),
        "full_crit_kills": sum(kill["crit_type"] == 2 for kill in kills),
    }
    return round(max(0.0, score), 2), sorted(tags), metrics


def build_candidates(deaths: List[Dict[str, Any]], rounds: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    by_attacker: Dict[Tuple[int, int], List[Dict[str, Any]]] = defaultdict(list)
    for death in deaths:
        by_attacker[(death["round_index"], death["attacker_user_id"])].append(death)
    round_lookup = {item["round_index"]: item for item in rounds}
    candidates: List[Dict[str, Any]] = []
    for (round_index, attacker), kills in by_attacker.items():
        kills.sort(key=lambda item: item["tick"])
        groups: List[List[Dict[str, Any]]] = []
        current: List[Dict[str, Any]] = []
        for kill in kills:
            if current and kill["tick"] - current[-1]["tick"] > SEQUENCE_GAP_TICKS:
                groups.append(current)
                current = []
            current.append(kill)
        if current:
            groups.append(current)
        round_data = round_lookup[round_index]
        for group in groups:
            score, tags, metrics = score_candidate(group, round_data)
            # Keep single kills only when they contain a meaningful known signal.
            if len(group) == 1 and score < 25.0:
                continue
            first_tick = group[0]["tick"]
            last_tick = group[-1]["tick"]
            candidates.append({
                "candidate_id": "r{}-p{}-t{}".format(round_index, attacker, first_tick),
                "round_index": round_index,
                "live_round": True,
                "start_tick": max(round_data["live_start_tick"], first_tick - PRE_ROLL_TICKS),
                "point_of_kill_ticks": [kill["tick"] for kill in group],
                "end_tick": min(round_data["end_tick"], last_tick + POST_ROLL_TICKS),
                "attacker_user_id": attacker,
                "attacker_class": group[0]["attacker_class"],
                "overall_score": score,
                "tags": tags,
                "metrics": metrics,
                "kills": group,
                "state_pass": {
                    "status": "pending",
                    "next": ["airshot confirmation", "projectile flight", "health", "position", "objective proximity"],
                },
            })
    return sorted(candidates, key=lambda item: (-item["overall_score"], item["start_tick"]))


def write_ndjson(path: Path, values: Iterable[Dict[str, Any]]) -> None:
    with path.open("w", encoding="utf-8", newline="\n") as output:
        for value in values:
            output.write(json.dumps(value, separators=(",", ":"), ensure_ascii=False))
            output.write("\n")


def main() -> int:
    parser = argparse.ArgumentParser(description="Rank live-round TF2 frag candidates from events.ndjson.")
    parser.add_argument("export_directory", type=Path, help="Folder produced by export_all.exe")
    arguments = parser.parse_args()
    export_directory = arguments.export_directory.resolve()
    events_path = export_directory / "events.ndjson"
    if not events_path.is_file():
        raise FileNotFoundError("events.ndjson is missing. Rebuild and run the updated parser first: {}".format(events_path))
    events = read_events(events_path)
    rounds = build_rounds(events)
    classes = class_history(events)
    deaths = normalized_deaths(events, rounds, classes)
    candidates = build_candidates(deaths, rounds)
    write_ndjson(export_directory / "frag_candidates.ndjson", candidates)
    summary = {
        "format": "tf2-frag-candidates",
        "format_version": 1,
        "source": "events.ndjson",
        "ticks_per_second_assumption": TICKS_PER_SECOND,
        "live_rounds": len(rounds),
        "live_round_kills": len(deaths),
        "candidate_count": len(candidates),
        "limitations": [
            "This event-only pass does not confirm airshots.",
            "Airshot, position, health, projectile-flight, and objective-proximity scoring require the planned packet-state pass.",
            "Only kills inside closed live-round intervals are candidates.",
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
