# Upstream PR documentation (Phase 0 Week 16)

Draft submission packages for in-tool plugins contributed to host OSS projects (v8 policy: no long-lived forks).

| Host tool | Draft PR doc | Milestone |
|-----------|----------------|-----------|
| KiCad | [`kicad.md`](kicad.md) | M2 submission |
| FreeCAD | [`freecad.md`](freecad.md) | M2 submission |
| KLayout, ngspice, Yosys, Verilator, Magic, OpenROAD | harness-only; upstream PRs planned post-M2 | M3+ |
| Xschem, OpenEMS, Elmer, Qucs-S, PlatformIO | harness-only; upstream PRs planned post-M2 | M3+ |

Release gate for bridge matrix: `make hb-verify-bridge` from `hb-platform` (runs `cargo test` in `hb-bridge`).
