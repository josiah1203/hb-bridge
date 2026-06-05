#!/usr/bin/env bash
# M1 headless roundtrip verify (no KiCad/FreeCAD install required).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
echo "==> roundtrip-harness (corpus)"
cargo test -p roundtrip-harness -q
echo "==> adapter crates"
cargo test -p hnf-kicad -p hnf-freecad -p hnf-phase0-tools -q
echo "==> sidecar libraries"
(cd rust && cargo test -p kicad-sidecar -p freecad-sidecar -q)
echo "ok: headless roundtrip"
