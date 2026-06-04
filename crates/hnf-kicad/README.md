# hnf-kicad

KiCad mutation → scene-graph mapping and export MIME normalization for HCP.

**Version pin:** `hnf-adapter-sdk = 0.1.0` (path or crates.io after publish).

**Upstream binary:** KiCad 8.x `kicad-cli` from [`hcp-oss/kicad`](https://github.com/hcp-oss/kicad) branch `hcp/integration` (not vendored in monorepo).

## Sidecar usage

`rust/crates/kicad-sidecar` imports this crate for `map_mutation_to_scene_delta`, `export_content_type`, and `host_trace_event` (`HCP_HOST_OSS:` stderr lines).
