from __future__ import annotations

import sys
from pathlib import Path

_REGRESSION_DIR = Path(__file__).resolve().parents[1]
if str(_REGRESSION_DIR) not in sys.path:
    sys.path.insert(0, str(_REGRESSION_DIR))

from mutation_hook import _deterministic_mutations, _expected_kicad_upserts
from sidecar_client import SidecarSession


def test_deterministic_mutations_are_stable() -> None:
    seed = Path("/tmp/seed-a.json")
    a = _deterministic_mutations(seed, count=3, sidecar="kicad")
    b = _deterministic_mutations(seed, count=3, sidecar="kicad")
    assert a == b
    assert a[0]["kind"] == "schematic.symbol.upsert"


def test_wait_for_scene_traces_returns_when_counts_met() -> None:
    session = SidecarSession(binary=Path("/nonexistent"))
    session.scene_traces = [
        {"method": "hcp/sceneGraph/upsertNodes", "params": {}},
        {"method": "hcp/sceneGraph/upsertEdges", "params": {}},
    ]
    session.wait_for_scene_traces(node_upserts=1, edge_upserts=1, timeout_s=0.5)


def test_expected_kicad_node_types() -> None:
    doc = "hcp://doc/board"
    mutations = [
        {"kind": "schematic.symbol.upsert", "payload": {}},
        {"kind": "pcb.track.upsert", "payload": {}},
    ]
    expected = _expected_kicad_upserts(doc, mutations)
    assert expected[0]["nodes"]["nodeTypes"] == ["kicad.schematic.element"]
    assert expected[1]["nodes"]["nodeTypes"] == ["kicad.pcb.element"]
