# Phase 0 bridge tools matrix

Roundtrip CI **blocks release** when a plugin version fails (v8 policy).

| Tool | Phase 0 target | Status | Crate / path | Notes |
|------|----------------|--------|--------------|-------|
| KiCad | M1 | done | `crates/hnf-kicad` | save/load v0.1 — see `kicad/README.md` |
| FreeCAD | M1 | done | `crates/hnf-freecad` | save/load v0.1 — see `freecad/README.md` |
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

Phase 1 built-environment tools: [`phase1/README.md`](phase1/README.md).
