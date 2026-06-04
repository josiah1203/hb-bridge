#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

_REGRESSION_DIR = Path(__file__).resolve().parent
if str(_REGRESSION_DIR) not in sys.path:
    sys.path.insert(0, str(_REGRESSION_DIR))


@dataclass(frozen=True)
class SuiteResult:
    suite: str
    ok: bool
    details: dict[str, Any]


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def run_roundtrip(args: argparse.Namespace) -> SuiteResult:
    from roundtrip import run_roundtrip_suite

    trace = run_roundtrip_suite(corpus=Path(args.corpus))
    trace_path = Path(args.out).with_name("roundtrip_trace.json")
    _write_json(trace_path, trace)
    return SuiteResult(
        suite="roundtrip",
        ok=bool(trace.get("ok")),
        details={
            "corpus": args.corpus,
            "tracePath": str(trace_path),
            "cases": [c.get("id") for c in trace.get("cases", [])],
        },
    )


def run_mutation_hook(args: argparse.Namespace) -> SuiteResult:
    from mutation_hook import ensure_sidecars_built, run_mutation_hook_suite

    ensure_sidecars_built()
    trace = run_mutation_hook_suite(
        seed=Path(args.seed),
        mutations_count=args.mutations,
        sidecars=args.sidecars,
    )
    trace_path = Path(args.out).with_name("mutation_hook_trace.json")
    _write_json(trace_path, trace)
    return SuiteResult(
        suite="mutation-hook",
        ok=bool(trace.get("ok")),
        details={
            "seed": args.seed,
            "mutations": args.mutations,
            "tracePath": str(trace_path),
            "sidecars": [s.get("sidecar") for s in trace.get("sidecars", [])],
        },
    )


def run_drc(args: argparse.Namespace) -> SuiteResult:
    from drc import run_drc_suite

    trace = run_drc_suite(corpus=Path(args.corpus), goldens=Path(args.goldens))
    trace_path = Path(args.out).with_name("drc_trace.json")
    _write_json(trace_path, trace)
    return SuiteResult(
        suite="drc",
        ok=bool(trace.get("ok")),
        details={
            "corpus": args.corpus,
            "goldens": args.goldens,
            "tracePath": str(trace_path),
            "cases": [c.get("id") for c in trace.get("cases", [])],
        },
    )


def run_simulation_stability(args: argparse.Namespace) -> SuiteResult:
    from simulation_stability import run_simulation_stability_suite

    trace = run_simulation_stability_suite(
        corpus=Path(args.corpus),
        goldens=Path(args.goldens),
    )
    trace_path = Path(args.out).with_name("simulation_stability_trace.json")
    _write_json(trace_path, trace)
    return SuiteResult(
        suite="simulation-stability",
        ok=bool(trace.get("ok")),
        details={
            "corpus": args.corpus,
            "goldens": args.goldens,
            "tracePath": str(trace_path),
            "cases": [c.get("id") for c in trace.get("cases", [])],
        },
    )


def main() -> int:
    parser = argparse.ArgumentParser(prog="run_suite.py")
    sub = parser.add_subparsers(dest="suite", required=True)

    p1 = sub.add_parser("roundtrip")
    p1.add_argument("--corpus", required=True)
    p1.add_argument("--out", default="out/regression/roundtrip.json")
    p1.set_defaults(_fn=run_roundtrip)

    p2 = sub.add_parser("mutation-hook")
    p2.add_argument("--seed", required=True)
    p2.add_argument("--mutations", type=int, default=100)
    p2.add_argument(
        "--sidecars",
        nargs="+",
        choices=["kicad", "freecad"],
        default=None,
        help="Sidecars to exercise (default: kicad and freecad)",
    )
    p2.add_argument("--out", default="out/regression/mutation_hook.json")
    p2.set_defaults(_fn=run_mutation_hook)

    p3 = sub.add_parser("drc")
    p3.add_argument("--corpus", required=True)
    p3.add_argument("--goldens", required=True)
    p3.add_argument("--out", default="out/regression/drc.json")
    p3.set_defaults(_fn=run_drc)

    p4 = sub.add_parser("simulation-stability")
    p4.add_argument("--corpus", required=True)
    p4.add_argument("--goldens", required=True)
    p4.add_argument("--out", default="out/regression/simulation_stability.json")
    p4.set_defaults(_fn=run_simulation_stability)

    args = parser.parse_args()
    result: SuiteResult = args._fn(args)

    _write_json(Path(args.out), asdict(result))
    if not result.ok:
        print(json.dumps(asdict(result), indent=2), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

