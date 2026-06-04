"""Headless roundtrip integration tests (M1 harness)."""

from __future__ import annotations

import sys
import tempfile
from pathlib import Path

_REGRESSION = Path(__file__).resolve().parents[1]
_REPO = _REGRESSION.parents[1]
if str(_REGRESSION) not in sys.path:
    sys.path.insert(0, str(_REGRESSION))

from mutation_hook import ensure_sidecars_built  # noqa: E402
from roundtrip import run_roundtrip_suite  # noqa: E402


def test_headless_kicad_roundtrip_corpus() -> None:
    ensure_sidecars_built()
    corpus = _REPO / "corpora" / "roundtrip" / "manifest.json"
    trace = run_roundtrip_suite(corpus=corpus)
    kicad_cases = [
        r
        for case in trace.get("cases", [])
        for r in case.get("results", [])
        if r.get("sidecar") == "kicad"
    ]
    assert kicad_cases, "expected KiCad roundtrip case"
    assert all(r.get("ok") for r in kicad_cases), kicad_cases


def test_headless_freecad_roundtrip_corpus() -> None:
    ensure_sidecars_built()
    corpus = _REPO / "corpora" / "roundtrip" / "manifest.json"
    trace = run_roundtrip_suite(corpus=corpus)
    freecad_cases = [
        r
        for case in trace.get("cases", [])
        for r in case.get("results", [])
        if r.get("sidecar") == "freecad"
    ]
    assert freecad_cases, "expected FreeCAD roundtrip case"
    assert all(r.get("ok") for r in freecad_cases), freecad_cases


def test_roundtrip_export_fingerprint_stable_per_sidecar() -> None:
    """Double-export fingerprint stability for each tool case."""
    ensure_sidecars_built()
    with tempfile.TemporaryDirectory() as tmp:
        corpus = Path(tmp) / "manifest.json"
        corpus.write_text(
            (_REPO / "corpora" / "roundtrip" / "manifest.json").read_text()
        )
        trace = run_roundtrip_suite(corpus=corpus)
    assert trace.get("ok"), trace
