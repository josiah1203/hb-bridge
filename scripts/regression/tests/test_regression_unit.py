"""Unit tests for roundtrip, DRC, and simulation regression helpers."""

from __future__ import annotations

import json
import sys
from pathlib import Path

_REGRESSION = Path(__file__).resolve().parents[1]
if str(_REGRESSION) not in sys.path:
    sys.path.insert(0, str(_REGRESSION))

from common import (  # noqa: E402
    fingerprint_artifacts,
    load_manifest,
    normalize_sim_envelope,
    normalize_violations,
)
from drc import derive_violations_from_mutations  # noqa: E402
from simulation_stability import (  # noqa: E402
    _default_stub_command,
    _envelopes_match,
    _run_simulation_config,
)


def test_load_roundtrip_manifest() -> None:
    corpus = _REGRESSION / "fixtures" / "roundtrip_corpus"
    manifest = load_manifest(corpus)
    assert len(manifest["cases"]) >= 1


def test_fingerprint_artifacts_stable() -> None:
    artifacts = [
        {"path": "/tmp/a.step", "contentType": "model/step"},
        {"path": "/tmp/b.svg", "contentType": "image/svg+xml"},
    ]
    assert fingerprint_artifacts(artifacts) == fingerprint_artifacts(
        list(reversed(artifacts))
    )


def test_derive_violations_from_mutations() -> None:
    mutations = [
        {"kind": "pcb.track.upsert", "payload": {"net": "VCC"}},
        {"kind": "pcb.via.upsert", "payload": {"diameter_mm": 0.3}},
    ]
    rules = [
        {
            "rule": "pcb.track.width",
            "severity": "warning",
            "whenKind": "pcb.track.upsert",
            "whenField": "net",
            "whenEquals": "VCC",
            "location": "pcb/track/0",
            "message": "track width below recommended minimum",
        },
        {
            "rule": "pcb.via.diameter",
            "severity": "error",
            "whenKind": "pcb.via.upsert",
            "whenField": "diameter_mm",
            "whenEquals": "0.3",
            "location": "pcb/via/3",
            "message": "via diameter below process minimum",
        },
    ]
    violations = derive_violations_from_mutations(mutations, rules=rules)
    golden = (
        _REGRESSION / "fixtures" / "drc_goldens" / "kicad-clearance-warning.json"
    ).read_text()
    expected = json.loads(golden)["violations"]
    assert normalize_violations(violations) == normalize_violations(expected)


def test_default_stub_command_matches_rust_shape() -> None:
    program, args = _default_stub_command("ngspice", "job-1")
    assert program == "sh"
    assert "stub-run ngspice job-1" in args[-1]


def test_run_simulation_config_stub(tmp_path: Path) -> None:
    config = {
        "engine": "ngspice",
        "job_id": "regression-stub-1",
        "output_dir": str(tmp_path),
    }
    envelope = _run_simulation_config(config, work_dir=tmp_path)
    normalized = normalize_sim_envelope(envelope)
    golden = {
        "engine": "ngspice",
        "job_id": "regression-stub-1",
        "status": "succeeded",
        "exit_code": 0,
        "artifacts": [
            {
                "path": "ngspice-regression-stub-1.json",
                "content_type": "application/json",
                "role": "result-manifest",
            }
        ],
    }
    assert _envelopes_match(normalized, golden)


def test_normalize_violations_sort_order() -> None:
    a = normalize_violations(
        [{"rule": "b", "severity": "error", "location": "1", "message": ""}]
    )
    b = normalize_violations(
        [{"rule": "a", "severity": "warning", "location": "2", "message": ""}]
    )
    assert a != b
