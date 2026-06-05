# FreeCAD upstream contribution — draft PR (M2)

**Target repo:** https://github.com/FreeCAD/FreeCAD  
**Pin:** FreeCAD 1.x (`main`)  
**HB Bridge source:** `hb-bridge/plugins/freecad/` + `crates/hnf-freecad`

## PR title (draft)

```
Add HB Bridge workbench stub for HNF v0.1 mechanical export/import
```

## Summary

Contribute HummingBird **HNF v0.1** mechanical domain save/load as an optional FreeCAD workbench/macro. No HummingBird fork of FreeCAD. Rust adapter and headless CI remain in `hb-bridge`; upstream PR adds workbench registration and user-facing commands after maintainer review.

## Scope (PR 1 — landed in hb-bridge, upstream submission M2)

- [x] `hb_bridge.py` — macro entrypoints for HNF export/import
- [x] `hnf_client.py` — invokes `hb-bridge-hnf` CLI
- [ ] Assembly domain hooks (PR 2)
- [ ] Bridge panel task panel using `plugins/freecad/panel/` (PR 3)

## Upstream checklist

| Item | Notes |
|------|-------|
| License | LGPL-2.1+ (FreeCAD) vs Apache-2.0 (hb-bridge) — dual-license note in PR body |
| Packaging | Ship as `Mod/HBBridge` or external addon per maintainer preference |
| Dependencies | `freecadcmd` + built `hb-bridge-hnf` on PATH for CLI bridge |
| Tests | FreeCAD CI not blocked; hb-bridge `cargo test -p roundtrip-harness` is release gate |
| Docs | Wiki page link from workbench "Help" |

## Test plan (reviewer)

1. Build `hb-bridge-hnf` from `hb-bridge`.
2. Add workbench per `plugins/freecad/README.md`.
3. Open sample Part → export HNF → verify JSON sidecar.
4. Import sidecar → geometry restored (v0.1 bbox/body stubs).
5. Optional: `HB_BRIDGE_HOST_FREECAD=1 cargo test -p roundtrip-harness freecad_host_hnf_roundtrip -- --ignored`

## References

- HNF mechanical domain: `hnf/schemas/domains/mechanical.json`
- Matrix status: `hb-bridge/plugins/README.md`
- Panel stub: `hb-bridge/plugins/freecad/panel/bridge_panel.py`
