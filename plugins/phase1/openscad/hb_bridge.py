"""Phase 1 openscad bridge stub — headless harness only."""

from __future__ import annotations

TOOL = "openscad"


def roundtrip_status() -> dict[str, str]:
    return {"tool": TOOL, "status": "harness-only", "engine": "hnf-phase1-tools"}
