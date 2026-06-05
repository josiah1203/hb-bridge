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

"""Invoke `hb-bridge-hnf` CLI for HNF v0.1 save/load (hnf-kicad crate)."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from typing import Any


def _hnf_cli_path() -> Path:
    override = os.environ.get("HB_BRIDGE_HNF_CLI")
    if override:
        return Path(override)
    repo_root = Path(__file__).resolve().parents[2]
    for candidate in (
        repo_root / "target" / "debug" / "hb-bridge-hnf",
        repo_root / "target" / "release" / "hb-bridge-hnf",
    ):
        if candidate.is_file():
            return candidate
    return candidate


def _run(tool: str, command: str, payload: dict[str, Any]) -> Any:
    cli = _hnf_cli_path()
    if not cli.is_file():
        raise FileNotFoundError(
            f"hb-bridge-hnf not found at {cli}; run `cargo build -p hb-bridge-hnf` "
            "or set HB_BRIDGE_HNF_CLI"
        )
    proc = subprocess.run(
        [str(cli), tool, command],
        input=json.dumps(payload),
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"hb-bridge-hnf {tool} {command} failed: {proc.stderr.strip() or proc.stdout}"
        )
    return json.loads(proc.stdout)


def kicad_export_layout(project_path: str, mutations: list[dict[str, Any]]) -> dict[str, Any]:
    return _run(
        "kicad",
        "export-layout",
        {"project_path": project_path, "mutations": mutations},
    )


def kicad_import_layout(hnf_document: dict[str, Any]) -> list[dict[str, Any]]:
    out = _run("kicad", "import-layout", {"hnf_document": hnf_document})
    return out["mutations"]


def kicad_export_schematic(project_path: str, mutations: list[dict[str, Any]]) -> dict[str, Any]:
    return _run(
        "kicad",
        "export-schematic",
        {"project_path": project_path, "mutations": mutations},
    )


def kicad_import_schematic(hnf_document: dict[str, Any]) -> list[dict[str, Any]]:
    out = _run("kicad", "import-schematic", {"hnf_document": hnf_document})
    return out["mutations"]
