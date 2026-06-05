//! Headless roundtrip harness: corpus manifests → adapter mapping stability (no CAD install).

use std::fs;
use std::path::PathBuf;

use hnf_core::{HnfMutation, SceneGraphDeltas};
use hnf_freecad::{map_mutation_to_deltas, roundtrip_mechanical};
use hnf_kicad::{map_mutation_to_scene_delta, roundtrip_layout, roundtrip_schematic, split_mutations};
use hnf_phase0_tools::{
    fingerprint_deltas as phase0_fingerprint, host_binary_available, host_tool_gate_enabled,
    map_mutation_to_scene_delta as map_phase0_mutation, PHASE0_TOOLS,
};
use serde::Deserialize;
use serde_json::Value;
use sidecar_protocol::Mutation;

#[derive(Debug, Deserialize)]
pub struct CorpusManifest {
    pub cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
pub struct CorpusCase {
    pub id: String,
    pub document_uri: String,
    #[serde(default)]
    pub sidecars: Vec<String>,
    pub mutations: Vec<CorpusMutation>,
}

#[derive(Debug, Deserialize)]
pub struct CorpusMutation {
    kind: String,
    payload: Value,
}

pub const M1_TOOLS: &[&str] = &["kicad", "freecad"];
pub const ALL_CORPUS_TOOLS: &[&str] = &[
    "kicad",
    "freecad",
    "klayout",
    "ngspice",
    "yosys",
    "verilator",
    "magic",
    "openroad",
    "xschem",
    "openems",
    "elmer",
    "qucs-s",
    "platformio",
];

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("hb-bridge root")
        .to_path_buf()
}

pub fn load_corpus(tool: &str) -> CorpusManifest {
    let path = repo_root().join("corpora").join(tool).join("manifest.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!("corpus manifest missing at {}: {err}", path.display())
    });
    serde_json::from_str(&raw).expect("valid corpus manifest JSON")
}

fn fingerprint_kicad_deltas(deltas: &SceneGraphDeltas) -> String {
    format!(
        "{}|{}|{}",
        deltas.commit_id,
        deltas.nodes.len(),
        deltas.edges.len()
    )
}

fn fingerprint_freecad_deltas(deltas: &SceneGraphDeltas) -> String {
    format!(
        "{}|{}|{}",
        deltas.commit_id,
        deltas
            .nodes
            .first()
            .map(|n| n.nodeId.as_str())
            .unwrap_or(""),
        deltas.edges.len()
    )
}

pub fn run_kicad_hnf_document_roundtrip(case: &CorpusCase) -> Result<(), String> {
    let mutations: Vec<Mutation> = case
        .mutations
        .iter()
        .map(|m| Mutation {
            kind: m.kind.clone(),
            payload: m.payload.clone(),
        })
        .collect();
    let (layout, schematic) = split_mutations(&mutations);
    let project = case.document_uri.clone();
    if !layout.is_empty() {
        roundtrip_layout(&project, &layout).map_err(|e| e.to_string())?;
    }
    if !schematic.is_empty() {
        roundtrip_schematic(&project, &schematic).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn run_freecad_hnf_document_roundtrip(case: &CorpusCase) -> Result<(), String> {
    let mutations: Vec<HnfMutation> = case
        .mutations
        .iter()
        .map(|m| HnfMutation {
            kind: m.kind.clone(),
            payload: m.payload.clone(),
        })
        .collect();
    roundtrip_mechanical(&case.document_uri, &mutations).map_err(|e| e.to_string())
}

pub fn run_kicad_case(case: &CorpusCase) -> Result<(), String> {
    let mut fingerprints: Vec<String> = Vec::new();
    for (index, mutation) in case.mutations.iter().enumerate() {
        let m = Mutation {
            kind: mutation.kind.clone(),
            payload: mutation.payload.clone(),
        };
        let deltas = map_mutation_to_scene_delta(&case.document_uri, index, &m);
        fingerprints.push(fingerprint_kicad_deltas(&deltas));
    }
    let mut replay: Vec<String> = Vec::new();
    for (index, mutation) in case.mutations.iter().enumerate() {
        let m = Mutation {
            kind: mutation.kind.clone(),
            payload: mutation.payload.clone(),
        };
        let deltas = map_mutation_to_scene_delta(&case.document_uri, index, &m);
        replay.push(fingerprint_kicad_deltas(&deltas));
    }
    if fingerprints != replay {
        return Err(format!(
            "KiCad roundtrip fingerprint drift for case {}",
            case.id
        ));
    }
    run_kicad_hnf_document_roundtrip(case)?;
    Ok(())
}

pub fn run_freecad_case(case: &CorpusCase) -> Result<(), String> {
    let mut fingerprints: Vec<String> = Vec::new();
    for mutation in &case.mutations {
        let m = HnfMutation {
            kind: mutation.kind.clone(),
            payload: mutation.payload.clone(),
        };
        let deltas = map_mutation_to_deltas(&m).map_err(|e| e.to_string())?;
        fingerprints.push(fingerprint_freecad_deltas(&deltas));
    }
    let mut replay: Vec<String> = Vec::new();
    for mutation in &case.mutations {
        let m = HnfMutation {
            kind: mutation.kind.clone(),
            payload: mutation.payload.clone(),
        };
        let deltas = map_mutation_to_deltas(&m).map_err(|e| e.to_string())?;
        replay.push(fingerprint_freecad_deltas(&deltas));
    }
    if fingerprints != replay {
        return Err(format!(
            "FreeCAD roundtrip fingerprint drift for case {}",
            case.id
        ));
    }
    run_freecad_hnf_document_roundtrip(case)?;
    Ok(())
}

pub fn run_phase0_tool_case(tool: &str, case: &CorpusCase) -> Result<(), String> {
    if !PHASE0_TOOLS.contains(&tool) {
        return Err(format!("not a Phase 0 stub tool: {tool}"));
    }
    let mut fingerprints: Vec<String> = Vec::new();
    for (index, mutation) in case.mutations.iter().enumerate() {
        let m = Mutation {
            kind: mutation.kind.clone(),
            payload: mutation.payload.clone(),
        };
        let deltas = map_phase0_mutation(tool, &case.document_uri, index, &m);
        fingerprints.push(phase0_fingerprint(&deltas));
    }
    let mut replay: Vec<String> = Vec::new();
    for (index, mutation) in case.mutations.iter().enumerate() {
        let m = Mutation {
            kind: mutation.kind.clone(),
            payload: mutation.payload.clone(),
        };
        let deltas = map_phase0_mutation(tool, &case.document_uri, index, &m);
        replay.push(phase0_fingerprint(&deltas));
    }
    if fingerprints != replay {
        return Err(format!(
            "{tool} roundtrip fingerprint drift for case {}",
            case.id
        ));
    }
    Ok(())
}

pub fn run_corpus(tool: &str) -> Result<(), String> {
    let manifest = load_corpus(tool);
    for case in &manifest.cases {
        match tool {
            "kicad" => run_kicad_case(case)?,
            "freecad" => run_freecad_case(case)?,
            t if PHASE0_TOOLS.contains(&t) => run_phase0_tool_case(t, case)?,
            other => return Err(format!("unsupported tool corpus: {other}")),
        }
    }
    Ok(())
}

pub fn corpora_dir() -> PathBuf {
    repo_root().join("corpora")
}

pub fn corpus_manifest_path(tool: &str) -> PathBuf {
    corpora_dir().join(tool).join("manifest.json")
}

/// Host program names for optional smoke tests (`#[ignore]` unless env gate set).
pub fn host_program_for_tool(tool: &str) -> Option<&'static str> {
    match tool {
        "kicad" => Some("kicad-cli"),
        "freecad" => Some("freecadcmd"),
        "klayout" => Some("klayout"),
        "ngspice" => Some("ngspice"),
        "yosys" => Some("yosys"),
        "verilator" => Some("verilator"),
        "magic" => Some("magic"),
        "openroad" => Some("openroad"),
        "xschem" => Some("xschem"),
        "openems" => Some("openems"),
        "elmer" => Some("ElmerSolver"),
        "qucs-s" => Some("qucs"),
        "platformio" => Some("pio"),
        _ => None,
    }
}

pub fn run_host_smoke_if_gated(tool: &str) -> Result<(), String> {
    if !host_tool_gate_enabled(tool) {
        return Err(format!("skipped: set HB_BRIDGE_HOST_{} to run", tool.to_ascii_uppercase()));
    }
    let program = host_program_for_tool(tool).ok_or_else(|| format!("no host binary for {tool}"))?;
    if !host_binary_available(program) {
        return Err(format!("host binary not on PATH: {program}"));
    }
    Ok(())
}

pub fn run_m1_host_hnf_roundtrip(tool: &str) -> Result<(), String> {
    run_host_smoke_if_gated(tool)?;
    let manifest = load_corpus(tool);
    for case in &manifest.cases {
        match tool {
            "kicad" => run_kicad_hnf_document_roundtrip(case)?,
            "freecad" => run_freecad_hnf_document_roundtrip(case)?,
            _ => return Err(format!("unsupported M1 host tool: {tool}")),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kicad_corpus_roundtrip_is_deterministic() {
        run_corpus("kicad").expect("kicad corpus");
    }

    #[test]
    fn freecad_corpus_roundtrip_is_deterministic() {
        run_corpus("freecad").expect("freecad corpus");
    }

    #[test]
    fn klayout_corpus_roundtrip_is_deterministic() {
        run_corpus("klayout").expect("klayout corpus");
    }

    #[test]
    fn ngspice_corpus_roundtrip_is_deterministic() {
        run_corpus("ngspice").expect("ngspice corpus");
    }

    #[test]
    fn yosys_corpus_roundtrip_is_deterministic() {
        run_corpus("yosys").expect("yosys corpus");
    }

    #[test]
    fn verilator_corpus_roundtrip_is_deterministic() {
        run_corpus("verilator").expect("verilator corpus");
    }

    #[test]
    fn magic_corpus_roundtrip_is_deterministic() {
        run_corpus("magic").expect("magic corpus");
    }

    #[test]
    fn openroad_corpus_roundtrip_is_deterministic() {
        run_corpus("openroad").expect("openroad corpus");
    }

    #[test]
    fn xschem_corpus_roundtrip_is_deterministic() {
        run_corpus("xschem").expect("xschem corpus");
    }

    #[test]
    fn openems_corpus_roundtrip_is_deterministic() {
        run_corpus("openems").expect("openems corpus");
    }

    #[test]
    fn elmer_corpus_roundtrip_is_deterministic() {
        run_corpus("elmer").expect("elmer corpus");
    }

    #[test]
    fn qucs_s_corpus_roundtrip_is_deterministic() {
        run_corpus("qucs-s").expect("qucs-s corpus");
    }

    #[test]
    fn platformio_corpus_roundtrip_is_deterministic() {
        run_corpus("platformio").expect("platformio corpus");
    }

    #[test]
    fn corpus_manifest_files_exist() {
        for tool in ALL_CORPUS_TOOLS {
            let path = corpus_manifest_path(tool);
            assert!(path.is_file(), "missing {}", path.display());
        }
    }

    #[test]
    fn fixture_json_exists_for_phase0_tools() {
        let root = repo_root();
        for rel in [
            "tests/fixtures/yosys/minimal_counter.json",
            "tests/fixtures/ngspice/rc_lowpass.json",
            "tests/fixtures/xschem/minimal_amp.json",
            "tests/fixtures/openems/minimal_patch.json",
            "tests/fixtures/elmer/minimal_heat.json",
            "tests/fixtures/qucs-s/rc_filter.json",
            "tests/fixtures/platformio/esp32_env.json",
        ] {
            let path = root.join(rel);
            assert!(path.is_file(), "missing fixture {}", path.display());
        }
    }

    #[test]
    #[ignore = "requires KLayout on PATH; run with HB_BRIDGE_HOST_KLAYOUT=1 and --ignored"]
    fn klayout_host_binary_smoke() {
        run_host_smoke_if_gated("klayout").expect("klayout host");
    }

    #[test]
    #[ignore = "requires ngspice on PATH; run with HB_BRIDGE_HOST_NGSPICE=1 and --ignored"]
    fn ngspice_host_binary_smoke() {
        run_host_smoke_if_gated("ngspice").expect("ngspice host");
    }

    #[test]
    #[ignore = "requires Yosys on PATH; run with HB_BRIDGE_HOST_YOSYS=1 and --ignored"]
    fn yosys_host_binary_smoke() {
        run_host_smoke_if_gated("yosys").expect("yosys host");
    }

    #[test]
    #[ignore = "requires Verilator on PATH; run with HB_BRIDGE_HOST_VERILATOR=1 and --ignored"]
    fn verilator_host_binary_smoke() {
        run_host_smoke_if_gated("verilator").expect("verilator host");
    }

    #[test]
    #[ignore = "requires Magic on PATH; run with HB_BRIDGE_HOST_MAGIC=1 and --ignored"]
    fn magic_host_binary_smoke() {
        run_host_smoke_if_gated("magic").expect("magic host");
    }

    #[test]
    #[ignore = "requires OpenROAD on PATH; run with HB_BRIDGE_HOST_OPENROAD=1 and --ignored"]
    fn openroad_host_binary_smoke() {
        run_host_smoke_if_gated("openroad").expect("openroad host");
    }

    #[test]
    #[ignore = "requires xschem on PATH; run with HB_BRIDGE_HOST_XSCHEM=1 and --ignored"]
    fn xschem_host_binary_smoke() {
        run_host_smoke_if_gated("xschem").expect("xschem host");
    }

    #[test]
    #[ignore = "requires OpenEMS on PATH; run with HB_BRIDGE_HOST_OPENEMS=1 and --ignored"]
    fn openems_host_binary_smoke() {
        run_host_smoke_if_gated("openems").expect("openems host");
    }

    #[test]
    #[ignore = "requires ElmerSolver on PATH; run with HB_BRIDGE_HOST_ELMER=1 and --ignored"]
    fn elmer_host_binary_smoke() {
        run_host_smoke_if_gated("elmer").expect("elmer host");
    }

    #[test]
    #[ignore = "requires qucs on PATH; run with HB_BRIDGE_HOST_QUCS_S=1 and --ignored"]
    fn qucs_s_host_binary_smoke() {
        run_host_smoke_if_gated("qucs-s").expect("qucs-s host");
    }

    #[test]
    #[ignore = "requires pio on PATH; run with HB_BRIDGE_HOST_PLATFORMIO=1 and --ignored"]
    fn platformio_host_binary_smoke() {
        run_host_smoke_if_gated("platformio").expect("platformio host");
    }

    #[test]
    fn kicad_hnf_document_roundtrip_from_corpus() {
        let manifest = load_corpus("kicad");
        for case in &manifest.cases {
            run_kicad_hnf_document_roundtrip(case).expect("kicad HNF document roundtrip");
        }
    }

    #[test]
    fn freecad_hnf_document_roundtrip_from_corpus() {
        let manifest = load_corpus("freecad");
        for case in &manifest.cases {
            run_freecad_hnf_document_roundtrip(case).expect("freecad HNF document roundtrip");
        }
    }

    #[test]
    #[ignore = "requires kicad-cli on PATH; run with HB_BRIDGE_HOST_KICAD=1 and --ignored"]
    fn kicad_host_hnf_roundtrip() {
        run_m1_host_hnf_roundtrip("kicad").expect("kicad host HNF roundtrip");
    }

    #[test]
    #[ignore = "requires freecadcmd on PATH; run with HB_BRIDGE_HOST_FREECAD=1 and --ignored"]
    fn freecad_host_hnf_roundtrip() {
        run_m1_host_hnf_roundtrip("freecad").expect("freecad host HNF roundtrip");
    }
}
