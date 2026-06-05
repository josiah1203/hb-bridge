# KiCad — HB Bridge plugin (v8)

**Phase 0 target:** M1 (save/load v0.1)  
**Status:** in-progress — `crates/hnf-kicad` + `plugin_stub.py` + `hb-bridge-hnf` CLI; headless roundtrip CI green; Pcbnew ActionPlugin v0.1 landed

## v8 approach (not a fork)

HB Bridge ships as a **KiCad action plugin / Python module** contributed upstream to KiCad. We do not maintain a long-lived KiCad fork.

| Track | Target | Status |
|-------|--------|--------|
| Rust adapter | `crates/hnf-kicad` HNF v0.1 save/load + scene-graph | landed |
| CLI bridge | `crates/hb-bridge-hnf` (`hb-bridge-hnf`) | landed |
| Headless CI | `crates/roundtrip-harness` + `rust/crates/kicad-sidecar` | landed |
| In-tool plugin | `plugin_stub.py` + `hnf_client.py` | v0.1 landed (layout); schematic API headless |
| Bridge panel | `panel/bridge_panel.py` stub (HBW link + last export) | M2 stub |

## Install path (KiCad 8)

1. Build the HNF CLI (once per machine or CI artifact):

```bash
cd /path/to/hb-bridge
cargo build -p hb-bridge-hnf
export HB_BRIDGE_HNF_CLI="$PWD/target/debug/hb-bridge-hnf"   # optional override
```

2. Copy or symlink this directory into the KiCad **8.0** user plugins folder:

| OS | Path |
|----|------|
| Linux | `~/.local/share/kicad/8.0/plugins/hb_bridge_kicad/` |
| macOS | `~/Library/Application Support/kicad/8.0/plugins/hb_bridge_kicad/` |
| Windows | `%APPDATA%\kicad\8.0\plugins\hb_bridge_kicad\` |

Required files: `plugin_stub.py`, `hnf_client.py`.

3. Restart KiCad. Pcbnew plugins appear under **Tools → External Plugins** (category **HB Bridge**):
   - **HB Bridge: Export layout (HNF)** — writes `<board>.layout.hnf.json`
   - **HB Bridge: Import layout (HNF)** — reads the sidecar and applies footprints

**Pin:** KiCad 8.x — target upstream KiCad master / 8.0 release branch.

## Plugin entrypoints (M1)

| Entrypoint | Domain | KiCad surface |
|------------|--------|---------------|
| `export_schematic_to_hnf` / `import_schematic_from_hnf` | `schematic` | Headless + batch (Eeschema ActionPlugin M2) |
| `export_layout_to_hnf` / `import_layout_from_hnf` | `layout` | Pcbnew `ActionPlugin` |

Rust HNF mapping lives in `crates/hnf-kicad`; Python calls `hb-bridge-hnf` via `hnf_client.py`.

## Roundtrip gate (release blocker)

| Gate | Command |
|------|---------|
| Headless Rust | `cargo test -p roundtrip-harness` |
| Plugin Python | `python3 -m pytest scripts/regression/tests/test_plugin_hnf_roundtrip.py -q` |
| Corpus | [`corpora/kicad/`](../corpora/kicad/) |

Optional **host** roundtrip (requires KiCad 8 + `kicad-cli` on PATH):

```bash
HB_BRIDGE_HOST_KICAD=1 cargo test -p roundtrip-harness kicad_host_hnf_roundtrip -- --ignored
```

## Upstream PR roadmap

| PR | Scope | Milestone |
|----|-------|-----------|
| 1 | HNF export/import hooks + ActionPlugin registration | M1 (this repo) |
| 2 | Eeschema save/load HNF `schematic` domain | M2 |
| 3 | Bridge panel (commit to HOS via `hb` CLI) | M2 |

Contributions target **KiCad upstream** (`kicad/kicad`), not a HummingBird fork.

Grafted from HCP `phase-0.5-beta-rc1` — see [`REUSE_FROM_HCP.md`](../REUSE_FROM_HCP.md).
