"""Phase 1 opensees bridge stub — headless harness only."""

from __future__ import annotations

TOOL = "opensees"


def roundtrip_status() -> dict[str, str]:
    return {"tool": TOOL, "status": "harness-only", "engine": "hnf-phase1-tools"}
