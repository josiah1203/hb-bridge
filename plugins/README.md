# Phase 0 bridge tools matrix

Roundtrip CI **blocks release** when a plugin version fails (v8 policy).

| Tool | Phase 0 target | Status | Crate / path | Notes |
|------|----------------|--------|--------------|-------|
| KiCad | M1 | in-progress | `crates/hnf-kicad`, `plugins/kicad/` | HNF v0.1 save/load + ActionPlugin; headless roundtrip CI; host gate `HB_BRIDGE_HOST_KICAD=1` |
| FreeCAD | M1 | in-progress | `crates/hnf-freecad`, `plugins/freecad/` | HNF v0.1 mechanical save/load + macro; headless roundtrip CI; host gate `HB_BRIDGE_HOST_FREECAD=1` |
| KLayout | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/klayout/` | ic_layout corpus; host binary smoke `#[ignore]` + `HB_BRIDGE_HOST_KLAYOUT` |
| ngspice | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/ngspice/` | simulation corpus + `tests/fixtures/ngspice/`; host smoke gated |
| Yosys | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/yosys/` | RTL corpus + `tests/fixtures/yosys/`; host smoke gated |
| Verilator | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/verilator/` | TB corpus; upstream plugin pending |
| Magic | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/magic/` | layout corpus; upstream plugin pending |
| OpenROAD | M2 | harness-only | `crates/hnf-phase0-tools`, `plugins/openroad/` | PnR corpus; upstream plugin pending |
| Xschem | M2 | pending | TBD | schematic |
| OpenEMS | M2 | pending | TBD | |
| Elmer | M2 | pending | TBD | |
| Qucs-S | M2 | pending | TBD | |
| PlatformIO | M2 | pending | TBD | firmware |

**Status key:** `harness-only` = headless roundtrip + Rust adapter land; in-tool upstream plugin not yet merged. `in-progress` = active upstream PR work. `done` = save/load + roundtrip green with host tool. `pending` = no corpus or adapter yet.

Phase 1 built-environment tools: [`phase1/README.md`](phase1/README.md).

## M1/M2 roundtrip

Corpus: [`corpora/`](../corpora/) (KiCad, FreeCAD, KLayout, ngspice, Yosys, Verilator, Magic, OpenROAD).  
Fixtures: [`tests/fixtures/`](../tests/fixtures/).  
CI harness: [`crates/roundtrip-harness`](../crates/roundtrip-harness/) + [`rust/crates/*-sidecar`](../rust/crates/) (headless only — not shipped to end users).

Optional host installs: `cargo test -p roundtrip-harness -- --ignored` with `HB_BRIDGE_HOST_<TOOL>=1` (e.g. `HB_BRIDGE_HOST_KICAD=1`, `HB_BRIDGE_HOST_FREECAD=1`, `HB_BRIDGE_HOST_YOSYS=1`).
