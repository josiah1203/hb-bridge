# Copyright 2026 HummingBird Labs
# SPDX-License-Identifier: Apache-2.0
"""
FreeCAD HB Bridge status panel (M2 stub).

Shows connection state, last HNF export path, and link to HBW. Workbench dock
widget ships with upstream PR 3 (see docs/upstream/freecad.md).
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from typing import Any

PANEL_VERSION = "0.1.0"
DEFAULT_HBW_URL = os.environ.get("HBW_URL", "http://127.0.0.1:5173/")


@dataclass
class BridgePanelState:
    connected: bool = False
    last_hnf_export: str | None = None
    hbw_url: str = field(default_factory=lambda: DEFAULT_HBW_URL)
    adapter: str = "hnf-freecad"

    def to_dict(self) -> dict[str, Any]:
        return {
            "version": PANEL_VERSION,
            "connected": self.connected,
            "last_hnf_export": self.last_hnf_export,
            "hbw_url": self.hbw_url,
            "adapter": self.adapter,
        }


_state = BridgePanelState()


def set_connected(connected: bool) -> None:
    _state.connected = connected


def record_hnf_export(path: str) -> None:
    _state.last_hnf_export = path
    _state.connected = True


def panel_status() -> dict[str, Any]:
    return _state.to_dict()


def hbw_deep_link(commit_id: str | None = None) -> str:
    base = _state.hbw_url.rstrip("/")
    if commit_id:
        return f"{base}/history?commit={commit_id}"
    return f"{base}/repos"
