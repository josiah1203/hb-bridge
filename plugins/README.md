# Phase 0 bridge tools matrix

Roundtrip CI **blocks release** when a plugin version fails (v8 policy).

| Tool | Phase 0 target | Status | Crate / path | Notes |
|------|----------------|--------|--------------|-------|
| KiCad | M1 | harness-only | `crates/hnf-kicad`, `plugins/kicad/` | Rust mapping + plugin stub; headless roundtrip CI; in-tool save/load WIP |
| FreeCAD | M1 | harness-only | `crates/hnf-freecad`, `plugins/freecad/` | Rust mapping; headless roundtrip CI; workbench plugin WIP |
| KLayout | M2 | pending | TBD | ic_layout |
| OpenROAD | M2 | pending | TBD | |
| Magic | M2 | pending | TBD | |
| Xschem | M2 | pending | TBD | schematic |
| ngspice | M2 | pending | TBD | simulation |
| OpenEMS | M2 | pending | TBD | |
| Elmer | M2 | pending | TBD | |
| Qucs-S | M2 | pending | TBD | |
| Verilator | M2 | pending | TBD | |
| Yosys | M2 | pending | TBD | |
| PlatformIO | M2 | pending | TBD | firmware |

**Status key:** `harness-only` = headless roundtrip + Rust adapter land; in-tool upstream plugin not yet merged. `in-progress` = active upstream PR work. `done` = save/load + roundtrip green with host tool.

Phase 1 built-environment tools: [`phase1/README.md`](phase1/README.md).

## M1 roundtrip

Corpus: [`corpora/`](../corpora/). CI harness: [`rust/crates/*-sidecar`](../rust/crates/) (headless only — not shipped to end users).
