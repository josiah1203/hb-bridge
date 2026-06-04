# hb-bridge — in-tool plugins + roundtrip CI

Apache 2.0 workspace for **HummingBird Bridge** plugins (v8: plugins inside upstream tools, not long-lived forks).

| Crate | Role |
|-------|------|
| `crates/hnf-kicad` | KiCad plugin / mapping |
| `crates/hnf-freecad` | FreeCAD plugin / mapping |
| `crates/protocol` | Headless JSON-RPC harness for CI only |

- **Plugins matrix:** `plugins/README.md`
- **Regression:** `scripts/regression/` (from HCP)
- **Roundtrip:** `roundtrip/tests/` (placeholder)

See `REUSE_FROM_HCP.md` for graft provenance.
