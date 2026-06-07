# Copyright 2026 HummingBird Labs
"""Phase 1 harness-only roundtrip for Qgis (geospatial domain)."""

from __future__ import annotations

from typing import Any

PLUGIN_ID = "hb_bridge_qgis"
PLUGIN_VERSION = "0.1.0"
DOMAIN = "geospatial"
MUTATION_KIND = "geo/layer/upsert"


def _mutation(kind: str, payload: dict[str, Any]) -> dict[str, Any]:
    return {"kind": kind, "payload": payload}


def export_to_hnf(
    project_path: str,
    *,
    mutations: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    """Headless export stub — returns HNF v0.1 manifest + domain payload."""
    del project_path
    if mutations is None:
        mutations = [
            _mutation(
                MUTATION_KIND,
                {"commitId": "qgis-export", "id": "site", "name": "Qgis sample"},
            )
        ]
    return {
        "manifest": {
            "hnf_version": "0.1",
            "doc_id": "phase1-qgis-001",
            "disciplines": [DOMAIN],
        },
        "document_uri": "hbp://docs/qgis/sample",
        "objects": [
            {
                "id": "obj-qgis",
                "kind": f"{DOMAIN}.sample",
                "properties": {"domain": DOMAIN, "mutations": mutations},
            }
        ],
    }


def import_from_hnf(
    project_path: str,
    hnf_document: dict[str, Any],
    *,
    merge: bool = False,
) -> list[dict[str, Any]]:
    """Headless import stub — replays mutations from HNF document."""
    del project_path, merge
    for obj in hnf_document.get("objects", []):
        props = obj.get("properties", {})
        if props.get("domain") == DOMAIN:
            return list(props.get("mutations", []))
    return []
