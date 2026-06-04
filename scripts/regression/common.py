"""Shared helpers for regression suite drivers."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_DIR = REPO_ROOT / "rust"


def repo_root() -> Path:
    return REPO_ROOT


def load_manifest(corpus: Path) -> dict[str, Any]:
    """Load manifest.json from a corpus directory or file."""
    corpus = corpus.resolve()
    if corpus.is_file():
        return json.loads(corpus.read_text())
    manifest = corpus / "manifest.json"
    if not manifest.is_file():
        raise FileNotFoundError(f"corpus missing manifest.json: {corpus}")
    return json.loads(manifest.read_text())


def load_goldens(goldens: Path) -> dict[str, Any]:
    """Load goldens index or a single golden JSON file."""
    goldens = goldens.resolve()
    if goldens.is_file():
        data = json.loads(goldens.read_text())
        if "cases" not in data and "violations" in data:
            return {"cases": [{"id": goldens.stem, **data}]}
        return data
    index = goldens / "index.json"
    if index.is_file():
        return json.loads(index.read_text())
    cases: list[dict[str, Any]] = []
    for path in sorted(goldens.glob("*.json")):
        if path.name == "index.json":
            continue
        entry = json.loads(path.read_text())
        entry.setdefault("id", path.stem)
        cases.append(entry)
    if not cases:
        raise FileNotFoundError(f"no golden JSON files under {goldens}")
    return {"cases": cases}


def fingerprint_artifacts(artifacts: list[dict[str, Any]]) -> list[dict[str, str]]:
    """Stable fingerprint for export artifacts (basename + content type)."""
    rows: list[dict[str, str]] = []
    for artifact in sorted(artifacts, key=lambda a: a.get("path", "")):
        path = str(artifact.get("path", ""))
        rows.append(
            {
                "basename": Path(path).name,
                "contentType": str(artifact.get("contentType", "")),
            }
        )
    return rows


# Back-compat alias
fingerprint_export = fingerprint_artifacts


def normalize_violations(violations: list[dict[str, Any]]) -> list[dict[str, str]]:
    """Sortable normalized violation records for golden comparison."""
    normalized: list[dict[str, str]] = []
    for item in violations:
        normalized.append(
            {
                "rule": str(item.get("rule", item.get("code", ""))),
                "severity": str(item.get("severity", "unknown")),
                "location": str(item.get("location", "")),
                "message": str(item.get("message", "")),
            }
        )
    return sorted(
        normalized, key=lambda v: (v["rule"], v["location"], v["message"])
    )


def normalize_sim_envelope(envelope: dict[str, Any]) -> dict[str, Any]:
    """Normalize simulation envelope fields for golden comparison."""
    artifacts = []
    for art in envelope.get("artifacts") or []:
        path = str(art.get("path", ""))
        artifacts.append(
            {
                "path": Path(path).name,
                "content_type": str(
                    art.get("content_type", art.get("contentType", ""))
                ),
                "role": str(art.get("role", "")),
            }
        )
    artifacts.sort(key=lambda a: a["path"])
    return {
        "engine": str(envelope.get("engine", "")),
        "job_id": str(envelope.get("job_id", envelope.get("jobId", ""))),
        "status": str(envelope.get("status", "")),
        "exit_code": int(envelope.get("exit_code", envelope.get("exitCode", -1))),
        "artifacts": artifacts,
        "stdout": str(envelope.get("stdout", "")).strip(),
    }


def ensure_sidecar_binary(crate: str, bin_name: str) -> Path:
    debug = RUST_DIR / "target" / "debug" / bin_name
    if debug.is_file():
        return debug
    subprocess.run(
        ["cargo", "build", "-p", crate, "--bin", bin_name],
        cwd=RUST_DIR,
        check=True,
        capture_output=True,
        text=True,
    )
    if not debug.is_file():
        raise FileNotFoundError(f"expected sidecar binary at {debug}")
    return debug


def compare_numeric(
    actual: float, expected: float, *, rel_tol: float = 1e-6, abs_tol: float = 1e-9
) -> bool:
    diff = abs(actual - expected)
    if diff <= abs_tol:
        return True
    scale = max(abs(expected), abs(actual), 1.0)
    return diff / scale <= rel_tol
