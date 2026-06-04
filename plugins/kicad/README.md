# KiCad — HB Bridge plugin (v8)

**Phase 0 target:** M1 (save/load v0.1)  
**Status:** In progress — `crates/hnf-kicad`

## v8 approach (not a fork)

HB Bridge ships as a **KiCad action plugin / Python module** inside upstream KiCad. We do not maintain a long-lived KiCad fork.

## Roadmap

| Milestone | Deliverable |
|-----------|-------------|
| M1 | Export/import HNF `schematic` + `layout` domains; roundtrip CI |
| M2 | Optional Bridge panel (file → HOS commit) |
| M2 | DRC/ERC workflow actions wired via `hbp-cloud` |

## Roundtrip policy

Release blocked when `roundtrip/tests/kicad_*` fails for the pinned KiCad version (see root `plugins/README.md`).

## Upstream

Target upstream contribution: HNF export hook in KiCad ecosystem (Phase 1 Week 44).
