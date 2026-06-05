"""M2 bridge panel stubs (KiCad + FreeCAD)."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]


def _load_panel(module_path: Path, module_name: str):
    spec = importlib.util.spec_from_file_location(module_name, module_path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = mod
    spec.loader.exec_module(mod)
    return mod


def test_kicad_panel_stub_roundtrip() -> None:
    panel = _load_panel(
        _REPO / "plugins/kicad/panel/bridge_panel.py",
        "hb_kicad_panel",
    )
    panel.set_connected(False)
    panel.record_hnf_export("/tmp/board.layout.hnf.json")
    status = panel.panel_status()
    assert status["connected"] is True
    assert status["last_hnf_export"].endswith(".hnf.json")
    assert "hbw" in panel.hbw_deep_link().lower() or "127.0.0.1" in panel.hbw_deep_link()


def test_freecad_panel_stub_roundtrip() -> None:
    panel = _load_panel(
        _REPO / "plugins/freecad/panel/bridge_panel.py",
        "hb_freecad_panel",
    )
    panel.set_connected(False)
    panel.record_hnf_export("/tmp/part.mechanical.hnf.json")
    status = panel.panel_status()
    assert status["adapter"] == "hnf-freecad"
    assert status["connected"] is True
