#!/usr/bin/env python3
"""Benchmark v32.2 candidate analysis against the optimized analyzer.

The raw parser exports are never modified. The analyzer output files are backed
up, the reference analyzer runs, its candidate bytes are captured, then the
optimized analyzer runs and becomes the final output. Results are written to
CSV and JSON so they can be analyzed later.
"""
from __future__ import annotations

import argparse
import csv
import json
import shutil
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / "tools" / "reference" / "analyze_frags_v32_2.py"
OPTIMIZED = ROOT / "analyze_frags.py"
OUTPUT_NAMES = ("frag_candidates.ndjson", "frag_summary.json", "analysis_profile.json")


def run_analyzer(script: Path, export: Path, item_schema: Optional[Path], candidate_workers: int = 1) -> Dict[str, Any]:
    command = [sys.executable, str(script)]
    if script == OPTIMIZED:
        command += ["--candidate-workers", str(max(1, candidate_workers))]
    if item_schema is not None:
        command += ["--item-schema", str(item_schema)]
    command.append(str(export))
    started = time.perf_counter()
    process = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    elapsed = time.perf_counter() - started
    return {"return_code": process.returncode, "seconds": elapsed, "output": process.stdout}


def read_json(path: Path) -> Dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
        return value if isinstance(value, dict) else {}
    except Exception:
        return {}


def candidate_count(export: Path) -> int:
    summary = read_json(export / "frag_summary.json")
    return int(summary.get("candidate_count") or 0)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("exports", nargs="+", type=Path, help="Parsed export folders to benchmark")
    parser.add_argument("--item-schema", type=Path)
    parser.add_argument("--candidate-workers", type=int, default=1, help="Candidate-group workers for the optimized analyzer")
    parser.add_argument("--output", type=Path, help="Output directory for benchmark CSV/JSON")
    args = parser.parse_args()

    output_dir = (args.output or (ROOT / "benchmark_results")).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    rows: List[Dict[str, Any]] = []

    for raw_export in args.exports:
        export = raw_export.resolve()
        if not (export / "events.ndjson").is_file():
            print("SKIP {}: events.ndjson missing".format(export))
            continue

        backups: Dict[str, Optional[bytes]] = {}
        for name in OUTPUT_NAMES:
            path = export / name
            backups[name] = path.read_bytes() if path.is_file() else None

        print("Reference: {}".format(export))
        reference = run_analyzer(REFERENCE, export, args.item_schema, 1)
        if reference["return_code"] != 0:
            print(reference["output"])
            raise SystemExit("Reference analyzer failed for {}".format(export))
        reference_candidates = (export / "frag_candidates.ndjson").read_bytes()
        reference_count = candidate_count(export)

        print("Optimized: {}".format(export))
        optimized = run_analyzer(OPTIMIZED, export, args.item_schema, args.candidate_workers)
        if optimized["return_code"] != 0:
            print(optimized["output"])
            # Restore previous user outputs if the optimized analyzer fails.
            for name, content in backups.items():
                path = export / name
                if content is None:
                    if path.exists():
                        path.unlink()
                else:
                    path.write_bytes(content)
            raise SystemExit("Optimized analyzer failed for {}".format(export))

        optimized_candidates = (export / "frag_candidates.ndjson").read_bytes()
        optimized_count = candidate_count(export)
        profile = read_json(export / "analysis_profile.json")
        death_rejections = profile.get("death_rejections", {}) if isinstance(profile.get("death_rejections"), dict) else {}
        stage = profile.get("stage_seconds", {}) if isinstance(profile.get("stage_seconds"), dict) else {}
        old_seconds = float(reference["seconds"])
        new_seconds = float(optimized["seconds"])
        row = {
            "export": str(export),
            "capture_type": profile.get("capture_type", ""),
            "analysis_scope": profile.get("analysis_scope", ""),
            "reference_seconds": round(old_seconds, 6),
            "optimized_seconds": round(new_seconds, 6),
            "speedup_x": round(old_seconds / new_seconds, 4) if new_seconds > 0 else 0.0,
            "reference_candidate_count": reference_count,
            "optimized_candidate_count": optimized_count,
            "candidate_bytes_identical": reference_candidates == optimized_candidates,
            "total_player_death_events": profile.get("total_player_death_events", 0),
            "accepted_live_scope_kills": profile.get("accepted_live_scope_kills", 0),
            "rejected_outside_live_round": death_rejections.get("outside_live_round", 0),
            "rejected_not_pov_attacker": death_rejections.get("not_pov_attacker", 0),
            "state_lookup_count": profile.get("state_lookup_count", 0),
            "unindexed_state_lookup_count": profile.get("unindexed_state_lookup_count", 0),
            "projectile_tracks_total": profile.get("projectile_tracks_total", 0),
            "projectile_tracks_examined": profile.get("projectile_tracks_examined", 0),
            "state_enrichment_seconds": stage.get("state_enrichment", 0),
            "candidate_scoring_seconds": stage.get("candidate_grouping_and_scoring", 0),
        }
        rows.append(row)
        print("  {:.2f}s -> {:.2f}s ({:.2f}x), candidates={}, parity={}".format(
            old_seconds, new_seconds, row["speedup_x"], optimized_count, row["candidate_bytes_identical"]
        ))

    if not rows:
        print("No valid exports were benchmarked.")
        return 1

    csv_path = output_dir / ("candidate_analyzer_benchmark_{}.csv".format(stamp))
    json_path = output_dir / ("candidate_analyzer_benchmark_{}.json".format(stamp))
    with csv_path.open("w", newline="", encoding="utf-8") as output:
        writer = csv.DictWriter(output, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)
    json_path.write_text(json.dumps({"format": "tf2-candidate-analyzer-benchmark", "format_version": 1, "runs": rows}, indent=2), encoding="utf-8")
    print("CSV: {}".format(csv_path))
    print("JSON: {}".format(json_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
