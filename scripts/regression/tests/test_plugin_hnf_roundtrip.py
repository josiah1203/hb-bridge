"""Headless HNF v0.1 plugin roundtrip tests (M1 KiCad/FreeCAD)."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]
_KICAD_PLUGIN = _REPO / "plugins" / "kicad"
_FREECAD_PLUGIN = _REPO / "plugins" / "freecad"


def _ensure_hnf_cli() -> Path:
    cli = _REPO / "target" / "debug" / "hb-bridge-hnf"
    if cli.is_file():
        return cli
    subprocess.run(
        ["cargo", "build", "-q", "-p", "hb-bridge-hnf"],
        cwd=_REPO,
        check=True,
    )
    assert cli.is_file(), f"missing CLI: {cli}"
    return cli


def test_kicad_layout_hnf_roundtrip_via_plugin() -> None:
    _ensure_hnf_cli()
    sys.path.insert(0, str(_KICAD_PLUGIN))
    from plugin_stub import export_layout_to_hnf, import_layout_from_hnf  # noqa: WPS433

    mutations = [
        {"kind": "pcb.track.upsert", "payload": {"net": "GND", "layer": "F.Cu"}},
    ]
    doc = export_layout_to_hnf("/tmp/test.kicad_pcb", mutations=mutations)
    assert doc.get("manifest", {}).get("hnf_version") == "0.1"
    assert doc.get("manifest", {}).get("disciplines") == ["layout"]
    replay = import_layout_from_hnf("/tmp/test.kicad_pcb", doc)
    assert any(m.get("kind") == "pcb.track.upsert" for m in replay)


def test_kicad_schematic_hnf_roundtrip_via_plugin() -> None:
    _ensure_hnf_cli()
    sys.path.insert(0, str(_KICAD_PLUGIN))
    from plugin_stub import export_schematic_to_hnf, import_schematic_from_hnf  # noqa: WPS433

    mutations = [
        {"kind": "schematic.symbol.upsert", "payload": {"ref": "R1", "value": "10k"}},
    ]
    doc = export_schematic_to_hnf("/tmp/test.kicad_sch", mutations=mutations)
    assert doc.get("manifest", {}).get("disciplines") == ["schematic"]
    replay = import_schematic_from_hnf("/tmp/test.kicad_sch", doc)
    assert any(m.get("payload", {}).get("refdes") == "R1" for m in replay)


def test_freecad_mechanical_hnf_roundtrip_via_plugin() -> None:
    _ensure_hnf_cli()
    import importlib.util

    mod_path = _FREECAD_PLUGIN / "hb_bridge.py"
    spec = importlib.util.spec_from_file_location("hb_bridge_freecad", mod_path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    export_mechanical_to_hnf = mod.export_mechanical_to_hnf
    import_mechanical_from_hnf = mod.import_mechanical_from_hnf

    mutations = [
        {
            "kind": "mechanical/solid/upsert",
            "payload": {
                "commitId": "test",
                "solid_id": "solid-bracket-1",
                "name": "Bracket",
                "material": "Aluminum",
                "volume_mm3": 42.0,
            },
        }
    ]
    doc = export_mechanical_to_hnf("/tmp/bracket.FCStd", mutations=mutations)
    assert doc.get("manifest", {}).get("disciplines") == ["mechanical"]
    replay = import_mechanical_from_hnf("/tmp/bracket.FCStd", doc)
    assert replay[0]["payload"]["solid_id"] == "solid-bracket-1"


def test_corpus_fixtures_match_plugin_roundtrip() -> None:
    kicad_fixture = json.loads((_REPO / "tests/fixtures/kicad/minimal_board.json").read_text())
    freecad_fixture = json.loads(
        (_REPO / "tests/fixtures/freecad/minimal_bracket.json").read_text()
    )
    _ensure_hnf_cli()
    import importlib.util

    sys.path.insert(0, str(_KICAD_PLUGIN))
    from plugin_stub import export_layout_to_hnf, import_layout_from_hnf  # noqa: WPS433

    mod_path = _FREECAD_PLUGIN / "hb_bridge.py"
    spec = importlib.util.spec_from_file_location("hb_bridge_freecad", mod_path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    export_mechanical_to_hnf = mod.export_mechanical_to_hnf
    import_mechanical_from_hnf = mod.import_mechanical_from_hnf

    layout_mutations = [m for m in kicad_fixture["mutations"] if m["kind"].startswith("pcb.")]
    doc = export_layout_to_hnf(kicad_fixture["documentUri"], mutations=layout_mutations)
    import_layout_from_hnf(kicad_fixture["documentUri"], doc)

    doc = export_mechanical_to_hnf(
        freecad_fixture["documentUri"], mutations=freecad_fixture["mutations"]
    )
    import_mechanical_from_hnf(freecad_fixture["documentUri"], doc)
