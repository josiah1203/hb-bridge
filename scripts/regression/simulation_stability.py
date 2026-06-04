"""Simulation stability regression: stub/host subprocess + golden envelope comparison."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any

from common import load_goldens, load_manifest, normalize_sim_envelope


def _default_stub_command(engine: str, job_id: str) -> tuple[str, list[str]]:
    """Mirror rust/crates/simulation-sidecars default_stub_command."""
    return (
        "sh",
        [
            "-lc",
            f"printf 'stub-run {engine} {job_id}\\n'",
        ],
    )


def _run_simulation_config(
    config: dict[str, Any], *, work_dir: Path | None = None
) -> dict[str, Any]:
    engine = str(config.get("engine", "ngspice"))
    job_id = str(config.get("job_id", config.get("jobId", "regression-job")))
    command = config.get("command")
    if command:
        program, args = str(command[0]), list(command[1:])
    else:
        program, args = _default_stub_command(engine, job_id)

    outcome = subprocess.run(
        [program, *args],
        capture_output=True,
        text=True,
        check=False,
        cwd=str(work_dir) if work_dir else config.get("output_dir"),
    )
    exit_code = int(outcome.returncode)
    output_dir = str(work_dir or config.get("output_dir") or ".").rstrip("/")
    artifacts = config.get("expected_artifacts") or config.get("expectedArtifacts")
    if not artifacts:
        artifacts = [
            {
                "path": f"{output_dir}/{engine}-{job_id}.json",
                "content_type": "application/json",
                "role": "result-manifest",
            }
        ]

    return {
        "engine": engine,
        "job_id": job_id,
        "status": "succeeded" if exit_code == 0 else "failed",
        "exit_code": exit_code,
        "artifacts": artifacts,
        "stdout": outcome.stdout,
        "stderr": outcome.stderr,
    }


def _envelopes_match(observed: dict[str, Any], golden: dict[str, Any]) -> bool:
    obs = normalize_sim_envelope(observed)
    gld = normalize_sim_envelope(golden)
    if not gld.get("stdout"):
        obs = {k: v for k, v in obs.items() if k != "stdout"}
        gld = {k: v for k, v in gld.items() if k != "stdout"}
    return obs == gld


def run_simulation_job(config: dict[str, Any]) -> dict[str, Any]:
    """Public entry used by CLI; writes under output_dir when set."""
    work = Path(config["output_dir"]) if config.get("output_dir") else None
    return _run_simulation_config(config, work_dir=work)


def run_simulation_stability_suite(*, corpus: Path, goldens: Path) -> dict[str, Any]:
    manifest = load_manifest(corpus)
    golden_index = load_goldens(goldens)
    corpus_dir = corpus if corpus.is_dir() else corpus.parent

    golden_by_id = {
        entry.get("id"): entry for entry in golden_index.get("cases") or []
    }

    trace: dict[str, Any] = {
        "suite": "simulation-stability",
        "corpus": str(corpus.resolve()),
        "goldens": str(goldens.resolve()),
        "cases": [],
        "ok": True,
        "mode": "subprocess-stub",
    }

    for case in manifest.get("cases") or []:
        case_id = case.get("id", "unknown")
        case_path = corpus_dir / f"{case_id}.json"
        if case_path.is_file():
            import json

            config = json.loads(case_path.read_text())
        else:
            config = case.get("simulation") or case

        work_dir = Path(
            str(config.get("output_dir", config.get("outputDir", "/tmp/hcp-sim")))
        )
        work_dir.mkdir(parents=True, exist_ok=True)

        observed = _run_simulation_config(config, work_dir=work_dir)
        golden = golden_by_id.get(case_id)
        errors: list[str] = []
        if golden:
            if not _envelopes_match(observed, golden):
                errors.append(
                    f"envelope mismatch observed={normalize_sim_envelope(observed)} "
                    f"golden={normalize_sim_envelope(golden)}"
                )
        elif observed.get("status") != "succeeded":
            errors.append(f"simulation failed: {observed}")

        case_result = {
            "caseId": case_id,
            "engine": observed.get("engine"),
            "ok": not errors,
            "errors": errors,
            "envelope": normalize_sim_envelope(observed),
        }
        trace["ok"] = trace["ok"] and case_result["ok"]
        trace["cases"].append(case_result)

    return trace
