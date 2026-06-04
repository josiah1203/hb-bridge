# hb-bridge — in-tool plugins + roundtrip CI

Apache 2.0 workspace for **HummingBird Bridge** plugins (v8: plugins inside upstream tools, not long-lived forks).

| Crate / path | Role |
|--------------|------|
| `crates/hnf-kicad` | KiCad mutation → scene-graph mapping |
| `crates/hnf-freecad` | FreeCAD mechanical mapping |
| `crates/protocol` | JSON-RPC types (headless CI harness) |
| `crates/roundtrip-harness` | M1 corpus roundtrip gate (no CAD install) |
| `rust/crates/*-sidecar` | Stdio sidecars (stub bindings; optional host OSS) |

- **Plugins matrix:** `plugins/README.md`
- **Corpora:** `corpora/kicad/`, `corpora/freecad/`
- **Regression:** `scripts/regression/` (grafted from HCP — see `REUSE_FROM_HCP.md`)
- **Verify:** `scripts/headless-roundtrip.sh` or `make hb-verify-bridge` from `hb-platform` (prefers `hb-v8-bridge` worktree when present)

See `REUSE_FROM_HCP.md` for graft provenance (`phase-0.5-beta-rc1`).
