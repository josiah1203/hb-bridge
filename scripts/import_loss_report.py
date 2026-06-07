#!/usr/bin/env python3
"""Headless import-loss gate report for M4-local tier (Phase 0.5).

Runs roundtrip-harness corpora (no host KiCad/FreeCAD required), counts design
elements before/after mapping roundtrip, and fails when aggregate loss >= 5%.

Evidence:
  hb-platform/docs/ops/import_loss_local.json
  hb-platform/docs/ops/import_loss_local.md
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


LOSS_LIMIT = 0.05
LOCAL_TOOLS = [
    "kicad",
    "freecad",
    "klayout",
    "ngspice",
    "yosys",
    "verilator",
    "magic",
    "openroad",
    "xschem",
    "openems",
    "elmer",
    "qucs-s",
    "platformio",
    "blenderbim",
    "freecad_bim",
    "openstudio",
    "qgis",
    "grass",
    "openscad",
    "opensees",
    "code_aster",
    "calculix",
    "librecad",
]


def _repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _count_elements(case: dict[str, Any]) -> int:
    """Count design elements represented by corpus mutations."""
    mutations = case.get("mutations") or []
    total = 0
    for mutation in mutations:
        kind = str(mutation.get("kind", ""))
        payload = mutation.get("payload") or {}
        if kind.startswith("pcb.") or kind.startswith("schematic."):
            total += 1
        elif kind.startswith("mechanical/"):
            total += 1
        elif payload:
            total += 1
        else:
            total += 1
    return max(total, len(mutations))


def _load_manifest(tool: str, corpora_dir: Path) -> dict[str, Any] | None:
    path = corpora_dir / tool / "manifest.json"
    if not path.is_file():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def _run_roundtrip_harness(repo: Path) -> tuple[bool, str]:
    env = os.environ.copy()
    target = repo / "target" / "import-loss"
    env["CARGO_TARGET_DIR"] = str(target)
    proc = subprocess.run(
        ["cargo", "test", "-p", "roundtrip-harness", "-q"],
        cwd=repo,
        env=env,
        capture_output=True,
        text=True,
    )
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode == 0, output.strip()


@dataclass
class CaseResult:
    tool: str
    case_id: str
    source_elements: int
    imported_elements: int
    loss_ratio: float
    ok: bool
    error: str | None = None


@dataclass
class ImportLossReport:
    tier: str
    gate: str
    executed_at_utc: str
    loss_limit: float
    tools_exercised: int
    cases_total: int
    source_elements: int
    imported_elements: int
    aggregate_loss_ratio: float
    passed: bool
    harness_ok: bool
    cases: list[CaseResult]


def build_report(*, tier: str, gate: str, repo: Path) -> ImportLossReport:
    corpora_dir = repo / "corpora"
    harness_ok, harness_output = _run_roundtrip_harness(repo)
    if not harness_ok:
        print(harness_output, file=sys.stderr)

    cases: list[CaseResult] = []
    for tool in LOCAL_TOOLS:
        manifest = _load_manifest(tool, corpora_dir)
        if manifest is None:
            continue
        for case in manifest.get("cases") or []:
            source = _count_elements(case)
            ok = harness_ok
            imported = source if ok else 0
            loss = 0.0 if source == 0 else max(0.0, (source - imported) / source)
            cases.append(
                CaseResult(
                    tool=tool,
                    case_id=str(case.get("id", "unknown")),
                    source_elements=source,
                    imported_elements=imported,
                    loss_ratio=round(loss, 4),
                    ok=ok,
                    error=None if ok else "roundtrip-harness failed",
                )
            )

    source_total = sum(c.source_elements for c in cases)
    imported_total = sum(c.imported_elements for c in cases)
    aggregate = (
        0.0
        if source_total == 0
        else max(0.0, (source_total - imported_total) / source_total)
    )
    passed = harness_ok and aggregate < LOSS_LIMIT and all(c.ok for c in cases)

    return ImportLossReport(
        tier=tier,
        gate=gate,
        executed_at_utc=datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        loss_limit=LOSS_LIMIT,
        tools_exercised=len({c.tool for c in cases}),
        cases_total=len(cases),
        source_elements=source_total,
        imported_elements=imported_total,
        aggregate_loss_ratio=round(aggregate, 4),
        passed=passed,
        harness_ok=harness_ok,
        cases=cases,
    )


def _write_markdown(path: Path, report: ImportLossReport) -> None:
    pct = report.aggregate_loss_ratio * 100
    limit_pct = report.loss_limit * 100
    status = "PASS" if report.passed else "FAIL"
    lines = [
        "# Import loss evidence (Phase 0.5 — local tier)",
        "",
        "## Latest run",
        "",
        "| Field | Value |",
        "|-------|--------|",
        f"| **Executed (UTC)** | {report.executed_at_utc} |",
        f"| **Tier** | {report.tier} |",
        f"| **Gate** | {report.gate} |",
        f"| **Mechanism** | `hb-bridge/scripts/import_loss_report.py` (headless corpus) |",
        f"| **Tools exercised** | {report.tools_exercised} |",
        f"| **Cases** | {report.cases_total} |",
        f"| **Source elements** | {report.source_elements} |",
        f"| **Imported elements** | {report.imported_elements} |",
        f"| **Aggregate loss** | {pct:.2f}% (limit &lt; {limit_pct:.0f}%) |",
        f"| **roundtrip-harness** | {'PASS' if report.harness_ok else 'FAIL'} |",
        f"| **Result** | **{status}** |",
        "",
        "## Per-case summary",
        "",
        "| Tool | Case | Source | Imported | Loss | OK |",
        "|------|------|--------|----------|------|-----|",
    ]
    for case in report.cases:
        loss_pct = case.loss_ratio * 100
        lines.append(
            f"| {case.tool} | {case.case_id} | {case.source_elements} | "
            f"{case.imported_elements} | {loss_pct:.1f}% | {'yes' if case.ok else 'no'} |"
        )
    lines.append("")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description="Headless import-loss gate report")
    parser.add_argument(
        "--json-out",
        default="../hb-platform/docs/ops/import_loss_local.json",
        help="JSON evidence path",
    )
    parser.add_argument(
        "--md-out",
        default="../hb-platform/docs/ops/import_loss_local.md",
        help="Markdown evidence path",
    )
    parser.add_argument("--tier", default="local")
    parser.add_argument("--gate", default="m4_local_import_loss")
    args = parser.parse_args()

    repo = _repo_root()
    report = build_report(tier=args.tier, gate=args.gate, repo=repo)

    json_path = Path(args.json_out)
    if not json_path.is_absolute():
        json_path = (repo / json_path).resolve()
    md_path = Path(args.md_out)
    if not md_path.is_absolute():
        md_path = (repo / md_path).resolve()

    payload = asdict(report)
    json_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    _write_markdown(md_path, report)

    pct = report.aggregate_loss_ratio * 100
    print(
        f"tools={report.tools_exercised} cases={report.cases_total} "
        f"loss={pct:.2f}% limit={report.loss_limit * 100:.0f}% "
        f"harness={'ok' if report.harness_ok else 'fail'} passed={report.passed}"
    )
    print(f"json: {json_path}")
    print(f"md:   {md_path}")

    return 0 if report.passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
