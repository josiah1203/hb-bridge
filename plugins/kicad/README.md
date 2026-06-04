# KiCad — HB Bridge plugin (v8)

**Phase 0 target:** M1 (save/load v0.1)  
**Status:** harness-only — `crates/hnf-kicad` + `plugin_stub.py`; headless roundtrip CI green; in-tool ActionPlugin WIP

## v8 approach (not a fork)

HB Bridge ships as a **KiCad action plugin / Python module** contributed upstream to KiCad. We do not maintain a long-lived KiCad fork.

| Track | Target | Status |
|-------|--------|--------|
| Rust adapter | `crates/hnf-kicad` mutation → scene-graph | landed |
| Headless CI | `rust/crates/kicad-sidecar` (stub binding) | landed — see [`corpora/kicad/`](../corpora/kicad/) |
| In-tool plugin | `plugin_stub.py` → upstream PR | in progress |
| Bridge panel | optional file → HOS commit | M2 |

## Install path (KiCad 8)

Copy or symlink this directory’s `plugin_stub.py` (and future `hb_bridge/` package files) into the KiCad **8.0** user plugins folder:

| OS | Path |
|----|------|
| Linux | `~/.local/share/kicad/8.0/plugins/` |
| macOS | `~/Library/Application Support/kicad/8.0/plugins/` |
| Windows | `%APPDATA%\kicad\8.0\plugins\` |

After install, restart KiCad. Pcbnew plugins appear under **Tools → External Plugins** (category **HB Bridge**). Schematic plugins will register the same tree once eeschema `ActionPlugin` hooks land (see stub TODOs).

**Pin:** KiCad 8.x — target upstream KiCad master / 8.0 release branch (not an HCP fork).

## Plugin entrypoints (M1)

`plugin_stub.py` documents the HNF save/load contract:

| Entrypoint | Domain | KiCad surface |
|------------|--------|---------------|
| `export_schematic_to_hnf` / `import_schematic_from_hnf` | `schematic` | Eeschema (TODO: ActionPlugin) |
| `export_layout_to_hnf` / `import_layout_from_hnf` | `layout` | Pcbnew `ActionPlugin` stubs |

Full implementation follows in upstream PRs; Rust mutation mapping remains in `crates/hnf-kicad`.

## Upstream PR roadmap

| PR | Scope | Milestone |
|----|-------|-----------|
| 1 | Document HNF export/import hooks + stub ActionPlugin registration | M1 (this repo) |
| 2 | Pcbnew save/load HNF `layout` domain | M1 |
| 3 | Eeschema save/load HNF `schematic` domain | M2 |
| 4 | Optional Bridge panel (commit to HOS via `hb` CLI) | M2 |

Contributions target **KiCad upstream** (`kicad/kicad`), not a HummingBird fork.

## Roundtrip gate (release blocker)

v8 policy: **release blocked** when KiCad roundtrip fails for the pinned version.

| Gate | Location |
|------|----------|
| Corpus | [`corpora/kicad/minimal_layout.json`](../corpora/kicad/minimal_layout.json) |
| Headless harness | `python3 scripts/regression/run_suite.py roundtrip --corpus corpora/roundtrip/manifest.json` |
| Rust sidecar | `rust/crates/kicad-sidecar` (CI only) |

Optional host roundtrip: set `HBP_USE_HOST_OSS=1` and install KiCad 8 + `kicad-cli` on PATH.

Grafted from HCP `phase-0.5-beta-rc1` — see [`REUSE_FROM_HCP.md`](../REUSE_FROM_HCP.md).
