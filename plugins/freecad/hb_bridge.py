# Copyright 2026 HummingBird Labs
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""
FreeCAD workbench entry for HNF v0.1 mechanical save/load (HB Bridge M1).

Rust mapping: crates/hnf-freecad via hb-bridge-hnf CLI.
"""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from typing import Any


def _load_hnf_client():
    mod_path = Path(__file__).resolve().with_name("hnf_client.py")
    spec = importlib.util.spec_from_file_location("hb_freecad_hnf_client", mod_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"cannot load {mod_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_hnf = _load_hnf_client()
freecad_export_mechanical = _hnf.freecad_export_mechanical
freecad_import_mechanical = _hnf.freecad_import_mechanical

PLUGIN_ID = "hb_bridge_freecad"
PLUGIN_VERSION = "0.1.0"


def _mutation(kind: str, payload: dict[str, Any]) -> dict[str, Any]:
    return {"kind": kind, "payload": payload}


def _document_to_mutations(doc: Any) -> list[dict[str, Any]]:
    try:
        import FreeCAD  # type: ignore[import-untyped]
    except ImportError:
        return []

    mutations: list[dict[str, Any]] = []
    for obj in doc.Objects:
        if not hasattr(obj, "Shape") or obj.Shape.isNull():
            continue
        solid_id = f"solid-{obj.Name}"
        volume = float(obj.Shape.Volume) if hasattr(obj.Shape, "Volume") else 0.0
        material = getattr(obj, "Material", None)
        material_name = ""
        if material is not None and hasattr(material, "Label"):
            material_name = str(material.Label)
        mutations.append(
            _mutation(
                "mechanical/solid/upsert",
                {
                    "commitId": "freecad-export",
                    "solid_id": solid_id,
                    "name": str(obj.Label or obj.Name),
                    "material": material_name or "unknown",
                    "volume_mm3": volume,
                },
            )
        )
    if not mutations:
        mutations.append(
            _mutation(
                "mechanical/solid/upsert",
                {
                    "commitId": "freecad-export",
                    "solid_id": "solid-placeholder",
                    "name": "Placeholder",
                    "material": "unknown",
                    "volume_mm3": 0.0,
                },
            )
        )
    del FreeCAD
    return mutations


def export_mechanical_to_hnf(
    project_path: str,
    *,
    hnf_version: str = "0.1",
    mutations: list[dict[str, Any]] | None = None,
    step_content_hash: str | None = None,
) -> dict[str, Any]:
    """Export HNF mechanical domain JSON (v0.1) with optional STEP blob ref (ADR-0002)."""
    del hnf_version
    if mutations is None:
        try:
            import FreeCAD  # type: ignore[import-untyped]

            doc = FreeCAD.ActiveDocument
            if doc is not None:
                mutations = _document_to_mutations(doc)
        except ImportError:
            mutations = None
    if mutations is None:
        mutations = [
            _mutation(
                "mechanical/solid/upsert",
                {
                    "commitId": "freecad-export",
                    "solid_id": "solid-placeholder",
                    "name": "Placeholder",
                    "material": "unknown",
                    "volume_mm3": 0.0,
                },
            )
        ]
    doc = freecad_export_mechanical(project_path, mutations)
    if step_content_hash and isinstance(doc.get("objects"), list):
        for obj in doc["objects"]:
            props = obj.get("properties", {})
            if props.get("domain") == "mechanical":
                solids = props.get("solids", [])
                if solids:
                    solids[0].setdefault("geometry_blobs", []).append(
                        {"format": "step", "content_hash": step_content_hash}
                    )
                props.setdefault("content_hash", step_content_hash)
                obj["properties"] = props
    return doc


def import_mechanical_from_hnf(
    project_path: str,
    hnf_document: dict[str, Any],
    *,
    merge: bool = False,
) -> list[dict[str, Any]]:
    """Apply HNF mechanical domain; returns normalized mutations."""
    del project_path, merge
    return freecad_import_mechanical(hnf_document)


def write_hnf_sidecar(project_path: str, document: dict[str, Any]) -> Path:
    path = Path(project_path).with_suffix(".mechanical.hnf.json")
    path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    return path
