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
KiCad 8 action-plugin stub for HNF save/load (HB Bridge M1).

v8 policy: upstream KiCad plugin, not a long-lived fork. This module documents
the entrypoints the full plugin will implement; roundtrip CI gates release.

Domains (HNF v0.1): schematic (.kicad_sch), layout (.kicad_pcb).
Rust mapping lives in crates/hnf-kicad (mutation → scene-graph).
"""

from __future__ import annotations

from typing import Any

# KiCad 8 loads action plugins from the user plugins directory (see README).
# Import only when running inside KiCad; keep importable for static checks.
try:
    import pcbnew  # type: ignore[import-untyped]
except ImportError:  # pragma: no cover - dev/CI without KiCad
    pcbnew = None  # type: ignore[assignment]

PLUGIN_ID = "hb_bridge_kicad"
PLUGIN_VERSION = "0.1.0-m1-stub"
HNF_DOMAIN_SCHEMATIC = "schematic"
HNF_DOMAIN_LAYOUT = "layout"


# ---------------------------------------------------------------------------
# HNF save/load entrypoints (M1 contract — implement in follow-up PRs)
# ---------------------------------------------------------------------------


def export_schematic_to_hnf(
    project_path: str,
    *,
    hnf_version: str = "0.1",
) -> dict[str, Any]:
    """
    KiCad 8 API hook: read active schematic and emit HNF schematic domain JSON.

    Expected callers: Eeschema action plugin Run() or batch via kicad-cli wrapper.
    """
    # TODO: open .kicad_sch via KiCad project API; serialize to HNF schematic schema
    raise NotImplementedError("M1 stub: export_schematic_to_hnf")


def import_schematic_from_hnf(
    project_path: str,
    hnf_document: dict[str, Any],
    *,
    merge: bool = False,
) -> None:
    """
    KiCad 8 API hook: apply HNF schematic domain into the active project.

    merge=False replaces project schematic; merge=True applies semantic patches.
    """
    # TODO: validate hnf_document; map symbols/nets to eeschema objects
    raise NotImplementedError("M1 stub: import_schematic_from_hnf")


def export_layout_to_hnf(
    project_path: str,
    *,
    hnf_version: str = "0.1",
) -> dict[str, Any]:
    """
    KiCad 8 API hook: read active board and emit HNF layout domain JSON.

    Expected callers: Pcbnew ActionPlugin Run() or pcbnew scripting shell.
    """
    # TODO: walk BOARD; emit footprints, tracks, zones per HNF layout schema
    raise NotImplementedError("M1 stub: export_layout_to_hnf")


def import_layout_from_hnf(
    project_path: str,
    hnf_document: dict[str, Any],
    *,
    merge: bool = False,
) -> None:
    """KiCad 8 API hook: apply HNF layout domain into the active board."""
    # TODO: validate hnf_document; apply footprints/tracks without breaking DRC state
    raise NotImplementedError("M1 stub: import_layout_from_hnf")


# ---------------------------------------------------------------------------
# KiCad 8 ActionPlugin registration (pcbnew)
# ---------------------------------------------------------------------------


if pcbnew is not None:

    class HbBridgeExportLayout(pcbnew.ActionPlugin):
        """Tools → External Plugins → HB Bridge: Export layout to HNF."""

        def defaults(self) -> None:
            self.name = "HB Bridge: Export layout (HNF)"
            self.category = "HB Bridge"
            self.description = "Export current board to HNF layout domain (M1 stub)"
            self.show_toolbar_button = False

        def Run(self) -> None:
            # TODO: resolve project path; call export_layout_to_hnf; write .hnf sidecar
            raise NotImplementedError("M1 stub: HbBridgeExportLayout.Run")

    class HbBridgeImportLayout(pcbnew.ActionPlugin):
        """Tools → External Plugins → HB Bridge: Import layout from HNF."""

        def defaults(self) -> None:
            self.name = "HB Bridge: Import layout (HNF)"
            self.category = "HB Bridge"
            self.description = "Import HNF layout domain into current board (M1 stub)"
            self.show_toolbar_button = False

        def Run(self) -> None:
            # TODO: file picker → import_layout_from_hnf
            raise NotImplementedError("M1 stub: HbBridgeImportLayout.Run")

    HbBridgeExportLayout().register()
    HbBridgeImportLayout().register()


# ---------------------------------------------------------------------------
# Eeschema action plugin entrypoints (schematic) — same plugin package
# ---------------------------------------------------------------------------
# KiCad 8 registers schematic plugins separately under plugins/; mirror pcbnew
# with eeschema.ActionPlugin subclasses when the schematic API surface is wired.
#
# TODO: class HbBridgeExportSchematic(eeschema.ActionPlugin): ...
# TODO: class HbBridgeImportSchematic(eeschema.ActionPlugin): ...
# TODO: register schematic plugins on load (import eeschema guard like pcbnew)


def register_plugins() -> None:
    """
    Explicit registration hook for packaging tests.

    pcbnew plugins self-register on import when pcbnew is available.
    """
    # TODO: register eeschema ActionPlugin pair when schematic hooks land
