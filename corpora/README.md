# Roundtrip corpora (M1)

Headless roundtrip fixtures for KiCad and FreeCAD HB Bridge adapters. These drive the **CI harness only** (`rust/crates/*-sidecar` + `scripts/regression/run_suite.py`); they do not require installed KiCad/FreeCAD unless `HBP_USE_HOST_OSS=1`.

| Tool | Fixture | Harness |
|------|---------|---------|
| KiCad | [`kicad/minimal_layout.json`](kicad/minimal_layout.json) | `kicad-sidecar` stub binding |
| FreeCAD | [`freecad/minimal_solid.json`](freecad/minimal_solid.json) | `freecad-sidecar` stub binding |

Combined manifest: [`roundtrip/manifest.json`](roundtrip/manifest.json).

```bash
# Local headless roundtrip (builds sidecars, no CAD install required)
python3 scripts/regression/run_suite.py roundtrip --corpus corpora/roundtrip/manifest.json
```

Grafted from HCP `phase-0.5-beta-rc1` — see [`REUSE_FROM_HCP.md`](../REUSE_FROM_HCP.md).
