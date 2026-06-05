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
KiCad 8 action plugin for HNF v0.1 save/load (HB Bridge M1).

Domains: schematic (.kicad_sch), layout (.kicad_pcb).
Rust mapping: crates/hnf-kicad via hb-bridge-hnf CLI.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

try:
    from hnf_client import (
        kicad_export_layout,
        kicad_export_schematic,
        kicad_import_layout,
        kicad_import_schematic,
    )
except ImportError:  # pragma: no cover
    from .hnf_client import (  # type: ignore[no-redef]
        kicad_export_layout,
        kicad_export_schematic,
        kicad_import_layout,
        kicad_import_schematic,
    )

try:
    import pcbnew  # type: ignore[import-untyped]
except ImportError:  # pragma: no cover - dev/CI without KiCad
    pcbnew = None  # type: ignore[assignment]

PLUGIN_ID = "hb_bridge_kicad"
PLUGIN_VERSION = "0.1.0"


def _mutation(kind: str, payload: dict[str, Any]) -> dict[str, Any]:
    return {"kind": kind, "payload": payload}


def _board_to_layout_mutations(board: Any) -> list[dict[str, Any]]:
    mutations: list[dict[str, Any]] = []
    for footprint in board.GetFootprints():
        pos = footprint.GetPosition()
        mutations.append(
            _mutation(
                "pcb.footprint.upsert",
                {
                    "refdes": footprint.GetReference(),
                    "layer": footprint.GetLayerName(),
                    "x": pcbnew.ToMM(pos.x),
                    "y": pcbnew.ToMM(pos.y),
                    "rotation_deg": footprint.GetOrientation().AsDegrees(),
                },
            )
        )
    for track in board.GetTracks():
        if not hasattr(track, "GetNetname"):
            continue
        net = track.GetNetname()
        if not net:
            continue
        mutations.append(
            _mutation(
                "pcb.track.upsert",
                {
                    "net": net,
                    "layer": board.GetLayerName(track.GetLayer()),
                    "width_mm": pcbnew.ToMM(track.GetWidth()),
                },
            )
        )
    return mutations


def _apply_layout_mutations(board: Any, mutations: list[dict[str, Any]]) -> None:
    for entry in mutations:
        kind = entry.get("kind", "")
        payload = entry.get("payload", {})
        if kind != "pcb.footprint.upsert":
            continue
        refdes = payload.get("refdes") or payload.get("ref")
        if not refdes:
            continue
        footprint = board.FindFootprintByReference(str(refdes))
        if footprint is None:
            footprint = pcbnew.FOOTPRINT(board)
            footprint.SetReference(str(refdes))
            board.Add(footprint)
        if payload.get("layer"):
            footprint.SetLayerName(str(payload["layer"]))


def export_schematic_to_hnf(
    project_path: str,
    *,
    hnf_version: str = "0.1",
    mutations: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    """Export HNF schematic domain JSON (v0.1)."""
    del hnf_version
    if mutations is None:
        mutations = [
            _mutation("schematic.symbol.upsert", {"refdes": "R1", "value": "placeholder"}),
        ]
    return kicad_export_schematic(project_path, mutations)


def import_schematic_from_hnf(
    project_path: str,
    hnf_document: dict[str, Any],
    *,
    merge: bool = False,
) -> list[dict[str, Any]]:
    """Apply HNF schematic domain; returns normalized mutations."""
    del project_path, merge
    return kicad_import_schematic(hnf_document)


def export_layout_to_hnf(
    project_path: str,
    *,
    hnf_version: str = "0.1",
    mutations: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    """Export HNF layout domain JSON (v0.1)."""
    del hnf_version
    if mutations is None and pcbnew is not None:
        board = pcbnew.GetBoard()
        if board is not None:
            mutations = _board_to_layout_mutations(board)
    if mutations is None:
        mutations = [
            _mutation("pcb.track.upsert", {"net": "GND", "layer": "F.Cu"}),
        ]
    return kicad_export_layout(project_path, mutations)


def import_layout_from_hnf(
    project_path: str,
    hnf_document: dict[str, Any],
    *,
    merge: bool = False,
) -> list[dict[str, Any]]:
    """Apply HNF layout domain into the active board when pcbnew is available."""
    mutations = kicad_import_layout(hnf_document)
    if pcbnew is not None and not merge:
        board = pcbnew.GetBoard()
        if board is not None:
            _apply_layout_mutations(board, mutations)
    del project_path
    return mutations


def write_hnf_sidecar(project_path: str, domain: str, document: dict[str, Any]) -> Path:
    """Write `<project>.<domain>.hnf.json` next to the KiCad project."""
    path = Path(project_path).with_suffix(f".{domain}.hnf.json")
    path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
    return path


if pcbnew is not None:

    class HbBridgeExportLayout(pcbnew.ActionPlugin):
        """Tools → External Plugins → HB Bridge: Export layout to HNF."""

        def defaults(self) -> None:
            self.name = "HB Bridge: Export layout (HNF)"
            self.category = "HB Bridge"
            self.description = "Export current board to HNF layout domain v0.1"
            self.show_toolbar_button = False

        def Run(self) -> None:
            project = pcbnew.GetBoard().GetFileName() if pcbnew.GetBoard() else "untitled.kicad_pcb"
            doc = export_layout_to_hnf(project)
            write_hnf_sidecar(project, "layout", doc)

    class HbBridgeImportLayout(pcbnew.ActionPlugin):
        """Tools → External Plugins → HB Bridge: Import layout from HNF."""

        def defaults(self) -> None:
            self.name = "HB Bridge: Import layout (HNF)"
            self.category = "HB Bridge"
            self.description = "Import HNF layout domain into current board"
            self.show_toolbar_button = False

        def Run(self) -> None:
            project = pcbnew.GetBoard().GetFileName() if pcbnew.GetBoard() else "untitled.kicad_pcb"
            sidecar = Path(project).with_suffix(".layout.hnf.json")
            if not sidecar.is_file():
                raise FileNotFoundError(f"missing sidecar: {sidecar}")
            doc = json.loads(sidecar.read_text(encoding="utf-8"))
            import_layout_from_hnf(project, doc)
            pcbnew.Refresh()

    HbBridgeExportLayout().register()
    HbBridgeImportLayout().register()


def register_plugins() -> None:
    """Explicit registration hook for packaging tests."""
    if pcbnew is None:
        return
