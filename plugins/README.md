# Phase 0 bridge tools matrix

Roundtrip CI **blocks release** when a plugin version fails (v8 policy).

| Tool | Phase 0 target | Status | Crate / path | Notes |
|------|----------------|--------|--------------|-------|
| KiCad | M1 | in-progress | `crates/hnf-kicad`, `plugins/kicad/` | HNF v0.1 save/load + ActionPlugin; panel stub `plugins/kicad/panel/`; upstream [`docs/upstream/kicad.md`](../docs/upstream/kicad.md) |
| FreeCAD | M1 | in-progress | `crates/hnf-freecad`, `plugins/freecad/` | HNF v0.1 mechanical save/load; panel stub `plugins/freecad/panel/`; upstream [`docs/upstream/freecad.md`](../docs/upstream/freecad.md) |
| KLayout | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/klayout/` | ic_layout corpus; host binary smoke `#[ignore]` + `HB_BRIDGE_HOST_KLAYOUT` |
| ngspice | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/ngspice/` | simulation corpus + `tests/fixtures/ngspice/` |
| Yosys | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/yosys/` | RTL corpus + `tests/fixtures/yosys/` |
| Verilator | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/verilator/` | TB corpus; upstream plugin planned |
| Magic | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/magic/` | layout corpus; upstream plugin planned |
| OpenROAD | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/openroad/` | PnR corpus; upstream plugin planned |
| Xschem | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/xschem/` | schematic corpus `corpora/xschem/` |
| OpenEMS | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/openems/` | FDTD corpus `corpora/openems/` |
| Elmer | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/elmer/` | FEM corpus `corpora/elmer/` |
| Qucs-S | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/qucs-s/` | circuit corpus `corpora/qucs-s/` |
| PlatformIO | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/platformio/` | firmware corpus `corpora/platformio/` |

**Status key:** `harness-only` = headless roundtrip + Rust adapter land; in-tool upstream plugin not yet merged. `in-progress` = active upstream PR work. `done` = save/load + roundtrip green with host tool.

Phase 1 built-environment tools: [`phase1/README.md`](phase1/README.md).

## M1/M2 roundtrip

Corpus: [`corpora/`](../corpora/) (all 13 Phase 0 tools).  
Fixtures: [`tests/fixtures/`](../tests/fixtures/).  
CI harness: [`crates/roundtrip-harness`](../crates/roundtrip-harness/) + [`rust/crates/*-sidecar`](../rust/crates/) (headless only — not shipped to end users).

Optional host installs: `cargo test -p roundtrip-harness -- --ignored` with `HB_BRIDGE_HOST_<TOOL>=1` (e.g. `HB_BRIDGE_HOST_KICAD=1`, `HB_BRIDGE_HOST_QUCS_S=1`).

Upstream draft PRs (KiCad, FreeCAD): [`docs/upstream/`](../docs/upstream/).
