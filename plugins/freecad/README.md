# FreeCAD — HB Bridge plugin (v8)

**Phase 0 target:** M1 (save/load v0.1)  
**Status:** harness-only — `crates/hnf-freecad`; headless roundtrip CI green; workbench plugin WIP

## v8 approach (not a fork)

HB Bridge is a **FreeCAD workbench / macro** contributed upstream to FreeCAD. No maintained FreeCAD fork.

| Track | Target | Status |
|-------|--------|--------|
| Rust adapter | `crates/hnf-freecad` mechanical mutations → scene-graph | landed |
| Headless CI | `rust/crates/freecad-sidecar` (stub binding) | landed — see [`corpora/freecad/`](../corpora/freecad/) |
| In-tool workbench | FreeCAD macro / workbench → upstream PR | in progress |
| Bridge panel | optional commit to HOS | M2 |

## Upstream PR roadmap

| PR | Scope | Milestone |
|----|-------|-----------|
| 1 | HNF `mechanical` domain export/import macro stub | M1 |
| 2 | Workbench integration + solid/constraint mapping | M1 |
| 3 | Bridge panel (HOS commit via `hb` CLI) | M2 |
| 4 | Cross-domain warnings with schematic/BOM (cloud events) | M3 |

Contributions target **FreeCAD upstream** (`FreeCAD/FreeCAD`), not a HummingBird fork.

## Roundtrip gate (release blocker)

v8 policy: **release blocked** when FreeCAD roundtrip fails for the pinned version.

| Gate | Location |
|------|----------|
| Corpus | [`corpora/freecad/minimal_solid.json`](../corpora/freecad/minimal_solid.json) |
| Headless harness | `python3 scripts/regression/run_suite.py roundtrip --corpus corpora/roundtrip/manifest.json` |
| Rust sidecar | `rust/crates/freecad-sidecar` (CI only) |

Optional host roundtrip: set `HBP_USE_HOST_OSS=1`, provide a workspace dir, and install `freecadcmd` on PATH.

## Phase 1

FreeCAD BIM workbench path tracked in `plugins/phase1/README.md` (`bim` domain).

Grafted from HCP `phase-0.5-beta-rc1` — see [`REUSE_FROM_HCP.md`](../REUSE_FROM_HCP.md).
