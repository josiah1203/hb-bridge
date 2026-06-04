"""DRC regression: rule-based violations (best-effort) with optional sidecar scene traces."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from common import (
    ensure_sidecar_binary,
    load_goldens,
    load_manifest,
    normalize_violations,  # re-exported for unit tests
)

__all__ = [
    "derive_violations_from_mutations",
    "normalize_violations",
    "run_drc_suite",
]
from sidecar_client import SidecarSession


def derive_violations_from_mutations(
    mutations: list[dict[str, Any]],
    *,
    rules: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    """Match corpus mutations against declarative DRC rules (stub / best-effort)."""
    violations: list[dict[str, Any]] = []
    for mutation in mutations:
        kind = mutation.get("kind", "")
        payload = mutation.get("payload") or {}
        for rule in rules:
            if rule.get("whenKind") and rule["whenKind"] != kind:
                continue
            field = rule.get("whenField")
            if field is not None:
                actual = str(payload.get(field, ""))
                expected = str(rule.get("whenEquals", ""))
                if actual != expected:
                    continue
            violations.append(
                {
                    "rule": rule["rule"],
                    "severity": rule["severity"],
                    "location": rule.get("location", ""),
                    "message": rule.get("message", ""),
                }
            )
    return violations


def _run_sidecar_smoke(
    *,
    case: dict[str, Any],
    corpus_dir: Path,
) -> list[str]:
    """Optional KiCad sidecar smoke: mutations apply without JSON-RPC errors."""
    errors: list[str] = []
    mutations = case.get("mutations") or []
    if not mutations:
        return errors

    binary = ensure_sidecar_binary("kicad-sidecar", "kicad-sidecar")
    document_uri = case["documentUri"]
    session = SidecarSession(binary=binary)
    try:
        session.start()
        session.ping()
        session.project_open(
            project_id=case.get("projectId", "drc-regression"),
            workspace_root=case.get("workspaceRoot", str(corpus_dir)),
        )
        apply = session.apply_mutations(
            document_uri=document_uri, mutations=mutations
        )
        if apply.get("errors"):
            errors.append(f"sidecar mutations: {apply['errors']}")
        session.wait_for_scene_traces(
            node_upserts=len(mutations),
            edge_upserts=len(mutations),
            timeout_s=5.0,
        )
    finally:
        session.close()
    return errors


def _golden_for_case(goldens: dict[str, Any], case_id: str) -> dict[str, Any] | None:
    for entry in goldens.get("cases") or []:
        if entry.get("id") == case_id:
            return entry
    return None


def run_drc_suite(*, corpus: Path, goldens: Path) -> dict[str, Any]:
    from mutation_hook import ensure_sidecars_built

    ensure_sidecars_built()
    manifest = load_manifest(corpus)
    golden_index = load_goldens(goldens)
    corpus_dir = corpus if corpus.is_dir() else corpus.parent

    trace: dict[str, Any] = {
        "suite": "drc",
        "corpus": str(corpus.resolve()),
        "goldens": str(goldens.resolve()),
        "cases": [],
        "ok": True,
        "mode": "rule-based-best-effort",
    }

    for case in manifest.get("cases") or []:
        case_id = case.get("id", "")
        golden = _golden_for_case(golden_index, case_id)
        rules = case.get("rules") or (golden or {}).get("rules") or []
        mutations = case.get("mutations") or []

        errors: list[str] = []
        observed = normalize_violations(
            derive_violations_from_mutations(mutations, rules=rules)
        )

        if golden is not None:
            expected = normalize_violations(golden.get("violations") or [])
            if observed != expected:
                errors.append(
                    f"violation mismatch expected={expected} observed={observed}"
                )
        elif rules and not observed:
            errors.append("rules configured but no violations derived")

        errors.extend(_run_sidecar_smoke(case=case, corpus_dir=corpus_dir))

        outcome = {
            "caseId": case_id,
            "ok": not errors,
            "errors": errors,
            "observedViolations": observed,
        }
        trace["ok"] = trace["ok"] and outcome["ok"]
        trace["cases"].append(outcome)

    return trace
