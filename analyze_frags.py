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
import math
import os
import re
import sys
import time
from concurrent.futures import ProcessPoolExecutor
from bisect import bisect_left, bisect_right
from collections import defaultdict
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional, Tuple


TICKS_PER_SECOND = 66.6666667
SEQUENCE_GAP_TICKS = round(TICKS_PER_SECOND * 4.0)
PRE_ROLL_TICKS = round(TICKS_PER_SECOND * 5.0)
POST_ROLL_TICKS = round(TICKS_PER_SECOND * 3.0)
BOOKMARK_SCORE = 30.0
BOOKMARK_LINK_BEFORE_TICKS = round(TICKS_PER_SECOND * 8.0)
BOOKMARK_LINK_AFTER_TICKS = round(TICKS_PER_SECOND * 2.0)
OBJECTIVE_CONVERSION_TICKS = round(TICKS_PER_SECOND * 8.0)
CAPTURE_DENIAL_TICKS = round(TICKS_PER_SECOND * 2.0)
ROUND_CLINCH_TICKS = round(TICKS_PER_SECOND * 3.0)
SACK_RECOVERY_TICKS = round(TICKS_PER_SECOND * 10.0)
DUPLICATE_DEATH_TICKS = round(TICKS_PER_SECOND * 2.0)
SACK_MIN_FRIENDLY_LOSSES = 2
UBER_ADVANTAGE_CHARGE_GAP = 25
MEDIC_FORCE_FOLLOWUP_TICKS = round(TICKS_PER_SECOND * 4.0)
MEDIC_FORCE_PRESSURE_TICKS = round(TICKS_PER_SECOND * 2.0)
KRITZKRIEG_DURATION_TICKS = round(TICKS_PER_SECOND * 8.0)
DOUBLE_DONK_WINDOW_TICKS = round(TICKS_PER_SECOND * 0.5)
PLAYER_SWING_MIN_WINDOW_TICKS = round(TICKS_PER_SECOND * 4.0)
CHARGE_MELEE_FOLLOWUP_TICKS = round(TICKS_PER_SECOND * 0.85)
BACKSTAB_CUSTOM_KILL = 2
SHIELD_BASH_CUSTOM_KILL = 23
COMPETITIVE_CLASSES = {"scout", "soldier", "pyro", "demoman", "heavy", "engineer", "medic", "sniper", "spy"}
SIXES_COMPOSITION = {"scout": 2, "soldier": 2, "demoman": 1, "medic": 1}

ROUND_END_EVENTS = {
    "teamplay_round_win", "teamplay_round_stalemate", "teamplay_game_over",
    "tf_game_over", "game_end", "round_end",
}
ROUND_ACTIVATION_EVENTS = {
    "teamplay_round_start", "teamplay_restart_round", "teamplay_ready_restart",
    "teamplay_round_restart_seconds", "teamplay_waiting_ends", "round_start",
}
ROUND_RESET_EVENTS = {
    "teamplay_waiting_begins",
}
BUILDING_DESTRUCTION_EVENTS = {
    "object_destroyed", "building_destroyed", "building_destruction",
}
OBJECTIVE_CAPTURE_EVENTS = {"teamplay_point_captured"}
PAYLOAD_PROGRESS_EVENTS = {"payload_pushed"}
CAPTURE_DENIAL_EVENTS = {"teamplay_capture_blocked"}
READY_TEAM_NAMES = {2: "red", 3: "blu"}
PROJECTILE_WEAPONS = {
    "rocketlauncher", "directhit", "blackbox", "liberty_launcher", "airstrike",
    "grenadelauncher", "loch_n_load", "iron_bomber", "stickybomb_launcher",
    "quickiebomb_launcher", "flaregun", "detonator", "scorch_shot", "compound_bow",
    "crusaders_crossbow", "syringegun_medic", "rescue_ranger", "righteous_bison",
}
LOOSE_CANNON_WEAPONS = {"loose_cannon", "loose_cannon_impact", "loose_cannon_explosion"}
AIRSHOT_PROJECTILE_WEAPONS = {
    "rocketlauncher", "directhit", "blackbox", "liberty_launcher", "airstrike",
    "grenadelauncher", "loch_n_load", "iron_bomber", *LOOSE_CANNON_WEAPONS,
    "flaregun", "detonator", "scorch_shot", "compound_bow", "huntsman",
}
SPECIAL_WEAPON_TAGS = {
    "market_gardener": "market_gardener",
    "axtinguisher": "axtinguisher",
    "backburner": "backburner",
    "ambassador": "ambassador",
    "kunai": "kunai",
    "eternal_reward": "eternal_reward",
    "tribalkukri": "tribalman's_shiv",
}
DEMOMAN_MELEE_WEAPONS = {
    "bottle", "sword", "eyelander", "headtaker", "golfclub",
    "scotsmans_skullcutter", "skullcutter", "paintrain", "pain_train",
    "ullapool_caber", "battleaxe", "claidheamh_mor", "claidheamohmor", "half_zatoichi",
    "katana", "persian_persuader", "fryingpan", "golden_fryingpan",
    "saxxy", "conscientious_objector", "freedom_staff", "ham_shank",
    "memory_maker", "necro_smasher", "crossing_guard", "prinny_machete",
}
MELEE_WEAPONS = DEMOMAN_MELEE_WEAPONS | {
    "bat", "bat_wood", "sandman", "wrap_assassin", "atomizer", "fan_o_war",
    "holy_mackerel", "boston_basher", "three_rune_blade", "sun_on_a_stick",
    "candy_cane", "fish", "fists", "gloves", "holiday_punch", "kgb",
    "warrior_spirit", "eviction_notice", "fireaxe", "back_scratcher", "powerjack",
    "homewrecker", "maul", "neon_annihilator", "thirddegree", "volcano_fragment",
    "axtinguisher", "postal_pummeler", "shovel", "equalizer", "escape_plan",
    "market_gardener", "disciplinary_action", "wrench", "gunslinger",
    "southern_hospitality", "jag", "eureka_effect", "bonesaw", "ubersaw",
    "vita_saw", "amputator", "solemn_vow", "knife", "kunai", "eternal_reward",
    "wanga_prick", "big_earner", "spy_cicle", "black_rose", "sharp_dresser",
    "tribalkukri", "shahanshah", "bushwacka", "kukri",
}
# ETFWeaponType values from Valve's TF2 shared definitions. Unlike the
# killfeed/icon string, this identifies the weapon implementation and remains
# the same across reskins and item-schema display-name changes. Name matching
# remains a compatibility fallback for older exports that omitted weapon_id.
MELEE_WEAPON_IDS = {
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
    64,   # TF_WEAPON_SWORD
    72,   # TF_WEAPON_BAT_FISH
    74,   # TF_WEAPON_STICKBOMB (Ullapool Caber)
    82,   # TF_WEAPON_BAT_GIFTWRAP
    96,   # TF_WEAPON_HARVESTER_SAW
    104,  # TF_WEAPON_BREAKABLE_SIGN
    106,  # TF_WEAPON_SLAP
}
# `player_death.custom_kill` is the authoritative killfeed classification. Do
# not infer taunts from the equipped weapon: several taunt kills keep the
# weapon's ordinary name in the event. These are the legacy taunt kill values
# plus the dedicated taunt-attack values emitted by newer taunts.
TAUNT_CUSTOM_KILL_NAMES = {
    7: "hadouken", 9: "high_noon", 10: "grand_slam", 13: "fencing",
    15: "arrow_stab", 21: "grenade_taunt", 24: "barbarian_swing",
    29: "uberslice", 33: "engineer_guitar_smash", 38: "engineer_arm_impale",
    52: "armageddon", 60: "allclass_guitar_riff", 80: "gas_blast",
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


def canonical_team(value: Any) -> str:
    """Return the stable spelling used by event and state evidence."""
    team = as_text(value).lower()
    return "blu" if team in {"blue", "blu"} else team


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


KEYVALUES_TOKEN = re.compile(r'"(?:\\.|[^"\\])*"|//[^\r\n]*|[{}]|[^\s{}"]+')


def keyvalues_token_value(token: str) -> str:
    if token.startswith('"') and token.endswith('"'):
        return token[1:-1].replace(r'\"', '"').replace(r'\\', '\\')
    return token


def parse_keyvalues(text: str) -> Dict[str, Any]:
    """Parse the KeyValues subset used by TF2's signed items_game.txt."""
    tokens = [token for token in KEYVALUES_TOKEN.findall(text) if not token.startswith("//")]

    def parse_object(index: int, nested: bool) -> Tuple[Dict[str, Any], int]:
        result: Dict[str, Any] = {}
        while index < len(tokens):
            token = tokens[index]
            if token == "}":
                if not nested:
                    raise ValueError("Unexpected closing brace in item schema")
                return result, index + 1
            if token == "{" or (token.startswith("[") and token.endswith("]")):
                index += 1
                continue
            key = keyvalues_token_value(token).lower()
            index += 1
            if index >= len(tokens):
                raise ValueError("Missing value for KeyValues key {!r}".format(key))
            if tokens[index] == "{":
                value, index = parse_object(index + 1, True)
            else:
                value = keyvalues_token_value(tokens[index])
                index += 1
            result[key] = value
            if index < len(tokens) and tokens[index].startswith("[") and tokens[index].endswith("]"):
                index += 1
        if nested:
            raise ValueError("Unclosed object in item schema")
        return result, index

    result, _ = parse_object(0, False)
    return result


def item_schema_slots(path: Path) -> Dict[int, str]:
    """Resolve defindex -> loadout slot, including inherited prefab fields."""
    with path.open("r", encoding="utf-8-sig", errors="replace") as source:
        parsed = parse_keyvalues(source.read())
    root = parsed.get("items_game", parsed)
    if not isinstance(root, dict):
        raise ValueError("items_game root object is missing")
    prefabs = root.get("prefabs", {})
    items = root.get("items", {})
    if not isinstance(prefabs, dict) or not isinstance(items, dict):
        raise ValueError("items_game prefabs/items objects are missing")

    def inherited_slot(record: Any, visited: Optional[set] = None) -> Optional[str]:
        if not isinstance(record, dict):
            return None
        direct = as_text(record.get("item_slot")).lower()
        if direct:
            return direct
        visited = set() if visited is None else set(visited)
        prefab_names = as_text(record.get("prefab")).split()
        # Later prefabs override earlier ones in Valve's merge order.
        for prefab_name in reversed(prefab_names):
            normalized = prefab_name.lower()
            if normalized in visited:
                continue
            visited.add(normalized)
            slot = inherited_slot(prefabs.get(normalized), visited)
            if slot:
                return slot
        return None

    slots: Dict[int, str] = {}
    for raw_defindex, record in items.items():
        defindex = as_int(raw_defindex, -1)
        if defindex < 0:
            continue
        slot = inherited_slot(record)
        if slot:
            slots[defindex] = slot
    return slots


def discover_item_schema(export_directory: Path, explicit_path: Optional[Path] = None) -> Optional[Path]:
    """Find TF2's current signed client schema without requiring a web API."""
    candidates: List[Path] = []
    if explicit_path is not None:
        candidates.append(explicit_path.expanduser())
    environment_path = os.environ.get("TF2_ITEM_SCHEMA", "").strip()
    if environment_path:
        candidates.append(Path(environment_path).expanduser())
    candidates.append(export_directory / "items_game.txt")

    manifest = read_json_if_present(export_directory / "manifest.json")
    source_demo = as_text(manifest.get("source_demo"))
    if source_demo:
        demo_path = Path(source_demo).expanduser()
        # Only infer the live TF2 schema when the demo is actually stored in
        # that installation's tf/demos tree.  An arbitrary uploaded demo can
        # share parent folder names with a TF2 install, so it must not cause us
        # to reach outside the selected input; the explicit schema option is
        # still available for that case.
        for parent in [demo_path.parent, *demo_path.parents]:
            if parent.name.lower() == "demos" and parent.parent.name.lower() == "tf":
                candidates.append(parent.parent / "scripts" / "items" / "items_game.txt")
                break

    seen = set()
    for candidate in candidates:
        try:
            resolved = candidate.resolve()
        except OSError:
            resolved = candidate
        normalized = os.path.normcase(str(resolved))
        if normalized in seen:
            continue
        seen.add(normalized)
        if resolved.is_file():
            return resolved
    return None


def load_item_schema(export_directory: Path, explicit_path: Optional[Path] = None, debug: bool = False) -> Tuple[Dict[int, str], Dict[str, Any]]:
    path = discover_item_schema(export_directory, explicit_path)
    if path is None:
        info = {
            "status": "unavailable",
            "path": None,
            "resolved_item_slots": 0,
            "fallback": "weapon_id_then_weapon_name",
        }
        if debug:
            print("[candidate-debug] item schema unavailable; weapon slot fallback=weapon_id_then_weapon_name")
        return {}, info
    try:
        slots = item_schema_slots(path)
    except (OSError, ValueError) as error:
        info = {
            "status": "invalid",
            "path": str(path),
            "resolved_item_slots": 0,
            "error": str(error),
            "fallback": "weapon_id_then_weapon_name",
        }
        if debug:
            print("[candidate-debug] item schema invalid path={} error={} fallback=weapon_id_then_weapon_name".format(path, error))
        return {}, info
    info = {
        "status": "loaded",
        "path": str(path),
        "resolved_item_slots": len(slots),
        "fallback": "weapon_id_then_weapon_name",
    }
    if debug:
        print("[candidate-debug] item schema loaded path={} resolved_item_slots={}".format(path, len(slots)))
    return slots, info


class StateTimeline:
    """Index parser-reconstructed state deltas by authoritative analysis tick."""

    def __init__(self) -> None:
        self.players: Dict[int, List[Tuple[int, Dict[str, Any]]]] = defaultdict(list)
        self.projectiles: Dict[int, List[Tuple[int, Dict[str, Any]]]] = defaultdict(list)
        self.projectile_removals: Dict[int, List[int]] = defaultdict(list)
        self.player_ticks: Dict[int, List[int]] = {}
        self.projectile_ticks: Dict[int, List[int]] = {}
        self.projectiles_by_launcher: Dict[int, set] = defaultdict(set)
        self.sample_count = 0
        self.ignored_out_of_range_samples = 0
        self.state_lookup_count = 0
        self.unindexed_state_lookup_count = 0
        self.projectile_lookup_calls = 0
        self.projectile_tracks_examined = 0

    def _at(self, history: List[Tuple[int, Dict[str, Any]]], tick: int, require_alive: bool = False,
            ticks: Optional[List[int]] = None) -> Optional[Dict[str, Any]]:
        if not history:
            return None
        self.state_lookup_count += 1
        if ticks is None:
            self.unindexed_state_lookup_count += 1
            ticks = [item[0] for item in history]
        index = bisect_right(ticks, tick) - 1
        while index >= 0:
            state = history[index][1]
            if not require_alive or (as_text(state.get("life_state")).lower() == "alive" and as_int(state.get("health")) > 0):
                return state
            index -= 1
        return None

    def player_at(self, user_id: int, tick: int, require_alive: bool = False) -> Optional[Dict[str, Any]]:
        return self._at(self.players.get(user_id, []), tick, require_alive, self.player_ticks.get(user_id))

    def projectile_at(self, entity_id: int, tick: int) -> Optional[Dict[str, Any]]:
        return self._at(self.projectiles.get(entity_id, []), tick, False, self.projectile_ticks.get(entity_id))

    def projectile_entities_for_handles(self, handles: Iterable[int]) -> List[int]:
        self.projectile_lookup_calls += 1
        # Unit tests and callers may construct StateTimeline directly instead
        # of using read_state_timeline(), in which case the launcher index has
        # not been finalized. Preserve the old correct behavior as a fallback.
        if self.projectiles and not self.projectiles_by_launcher:
            result = sorted(self.projectiles.keys())
            self.projectile_tracks_examined += len(result)
            return result
        entity_ids = set()
        for handle in handles:
            entity_ids.update(self.projectiles_by_launcher.get(as_int(handle), set()))
        result = sorted(entity_ids)
        self.projectile_tracks_examined += len(result)
        return result

    def last_player_flag_tick(self, user_id: int, tick: int, field: str, window_ticks: int) -> Optional[int]:
        """Find recent positive reconstructed state for a short causal window."""
        for state_tick, state in reversed(self.players.get(user_id, [])):
            if state_tick > tick:
                continue
            if tick - state_tick > window_ticks:
                break
            if bool(state.get(field)):
                return state_tick
        return None

    def team_counts_at(self, tick: int) -> Tuple[Dict[str, int], Dict[str, int]]:
        alive: Dict[str, int] = defaultdict(int)
        roster: Dict[str, int] = defaultdict(int)
        for user_id, history in self.players.items():
            state = self._at(history, tick, False, self.player_ticks.get(user_id))
            if state is None:
                continue
            team = canonical_team(state.get("team"))
            if team not in {"red", "blue", "blu"}:
                continue
            roster[team] += 1
            if as_text(state.get("life_state")).lower() == "alive" and as_int(state.get("health")) > 0:
                alive[team] += 1
        return dict(alive), dict(roster)

    def medic_charge_at(self, team: str, tick: int) -> Optional[int]:
        """Return the highest alive Medic charge for a team, if state knows it.

        We deliberately require an alive Medic. A stale 100% charge left on a
        dead entity is not an Uber advantage and must not create a sack tag.
        """
        charges = []
        for user_id, history in self.players.items():
            state = self._at(history, tick, require_alive=True, ticks=self.player_ticks.get(user_id))
            if state is None:
                continue
            if canonical_team(state.get("team")) != canonical_team(team):
                continue
            if as_text(state.get("class")).lower() != "medic":
                continue
            charge = state.get("medic_charge")
            if charge is not None:
                charges.append(as_int(charge))
        return max(charges) if charges else None

    def next_alive_tick(self, user_id: int, tick: int) -> Optional[int]:
        """Find the first observed respawn after a player's death tick."""
        for state_tick, state in self.players.get(user_id, []):
            if state_tick <= tick:
                continue
            if as_text(state.get("life_state")).lower() == "alive" and as_int(state.get("health")) > 0:
                return state_tick
        return None

    def respawn_ticks_for_dead_team(self, team: str, tick: int) -> List[int]:
        """Return observed future respawns for players dead at `tick`."""
        respawns: List[int] = []
        wanted_team = canonical_team(team)
        for user_id, history in self.players.items():
            state = self._at(history, tick, False, self.player_ticks.get(user_id))
            if state is None or canonical_team(state.get("team")) != wanted_team:
                continue
            alive = as_text(state.get("life_state")).lower() == "alive" and as_int(state.get("health")) > 0
            if alive:
                continue
            next_tick = self.next_alive_tick(user_id, tick)
            if next_tick is not None:
                respawns.append(next_tick)
        return sorted(respawns)


def read_state_timeline(path: Path, debug: bool = False, max_demo_tick: Optional[int] = None) -> StateTimeline:
    """Read state samples without letting a split-demo bootstrap tick poison state joins.

    Some map-change continuations begin with a signon/bootstrap sample whose
    ``demo_tick`` belongs to the preceding stream.  Prefer an explicit server
    tick whenever one exists.  If it is absent, reject a demo tick outside the
    current demo header's tick range instead of treating it as future state.
    """
    timeline = StateTimeline()
    if not path.is_file():
        if debug:
            print("[candidate-debug] state timeline unavailable path={}".format(path))
        return timeline
    entity_to_user: Dict[int, int] = {}
    with path.open("r", encoding="utf-8") as source:
        for line_number, line in enumerate(source, 1):
            line = line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValueError("Invalid JSON on state_samples.ndjson line {}: {}".format(line_number, error))
            server_tick_value = record.get("server_tick")
            has_server_tick = server_tick_value is not None
            demo_tick = as_int(record.get("demo_tick"))
            tick = as_int(server_tick_value) if has_server_tick else demo_tick
            if not has_server_tick and max_demo_tick is not None and not 0 <= demo_tick <= max_demo_tick:
                timeline.ignored_out_of_range_samples += 1
                continue
            timeline.sample_count += 1
            for player in record.get("players", []):
                if not isinstance(player, dict):
                    continue
                entity_id = as_int(player.get("entity_id"))
                user_id = as_int(player.get("user_id"))
                if user_id:
                    entity_to_user[entity_id] = user_id
                else:
                    user_id = entity_to_user.get(entity_id, 0)
                if user_id:
                    sample = dict(player)
                    sample["state_tick"] = tick
                    timeline.players[user_id].append((tick, sample))
            for projectile in record.get("projectiles", []):
                if not isinstance(projectile, dict):
                    continue
                entity_id = as_int(projectile.get("entity_id"))
                if entity_id:
                    sample = dict(projectile)
                    sample["state_tick"] = tick
                    timeline.projectiles[entity_id].append((tick, sample))
            for entity_id in record.get("removed_projectiles", []):
                timeline.projectile_removals[as_int(entity_id)].append(tick)
    # Packet exports are normally chronological, but a split-demo bootstrap
    # sample can precede the new stream's tick zero.  Binary searches require
    # the histories to be monotonic regardless of input order.
    for history in timeline.players.values():
        history.sort(key=lambda item: item[0])
    for user_id, history in timeline.players.items():
        timeline.player_ticks[user_id] = [item[0] for item in history]
    for entity_id, history in timeline.projectiles.items():
        history.sort(key=lambda item: item[0])
        timeline.projectile_ticks[entity_id] = [item[0] for item in history]
        for _, state in history:
            launcher_handle = as_int(state.get("launcher_handle"))
            if launcher_handle:
                timeline.projectiles_by_launcher[launcher_handle].add(entity_id)
    for removals in timeline.projectile_removals.values():
        removals.sort()
    if debug:
        print("[candidate-debug] state timeline samples={} ignored_out_of_range={} players={} projectiles={}".format(timeline.sample_count, timeline.ignored_out_of_range_samples, len(timeline.players), len(timeline.projectiles)))
    return timeline


def vector3(value: Any) -> Tuple[float, float, float]:
    if not isinstance(value, list) or len(value) < 3:
        return 0.0, 0.0, 0.0
    try:
        return float(value[0]), float(value[1]), float(value[2])
    except (TypeError, ValueError):
        return 0.0, 0.0, 0.0


def projectile_type_matches_weapon(projectile_type: str, weapon: str) -> bool:
    projectile_type = projectile_type.lower()
    if weapon in {"rocketlauncher", "directhit", "blackbox", "liberty_launcher", "airstrike"}:
        return projectile_type == "rocket"
    if weapon in {"grenadelauncher", "loch_n_load", "iron_bomber"}:
        return projectile_type == "pipe"
    if weapon in LOOSE_CANNON_WEAPONS:
        return projectile_type == "loosecannon"
    if weapon in {"flaregun", "detonator", "scorch_shot"}:
        return projectile_type == "flare"
    if weapon in {"compound_bow", "huntsman"}:
        return projectile_type in {"arrow", "unknown"}
    return False


def pipe_flight_state(history: List[Tuple[int, Dict[str, Any]]], impact_tick: int) -> str:
    """Classify pipe movement near impact without pretending a floor trap flew.

    Pipe entities do not expose a reliable per-tick velocity in the exported
    state, so this uses their reconstructed positions. A pipe that has not
    moved for several ticks, or that is moving flat along one Z level, is a
    grounded trap/roller rather than an airshot projectile.
    """
    recent = [item for item in history if impact_tick - round(TICKS_PER_SECOND * 0.35) <= item[0] <= impact_tick + 3]
    if not recent:
        return "insufficient_motion"
    last_tick, last_state = recent[-1]
    if impact_tick - last_tick > 4:
        return "grounded_or_stationary"
    if len(recent) < 2:
        return "insufficient_motion"
    z_values = [vector3(state.get("position"))[2] for _, state in recent]
    planar_distance = 0.0
    vertical_distance = 0.0
    for previous, current in zip(recent, recent[1:]):
        previous_position = vector3(previous[1].get("position"))
        current_position = vector3(current[1].get("position"))
        planar_distance += math.hypot(current_position[0] - previous_position[0], current_position[1] - previous_position[1])
        vertical_distance += abs(current_position[2] - previous_position[2])
    # A pipe travelling over ground can have horizontal movement, but no
    # meaningful vertical movement. Require a visible arc/bounce component.
    if vertical_distance < 6.0 and max(z_values) - min(z_values) < 6.0:
        return "grounded_or_stationary"
    if planar_distance < 2.0 and vertical_distance < 6.0:
        return "grounded_or_stationary"
    return "in_flight"


def matching_projectile(timeline: StateTimeline, kill: Dict[str, Any], attacker_state: Optional[Dict[str, Any]], victim_state: Optional[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    if attacker_state is None or victim_state is None:
        return None
    tick = kill["event_tick"]
    handles = {as_int(value) for value in attacker_state.get("weapon_handles", []) if as_int(value)}
    if not handles:
        return None
    victim_position = vector3(victim_state.get("position"))
    best: Optional[Tuple[float, Dict[str, Any]]] = None
    # Launcher handles are stable enough to pre-index projectile entities.
    # This avoids scanning every projectile track in the demo for every kill.
    for entity_id in timeline.projectile_entities_for_handles(handles):
        history = timeline.projectiles.get(entity_id, [])
        projectile = timeline.projectile_at(entity_id, tick + 3)
        if projectile is None or as_int(projectile.get("launcher_handle")) not in handles:
            continue
        if not projectile_type_matches_weapon(as_text(projectile.get("projectile_type")), kill["weapon"]):
            continue
        projectile_tick = as_int(projectile.get("state_tick"))
        removals = timeline.projectile_removals.get(entity_id, [])
        removal_distance = min((abs(removed_tick - tick) for removed_tick in removals), default=999999)
        if removal_distance > 5 and abs(projectile_tick - tick) > 5:
            continue
        projectile_position = vector3(projectile.get("position"))
        distance = math.sqrt(sum((projectile_position[index] - victim_position[index]) ** 2 for index in range(3)))
        if distance > 220.0:
            continue
        # Entity IDs are recycled. Keep only the lifetime containing the
        # projectile state matched to this kill; otherwise a new cannonball
        # could inherit the path and flight time of an older projectile with
        # the same entity ID.
        previous_removal = max((removed_tick for removed_tick in removals if removed_tick < projectile_tick), default=None)
        usable_history = [
            item for item in history
            if item[0] <= tick + 3 and (previous_removal is None or item[0] > previous_removal)
        ]
        path_distance = 0.0
        for previous, current in zip(usable_history, usable_history[1:]):
            previous_position = vector3(previous[1].get("position"))
            current_position = vector3(current[1].get("position"))
            path_distance += math.sqrt(sum((current_position[index] - previous_position[index]) ** 2 for index in range(3)))
        launch_tick = usable_history[0][0] if usable_history else projectile_tick
        impact_tick = min((removed_tick for removed_tick in removals if removed_tick >= projectile_tick), default=tick)
        projectile_type = as_text(projectile.get("projectile_type"))
        flight_state = pipe_flight_state(usable_history, tick) if projectile_type == "pipe" else "in_flight"
        evidence = {
            "entity_id": entity_id,
            "projectile_type": projectile_type,
            "launcher_handle": as_int(projectile.get("launcher_handle")),
            "last_state_tick": projectile_tick,
            "nearest_removal_tick_distance": removal_distance if removal_distance < 999999 else None,
            "distance_to_victim": round(distance, 2),
            "impact_proximity": "direct" if distance <= 64.0 else "splash",
            "launch_tick": launch_tick,
            "flight_ticks": max(0, impact_tick - launch_tick),
            "flight_seconds": round(max(0, impact_tick - launch_tick) / TICKS_PER_SECOND, 3),
            "tracked_path_distance": round(path_distance, 2),
            "flight_state": flight_state,
            "airshot_eligible": flight_state == "in_flight",
        }
        if best is None or distance < best[0]:
            best = distance, evidence
    return best[1] if best is not None else None


def enrich_state_evidence(deaths: List[Dict[str, Any]], events: List[Dict[str, Any]], timeline: StateTimeline, debug: bool = False, rounds: Optional[List[Dict[str, Any]]] = None, teams: Optional[Dict[int, List[Tuple[int, Optional[str]]]]] = None, context: Optional[Dict[str, Any]] = None) -> None:
    # Keep the original deployment event with its actor. A force must be
    # attributed to the Medic who deployed, not merely to an Uber-like state
    # seen on the POV player's team.
    deploys: Dict[int, List[Dict[str, Any]]] = defaultdict(list)
    deploys_by_target: Dict[int, List[Dict[str, Any]]] = defaultdict(list)
    deploy_events: List[Dict[str, Any]] = []
    hurt_events_by_pair: Dict[Tuple[int, int], List[Dict[str, Any]]] = defaultdict(list)
    charged_deaths = set()
    friendly_losses_by_round_team: Dict[Tuple[int, str], List[Tuple[int, int]]] = defaultdict(list)
    team_history = teams or {}
    pov_user_id = as_int((context or {}).get("pov_player_user_id")) if (context or {}).get("analysis_scope") == "pov_player_only" else 0
    for event in events:
        fields = event_fields(event)
        if event_name(event) == "player_chargedeployed":
            medic_user_id = as_int(fields.get("user_id", fields.get("userid")))
            if medic_user_id:
                deploy = {
                    "event_tick": event["tick"],
                    "target_user_id": as_int(fields.get("target_id", fields.get("targetid"))),
                    "medic_user_id": medic_user_id,
                }
                deploys[medic_user_id].append(deploy)
                deploy_events.append(deploy)
                if deploy["target_user_id"]:
                    deploys_by_target[deploy["target_user_id"]].append(deploy)
        elif event_name(event) == "player_hurt":
            attacker = as_int(fields.get("attacker"))
            victim = as_int(fields.get("user_id", fields.get("userid")))
            if attacker and victim and attacker != victim:
                hurt = {
                    "event_tick": event["tick"],
                    "attacker_user_id": attacker,
                    "victim_user_id": victim,
                    "weapon_id": as_int(fields.get("weapon_id", fields.get("weaponid"))),
                    "damage_amount": as_int(fields.get("damage_amount", fields.get("damageamount"))),
                    "mini_crit": as_bool(fields.get("mini_crit", fields.get("minicrit"))),
                }
                hurt_events_by_pair[(attacker, victim)].append(hurt)
        elif event_name(event) == "medic_death" and as_bool(fields.get("charged"), False):
            charged_deaths.add((event["tick"], as_int(fields.get("user_id", fields.get("userid")))))
        elif event_name(event) == "player_death":
            tick = event["tick"]
            round_data = round_for_tick(rounds, tick) if rounds is not None else None
            if rounds is not None and round_data is None:
                continue
            victim = as_int(fields.get("user_id", fields.get("userid")))
            attacker = as_int(fields.get("attacker"))
            # A recovery should follow an enemy's successful sacrifice, not a
            # killbind, world death, or an unrelated respawn transition.
            if not victim or not attacker or attacker == victim:
                continue
            victim_state = timeline.player_at(victim, max(0, tick - 1), require_alive=True)
            victim_team = canonical_team(victim_state.get("team")) if victim_state is not None else ""
            if victim_team in {"red", "blu"}:
                friendly_losses_by_round_team[(as_int(round_data.get("round_index")) if round_data else 0, victim_team)].append((tick, victim))

    deploy_events.sort(key=lambda item: item["event_tick"])
    deploy_event_ticks = [item["event_tick"] for item in deploy_events]
    for values in deploys.values():
        values.sort(key=lambda item: item["event_tick"])
    for values in deploys_by_target.values():
        values.sort(key=lambda item: item["event_tick"])
    for values in hurt_events_by_pair.values():
        values.sort(key=lambda item: item["event_tick"])
    friendly_loss_ticks: Dict[Tuple[int, str], List[int]] = {}
    for key, values in friendly_losses_by_round_team.items():
        values.sort(key=lambda item: item[0])
        friendly_loss_ticks[key] = [item[0] for item in values]

    for kill in deaths:
        tick = kill["event_tick"]
        attacker = kill["attacker_user_id"]
        victim = kill["victim_user_id"]
        attacker_state = timeline.player_at(attacker, tick, require_alive=True)
        victim_state = timeline.player_at(victim, tick, require_alive=True)
        if attacker_state is not None:
            kill["attacker_class"] = kill.get("attacker_class") or as_text(attacker_state.get("class")) or None
            state_team = as_text(attacker_state.get("team")).lower()
            kill["attacker_team"] = kill.get("attacker_team") or ("blu" if state_team == "blue" else state_team or None)
        if victim_state is not None:
            kill["victim_class"] = kill.get("victim_class") or as_text(victim_state.get("class")) or None
            state_team = as_text(victim_state.get("team")).lower()
            kill["victim_team"] = kill.get("victim_team") or ("blu" if state_team == "blue" else state_team or None)

        airborne = bool(victim_state is not None and victim_state.get("on_ground") is False)
        projectile_evidence = None
        if airborne and kill["weapon"] in AIRSHOT_PROJECTILE_WEAPONS:
            projectile_evidence = matching_projectile(timeline, kill, attacker_state, victim_state)
        medic_charge = as_int(victim_state.get("medic_charge")) if victim_state is not None else 0
        deployed_recently = any(
            0 <= tick - deploy["event_tick"] <= round(TICKS_PER_SECOND * 2.0)
            for deploy in deploys.get(victim, [])
        )
        uber_drop = kill.get("victim_class") == "medic" and (
            (tick, victim) in charged_deaths or (medic_charge >= 95 and not deployed_recently)
        )
        # State deltas are written after the packet is applied. Use the prior
        # server tick for pre-frag alive counts so the victim is not already
        # removed from the situation we are measuring.
        alive_counts, roster_counts = timeline.team_counts_at(max(0, tick - 1))
        attacker_team = canonical_team(kill.get("attacker_team"))
        victim_team = canonical_team(kill.get("victim_team"))
        current_round_index = as_int(kill.get("round_index"))
        loss_key = (current_round_index, attacker_team)
        loss_values = friendly_losses_by_round_team.get(loss_key, [])
        loss_ticks = friendly_loss_ticks.get(loss_key, [])
        loss_start = bisect_left(loss_ticks, tick - SACK_RECOVERY_TICKS)
        loss_end = bisect_left(loss_ticks, tick)
        recent_losses = [item for item in loss_values[loss_start:loss_end] if 0 < tick - item[0] <= SACK_RECOVERY_TICKS]
        # A teammate can die twice in a long enough window. Count distinct
        # players so the label really means at least two teammates were lost.
        recent_loss_users = sorted({loss_user_id for _, loss_user_id in recent_losses})
        friendly_medic_charge = timeline.medic_charge_at(attacker_team, max(0, tick - 1))
        enemy_medic_charge = timeline.medic_charge_at(victim_team, max(0, tick - 1))
        enemy_uber_advantage = enemy_medic_charge is not None and (
            (enemy_medic_charge >= 95 and (friendly_medic_charge is None or friendly_medic_charge < 95))
            or (enemy_medic_charge >= 75 and (friendly_medic_charge is None or enemy_medic_charge - friendly_medic_charge >= UBER_ADVANTAGE_CHARGE_GAP))
        )
        force_followups = []
        deploy_start = bisect_left(deploy_event_ticks, tick)
        deploy_end = bisect_right(deploy_event_ticks, tick + MEDIC_FORCE_FOLLOWUP_TICKS)
        for deploy in deploy_events[deploy_start:deploy_end]:
                medic_user_id = as_int(deploy.get("medic_user_id"))
                deploy_tick = deploy["event_tick"]
                # State samples represent applied packet deltas. Read just
                # before the event so this validates the Medic's active team
                # and class at the actual deployment moment.
                medic_state = timeline.player_at(medic_user_id, max(0, deploy_tick - 1), require_alive=True)
                if medic_state is None:
                    if debug:
                        print("[candidate-debug] reject medic_force tick={} medic={} reason=no_active_medic_state".format(deploy_tick, medic_user_id))
                    continue
                medic_team = canonical_team(medic_state.get("team"))
                event_team = canonical_team(player_team_at(team_history, medic_user_id, deploy_tick))
                if medic_team not in {"red", "blu"} or as_text(medic_state.get("class")).lower() != "medic":
                    if debug:
                        print("[candidate-debug] reject medic_force tick={} medic={} team={} class={} reason=unresolved_or_not_medic".format(deploy_tick, medic_user_id, medic_team or "unknown", as_text(medic_state.get("class")) or "unknown"))
                    continue
                # The game-event team history is the stable ownership source.
                # If it disagrees with a packet snapshot, do not guess: a
                # stale team on a preserved entity must not turn a friendly
                # deployment into an enemy-Medic force.
                if event_team in {"red", "blu"} and event_team != medic_team:
                    if debug:
                        print("[candidate-debug] reject medic_force tick={} medic={} state_team={} event_team={} reason=team_state_disagreement".format(deploy_tick, medic_user_id, medic_team, event_team))
                    continue
                resolved_medic_team = event_team or medic_team
                # STV candidates use the fragger's team at the deploy tick.
                # POV candidates use the recorder's team at that same tick;
                # the historical kill team can be stale or precede a team
                # transition.
                reference_user_id = pov_user_id or attacker
                reference_team = resolved_team_at(timeline, team_history, reference_user_id, deploy_tick)
                if reference_team is None:
                    if debug:
                        print("[candidate-debug] reject medic_force tick={} medic={} reference_player={} reason=unresolved_reference_team".format(deploy_tick, medic_user_id, reference_user_id))
                    continue
                if resolved_medic_team == reference_team:
                    if debug:
                        print("[candidate-debug] reject medic_force tick={} medic={} team={} reference_team={} reference_player={} reason=friendly_medic_deployment".format(deploy_tick, medic_user_id, resolved_medic_team, reference_team, reference_user_id))
                    continue
                # A force is a response to pressure, not simply an Uber that
                # happened later in the fight. Require the candidate attacker
                # to have damaged either the enemy Medic or the target named
                # by this deployment immediately beforehand. This deliberately
                # avoids crediting a POV player's unrelated Engineer pick for
                # their own team's or an independent enemy Uber.
                target_user_id = deploy.get("target_user_id", 0)
                pressure_events = []
                pressure_seen = set()
                for pressure_victim in {medic_user_id, target_user_id}:
                    if not pressure_victim:
                        continue
                    for hurt in hurt_events_by_pair.get((attacker, pressure_victim), []):
                        hurt_tick = hurt["event_tick"]
                        if hurt_tick > deploy_tick:
                            break
                        if deploy_tick - hurt_tick > MEDIC_FORCE_PRESSURE_TICKS:
                            continue
                        pressure_key = (hurt_tick, pressure_victim, hurt.get("weapon_id"), hurt.get("damage_amount"), hurt.get("mini_crit"))
                        if pressure_key not in pressure_seen:
                            pressure_seen.add(pressure_key)
                            pressure_events.append(hurt)
                if not pressure_events:
                    if debug:
                        print("[candidate-debug] reject medic_force tick={} medic={} target={} attacker={} reason=no_direct_candidate_pressure".format(deploy_tick, medic_user_id, target_user_id or "unknown", attacker))
                    continue
                force_followups.append({
                    "event_tick": deploy_tick,
                    "medic_user_id": medic_user_id,
                    "medic_team": resolved_medic_team,
                    "forced_by_team": reference_team,
                    "reference_player_user_id": reference_user_id,
                    "target_user_id": target_user_id or None,
                    "pressure_event_ticks": [hurt["event_tick"] for hurt in pressure_events],
                    "charge_before_sequence": enemy_medic_charge,
                    "seconds_after_kill": round((deploy_tick - tick) / TICKS_PER_SECOND, 3),
                })
        victim_respawn_tick = timeline.next_alive_tick(victim, tick)
        friendly_respawn_ticks = timeline.respawn_ticks_for_dead_team(attacker_team, tick)
        shield_charge_active = bool(attacker_state is not None and attacker_state.get("shield_charging"))
        recent_shield_charge_tick = tick if shield_charge_active else timeline.last_player_flag_tick(
            attacker, tick, "shield_charging", CHARGE_MELEE_FOLLOWUP_TICKS
        )
        kritz_deployments = []
        if attacker_state is not None and bool(attacker_state.get("kritz_boosted")):
            for deploy in deploys_by_target.get(attacker, []):
                    deploy_tick = as_int(deploy.get("event_tick"))
                    if deploy_tick > tick:
                        break
                    if tick - deploy_tick > KRITZKRIEG_DURATION_TICKS:
                        continue
                    medic_user_id = as_int(deploy.get("medic_user_id"))
                    medic_state = timeline.player_at(medic_user_id, max(0, deploy_tick - 1), require_alive=True)
                    if medic_state is None or as_text(medic_state.get("class")).lower() != "medic":
                        continue
                    if as_text(medic_state.get("medigun")).lower() != "kritzkrieg":
                        continue
                    if canonical_team(medic_state.get("team")) != canonical_team(attacker_state.get("team")):
                        continue
                    kritz_deployments.append({
                        "medic_user_id": medic_user_id,
                        "event_tick": deploy_tick,
                        "seconds_before_kill": round((tick - deploy_tick) / TICKS_PER_SECOND, 3),
                    })
        double_donk_events = []
        if kill.get("weapon") in LOOSE_CANNON_WEAPONS:
            kill_weapon_id = as_int(kill.get("weapon_id"))
            matching_hurts = [
                hurt for hurt in hurt_events_by_pair.get((attacker, victim), [])
                if hurt["event_tick"] <= tick
                and (not kill_weapon_id or hurt["weapon_id"] == kill_weapon_id)
            ]
            for impact in matching_hurts:
                if impact["mini_crit"]:
                    continue
                for explosion in matching_hurts:
                    if not explosion["mini_crit"]:
                        continue
                    if not (impact["event_tick"] <= explosion["event_tick"] <= tick):
                        continue
                    if explosion["event_tick"] - impact["event_tick"] > DOUBLE_DONK_WINDOW_TICKS:
                        continue
                    # The Mini-Crit explosion must be the damage that actually
                    # produces this death, not an earlier unrelated donk.
                    if tick - explosion["event_tick"] > 1:
                        continue
                    double_donk_events.append({
                        "impact_tick": impact["event_tick"],
                        "explosion_tick": explosion["event_tick"],
                        "window_seconds": round((explosion["event_tick"] - impact["event_tick"]) / TICKS_PER_SECOND, 3),
                        "impact_damage": impact["damage_amount"],
                        "explosion_damage": explosion["damage_amount"],
                    })
        evidence = {
            "state_available": attacker_state is not None and victim_state is not None,
            "attacker_airborne": bool(attacker_state is not None and attacker_state.get("on_ground") is False),
            "attacker_vertical_velocity": vector3(attacker_state.get("velocity"))[2] if attacker_state is not None else 0.0,
            "attacker_scoped": bool(attacker_state is not None and attacker_state.get("scoped")),
            "attacker_blast_jumping": bool(attacker_state is not None and attacker_state.get("blast_jumping")),
            "attacker_kritz_boosted": bool(attacker_state is not None and attacker_state.get("kritz_boosted")),
            "double_donk_events": double_donk_events,
            "confirmed_double_donk": bool(double_donk_events),
            "kritzkrieg_deployments": kritz_deployments,
            "confirmed_kritzkrieg_boost": bool(kritz_deployments),
            "attacker_shield_charging": shield_charge_active,
            "attacker_recent_shield_charge_tick": recent_shield_charge_tick,
            "attacker_seconds_since_shield_charge": round((tick - recent_shield_charge_tick) / TICKS_PER_SECOND, 3) if recent_shield_charge_tick is not None else None,
            "victim_airborne": airborne,
            "confirmed_airshot": bool(projectile_evidence and projectile_evidence.get("airshot_eligible")),
            "projectile": projectile_evidence,
            "medic_charge_before_death": medic_charge if kill.get("victim_class") == "medic" else None,
            "uber_deployed_recently": deployed_recently if kill.get("victim_class") == "medic" else None,
            "confirmed_uber_drop": uber_drop,
            "friendly_alive_before": alive_counts.get(attacker_team, 0),
            "enemy_alive_before": alive_counts.get(victim_team, 0),
            "friendly_state_roster": roster_counts.get(attacker_team, 0),
            "enemy_state_roster": roster_counts.get(victim_team, 0),
            "recent_friendly_death_ticks": [loss_tick for loss_tick, _ in recent_losses],
            "recent_friendly_death_user_ids": recent_loss_users,
            "recent_friendly_death_count": len(recent_loss_users),
            "sack_recovery_window_seconds": round(SACK_RECOVERY_TICKS / TICKS_PER_SECOND, 2),
            "player_disadvantage_before": max(0, alive_counts.get(victim_team, 0) - alive_counts.get(attacker_team, 0)),
            "friendly_medic_charge_before": friendly_medic_charge,
            "enemy_medic_charge_before": enemy_medic_charge,
            "enemy_uber_advantage_before": enemy_uber_advantage,
            "enemy_medic_force_followups": force_followups,
            "victim_next_respawn_tick": victim_respawn_tick,
            "victim_respawn_seconds": round((victim_respawn_tick - tick) / TICKS_PER_SECOND, 3) if victim_respawn_tick is not None else None,
            "friendly_pending_respawn_ticks": friendly_respawn_ticks,
        }
        kill["state_evidence"] = evidence
        if debug:
            print("[candidate-debug] state kill tick={} attacker={} victim={} airborne={} projectile_match={} double_donk={} uber_drop={} kritz_boost={} alive={}:{} recent_friendly_deaths={} uber_disadvantage={} force_followups={} victim_respawn={}".format(tick, attacker, victim, airborne, bool(projectile_evidence), bool(double_donk_events), uber_drop, bool(kritz_deployments), evidence["friendly_alive_before"], evidence["enemy_alive_before"], evidence["recent_friendly_death_count"], enemy_uber_advantage, len(force_followups), victim_respawn_tick or "unknown"))


def build_rounds(events: Iterable[Dict[str, Any]], timeline: Optional[StateTimeline] = None, teams: Optional[Dict[int, List[Tuple[int, Optional[str]]]]] = None) -> List[Dict[str, Any]]:
    """Build closed playable intervals and retain tournament ready-up evidence.

    `teamplay_team_ready` only means a team has readied. It never opens a
    highlight window. A bare `teamplay_round_active` is also insufficient:
    TF2 emits one during map/warmup initialization. A window opens only when
    `teamplay_round_active` follows a real round-transition event, including
    Casual's end-of-waiting transition, then moves to
    `teamplay_setup_finished` on maps where setup gates real combat.
    """
    event_list = list(events)
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

    for item in event_list:
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
    last_tick = max((as_int(item.get("tick")) for item in event_list), default=0)
    if current is not None and last_tick >= current["live_start_tick"]:
        # A map change or a manual/demo-support stop can end the file while
        # the server round is still live. The next numbered demo is a new
        # stream, so close this file's confirmed interval at its own end.
        current["end_tick"] = max(current["live_start_tick"] + 1, last_tick + 1)
        current["end_reason"] = "demo_end_while_event_confirmed_round_active"
        rounds.append(current)

    closed_rounds = [item for item in rounds if item["end_tick"] is not None and item["end_tick"] > item["live_start_tick"]]
    if closed_rounds or timeline is None or not timeline.sample_count:
        return closed_rounds

    # A sign-on / warmup `teamplay_round_active` is not evidence that a match
    # has started.  This matters on tournament servers: players can fight
    # while the F4 ready-up prompt is still visible.  If that initial active
    # event was seen but it never followed a real round transition, require a
    # later explicit transition instead of falling back to the first PvP kill.
    # The public-server fallback remains available for recordings that begin
    # mid-round and contain no round-state event at all.
    if any(event_name(item) == "teamplay_round_active" for item in event_list):
        return closed_rounds

    # Community/public demos commonly begin after a CTF or payload round is
    # already underway and end before it resets. In that case there is no
    # round-transition event inside the recording to form a closed interval.
    # Do not assume that a demo begins live: require a real PvP death whose
    # participants are independently resolved to opposing TF2 teams by the
    # reconstructed packet state (or, when available, player_team events).
    team_history = teams or {}
    first_combat_tick: Optional[int] = None
    for item in event_list:
        if event_name(item) != "player_death":
            continue
        fields = event_fields(item)
        attacker = as_int(fields.get("attacker"))
        victim = as_int(fields.get("user_id", fields.get("userid")))
        tick = as_int(item.get("tick"))
        if not attacker or not victim or attacker == victim:
            continue
        attacker_state = timeline.player_at(attacker, max(0, tick - 1))
        victim_state = timeline.player_at(victim, max(0, tick - 1))
        attacker_team = canonical_team(attacker_state.get("team")) if attacker_state is not None else None
        victim_team = canonical_team(victim_state.get("team")) if victim_state is not None else None
        attacker_team = attacker_team or player_team_at(team_history, attacker, tick)
        victim_team = victim_team or player_team_at(team_history, victim, tick)
        if attacker_team in {"red", "blu"} and victim_team in {"red", "blu"} and attacker_team != victim_team:
            first_combat_tick = tick
            break
    if first_combat_tick is None:
        return closed_rounds

    last_tick = max((as_int(item.get("tick")) for item in event_list), default=first_combat_tick)
    return [{
        "round_index": 1,
        "round_active_tick": first_combat_tick,
        "live_start_tick": first_combat_tick,
        "live_start_event": "in_progress_public_server",
        "setup_finished_tick": None,
        "ready_up": {
            "red_ready_tick": None,
            "blu_ready_tick": None,
            "both_teams_ready": False,
            "both_teams_ready_tick": None,
            "ready_restart_tick": None,
            "countdown_tick": None,
        },
        "activation_trigger": {
            "event": "state_confirmed_opposing_team_death",
            "tick": first_combat_tick,
        },
        "end_tick": max(first_combat_tick + 1, last_tick + 1),
        "end_reason": "demo_end_while_public_play_active",
    }]


class LiveRoundIndex:
    """Binary-searchable live-round lookup used by all early analysis gates."""

    def __init__(self, rounds: Iterable[Dict[str, Any]]) -> None:
        self.rounds = sorted(list(rounds), key=lambda item: as_int(item.get("live_start_tick")))
        self.starts = [as_int(item.get("live_start_tick")) for item in self.rounds]

    def at(self, tick: int) -> Optional[Dict[str, Any]]:
        if not self.rounds:
            return None
        index = bisect_right(self.starts, tick) - 1
        if index < 0:
            return None
        round_data = self.rounds[index]
        return round_data if as_int(round_data.get("live_start_tick")) <= tick < as_int(round_data.get("end_tick")) else None


def round_for_tick(rounds: Any, tick: int) -> Optional[Dict[str, Any]]:
    if isinstance(rounds, LiveRoundIndex):
        return rounds.at(tick)
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


def resolved_team_at(timeline: StateTimeline, teams: Dict[int, List[Tuple[int, Optional[str]]]], user_id: int, tick: int) -> Optional[str]:
    """Resolve a player's team at one tick without trusting a stale source."""
    state = timeline.player_at(user_id, max(0, tick - 1), require_alive=True)
    state_team = canonical_team(state.get("team")) if state is not None else ""
    event_team = canonical_team(player_team_at(teams, user_id, tick))
    if state_team in {"red", "blu"} and event_team in {"red", "blu"} and state_team != event_team:
        return None
    resolved = event_team or state_team
    return resolved if resolved in {"red", "blu"} else None


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


def competitive_roster_at(timeline: StateTimeline, tick: int) -> Dict[str, Dict[str, int]]:
    result: Dict[str, Dict[str, int]] = {"red": defaultdict(int), "blu": defaultdict(int)}
    for user_id in timeline.players:
        state = timeline.player_at(user_id, tick, require_alive=False)
        if state is None:
            continue
        team = canonical_team(state.get("team"))
        player_class = as_text(state.get("class")).lower()
        if team in result and player_class in COMPETITIVE_CLASSES:
            result[team][player_class] += 1
    return {team: dict(classes) for team, classes in result.items()}


def competitive_mode_context(export_directory: Path, events: List[Dict[str, Any]], timeline: StateTimeline, rounds: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Classify the demo from sustained team composition and positive server evidence.

    Config names are not an authoritative field in Source demos.  RGL is only
    asserted when an RGL signature is actually present; otherwise the result
    is the verified format rather than a league guess.
    """
    header = read_json_if_present(export_directory / "header.json")
    server_name = as_text(header.get("server"))
    evidence_text = server_name.lower()
    for item in events:
        # Server config announcements are usually emitted as text/chat events.
        # Keep this narrow and deterministic instead of assuming every STV is RGL.
        name = event_name(item)
        if name in {"player_say", "say_text", "server_message", "server_cvar"}:
            evidence_text += " " + json.dumps(event_fields(item), ensure_ascii=False).lower()
    rgl_signature = "rgl" in evidence_text
    valve_signature = any(token in evidence_text for token in ("valve", "matchmaking", "casual"))

    sample_ticks: List[int] = []
    for round_data in rounds:
        start = as_int(round_data.get("live_start_tick"))
        end = as_int(round_data.get("end_tick"))
        if end <= start:
            continue
        count = min(12, max(3, int((end - start) / max(1, round(TICKS_PER_SECOND * 30.0))) + 1))
        for index in range(1, count + 1):
            sample_ticks.append(start + ((end - start) * index // (count + 1)))

    sixes_samples = 0
    highlander_samples = 0
    observed_samples = 0
    for tick in sample_ticks:
        rosters = competitive_roster_at(timeline, tick)
        if not rosters["red"] or not rosters["blu"]:
            continue
        observed_samples += 1
        if all(rosters[team] == SIXES_COMPOSITION for team in ("red", "blu")):
            sixes_samples += 1
        if all(sum(rosters[team].values()) == 9 and set(rosters[team]) == COMPETITIVE_CLASSES and all(count == 1 for count in rosters[team].values()) for team in ("red", "blu")):
            highlander_samples += 1

    sixes_ratio = sixes_samples / observed_samples if observed_samples else 0.0
    highlander_ratio = highlander_samples / observed_samples if observed_samples else 0.0
    mode = "unknown"
    label = "Unknown / Mixed"
    confidence = "low"
    evidence = []
    if highlander_samples >= 3 and highlander_ratio >= 0.60:
        mode = "rgl_highlander" if rgl_signature else "highlander"
        label = "RGL Highlander" if rgl_signature else "Highlander Competitive"
        confidence = "high" if highlander_ratio >= 0.85 else "medium"
        evidence.append("{} of {} sampled live states were exact 9v9 one-of-each-class rosters.".format(highlander_samples, observed_samples))
    elif sixes_samples >= 3 and sixes_ratio >= 0.60:
        mode = "rgl_6v6" if rgl_signature else "6v6"
        label = "RGL 6v6" if rgl_signature else "6v6 Competitive"
        confidence = "high" if sixes_ratio >= 0.85 else "medium"
        evidence.append("{} of {} sampled live states were 2 Scout, 2 Soldier, 1 Demoman, 1 Medic per team.".format(sixes_samples, observed_samples))
    elif rgl_signature:
        mode = "rgl_competitive"
        label = "RGL Competitive"
        confidence = "medium"
        evidence.append("RGL signature found, but the captured roster was not stable enough to prove a format.")
    elif valve_signature:
        mode = "valve_public"
        label = "Valve Public"
        confidence = "medium"
        evidence.append("Valve/matchmaking signature found in the demo server metadata.")
    elif server_name.strip():
        mode = "community_public"
        label = "Community Public"
        confidence = "medium"
        evidence.append("Non-Valve server metadata with no sustained competitive roster pattern.")

    if rgl_signature:
        evidence.insert(0, "RGL signature found in recorded server or server-message data.")
    elif mode in {"6v6", "highlander"}:
        evidence.insert(0, "No RGL signature was recorded; format is roster-confirmed only.")
    return {
        "mode": mode,
        "mode_label": label,
        "mode_confidence": confidence,
        "mode_evidence": evidence,
        "mode_roster_samples": observed_samples,
        "mode_6v6_samples": sixes_samples,
        "mode_highlander_samples": highlander_samples,
        "server_name": server_name or None,
    }


def normalized_deaths(events: Iterable[Dict[str, Any]], rounds: Any, classes: Dict[int, List[Tuple[int, str]]], teams: Dict[int, List[Tuple[int, Optional[str]]]], names: Dict[int, List[Tuple[int, str]]], context: Dict[str, Any], debug: bool = False, item_slots: Optional[Dict[int, str]] = None, profile: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
    deaths: List[Dict[str, Any]] = []
    pov_user_id = context.get("pov_player_user_id") if context.get("analysis_scope") == "pov_player_only" else None
    last_death_by_victim: Dict[int, int] = {}
    reject_counts: Dict[str, int] = defaultdict(int)
    total_player_deaths = 0
    for item in events:
        if event_name(item) != "player_death":
            continue
        total_player_deaths += 1
        fields = event_fields(item)
        tick = item["tick"]
        round_data = round_for_tick(rounds, tick)
        if round_data is None:
            reject_counts["outside_live_round"] += 1
            continue
        attacker = as_int(fields.get("attacker"))
        victim = as_int(fields.get("user_id"))
        if attacker <= 0 or victim <= 0 or attacker == victim:
            reject_counts["invalid_or_world_damage"] += 1
            continue
        if pov_user_id is not None and victim == pov_user_id:
            reject_counts["pov_recorder_death"] += 1
            continue
        if context["analysis_scope"] == "pov_player_only" and attacker != context["pov_player_user_id"]:
            reject_counts["not_pov_attacker"] += 1
            continue
        previous_death_tick = last_death_by_victim.get(victim)
        if previous_death_tick is not None and 0 <= tick - previous_death_tick <= DUPLICATE_DEATH_TICKS:
            reject_counts["duplicate_victim_death"] += 1
            continue
        last_death_by_victim[victim] = tick
        weapon = as_text(fields.get("weapon")).lower()
        weapon_id = as_int(fields.get("weapon_id"))
        weapon_def_index = as_int(fields.get("weapon_def_index"))
        death = {
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
            "weapon_id": weapon_id,
            "weapon_def_index": weapon_def_index,
            "custom_kill": as_int(fields.get("custom_kill")),
            "crit_type": as_int(fields.get("crit_type")),
            "rocket_jump_victim": bool(scalar(fields.get("rocket_jump", False))),
            "kill_streak_total": as_int(fields.get("kill_streak_total")),
            "assister_user_id": as_int(fields.get("assister")),
        }
        schema_slot = (item_slots or {}).get(weapon_def_index)
        if schema_slot:
            death["weapon_slot"] = schema_slot
            death["weapon_slot_source"] = "item_schema"
        classification_source = melee_classification(death)
        if classification_source and not schema_slot:
            death["weapon_slot"] = "melee"
            death["weapon_slot_source"] = classification_source
        deaths.append(death)
        if debug:
            assister = as_int(fields.get("assister"))
            print("[candidate-debug] accept kill tick={} attacker={} victim={} assister={} weapon={} weapon_id={} defindex={} slot={} slot_source={}".format(
                tick, attacker, victim, assister or "none", weapon or "unknown", weapon_id or "unknown",
                weapon_def_index or "unknown", death.get("weapon_slot", "unknown"), death.get("weapon_slot_source", "unknown")
            ))
    if profile is not None:
        profile["total_player_death_events"] = total_player_deaths
        profile["accepted_live_scope_kills"] = len(deaths)
        profile["death_rejections"] = dict(reject_counts)
    if debug:
        print("[candidate-debug] death gate summary total={} accepted={} outside_live_round={} not_pov_attacker={} pov_recorder_death={} invalid_or_world_damage={} duplicate_victim_death={} scope={}".format(
            total_player_deaths,
            len(deaths),
            reject_counts.get("outside_live_round", 0),
            reject_counts.get("not_pov_attacker", 0),
            reject_counts.get("pov_recorder_death", 0),
            reject_counts.get("invalid_or_world_damage", 0),
            reject_counts.get("duplicate_victim_death", 0),
            context.get("analysis_scope", "unknown"),
        ))
    return deaths


def normalized_building_destructions(events: Iterable[Dict[str, Any]], rounds: Any, context: Dict[str, Any], debug: bool = False, profile: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
    """Normalize building/object destruction events without treating them as kills."""
    destructions: List[Dict[str, Any]] = []
    pov_user_id = context.get("pov_player_user_id") if context.get("analysis_scope") == "pov_player_only" else None
    rejected_outside = 0
    rejected_scope = 0
    for item in events:
        if event_name(item) not in BUILDING_DESTRUCTION_EVENTS:
            continue
        tick = item["tick"]
        if round_for_tick(rounds, tick) is None:
            rejected_outside += 1
            continue
        fields = event_fields(item)
        attacker = as_int(fields.get("attacker", fields.get("userid", fields.get("user_id"))))
        if context.get("analysis_scope") == "pov_player_only" and pov_user_id is not None and attacker != pov_user_id:
            rejected_scope += 1
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
    if profile is not None:
        profile["building_rejections"] = {
            "outside_live_round": rejected_outside,
            "not_pov_attacker": rejected_scope,
        }
    if debug and (rejected_outside or rejected_scope):
        print("[candidate-debug] building gate summary accepted={} outside_live_round={} not_pov_attacker={} scope={}".format(
            len(destructions), rejected_outside, rejected_scope, context.get("analysis_scope", "unknown")
        ))
    return destructions


def team_id(team: Any) -> int:
    """Normalize the team values used by game events and player state."""
    value = scalar(team)
    if isinstance(value, str):
        normalized = value.strip().lower()
        if normalized == "red":
            return 2
        if normalized in {"blu", "blue"}:
            return 3
    return as_int(value)


def normalized_objective_events(events: Iterable[Dict[str, Any]], rounds: Any, teams: Dict[int, List[Tuple[int, Optional[str]]]], debug: bool = False) -> List[Dict[str, Any]]:
    """Keep live-round objective progress as evidence for kill-to-objective conversions."""
    objectives: List[Dict[str, Any]] = []
    for item in events:
        name = event_name(item)
        if name not in OBJECTIVE_CAPTURE_EVENTS and name not in PAYLOAD_PROGRESS_EVENTS and name not in CAPTURE_DENIAL_EVENTS:
            continue
        tick = item["tick"]
        if round_for_tick(rounds, tick) is None:
            continue
        fields = event_fields(item)
        if name in OBJECTIVE_CAPTURE_EVENTS:
            objective = {
                "event_tick": tick,
                "event_type": name,
                "kind": "point_capture",
                "team": as_int(fields.get("team")),
                "point": as_int(fields.get("cp")),
                "point_name": as_text(fields.get("cp_name")),
                "cappers": as_text(fields.get("cappers")),
            }
        elif name in PAYLOAD_PROGRESS_EVENTS:
            pusher = as_int(fields.get("pusher"))
            objective = {
                "event_tick": tick,
                "event_type": name,
                "kind": "payload_progress",
                "team": team_id(player_team_at(teams, pusher, tick)),
                "pusher_user_id": pusher,
                "distance": as_int(fields.get("distance")),
            }
        else:
            blocker = as_int(fields.get("blocker"))
            objective = {
                "event_tick": tick,
                "event_type": name,
                "kind": "capture_denial",
                "team": team_id(player_team_at(teams, blocker, tick)),
                "blocker_user_id": blocker,
                "victim_user_id": as_int(fields.get("victim")),
                "point": as_int(fields.get("cp")),
                "point_name": as_text(fields.get("cp_name")),
            }
        objectives.append(objective)
        if debug:
            print("[candidate-debug] objective tick={} kind={} team={} data={}".format(tick, objective["kind"], objective.get("team"), objective))
    return objectives


def melee_classification(kill: Dict[str, Any]) -> Optional[str]:
    """Return the evidence source proving that a player kill used melee."""
    weapon_slot = as_text(kill.get("weapon_slot")).lower()
    if weapon_slot:
        if weapon_slot != "melee":
            return None
        return as_text(kill.get("weapon_slot_source")) or "weapon_slot"
    if as_int(kill.get("weapon_id")) in MELEE_WEAPON_IDS:
        return "weapon_id"
    if as_text(kill.get("weapon")).lower() in MELEE_WEAPONS:
        return "weapon_name"
    return None


def is_melee_kill(kill: Dict[str, Any]) -> bool:
    return melee_classification(kill) is not None


def is_backstab_kill(kill: Dict[str, Any]) -> bool:
    """TF_DMG_CUSTOM_BACKSTAB is the authoritative Spy-backstab event value."""
    return as_int(kill.get("custom_kill")) == BACKSTAB_CUSTOM_KILL


def weapon_tags(kill: Dict[str, Any]) -> List[str]:
    weapon = as_text(kill.get("weapon")).lower()
    tags: List[str] = []
    if weapon in PROJECTILE_WEAPONS or weapon in LOOSE_CANNON_WEAPONS:
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
    if is_backstab_kill(kill):
        tags.append("backstab")
    elif is_melee_kill(kill):
        tags.append("melee_kill")
    return tags


def taunt_kill_name(kill: Dict[str, Any]) -> Optional[str]:
    """Return the confirmed taunt type from TF2's custom-kill field."""
    return TAUNT_CUSTOM_KILL_NAMES.get(as_int(kill.get("custom_kill")))


def score_candidate(kills: List[Dict[str, Any]], round_data: Dict[str, Any], building_destructions: Optional[List[Dict[str, Any]]] = None, objective_events: Optional[List[Dict[str, Any]]] = None) -> Tuple[float, List[str], Dict[str, Any], List[Dict[str, Any]]]:
    """Score one sequence and expose every contribution for auditing."""
    tags = set()
    score = 10.0
    breakdown: List[Dict[str, Any]] = [{"reason": "candidate_base", "points": 10.0}]
    for kill in kills:
        tags.update(weapon_tags(kill))
        state_evidence = kill.get("state_evidence", {})
        kritzkrieg_kill = bool(state_evidence.get("confirmed_kritzkrieg_boost")) and as_int(kill.get("crit_type")) > 0
        market_garden = (
            kill.get("weapon") == "market_gardener"
            and as_int(kill.get("crit_type")) > 0
            and bool(state_evidence.get("attacker_blast_jumping"))
        )
        if kritzkrieg_kill:
            score += 8.0
            tags.add("kritzkrieg_kill")
            breakdown.append({"reason": "confirmed_kritzkrieg_boosted_kill", "points": 8.0, "event_tick": kill["event_tick"], "deployments": state_evidence.get("kritzkrieg_deployments", [])})
        if market_garden:
            score += 20.0
            tags.add("market_garden")
            breakdown.append({"reason": "confirmed_market_garden", "points": 20.0, "event_tick": kill["event_tick"]})
        if state_evidence.get("confirmed_double_donk"):
            score += 18.0
            tags.add("double_donk")
            breakdown.append({"reason": "confirmed_loose_cannon_double_donk", "points": 18.0, "event_tick": kill["event_tick"], "hits": state_evidence.get("double_donk_events", [])})
        drop_shot = (kill.get("attacker_class") == "sniper" and kill["weapon"] in {"sniperrifle", "sniperrifle_classic", "sniperrifle_decap"}
                     and bool(state_evidence.get("attacker_scoped"))
                     and bool(state_evidence.get("attacker_airborne"))
                     and float(state_evidence.get("attacker_vertical_velocity", 0)) < -20)
        if drop_shot:
            score += 18.0
            tags.add("sniper_dropshot")
            breakdown.append({"reason":"confirmed_sniper_dropshot","points":18.0,"event_tick":kill["event_tick"]})
        shield_bash = as_int(kill.get("custom_kill")) == SHIELD_BASH_CUSTOM_KILL
        charge_melee = (
            not shield_bash
            and kill.get("attacker_class") == "demoman"
            and is_melee_kill(kill)
            and state_evidence.get("attacker_recent_shield_charge_tick") is not None
        )
        if shield_bash:
            score += 22.0
            tags.update({"demoknight", "shield_bash_kill"})
            breakdown.append({"reason": "confirmed_shield_bash_kill", "points": 22.0, "event_tick": kill["event_tick"], "custom_kill": SHIELD_BASH_CUSTOM_KILL})
        elif charge_melee:
            score += 16.0
            tags.update({"demoknight", "charge_melee_kill"})
            breakdown.append({
                "reason": "shield_charge_followed_by_melee_kill",
                "points": 16.0,
                "event_tick": kill["event_tick"],
                "weapon": kill["weapon"],
                "weapon_id": as_int(kill.get("weapon_id")),
                "weapon_def_index": as_int(kill.get("weapon_def_index")),
                "weapon_slot": as_text(kill.get("weapon_slot")) or None,
                "classification_source": melee_classification(kill),
                "charge_tick": state_evidence.get("attacker_recent_shield_charge_tick"),
                "seconds_since_charge": state_evidence.get("attacker_seconds_since_shield_charge"),
            })
        taunt_name = taunt_kill_name(kill)
        backstab = is_backstab_kill(kill)
        ordinary_melee = (
            is_melee_kill(kill)
            and not taunt_name
            and not backstab
            and not shield_bash
            and not charge_melee
            and not market_garden
        )
        if not ordinary_melee:
            tags.discard("melee_kill")
        if ordinary_melee:
            score += 15.0
            tags.add("melee_kill")
            breakdown.append({
                "reason": "player_melee_kill",
                "points": 15.0,
                "event_tick": kill["event_tick"],
                "weapon": kill["weapon"],
                "weapon_id": as_int(kill.get("weapon_id")),
                "weapon_def_index": as_int(kill.get("weapon_def_index")),
                "classification_source": melee_classification(kill),
            })
        if backstab:
            score += 20.0
            tags.discard("melee_kill")
            tags.add("backstab")
            breakdown.append({
                "reason": "confirmed_spy_backstab",
                "points": 20.0,
                "event_tick": kill["event_tick"],
                "custom_kill": BACKSTAB_CUSTOM_KILL,
                "weapon": kill["weapon"],
            })
        if taunt_name:
            score += 25.0
            tags.add("taunt_kill")
            breakdown.append({
                "reason": "confirmed_taunt_kill",
                "points": 25.0,
                "event_tick": kill["event_tick"],
                "custom_kill": as_int(kill.get("custom_kill")),
                "taunt": taunt_name,
            })
        if kill["victim_class"] == "medic":
            score += 18.0
            tags.add("medic_pick")
            breakdown.append({"reason": "medic_pick", "points": 18.0, "event_tick": kill["event_tick"]})
            if state_evidence.get("confirmed_uber_drop"):
                score += 20.0
                tags.add("uber_drop")
                breakdown.append({"reason": "confirmed_uber_drop", "points": 20.0, "event_tick": kill["event_tick"], "charge": state_evidence.get("medic_charge_before_death")})
        if kill["victim_class"] == "demoman":
            score += 10.0
            tags.add("demoman_pick")
            breakdown.append({"reason": "demoman_pick", "points": 10.0, "event_tick": kill["event_tick"]})
        if state_evidence.get("confirmed_airshot"):
            score += 20.0
            tags.add("confirmed_airshot")
            breakdown.append({"reason": "state_confirmed_airshot", "points": 20.0, "event_tick": kill["event_tick"], "projectile": state_evidence.get("projectile")})
            projectile = state_evidence.get("projectile") or {}
            if projectile.get("impact_proximity") == "direct":
                score += 6.0
                tags.add("direct_airshot")
                breakdown.append({"reason": "direct_airshot_proximity", "points": 6.0, "event_tick": kill["event_tick"], "distance": projectile.get("distance_to_victim")})
            if float(projectile.get("flight_seconds") or 0.0) >= 0.5:
                score += 5.0
                tags.add("long_flight_airshot")
                breakdown.append({"reason": "long_flight_airshot", "points": 5.0, "event_tick": kill["event_tick"], "flight_seconds": projectile.get("flight_seconds")})
        elif state_evidence.get("victim_airborne") and kill["weapon"] in AIRSHOT_PROJECTILE_WEAPONS:
            score += 8.0
            tags.add("airborne_projectile_kill")
            breakdown.append({"reason": "state_confirmed_airborne_victim", "points": 8.0, "event_tick": kill["event_tick"]})
        elif kill["rocket_jump_victim"]:
            score += 10.0
            tags.add("rocket_jump_victim")
            breakdown.append({"reason": "rocket_jump_victim", "points": 10.0, "event_tick": kill["event_tick"]})
        if kill["kill_streak_total"] >= 10:
            score += 5.0
            tags.add("streak_10_plus")
            breakdown.append({"reason": "streak_10_plus", "points": 5.0, "event_tick": kill["event_tick"]})
        if (
            kill["crit_type"] == 2
            and not charge_melee
            and not kritzkrieg_kill
            and kill.get("weapon") != "market_gardener"
            and not backstab
        ):
            score -= 12.0
            tags.add("random_full_crit")
            breakdown.append({"reason": "random_full_crit", "points": -12.0, "event_tick": kill["event_tick"]})
    unique_kill_count = len({kill["victim_user_id"] for kill in kills})
    if unique_kill_count > 1:
        multi_points = 18.0 * (unique_kill_count - 1)
        score += multi_points
        tags.add("multi_kill")
        breakdown.append({"reason": "additional_kills", "points": multi_points, "count": unique_kill_count - 1})
    if unique_kill_count >= 3:
        score += 15.0
        tags.add("three_kill")
        breakdown.append({"reason": "three_kill", "points": 15.0})
    if unique_kill_count >= 4:
        score += 25.0
        tags.add("four_kill_plus")
        breakdown.append({"reason": "four_kill_plus", "points": 25.0})
    confirmed_airshot_count = sum(bool(kill.get("state_evidence", {}).get("confirmed_airshot")) for kill in kills)
    if confirmed_airshot_count >= 2:
        score += 15.0
        tags.add("double_airshot_sequence")
        breakdown.append({"reason": "multiple_confirmed_airshots", "points": 15.0, "count": confirmed_airshot_count})
    first_state = kills[0].get("state_evidence", {})
    enemy_alive_before = as_int(first_state.get("enemy_alive_before"))
    friendly_alive_before = as_int(first_state.get("friendly_alive_before"))
    enemy_state_roster = as_int(first_state.get("enemy_state_roster"))
    unique_group_victims = len({kill["victim_user_id"] for kill in kills})
    if enemy_state_roster >= 4 and enemy_alive_before > 0 and unique_group_victims >= enemy_alive_before:
        score += 18.0
        tags.add("team_wipe")
        if enemy_alive_before == 1:
            tags.add("last_enemy_alive")
        breakdown.append({"reason": "sequence_finished_enemy_team", "points": 18.0, "enemy_alive_before": enemy_alive_before})
    enemy_alive_after = max(0, enemy_alive_before - unique_group_victims)
    erased_player_disadvantage = friendly_alive_before > 0 and enemy_alive_before >= friendly_alive_before + 2 and enemy_alive_after <= friendly_alive_before
    last_kill_tick = kills[-1]["event_tick"]
    respawn_ticks = [
        as_int(kill.get("state_evidence", {}).get("victim_next_respawn_tick"))
        for kill in kills
        if as_int(kill.get("state_evidence", {}).get("victim_next_respawn_tick")) > last_kill_tick
    ]
    earliest_enemy_respawn_tick = min(respawn_ticks) if respawn_ticks else as_int(round_data.get("end_tick"))
    player_advantage_window_ticks = max(0, earliest_enemy_respawn_tick - last_kill_tick)
    friendly_respawns_before_enemy = sorted({
        respawn_tick
        for respawn_tick in kills[-1].get("state_evidence", {}).get("friendly_pending_respawn_ticks", [])
        if last_kill_tick < as_int(respawn_tick) <= earliest_enemy_respawn_tick
    })
    force_followups = {}
    for kill in kills:
        for followup in kill.get("state_evidence", {}).get("enemy_medic_force_followups", []):
            force_tick = as_int(followup.get("event_tick"))
            if last_kill_tick <= force_tick <= last_kill_tick + MEDIC_FORCE_FOLLOWUP_TICKS:
                force_followups[(force_tick, as_int(followup.get("medic_user_id")))] = followup
    medic_force = bool(force_followups)
    if medic_force:
        score += 16.0
        tags.add("medic_force")
        breakdown.append({
            "reason": "enemy_medic_forced_uber_after_sequence",
            "points": 16.0,
            "force_events": list(force_followups.values()),
        })
    player_count_swing = (
        erased_player_disadvantage
        and not medic_force
        and player_advantage_window_ticks >= PLAYER_SWING_MIN_WINDOW_TICKS
    )
    if player_count_swing:
        score += 16.0
        tags.add("player_count_swing")
        breakdown.append({
            "reason": "sequence_created_player_count_window",
            "points": 16.0,
            "friendly_alive_before": friendly_alive_before,
            "enemy_alive_before": enemy_alive_before,
            "enemy_alive_after": enemy_alive_after,
            "window_seconds": round(player_advantage_window_ticks / TICKS_PER_SECOND, 3),
            "earliest_enemy_respawn_tick": earliest_enemy_respawn_tick,
            "friendly_respawns_before_enemy": friendly_respawns_before_enemy,
        })
    recent_friendly_deaths = as_int(first_state.get("recent_friendly_death_count"))
    enemy_uber_advantage = bool(first_state.get("enemy_uber_advantage_before"))
    contains_medic_pick = any(kill.get("victim_class") == "medic" for kill in kills)
    # A sack tag represents an Uber equalization play, not a generic pick
    # after teammates died. It needs a verified enemy Uber advantage plus a
    # meaningful recovery of the player deficit or an enemy-Medic kill.
    sack_uber_recovery = (
        recent_friendly_deaths >= SACK_MIN_FRIENDLY_LOSSES
        and enemy_uber_advantage
        and (player_count_swing or contains_medic_pick)
    )
    if sack_uber_recovery:
        score += 16.0
        tags.add("sack_uber_recovery")
        breakdown.append({
            "reason": "sack_uber_recovery_after_losses",
            "points": 16.0,
            "recent_friendly_deaths": recent_friendly_deaths,
            "player_disadvantage_before": first_state.get("player_disadvantage_before"),
            "window_seconds": first_state.get("sack_recovery_window_seconds"),
            "death_ticks": first_state.get("recent_friendly_death_ticks", []),
            "friendly_medic_charge": first_state.get("friendly_medic_charge_before"),
            "enemy_medic_charge": first_state.get("enemy_medic_charge_before"),
        })
        if contains_medic_pick:
            score += 12.0
            tags.add("sack_uber_medic_equalizer")
            breakdown.append({
                "reason": "sack_uber_medic_equalizer",
                "points": 12.0,
            })
    duration_ticks = max(0, kills[-1]["event_tick"] - kills[0]["event_tick"])
    duration_seconds = duration_ticks / TICKS_PER_SECOND
    if unique_kill_count >= 2 and duration_seconds <= 2.0:
        score += 12.0
        tags.add("rapid_sequence")
        breakdown.append({"reason": "rapid_sequence", "points": 12.0})
    if any("projectile_kill" in weapon_tags(kill) for kill in kills):
        score += 8.0
        breakdown.append({"reason": "projectile_sequence", "points": 8.0})
    if round_data["end_tick"] - kills[-1]["event_tick"] <= round(TICKS_PER_SECOND * 8.0):
        score += 8.0
        tags.add("late_round")
        breakdown.append({"reason": "late_round", "points": 8.0})
    attacker_team = team_id(kills[0].get("attacker_team"))
    if (
        attacker_team
        and attacker_team == as_int(round_data.get("winning_team"))
        and 0 <= round_data["end_tick"] - kills[-1]["event_tick"] <= ROUND_CLINCH_TICKS
    ):
        score += 12.0
        tags.add("round_clinch")
        breakdown.append({"reason": "team_won_immediately_after_sequence", "points": 12.0, "event_tick": round_data["end_tick"]})
    linked_buildings = [
        building for building in (building_destructions or [])
        if building["attacker_user_id"] == kills[0]["attacker_user_id"]
        and 0 <= kills[0]["event_tick"] - building["event_tick"] <= round(TICKS_PER_SECOND * 2.0)
    ]
    if linked_buildings:
        score += 6.0
        tags.add("building_to_kill_sequence")
        breakdown.append({"reason": "building_destruction_led_to_kills", "points": 6.0, "count": len(linked_buildings), "event_tick": linked_buildings[-1]["event_tick"]})
    objective_followups = [
        objective for objective in (objective_events or [])
        if 0 <= objective["event_tick"] - kills[-1]["event_tick"] <= OBJECTIVE_CONVERSION_TICKS
        and objective.get("team", 0) == attacker_team
    ]
    # A point capture after the final kill is proof that the sequence secured
    # the cap; it is not evidence that the kill occurred while on the point.
    # Payload progress can be emitted more than once while a cart
    # moves, so score at most one of those signals and never stack it on a
    # completed capture from the same short sequence.
    point_capture = next((item for item in objective_followups if item["kind"] == "point_capture"), None)
    payload_progress = next((item for item in objective_followups if item["kind"] == "payload_progress"), None)
    capture_denial = next(
        (
            item for item in objective_followups
            if item["kind"] == "capture_denial"
            and item.get("blocker_user_id") == kills[0]["attacker_user_id"]
            and item["event_tick"] - kills[-1]["event_tick"] <= CAPTURE_DENIAL_TICKS
        ),
        None,
    )
    objective_score = 0.0
    objective_conversion_kind = ""
    if point_capture is not None:
        objective_score = 24.0
        objective_conversion_kind = "kills_to_secure_cap"
        score += objective_score
        tags.add("kills_to_secure_cap")
        breakdown.append({"reason": "kills_to_secure_cap", "points": objective_score, "event_tick": point_capture["event_tick"], "point": point_capture.get("point"), "point_name": point_capture.get("point_name")})
    elif capture_denial is not None:
        objective_score = 20.0
        objective_conversion_kind = "capture_denial"
        score += objective_score
        tags.add("capture_denial_followup")
        breakdown.append({"reason": "kill_sequence_blocked_capture", "points": objective_score, "event_tick": capture_denial["event_tick"], "point": capture_denial.get("point"), "point_name": capture_denial.get("point_name"), "victim_user_id": capture_denial.get("victim_user_id")})
    elif payload_progress is not None:
        objective_score = 16.0 if payload_progress.get("pusher_user_id") == kills[0]["attacker_user_id"] else 12.0
        objective_conversion_kind = "payload_progress"
        score += objective_score
        tags.add("payload_progress_followup")
        if payload_progress.get("pusher_user_id") == kills[0]["attacker_user_id"]:
            tags.add("payload_pusher")
        breakdown.append({"reason": "kill_sequence_led_to_payload_progress", "points": objective_score, "event_tick": payload_progress["event_tick"], "pusher_user_id": payload_progress.get("pusher_user_id"), "distance": payload_progress.get("distance")})
    raw_score = round(score, 2)
    metrics = {
        "kills": len(kills),
        "unique_victims": unique_kill_count,
        "duration_seconds": round(duration_seconds, 3),
        "unique_weapons": sorted({kill["weapon"] for kill in kills if kill["weapon"]}),
        "projectile_kills": sum("projectile_kill" in weapon_tags(kill) for kill in kills),
        "melee_kills": sum(
            is_melee_kill(kill)
            and not taunt_kill_name(kill)
            and not is_backstab_kill(kill)
            and as_int(kill.get("custom_kill")) != SHIELD_BASH_CUSTOM_KILL
            for kill in kills
        ),
        "backstab_kills": sum(is_backstab_kill(kill) for kill in kills),
        "taunt_kills": sum(bool(taunt_kill_name(kill)) for kill in kills),
        "shield_bash_kills": sum(as_int(kill.get("custom_kill")) == SHIELD_BASH_CUSTOM_KILL for kill in kills),
        "charge_melee_kills": sum(
            as_int(kill.get("custom_kill")) != SHIELD_BASH_CUSTOM_KILL
            and kill.get("attacker_class") == "demoman"
            and is_melee_kill(kill)
            and kill.get("state_evidence", {}).get("attacker_recent_shield_charge_tick") is not None
            for kill in kills
        ),
        "medic_kills": sum(kill["victim_class"] == "medic" for kill in kills),
        "demoman_kills": sum(kill["victim_class"] == "demoman" for kill in kills),
        "full_crit_kills": sum(kill["crit_type"] == 2 for kill in kills),
        "kritzkrieg_kills": sum(bool(kill.get("state_evidence", {}).get("confirmed_kritzkrieg_boost")) and as_int(kill.get("crit_type")) > 0 for kill in kills),
        "market_gardens": sum(
            kill.get("weapon") == "market_gardener"
            and as_int(kill.get("crit_type")) > 0
            and bool(kill.get("state_evidence", {}).get("attacker_blast_jumping"))
            for kill in kills
        ),
        "double_donks": sum(bool(kill.get("state_evidence", {}).get("confirmed_double_donk")) for kill in kills),
        "confirmed_airshots": confirmed_airshot_count,
        "direct_airshots": sum((kill.get("state_evidence", {}).get("projectile") or {}).get("impact_proximity") == "direct" for kill in kills),
        "airborne_projectile_kills": sum(bool(kill.get("state_evidence", {}).get("victim_airborne")) and kill["weapon"] in AIRSHOT_PROJECTILE_WEAPONS for kill in kills),
        "confirmed_uber_drops": sum(bool(kill.get("state_evidence", {}).get("confirmed_uber_drop")) for kill in kills),
        "friendly_alive_before": friendly_alive_before,
        "enemy_alive_before": enemy_alive_before,
        "enemy_alive_after_sequence": enemy_alive_after,
        "player_advantage_window_seconds": round(player_advantage_window_ticks / TICKS_PER_SECOND, 3),
        "earliest_enemy_respawn_tick": earliest_enemy_respawn_tick,
        "friendly_respawns_before_enemy": friendly_respawns_before_enemy,
        "medic_force": medic_force,
        "medic_force_followups": list(force_followups.values()),
        "recent_friendly_deaths_before": recent_friendly_deaths,
        "player_disadvantage_before": as_int(first_state.get("player_disadvantage_before")),
        "enemy_uber_advantage_before": enemy_uber_advantage,
        "sack_uber_recovery": sack_uber_recovery,
        "sack_uber_medic_equalizer": sack_uber_recovery and contains_medic_pick,
        "first_kill_tick": kills[0]["event_tick"],
        "last_kill_tick": last_kill_tick,
        "score_before_floor": raw_score,
        "score_floor_applied": raw_score < 0.0,
        "linked_building_destructions": len(linked_buildings),
        "objective_followups": len(objective_followups),
        "point_capture_followups": sum(item["kind"] == "point_capture" for item in objective_followups),
        "payload_progress_followups": sum(item["kind"] == "payload_progress" for item in objective_followups),
        "capture_denial_followups": sum(item["kind"] == "capture_denial" for item in objective_followups),
        "objective_followup_evidence": objective_followups,
        "objective_conversion_kind": objective_conversion_kind,
        "objective_score": objective_score,
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


def read_demo_bookmarks(export_directory: Path) -> List[Dict[str, Any]]:
    """Read TF2 Demo Support (ds_mark) sidecar data for this exported demo.

    TF2 stores Demo Support marks next to the .dem, not inside its packet
    stream.  The exact JSON nesting has changed across client builds, so this
    accepts both an explicit bookmarks collection and bookmark event records.
    """
    manifest = read_json_if_present(export_directory / "manifest.json")
    source_demo = as_text(manifest.get("source_demo"))
    if not source_demo:
        return []
    demo_path = Path(source_demo).expanduser()
    sidecars = [demo_path.with_suffix(".json"), Path(str(demo_path) + ".json")]
    seen_paths = set()
    records: List[Dict[str, Any]] = []
    for sidecar in sidecars:
        try:
            resolved = sidecar.resolve()
        except OSError:
            resolved = sidecar
        normalized = os.path.normcase(str(resolved))
        if normalized in seen_paths or not resolved.is_file():
            continue
        seen_paths.add(normalized)
        try:
            with resolved.open("r", encoding="utf-8-sig") as source:
                payload = json.load(source)
        except (OSError, ValueError, TypeError):
            continue
        records.extend(extract_bookmark_records(payload))

    unique: Dict[Tuple[int, str], Dict[str, Any]] = {}
    for record in records:
        tick = as_int(record.get("tick"), -1)
        if tick < 0:
            continue
        comment = as_text(record.get("comment")).strip() or "Bookmark"
        unique[(tick, comment)] = {"tick": tick, "comment": comment}
    return [unique[key] for key in sorted(unique)]


def extract_bookmark_records(value: Any, inherited_bookmark: bool = False) -> List[Dict[str, Any]]:
    if isinstance(value, list):
        result: List[Dict[str, Any]] = []
        for item in value:
            result.extend(extract_bookmark_records(item, inherited_bookmark))
        return result
    if not isinstance(value, dict):
        return []

    lowered = {str(key).lower(): item for key, item in value.items()}
    labels = " ".join(
        as_text(lowered.get(key)) for key in ("event", "event_type", "type", "kind", "category", "name")
    ).lower()
    is_bookmark = inherited_bookmark or "bookmark" in labels or any("bookmark" in key for key in lowered)
    tick = -1
    for key in ("tick", "demo_tick", "demotick", "demo tick"):
        if key in lowered:
            tick = as_int(lowered[key], -1)
            if tick >= 0:
                break
    result: List[Dict[str, Any]] = []
    if is_bookmark and tick >= 0:
        comment = ""
        for key in ("comment", "description", "text", "label", "message", "value"):
            candidate = lowered.get(key)
            if isinstance(candidate, str) and candidate.strip():
                comment = candidate.strip()
                break
        result.append({"tick": tick, "comment": comment or "Bookmark"})
    for key, child in value.items():
        if isinstance(child, (dict, list)):
            result.extend(extract_bookmark_records(child, is_bookmark or "bookmark" in str(key).lower()))
    return result


def linked_bookmark_candidate(tick: int, candidates: List[Dict[str, Any]]) -> Optional[Dict[str, Any]]:
    nearby = []
    for candidate in candidates:
        if candidate.get("candidate_type") == "bookmark":
            continue
        kill_ticks = candidate.get("point_of_kill_ticks") or []
        if not kill_ticks:
            continue
        first_tick, last_tick = min(kill_ticks), max(kill_ticks)
        if first_tick - BOOKMARK_LINK_AFTER_TICKS <= tick <= last_tick + BOOKMARK_LINK_BEFORE_TICKS:
            nearby.append(candidate)
    if not nearby:
        return None
    return min(nearby, key=lambda item: (abs(tick - max(item.get("point_of_kill_ticks") or [tick])), -float(item.get("overall_score", 0.0))))


def build_bookmark_candidates(bookmarks: List[Dict[str, Any]], rounds: List[Dict[str, Any]], context: Dict[str, Any], identified_candidates: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    candidates: List[Dict[str, Any]] = []
    for index, bookmark in enumerate(bookmarks, start=1):
        tick = as_int(bookmark.get("tick"), -1)
        if tick < 0:
            continue
        linked = linked_bookmark_candidate(tick, identified_candidates)
        round_data = next((item for item in rounds if item["live_start_tick"] <= tick <= item["end_tick"]), None)
        comment = as_text(bookmark.get("comment")).strip() or "Bookmark"
        if linked is not None:
            candidate = dict(linked)
            candidate["candidate_id"] = "bookmark-t{}-{}".format(tick, index)
            candidate["candidate_type"] = "bookmark"
            candidate["bookmark_comment"] = comment
            candidate["bookmark_tick"] = tick
            candidate["clip_start_tick"] = min(as_int(linked.get("clip_start_tick"), tick), max(0, tick - PRE_ROLL_TICKS))
            candidate["start_tick"] = candidate["clip_start_tick"]
            candidate["clip_end_tick"] = max(as_int(linked.get("clip_end_tick"), tick), tick + POST_ROLL_TICKS)
            candidate["end_tick"] = candidate["clip_end_tick"]
            candidate["overall_score"] = round(float(linked.get("overall_score", 0.0)) + BOOKMARK_SCORE, 2)
            candidate["tags"] = sorted(set(list(linked.get("tags") or []) + ["bookmark"]))
            candidate["score_breakdown"] = list(linked.get("score_breakdown") or []) + [{"reason": "manual_demo_bookmark", "points": BOOKMARK_SCORE, "event_tick": tick, "comment": comment}]
            metrics = dict(linked.get("metrics") or {})
            metrics.update({"bookmarks": 1, "bookmark_comment": comment, "bookmark_tick": tick, "bookmark_linked_candidate_id": linked.get("candidate_id")})
            candidate["metrics"] = metrics
            candidates.append(candidate)
            continue
        candidates.append({
            "candidate_id": "bookmark-t{}-{}".format(tick, index),
            "candidate_type": "bookmark",
            "round_index": round_data["round_index"] if round_data else -1,
            "live_round": round_data is not None,
            "demo_context": context,
            "round_state": {
                "classification": "live" if round_data else "bookmark_outside_live_round",
                "start_tick": round_data["live_start_tick"] if round_data else None,
                "start_event": round_data["live_start_event"] if round_data else None,
                "round_active_tick": round_data["round_active_tick"] if round_data else None,
                "setup_finished_tick": round_data["setup_finished_tick"] if round_data else None,
                "activation_trigger": round_data["activation_trigger"] if round_data else {},
                "ready_up": round_data["ready_up"] if round_data else {},
                "end_tick": round_data["end_tick"] if round_data else None,
                "end_event": round_data["end_reason"] if round_data else None,
            },
            "clip_start_tick": max(0, tick - PRE_ROLL_TICKS),
            "clip_end_tick": tick + POST_ROLL_TICKS,
            "start_tick": max(0, tick - PRE_ROLL_TICKS),
            "first_kill_tick": tick,
            "last_kill_tick": tick,
            "first_kill_server_tick": tick,
            "last_kill_server_tick": tick,
            "point_of_kill_ticks": [tick],
            "point_of_kill_server_ticks": [tick],
            "point_of_kill_events": [{"tick": tick, "demo_tick": tick, "server_tick": tick, "packet_sequence": -1, "event_index_in_packet": -1}],
            "end_tick": tick + POST_ROLL_TICKS,
            "attacker_user_id": 0,
            "attacker_class": "bookmark",
            "attacker_team": "bookmark",
            "overall_score": BOOKMARK_SCORE,
            "score_breakdown": [{"reason": "manual_demo_bookmark", "points": BOOKMARK_SCORE, "event_tick": tick, "comment": comment}],
            "tags": ["bookmark"],
            "metrics": {"bookmarks": 1, "bookmark_comment": comment, "bookmark_tick": tick},
            "bookmark_comment": comment,
            "bookmark_tick": tick,
            "kills": [],
            "building_destructions": [],
            "objective_followups": [],
            "state_pass": {"status": "not_applicable", "confirmed_airshots": 0, "confirmed_uber_drops": 0, "enemy_alive_before": 0, "enemy_alive_after_sequence": 0, "medic_force": False, "player_count_swing": False, "sack_uber_recovery": False, "sack_uber_medic_equalizer": False},
        })
    return candidates


def _score_group_work(item: Tuple[List[Dict[str, Any]], Dict[str, Any], Optional[List[Dict[str, Any]]], Optional[List[Dict[str, Any]]]]) -> Tuple[float, List[str], Dict[str, Any], List[Dict[str, Any]]]:
    group, round_data, building_destructions, objective_events = item
    return score_candidate(group, round_data, building_destructions, objective_events)


def build_candidates(deaths: List[Dict[str, Any]], rounds: List[Dict[str, Any]], context: Dict[str, Any], building_destructions: Optional[List[Dict[str, Any]]] = None, objective_events: Optional[List[Dict[str, Any]]] = None, debug: bool = False, candidate_workers: int = 1, profile: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
    by_attacker: Dict[Tuple[int, int], List[Dict[str, Any]]] = defaultdict(list)
    for death in deaths:
        by_attacker[(death["round_index"], death["attacker_user_id"])].append(death)
    round_lookup = {item["round_index"]: item for item in rounds}
    candidates: List[Dict[str, Any]] = []
    group_jobs: List[Tuple[int, int, List[Dict[str, Any]], Dict[str, Any]]] = []
    for (round_index, attacker), kills in by_attacker.items():
        kills.sort(key=lambda item: (item["event_tick"], item["packet_sequence"], item["event_index_in_packet"]))
        groups = group_kills(kills)
        if debug:
            print("[candidate-debug] grouping round={} attacker={} input_kills={} groups={}".format(round_index, attacker, len(kills), [[kill["event_tick"] for kill in group] for group in groups]))
        round_data = round_lookup[round_index]
        for group in groups:
            group_jobs.append((round_index, attacker, group, round_data))

    requested_workers = max(1, as_int(candidate_workers, 1))
    use_parallel = requested_workers > 1 and len(group_jobs) >= max(8, requested_workers * 2)
    if profile is not None:
        profile["candidate_group_jobs"] = len(group_jobs)
        profile["candidate_workers_requested"] = requested_workers
        profile["candidate_workers_used"] = requested_workers if use_parallel else 1

    if use_parallel:
        work = [
            (group, round_data, building_destructions, objective_events)
            for _, _, group, round_data in group_jobs
        ]
        with ProcessPoolExecutor(max_workers=requested_workers) as executor:
            scored = list(executor.map(_score_group_work, work, chunksize=1))
    else:
        scored = [
            score_candidate(group, round_data, building_destructions, objective_events)
            for _, _, group, round_data in group_jobs
        ]

    for job, score_result in zip(group_jobs, scored):
            round_index, attacker, group, round_data = job
            score, tags, metrics, score_breakdown = score_result
            objective_followups = metrics.get("objective_followup_evidence", [])
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
                "objective_followups": objective_followups,
                "state_pass": {
                    "status": "complete" if any(kill.get("state_evidence", {}).get("state_available") for kill in group) else "unavailable",
                    "confirmed_airshots": metrics.get("confirmed_airshots", 0),
                    "confirmed_uber_drops": metrics.get("confirmed_uber_drops", 0),
                    "enemy_alive_before": metrics.get("enemy_alive_before", 0),
                    "enemy_alive_after_sequence": metrics.get("enemy_alive_after_sequence", 0),
                    "medic_force": metrics.get("medic_force", False),
                    "player_count_swing": "player_count_swing" in tags,
                    "sack_uber_recovery": metrics.get("sack_uber_recovery", False),
                    "sack_uber_medic_equalizer": metrics.get("sack_uber_medic_equalizer", False),
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
    analyzer_started = time.perf_counter()
    parser = argparse.ArgumentParser(description="Rank live-round TF2 frag candidates from events.ndjson.")
    parser.add_argument("export_directory", type=Path, help="Folder produced by export_all.exe")
    parser.add_argument("--item-schema", type=Path, help="Optional path to TF2's current scripts/items/items_game.txt")
    parser.add_argument("--candidate-workers", type=int, default=1, help="Bounded child-process workers for independent candidate-group scoring")
    parser.add_argument("--debug", action="store_true", help="Print candidate acceptance, rejection, grouping, and scoring decisions")
    arguments = parser.parse_args()
    export_directory = arguments.export_directory.resolve()
    events_path = export_directory / "events.ndjson"
    if not events_path.is_file():
        raise FileNotFoundError("events.ndjson is missing. Rebuild and run the updated parser first: {}".format(events_path))
    profile: Dict[str, Any] = {
        "format": "tf2-frag-analysis-profile",
        "format_version": 1,
        "scope_behavior": "pov_player_only filters to the resolved POV attacker; STV/all_players retains all attackers",
        "stage_seconds": {},
    }
    stage_started = time.perf_counter()
    events = read_events(events_path)
    profile["stage_seconds"]["read_events"] = round(time.perf_counter() - stage_started, 6)
    header = read_json_if_present(export_directory / "header.json")
    header_ticks = as_int(header.get("ticks"))

    stage_started = time.perf_counter()
    teams = player_team_history(events)
    classes = class_history(events)
    names = player_name_history(events)
    context = analysis_context(export_directory, names)
    profile["analysis_scope"] = context.get("analysis_scope")
    profile["capture_type"] = context.get("capture_type")
    profile["pov_player_user_id"] = context.get("pov_player_user_id")
    profile["stage_seconds"]["event_indexes_and_scope"] = round(time.perf_counter() - stage_started, 6)

    stage_started = time.perf_counter()
    state_timeline = read_state_timeline(
        export_directory / "state_samples.ndjson",
        arguments.debug,
        max_demo_tick=header_ticks if header_ticks > 0 else None,
    )
    profile["stage_seconds"]["read_and_index_state"] = round(time.perf_counter() - stage_started, 6)

    stage_started = time.perf_counter()
    rounds = build_rounds(events, state_timeline, teams)
    live_round_index = LiveRoundIndex(rounds)
    context.update(competitive_mode_context(export_directory, events, state_timeline, rounds))
    profile["live_round_count"] = len(rounds)
    profile["mode"] = context["mode"]
    profile["mode_confidence"] = context["mode_confidence"]
    profile["stage_seconds"]["build_live_rounds"] = round(time.perf_counter() - stage_started, 6)

    stage_started = time.perf_counter()
    item_slots, item_schema_info = load_item_schema(export_directory, arguments.item_schema, arguments.debug)
    profile["stage_seconds"]["load_item_schema"] = round(time.perf_counter() - stage_started, 6)
    if arguments.debug:
        print("[candidate-debug] demo capture={} scope={} pov_user_id={} header_nick={}".format(context["capture_type"], context["analysis_scope"], context.get("pov_player_user_id") or "none", context.get("header_nick") or "unknown"))
        for round_data in rounds:
            print("[candidate-debug] live round #{} start={} ({}) end={} ({})".format(round_data["round_index"], round_data["live_start_tick"], round_data["live_start_event"], round_data["end_tick"], round_data["end_reason"]))

    stage_started = time.perf_counter()
    deaths = normalized_deaths(events, live_round_index, classes, teams, names, context, arguments.debug, item_slots, profile)
    profile["stage_seconds"]["early_death_gating"] = round(time.perf_counter() - stage_started, 6)

    stage_started = time.perf_counter()
    enrich_state_evidence(deaths, events, state_timeline, arguments.debug, live_round_index, teams, context)
    profile["stage_seconds"]["state_enrichment"] = round(time.perf_counter() - stage_started, 6)

    stage_started = time.perf_counter()
    building_destructions = normalized_building_destructions(events, live_round_index, context, arguments.debug, profile)
    objective_events = normalized_objective_events(events, live_round_index, teams, arguments.debug)
    profile["stage_seconds"]["auxiliary_events"] = round(time.perf_counter() - stage_started, 6)

    stage_started = time.perf_counter()
    candidates = build_candidates(deaths, rounds, context, building_destructions, objective_events, arguments.debug, arguments.candidate_workers, profile)
    bookmarks = read_demo_bookmarks(export_directory)
    bookmark_candidates = build_bookmark_candidates(bookmarks, rounds, context, candidates)
    candidates = sorted(candidates + bookmark_candidates, key=lambda item: (-item["overall_score"], item["start_tick"]))
    profile["demo_bookmark_count"] = len(bookmarks)
    profile["bookmark_candidate_count"] = len(bookmark_candidates)
    profile["stage_seconds"]["candidate_grouping_and_scoring"] = round(time.perf_counter() - stage_started, 6)

    profile["state_sample_count"] = state_timeline.sample_count
    profile["state_lookup_count"] = state_timeline.state_lookup_count
    profile["unindexed_state_lookup_count"] = state_timeline.unindexed_state_lookup_count
    profile["projectile_tracks_total"] = len(state_timeline.projectiles)
    profile["projectile_launcher_index_keys"] = len(state_timeline.projectiles_by_launcher)
    profile["projectile_lookup_calls"] = state_timeline.projectile_lookup_calls
    profile["projectile_tracks_examined"] = state_timeline.projectile_tracks_examined
    profile["candidate_count"] = len(candidates)

    stage_started = time.perf_counter()
    write_ndjson(export_directory / "frag_candidates.ndjson", candidates)
    profile["stage_seconds"]["write_candidates"] = round(time.perf_counter() - stage_started, 6)
    profile["total_seconds"] = round(time.perf_counter() - analyzer_started, 6)
    summary = {
        "format": "tf2-frag-candidates",
        "format_version": 1,
        "source": "events.ndjson",
        "demo_context": context,
        "item_schema": item_schema_info,
        "ticks_per_second_assumption": TICKS_PER_SECOND,
        "live_rounds": len(rounds),
        "live_round_kills": len(deaths),
        "live_round_building_destructions": len(building_destructions),
        "live_round_objective_events": len(objective_events),
        "demo_bookmarks": len(bookmarks),
        "bookmark_candidates": len(bookmark_candidates),
        "state_sample_count": state_timeline.sample_count,
        "state_samples_ignored_out_of_range": state_timeline.ignored_out_of_range_samples,
        "state_backed_analysis": state_timeline.sample_count > 0,
        "candidate_count": len(candidates),
        "analysis_performance": profile,
        "limitations": [
            "Confirmed airshots require an airborne victim plus a matching reconstructed projectile owner, type, timing, and impact proximity.",
            "When state_samples.ndjson is unavailable, the analyzer retains event-only scoring and does not invent state-backed tags.",
            "Kills must be inside an event-confirmed interval or a state-confirmed in-progress public-server interval; ready-up and countdown events never open an interval by themselves.",
            "Kill ticks are the original player_death event ticks. Packet sequence and event-index fields preserve ordering when multiple events share one tick.",
            "POV-only filtering is enabled only when the demo is identified as POV and its header nickname resolves to exactly one player event; otherwise all players are retained.",
            "Demo Support (ds_mark) bookmarks are read from the .json sidecar next to the source .dem. Each bookmark becomes a candidate worth 30 points and inherits identifiers from a nearby analyzed frag candidate when one is found.",
        ],
    }
    with (export_directory / "frag_summary.json").open("w", encoding="utf-8", newline="\n") as output:
        json.dump(summary, output, indent=2, ensure_ascii=False)
        output.write("\n")
    with (export_directory / "analysis_profile.json").open("w", encoding="utf-8", newline="\n") as output:
        json.dump(profile, output, indent=2, ensure_ascii=False)
        output.write("\n")
    print("Analyzed {} events: {} live rounds, {} live-round kills, {} candidates.".format(len(events), len(rounds), len(deaths), len(candidates)))
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print("ERROR: {}".format(error), file=sys.stderr)
        sys.exit(1)
