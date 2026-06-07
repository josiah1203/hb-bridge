"""Phase 1 freecad_bim bridge stub — headless harness only."""

from __future__ import annotations

TOOL = "freecad_bim"


def roundtrip_status() -> dict[str, str]:
    return {"tool": TOOL, "status": "harness-only", "engine": "hnf-phase1-tools"}
