"""Unit tests for mutation-hook helpers (no sidecar binaries required)."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

_REGRESSION = Path(__file__).resolve().parents[1]
if str(_REGRESSION) not in sys.path:
    sys.path.insert(0, str(_REGRESSION))

from mutation_hook import (  # noqa: E402
    _deterministic_mutations,
    _expected_kicad_upserts,
    _load_seed_config,
)


def test_load_seed_config(tmp_path: Path) -> None:
    seed_file = tmp_path / "mutation_seed.json"
    seed_file.write_text(
        '{"documentUri": "hcp://test/doc", "projectId": "p1", "sidecars": ["kicad"]}'
    )
    cfg = _load_seed_config(seed_file)
    assert cfg["documentUri"] == "hcp://test/doc"


def test_deterministic_mutations_stable(tmp_path: Path) -> None:
    seed = tmp_path / "mutation_seed.json"
    seed.write_text('{"documentUri": "hcp://x"}')
    a = _deterministic_mutations(seed, count=5, sidecar="kicad")
    b = _deterministic_mutations(seed, count=5, sidecar="kicad")
    assert a == b
    assert len(a) == 5


def test_expected_kicad_upserts_node_types() -> None:
    mutations = [
        {"kind": "schematic.symbol.upsert", "payload": {"ref": "R1"}},
        {"kind": "pcb.track.upsert", "payload": {"net": "GND"}},
    ]
    expected = _expected_kicad_upserts("hcp://doc/a", mutations)
    assert expected[0]["nodes"]["nodeTypes"] == ["kicad.schematic.element"]
    assert expected[1]["nodes"]["nodeTypes"] == ["kicad.pcb.element"]
