# hnf-freecad

FreeCAD mechanical HNF mutations → scene-graph nodes/edges.

**Version pin:** `hnf-adapter-sdk = 0.1.0` (path or crates.io after publish).

**Upstream binary:** FreeCAD 1.x `freecadcmd` from [`hcp-oss/freecad`](https://github.com/hcp-oss/freecad) (not vendored in monorepo).

## Sidecar usage

`rust/crates/freecad-sidecar` uses `map_mutation_to_deltas` in-process and `host_trace_event` when `SubprocessFreecadEngineBridge` probes `freecadcmd --version` on `hcp/project/open`.
