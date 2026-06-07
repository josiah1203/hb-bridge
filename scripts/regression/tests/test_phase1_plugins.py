"""Headless HNF v0.1 Phase 1 plugin roundtrip tests (M5-B)."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

_REPO = Path(__file__).resolve().parents[3]
_PHASE1 = _REPO / "plugins" / "phase1"

PHASE1_TOOLS = [
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


def _load_plugin(tool_id: str):
    mod_path = _PHASE1 / tool_id / "plugin_stub.py"
    spec = importlib.util.spec_from_file_location(f"hb_phase1_{tool_id}", mod_path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[f"hb_phase1_{tool_id}"] = mod
    spec.loader.exec_module(mod)
    return mod


@pytest.mark.parametrize("tool_id", PHASE1_TOOLS)
def test_phase1_plugin_hnf_roundtrip(tool_id: str) -> None:
    mod = _load_plugin(tool_id)
    doc = mod.export_to_hnf(f"/tmp/{tool_id}.proj")
    assert doc["manifest"]["hnf_version"] == "0.1"
    assert doc["manifest"]["disciplines"] == [mod.DOMAIN]
    replay = mod.import_from_hnf(f"/tmp/{tool_id}.proj", doc)
    assert len(replay) >= 1
    assert replay[0]["kind"] == mod.MUTATION_KIND


def test_phase1_matrix_has_ten_tools() -> None:
    assert len(PHASE1_TOOLS) == 10
