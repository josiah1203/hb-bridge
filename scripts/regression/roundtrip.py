"""Roundtrip regression: mutations → export → re-export fingerprint stability.

Headless mapping-only gate: ``cargo test -p roundtrip-harness`` (no sidecar binaries).
This module drives stdio sidecars built from ``rust/crates/*-sidecar``.
"""

from __future__ import annotations

import tempfile
from pathlib import Path
from typing import Any

from common import ensure_sidecar_binary, fingerprint_artifacts, load_manifest
from sidecar_client import SidecarSession


def _run_sidecar_roundtrip(
    *,
    sidecar: str,
    case: dict[str, Any],
    seed_dir: Path,
) -> dict[str, Any]:
    crate = "kicad-sidecar" if sidecar == "kicad" else "freecad-sidecar"
    binary = ensure_sidecar_binary(crate, f"{sidecar}-sidecar")
    document_uri = case["documentUri"]
    project_id = case.get("projectId", f"roundtrip-{sidecar}")
    workspace = case.get("workspaceRoot", str(seed_dir))
    mutations = case.get("mutations", [])
    export_format = case.get("exportFormat", "step" if sidecar == "kicad" else "step")

    result: dict[str, Any] = {
        "caseId": case.get("id", "unknown"),
        "sidecar": sidecar,
        "ok": False,
        "errors": [],
    }

    session = SidecarSession(binary=binary)
    try:
        session.start()
        session.ping()
        session.project_open(project_id=project_id, workspace_root=workspace)
        if mutations:
            apply = session.apply_mutations(
                document_uri=document_uri, mutations=mutations
            )
            if apply.get("errors"):
                result["errors"].append(f"seed mutations: {apply['errors']}")
            if sidecar == "kicad":
                session.wait_for_scene_traces(
                    node_upserts=len(mutations),
                    edge_upserts=len(mutations),
                    timeout_s=5.0,
                )

        with tempfile.TemporaryDirectory(prefix="hcp-roundtrip-") as tmp:
            out_dir = str(Path(tmp))
            first = session.export_document(
                document_uri=document_uri,
                format=export_format,
                output_dir=out_dir,
            )
            second = session.export_document(
                document_uri=document_uri,
                format=export_format,
                output_dir=out_dir,
            )

        fp1 = fingerprint_artifacts(first.get("artifacts") or [])
        fp2 = fingerprint_artifacts(second.get("artifacts") or [])
        if not fp1:
            result["errors"].append("export returned no artifacts")
        elif fp1 != fp2:
            result["errors"].append(
                f"export fingerprint drift: first={fp1} second={fp2}"
            )

        if sidecar == "kicad" and mutations:
            reapply = session.apply_mutations(
                document_uri=document_uri, mutations=mutations
            )
            if reapply.get("applied") != len(mutations):
                result["errors"].append(
                    f"re-import apply applied={reapply.get('applied')} "
                    f"expected={len(mutations)}"
                )

        result["exportFingerprint"] = fp1
        result["ok"] = not result["errors"]
    finally:
        session.close()

    return result


def run_roundtrip_suite(*, corpus: Path) -> dict[str, Any]:
    from mutation_hook import ensure_sidecars_built

    ensure_sidecars_built()
    manifest = load_manifest(corpus)
    corpus_dir = corpus if corpus.is_dir() else corpus.parent
    cases = manifest.get("cases") or []
    trace: dict[str, Any] = {
        "suite": "roundtrip",
        "corpus": str(corpus.resolve()),
        "cases": [],
        "ok": True,
    }

    for case in cases:
        sidecars = case.get("sidecars") or manifest.get("defaultSidecars") or ["kicad"]
        case_results: list[dict[str, Any]] = []
        for sidecar in sidecars:
            case_results.append(
                _run_sidecar_roundtrip(
                    sidecar=sidecar,
                    case=case,
                    seed_dir=corpus_dir,
                )
            )
        case_ok = all(r.get("ok") for r in case_results)
        trace["ok"] = trace["ok"] and case_ok
        trace["cases"].append(
            {
                "id": case.get("id"),
                "results": case_results,
                "ok": case_ok,
            }
        )

    return trace
