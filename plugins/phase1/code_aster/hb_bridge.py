"""Phase 1 code_aster bridge stub — headless harness only."""

from __future__ import annotations

TOOL = "code_aster"


def roundtrip_status() -> dict[str, str]:
    return {"tool": TOOL, "status": "harness-only", "engine": "hnf-phase1-tools"}
