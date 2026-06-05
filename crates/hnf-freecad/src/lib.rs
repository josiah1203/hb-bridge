//! FreeCAD mechanical mutation → scene-graph mapping.

mod hnf_io;

pub use hnf_io::{
    build_mechanical_hnf_document, export_hnf_json, extract_mechanical_domain, import_hnf_json,
    mechanical_domain_to_mutations, mechanical_roundtrip_fingerprint, mechanical_snapshot_from_domain,
    mutations_to_mechanical_domain, roundtrip_mechanical, FreecadHnfError, MechanicalSnapshot,
};

use hnf_core::{HnfMutation, SceneGraphDeltas};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sidecar_protocol::{SceneGraphEdge, SceneGraphNode};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalSolidUpsert {
    pub solid_id: String,
    pub name: String,
    pub material: String,
    pub volume_mm3: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MechanicalConstraintUpsert {
    pub constraint_id: String,
    pub from_solid_id: String,
    pub to_solid_id: String,
    pub constraint_type: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FreecadAdapterError {
    #[error("unsupported mutation kind: {0}")]
    UnsupportedMutation(String),
    #[error("invalid mutation payload: {0}")]
    InvalidPayload(String),
}

pub fn map_mutation_to_deltas(mutation: &HnfMutation) -> Result<SceneGraphDeltas, FreecadAdapterError> {
    let commit_id = mutation
        .payload
        .get("commitId")
        .and_then(Value::as_str)
        .unwrap_or("freecad-local-commit")
        .to_string();

    match mutation.kind.as_str() {
        "mechanical/solid/upsert" => {
            let solid: MechanicalSolidUpsert = serde_json::from_value(mutation.payload.clone())
                .map_err(|err| FreecadAdapterError::InvalidPayload(err.to_string()))?;

            let node = SceneGraphNode {
                nodeId: solid.solid_id.clone(),
                nodeType: "mechanical.solid".to_string(),
                attributes: json!({
                    "name": solid.name,
                    "material": solid.material,
                    "volumeMm3": solid.volume_mm3
                }),
            };

            Ok(SceneGraphDeltas {
                commit_id,
                nodes: vec![node],
                edges: vec![],
            })
        }
        "mechanical/constraint/upsert" => {
            let constraint: MechanicalConstraintUpsert =
                serde_json::from_value(mutation.payload.clone())
                    .map_err(|err| FreecadAdapterError::InvalidPayload(err.to_string()))?;

            let edge = SceneGraphEdge {
                edgeId: constraint.constraint_id,
                fromNodeId: constraint.from_solid_id,
                toNodeId: constraint.to_solid_id,
                edgeType: "mechanical.constraint".to_string(),
                attributes: json!({
                    "constraintType": constraint.constraint_type
                }),
            };

            Ok(SceneGraphDeltas {
                commit_id,
                nodes: vec![],
                edges: vec![edge],
            })
        }
        other => Err(FreecadAdapterError::UnsupportedMutation(other.to_string())),
    }
}

pub fn host_trace_event(event: &str, detail: Value) {
    let payload = json!({ "event": event, "detail": detail, "adapter": "hnf-freecad" });
    if let Ok(line) = serde_json::to_string(&payload) {
        eprintln!("HCP_HOST_OSS:{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_solid_upsert_to_scene_graph_node() {
        let mutation = HnfMutation {
            kind: "mechanical/solid/upsert".to_string(),
            payload: json!({
                "commitId": "c-1",
                "solid_id": "solid-123",
                "name": "Base Plate",
                "material": "Aluminum",
                "volume_mm3": 12345.0
            }),
        };

        let deltas = map_mutation_to_deltas(&mutation).expect("must map");
        assert_eq!(deltas.commit_id, "c-1");
        assert_eq!(deltas.nodes.len(), 1);
        assert_eq!(deltas.nodes[0].nodeType, "mechanical.solid");
    }
}
