# KiCad upstream contribution — draft PR (M2)

**Target repo:** https://github.com/kicad/kicad  
**Pin:** KiCad 8.x (`master` / `8.0` release branch)  
**HB Bridge source:** `hb-bridge/plugins/kicad/` + `crates/hnf-kicad`

## PR title (draft)

```
Add optional HNF v0.1 export/import hooks for Pcbnew (HB Bridge plugin)
```

## Summary

Contribute HummingBird **HNF v0.1** layout/schematic sidecar export as an optional KiCad ActionPlugin module. No fork; Apache-2.0 aligned with KiCad contribution policy. Headless mapping and CI live in the HummingBird `hb-bridge` repo; this PR adds in-tree registration only after review.

## Scope (PR 1 — landed in hb-bridge, upstream submission M2)

- [x] `plugin_stub.py` — Pcbnew export/import layout HNF JSON
- [x] `hnf_client.py` — invokes `hb-bridge-hnf` CLI
- [ ] Eeschema schematic ActionPlugin (PR 2)
- [ ] Bridge panel dock widget using `plugins/kicad/panel/` (PR 3)

## Upstream checklist

| Item | Notes |
|------|-------|
| License | Apache-2.0; matches KiCad third-party plugin guidance |
| Dependencies | Python 3 only in KiCad bundle; no extra pip deps in host |
| Offline default | Plugin inactive until user enables in preferences |
| Tests | KiCad QA not required for optional plugin; hb-bridge `cargo test -p roundtrip-harness` is release gate |
| Docs | KiCad docs/changelog entry: "Experimental HNF bridge plugin" |

## Test plan (reviewer)

1. Build `hb-bridge-hnf` from `hb-bridge` and set `HB_BRIDGE_HNF_CLI`.
2. Install plugin per `plugins/kicad/README.md`.
3. Open sample board → **HB Bridge: Export layout (HNF)** → verify `.layout.hnf.json`.
4. Import sidecar → footprints restored.
5. Optional: `HB_BRIDGE_HOST_KICAD=1 cargo test -p roundtrip-harness kicad_host_hnf_roundtrip -- --ignored`

## References

- HNF spec: `hnf/spec/spec-v0.1.md` (HummingBird format repo)
- Matrix status: `hb-bridge/plugins/README.md`
- Panel stub: `hb-bridge/plugins/kicad/panel/bridge_panel.py`
