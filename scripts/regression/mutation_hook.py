"""Mutation-hook regression suite: sidecar applyMutations + scene graph validation."""

from __future__ import annotations

import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

from sidecar_client import SidecarSession

REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_DIR = REPO_ROOT / "rust"

KICAD_MUTATION_KINDS = (
    ("schematic.symbol.upsert", {"ref": "R1"}),
    ("schematic.wire.upsert", {"net": "GND"}),
    ("pcb.track.upsert", {"net": "VCC"}),
    ("pcb.via.upsert", {"diameter_mm": 0.3}),
)

FREECAD_MUTATION_KINDS = (
    (
        "mechanical/solid/upsert",
        {
            "commitId": "regression-commit",
            "solid_id": "solid-1",
            "name": "Bracket",
            "material": "Aluminum",
            "volume_mm3": 42.0,
        },
    ),
    (
        "mechanical/constraint/upsert",
        {
            "commitId": "regression-commit",
            "constraint_id": "constraint-1",
            "from_solid_id": "solid-1",
            "to_solid_id": "solid-2",
            "constraint_type": "mate",
        },
    ),
)


def _ensure_sidecar_binary(crate: str, bin_name: str) -> Path:
    debug = RUST_DIR / "target" / "debug" / bin_name
    if debug.is_file():
        return debug
    subprocess.run(
        ["cargo", "build", "-p", crate, "--bin", bin_name],
        cwd=RUST_DIR,
        check=True,
        capture_output=True,
        text=True,
    )
    if not debug.is_file():
        raise FileNotFoundError(f"expected sidecar binary at {debug}")
    return debug


def repo_root() -> Path:
    return REPO_ROOT


def ensure_sidecars_built(_repo_root: Path | None = None) -> None:
    _ensure_sidecar_binary("kicad-sidecar", "kicad-sidecar")
    _ensure_sidecar_binary("freecad-sidecar", "freecad-sidecar")


def _load_seed_config(seed: Path) -> dict[str, Any]:
    if seed.is_dir():
        for name in ("mutation_seed.json", "minimal_seed.json"):
            candidate = seed / name
            if candidate.is_file():
                seed = candidate
                break
        else:
            raise FileNotFoundError(
                f"seed directory {seed} must contain mutation_seed.json or minimal_seed.json"
            )
    data = json.loads(seed.read_text())
    if "documentUri" not in data:
        raise ValueError("seed config requires documentUri")
    project = data.get("project") or {}
    if "projectId" not in data and project.get("projectId"):
        data["projectId"] = project["projectId"]
    if "workspaceRoot" not in data and project.get("workspaceRoot"):
        data["workspaceRoot"] = project["workspaceRoot"]
    return data


def _deterministic_mutations(
    seed_path: Path, *, count: int, sidecar: str
) -> list[dict[str, Any]]:
    digest = hashlib.sha256(f"{seed_path}:{sidecar}".encode()).digest()
    kinds = KICAD_MUTATION_KINDS if sidecar == "kicad" else FREECAD_MUTATION_KINDS
    mutations: list[dict[str, Any]] = []
    for i in range(count):
        kind, payload = kinds[i % len(kinds)]
        item = {"kind": kind, "payload": dict(payload)}
        if sidecar == "kicad":
            item["payload"]["index"] = i
            item["payload"]["seedByte"] = digest[i % len(digest)]
        mutations.append(item)
    return mutations


def _expected_kicad_upserts(
    document_uri: str, mutations: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    """Mirror rust/crates/kicad-sidecar map_mutation_to_scene_delta."""
    expected: list[dict[str, Any]] = []
    for index, mutation in enumerate(mutations):
        kind = mutation["kind"]
        safe_kind = kind.replace("/", ".").replace(" ", "_")
        commit_id = f"{document_uri}:{index}"
        node_id = f"node:{safe_kind}:{index}"
        if kind.startswith("schematic."):
            node_type = "kicad.schematic.element"
        elif kind.startswith("pcb."):
            node_type = "kicad.pcb.element"
        else:
            node_type = "kicad.mutation"
        expected.append(
            {
                "nodes": {
                    "commitId": commit_id,
                    "count": 1,
                    "nodeTypes": [node_type],
                    "nodeIds": [node_id],
                },
                "edges": {
                    "commitId": commit_id,
                    "count": 1,
                    "edgeType": "applies_mutation",
                    "fromNodeId": f"doc:{document_uri}",
                    "toNodeId": node_id,
                },
            }
        )
    return expected


def _validate_scene_node(params: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    nodes = params.get("nodes") or []
    for node in nodes:
        for key in ("nodeId", "nodeType", "attributes"):
            if key not in node:
                errors.append(f"missing node field {key}")
    return errors


def _validate_scene_edge(params: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    edges = params.get("edges") or []
    for edge in edges:
        for key in ("edgeId", "fromNodeId", "toNodeId", "edgeType", "attributes"):
            if key not in edge:
                errors.append(f"missing edge field {key}")
    return errors


def _run_kicad_sidecar(
    *,
    seed_path: Path,
    seed_config: dict[str, Any],
    mutations: list[dict[str, Any]],
) -> dict[str, Any]:
    binary = _ensure_sidecar_binary("kicad-sidecar", "kicad-sidecar")
    document_uri = seed_config["documentUri"]
    workspace = seed_config.get(
        "workspaceRoot",
        str(seed_path.parent if seed_path.is_file() else seed_path),
    )
    project_id = seed_config.get("projectId", "regression-project")

    session = SidecarSession(binary=binary)
    sidecar_result: dict[str, Any] = {"sidecar": "kicad", "ok": False, "errors": []}
    try:
        session.start()
        ping = session.ping()
        session.project_open(
            project_id=project_id,
            workspace_root=workspace,
        )
        apply_result = session.apply_mutations(
            document_uri=document_uri,
            mutations=mutations,
        )
        session.wait_for_scene_traces(
            node_upserts=len(mutations),
            edge_upserts=len(mutations),
        )

        expected = _expected_kicad_upserts(document_uri, mutations)
        node_traces = [
            t for t in session.scene_traces if t.get("method") == "hcp/sceneGraph/upsertNodes"
        ]
        edge_traces = [
            t for t in session.scene_traces if t.get("method") == "hcp/sceneGraph/upsertEdges"
        ]

        validation_errors: list[str] = []
        if apply_result.get("applied") != len(mutations):
            validation_errors.append(
                f"applied={apply_result.get('applied')} expected={len(mutations)}"
            )
        if apply_result.get("errors"):
            validation_errors.append(f"mutation errors: {apply_result['errors']}")
        if len(node_traces) != len(mutations):
            validation_errors.append(
                f"node upsert traces={len(node_traces)} expected={len(mutations)}"
            )
        if len(edge_traces) != len(mutations):
            validation_errors.append(
                f"edge upsert traces={len(edge_traces)} expected={len(mutations)}"
            )

        for trace in node_traces + edge_traces:
            params = trace.get("params") or {}
            validation_errors.extend(_validate_scene_node(params))
            validation_errors.extend(_validate_scene_edge(params))

        for idx, exp in enumerate(expected):
            if idx >= len(node_traces):
                break
            nparams = node_traces[idx]["params"]
            if nparams.get("commitId") != exp["nodes"]["commitId"]:
                validation_errors.append(f"commitId mismatch at mutation {idx}")
            node_types = [n["nodeType"] for n in nparams.get("nodes", [])]
            if node_types != exp["nodes"]["nodeTypes"]:
                validation_errors.append(f"nodeType mismatch at mutation {idx}")

        sidecar_result.update(
            {
                "ok": not validation_errors,
                "errors": validation_errors,
                "ping": ping,
                "apply": apply_result,
                "sceneTraces": session.scene_traces,
                "expectedUpserts": expected,
            }
        )
    finally:
        session.close()

    sidecar_result["rpc"] = [
        {"direction": ex.direction, "payload": ex.payload} for ex in session.exchanges
    ]
    return sidecar_result


def _run_freecad_sidecar(
    *,
    seed_path: Path,
    seed_config: dict[str, Any],
    mutations: list[dict[str, Any]],
) -> dict[str, Any]:
    binary = _ensure_sidecar_binary("freecad-sidecar", "freecad-sidecar")
    document_uri = seed_config["documentUri"]
    workspace = seed_config.get(
        "workspaceRoot",
        str(seed_path.parent if seed_path.is_file() else seed_path),
    )
    project_id = seed_config.get("projectId", "regression-project")

    session = SidecarSession(binary=binary)
    sidecar_result: dict[str, Any] = {"sidecar": "freecad", "ok": False, "errors": []}
    supported = {
        "mechanical/solid/upsert",
        "mechanical/constraint/upsert",
    }
    expected_applied = sum(1 for m in mutations if m["kind"] in supported)

    try:
        session.start()
        ping = session.ping()
        session.project_open(
            project_id=project_id,
            workspace_root=workspace,
        )
        apply_result = session.apply_mutations(
            document_uri=document_uri,
            mutations=mutations,
        )
        validation_errors: list[str] = []
        if apply_result.get("applied") != expected_applied:
            validation_errors.append(
                f"applied={apply_result.get('applied')} expected={expected_applied}"
            )
        sidecar_result.update(
            {
                "ok": not validation_errors,
                "errors": validation_errors,
                "ping": ping,
                "apply": apply_result,
            }
        )
    finally:
        session.close()

    sidecar_result["rpc"] = [
        {"direction": ex.direction, "payload": ex.payload} for ex in session.exchanges
    ]
    return sidecar_result


def run_mutation_hook_suite(
    *,
    seed: Path,
    mutations_count: int,
    sidecars: list[str] | None = None,
) -> dict[str, Any]:
    seed_path = seed.resolve()
    seed_config = _load_seed_config(seed_path)

    sidecar_list = sidecars or seed_config.get("sidecars", ["kicad", "freecad"])
    trace: dict[str, Any] = {
        "suite": "mutation-hook",
        "seed": str(seed_path),
        "mutationCount": mutations_count,
        "documentUri": seed_config["documentUri"],
        "sidecars": [],
    }

    all_ok = True
    for sidecar in sidecar_list:
        mutation_list = _deterministic_mutations(
            seed_path, count=mutations_count, sidecar=sidecar
        )
        if sidecar == "kicad":
            result = _run_kicad_sidecar(
                seed_path=seed_path,
                seed_config=seed_config,
                mutations=mutation_list,
            )
        elif sidecar == "freecad":
            result = _run_freecad_sidecar(
                seed_path=seed_path,
                seed_config=seed_config,
                mutations=mutation_list,
            )
        else:
            result = {
                "sidecar": sidecar,
                "ok": False,
                "errors": [f"unsupported sidecar: {sidecar}"],
            }
        all_ok = all_ok and bool(result.get("ok"))
        trace["sidecars"].append(result)

    trace["ok"] = all_ok
    return trace
