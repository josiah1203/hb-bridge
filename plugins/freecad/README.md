# FreeCAD — HB Bridge plugin (v8)

**Phase 0 target:** M1 (save/load v0.1)  
**Status:** In progress — `crates/hnf-freecad`

## v8 approach (not a fork)

HB Bridge is a **FreeCAD workbench / macro** installed into upstream FreeCAD. No maintained FreeCAD fork.

## Roadmap

| Milestone | Deliverable |
|-----------|-------------|
| M1 | Export/import HNF `mechanical` domain; roundtrip CI |
| M2 | Optional Bridge panel (commit to HOS) |
| M3 | Cross-domain warnings with schematic/BOM (via cloud events) |

## Roundtrip policy

Release blocked when `roundtrip/tests/freecad_*` fails for the pinned FreeCAD version.

## Phase 1

FreeCAD BIM workbench path tracked in `plugins/phase1/README.md` (`bim` domain).
