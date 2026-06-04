# Roundtrip tests (M1 headless harness)

Headless roundtrip gates for KiCad and FreeCAD adapters. **No KiCad/FreeCAD install required** for CI — sidecars use in-process stub bindings unless `HBP_USE_HOST_OSS=1`.

## Layout

| Path | Purpose |
|------|---------|
| `corpora/kicad/` | KiCad corpus fixture |
| `corpora/freecad/` | FreeCAD corpus fixture |
| `corpora/roundtrip/manifest.json` | Combined roundtrip manifest |
| `rust/crates/kicad-sidecar` | Headless KiCad JSON-RPC harness (CI only) |
| `rust/crates/freecad-sidecar` | Headless FreeCAD JSON-RPC harness (CI only) |
| `scripts/regression/run_suite.py roundtrip` | Python driver |

## Run locally

```bash
# Adapter unit tests
cargo test --all

# Sidecar harness + roundtrip
cd rust && cargo test --all && cargo build -p kicad-sidecar -p freecad-sidecar
python3 scripts/regression/run_suite.py roundtrip --corpus corpora/roundtrip/manifest.json

# Pytest (unit + headless roundtrip)
python3 -m pytest scripts/regression/tests/ -q
```

## Release gate

v8 policy: release **blocked** when KiCad or FreeCAD roundtrip fails for the pinned tool version. CI runs headless stubs on every push; optional host-OSS jobs may be added when KiCad 8 / FreeCAD are available in the runner image.
