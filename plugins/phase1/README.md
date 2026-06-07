# Phase 1 built-environment bridge matrix (M5-B)

HNF domains: `bim`, `geospatial`, `structural`, `energy_building`. All tools have harness-only roundtrip via `crates/hnf-phase1-tools`.

| Tool | Domain | Week | Status | Notes |
|------|--------|------|--------|-------|
| BlenderBIM | bim | 40 | **harness-only** | IFC import/export — `corpora/blenderbim/` |
| FreeCAD BIM | bim | 40 | **harness-only** | extends M1 FreeCAD plugin |
| OpenStudio / EnergyPlus | energy_building | 40 | **harness-only** | energy model roundtrip |
| QGIS | geospatial | 40 | **harness-only** | site / context layers |
| GRASS GIS | geospatial | 40 | **harness-only** | terrain / hydrology |
| OpenSCAD | mechanical | 40 | **harness-only** | parametric MCAD |
| OpenSees | structural | 40 | **harness-only** | FEA model exchange |
| Code_Aster | structural | 40 | **harness-only** | |
| CalculiX | structural | 40 | **harness-only** | |
| LibreCAD | layout | 40 | **harness-only** | 2D drafting |

**Verify:** `cargo test -p roundtrip-harness` (10 Phase 1 corpus tests)

**Week 44:** upstream PRs tracked in [`docs/upstream/`](../docs/upstream/); Foundation governance in `hb-platform/docs/FOUNDATION_GOVERNANCE.md`.

## Coverage matrix (ADR-0002)

| Domain | Tier A (HNF JSON) | Tier B (geometry blob) | Native parametric |
|--------|-------------------|------------------------|-------------------|
| bim | elements, storeys | IFC via `content_hash` | Revit/ArchiCAD — out of scope |
| geospatial | layers, CRS | GeoPackage/raster refs | — |
| structural | members, loads | mesh refs (future) | — |
| energy_building | zones, systems | IDF/OSM blobs (future) | — |
| mechanical (OpenSCAD) | solids, constraints | STEP via `geometry_blobs[]` | OpenSCAD native — Tier C |
