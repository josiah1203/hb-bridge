//! Headless roundtrip harness: corpus manifests → adapter mapping stability (no CAD install).

use std::fs;
use std::path::PathBuf;

use hnf_core::{HnfMutation, SceneGraphDeltas};
use hnf_freecad::map_mutation_to_deltas;
use hnf_kicad::map_mutation_to_scene_delta;
use serde::Deserialize;
use serde_json::Value;
use sidecar_protocol::Mutation;

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
struct CorpusCase {
    id: String,
    document_uri: String,
    #[serde(default)]
    sidecars: Vec<String>,
    mutations: Vec<CorpusMutation>,
}

#[derive(Debug, Deserialize)]
struct CorpusMutation {
    kind: String,
    payload: Value,
}

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
    // Re-apply same mutations; mapping must be deterministic.
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
    Ok(())
}

pub fn run_corpus(tool: &str) -> Result<(), String> {
    let manifest = load_corpus(tool);
    for case in &manifest.cases {
        match tool {
            "kicad" => run_kicad_case(case)?,
            "freecad" => run_freecad_case(case)?,
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
    fn corpus_manifest_files_exist() {
        for tool in ["kicad", "freecad"] {
            let path = corpus_manifest_path(tool);
            assert!(path.is_file(), "missing {}", path.display());
        }
    }
}
