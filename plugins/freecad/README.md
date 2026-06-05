# FreeCAD — HB Bridge plugin (v8)

**Phase 0 target:** M1 (save/load v0.1)  
**Status:** in-progress — `crates/hnf-freecad` + `hb_bridge.py` + `hb-bridge-hnf` CLI; headless roundtrip CI green; workbench macro v0.1 landed

## v8 approach (not a fork)

HB Bridge is a **FreeCAD workbench / macro** contributed upstream to FreeCAD. No maintained FreeCAD fork.

| Track | Target | Status |
|-------|--------|--------|
| Rust adapter | `crates/hnf-freecad` HNF v0.1 save/load + scene-graph | landed |
| CLI bridge | `crates/hb-bridge-hnf` | landed |
| Headless CI | `crates/roundtrip-harness` + `rust/crates/freecad-sidecar` | landed |
| In-tool workbench | `hb_bridge.py` + `hnf_client.py` | v0.1 macro API landed |
| Bridge panel | `panel/bridge_panel.py` stub (HBW link + last export) | M2 stub |

## Install path (FreeCAD 1.x)

1. Build the HNF CLI:

```bash
cd /path/to/hb-bridge
cargo build -p hb-bridge-hnf
export HB_BRIDGE_HNF_CLI="$PWD/target/debug/hb-bridge-hnf"   # optional override
```

2. Add this directory to FreeCAD's macro path, or copy `hb_bridge.py` and `hnf_client.py` into your user macro folder:

| OS | Typical macro path |
|----|-------------------|
| Linux | `~/.local/share/FreeCAD/Macro/` |
| macOS | `~/Library/Application Support/FreeCAD/Macro/` |
| Windows | `%APPDATA%\FreeCAD\Macro\` |

3. In FreeCAD, run **Macro → Macros…** and execute `hb_bridge.py`, or bind:

```python
from hb_bridge import export_mechanical_to_hnf, write_hnf_sidecar
doc = export_mechanical_to_hnf("/path/to/project.FCStd")
write_hnf_sidecar("/path/to/project.FCStd", doc)
```

When `FreeCAD.ActiveDocument` is open, solids are read from document objects; otherwise a placeholder corpus mutation is used (headless CI).

**Pin:** FreeCAD 1.x — target upstream `FreeCAD/FreeCAD` (not a HummingBird fork).

## Plugin entrypoints (M1)

| Entrypoint | Domain |
|------------|--------|
| `export_mechanical_to_hnf` / `import_mechanical_from_hnf` | `mechanical` |
| `write_hnf_sidecar` | writes `<project>.mechanical.hnf.json` |

## Roundtrip gate (release blocker)

| Gate | Command |
|------|---------|
| Headless Rust | `cargo test -p roundtrip-harness` |
| Plugin Python | `python3 -m pytest scripts/regression/tests/test_plugin_hnf_roundtrip.py -q` |
| Corpus | [`corpora/freecad/`](../corpora/freecad/) |

Optional **host** roundtrip (requires `freecadcmd` on PATH):

```bash
HB_BRIDGE_HOST_FREECAD=1 cargo test -p roundtrip-harness freecad_host_hnf_roundtrip -- --ignored
```

## Upstream PR roadmap

| PR | Scope | Milestone |
|----|-------|-----------|
| 1 | HNF `mechanical` domain export/import macro | M1 (this repo) |
| 2 | Workbench UI + constraint mapping | M1 |
| 3 | Bridge panel (HOS commit via `hb` CLI) | M2 |

Contributions target **FreeCAD upstream** (`FreeCAD/FreeCAD`), not a HummingBird fork.

## Phase 1

FreeCAD BIM workbench path tracked in `plugins/phase1/README.md` (`bim` domain).

Grafted from HCP `phase-0.5-beta-rc1` — see [`REUSE_FROM_HCP.md`](../REUSE_FROM_HCP.md).
