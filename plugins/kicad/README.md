# KiCad — HB Bridge plugin (v8)

**Phase 0 target:** M1 (save/load v0.1)  
**Status:** In progress — `crates/hnf-kicad` + `plugin_stub.py`

## v8 approach (not a fork)

HB Bridge ships as a **KiCad action plugin / Python module** inside upstream KiCad. We do not maintain a long-lived KiCad fork.

## Install path (KiCad 8)

Copy or symlink this directory’s `plugin_stub.py` (and future `hb_bridge/` package files) into the KiCad **8.0** user plugins folder:

| OS | Path |
|----|------|
| Linux | `~/.local/share/kicad/8.0/plugins/` |
| macOS | `~/Library/Application Support/kicad/8.0/plugins/` |
| Windows | `%APPDATA%\kicad\8.0\plugins\` |

After install, restart KiCad. Pcbnew plugins appear under **Tools → External Plugins** (category **HB Bridge**). Schematic plugins will register the same tree once eeschema `ActionPlugin` hooks land (see stub TODOs).

**Pin:** KiCad 8.x aligned with [`hcp-oss/kicad`](https://github.com/hcp-oss/kicad) branch `hcp/integration` (see `crates/hnf-kicad/README.md`).

## Plugin entrypoints (M1)

`plugin_stub.py` documents the HNF save/load contract:

| Entrypoint | Domain | KiCad surface |
|------------|--------|---------------|
| `export_schematic_to_hnf` / `import_schematic_from_hnf` | `schematic` | Eeschema (TODO: ActionPlugin) |
| `export_layout_to_hnf` / `import_layout_from_hnf` | `layout` | Pcbnew `ActionPlugin` stubs |

Full implementation follows in M1+ PRs; Rust mutation mapping remains in `crates/hnf-kicad`.

## Roadmap

| Milestone | Deliverable |
|-----------|-------------|
| M1 | Export/import HNF `schematic` + `layout` domains; roundtrip CI |
| M2 | Optional Bridge panel (file → HOS commit) |
| M2 | DRC/ERC workflow actions wired via `hbp-cloud` |

## Roundtrip gate (release blocker)

v8 policy: **release blocked** when KiCad roundtrip fails for the pinned version.

| Gate | Location |
|------|----------|
| M1+ corpus tests | `roundtrip/tests/kicad_*` (see `roundtrip/tests/README.md`) |
| Current regression runner | `scripts/regression/run_suite.py roundtrip` with `fixtures/roundtrip_corpus/` |

Until `roundtrip/tests/` is populated, CI and local verify use `cargo test` in `hb-bridge` plus the regression suite above. Failures in `roundtrip/tests/kicad_*` or the regression KiCad cases block release per root [`plugins/README.md`](../README.md).

## Upstream

Target upstream contribution: HNF export hook in KiCad ecosystem (Phase 1 Week 44).
