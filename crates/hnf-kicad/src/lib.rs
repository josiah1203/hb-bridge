//! KiCad mutation → scene-graph mapping for HCP sidecars.

use hnf_core::SceneGraphDeltas;
use serde_json::{json, Value};
use sidecar_protocol::{Mutation, SceneGraphEdge, SceneGraphNode};

pub fn map_mutation_kind_to_node_type(kind: &str) -> &'static str {
    if kind.starts_with("schematic.") {
        "kicad.schematic.element"
    } else if kind.starts_with("pcb.") {
        "kicad.pcb.element"
    } else {
        "kicad.mutation"
    }
}

pub fn map_mutation_to_scene_delta(
    document_uri: &str,
    index: usize,
    mutation: &Mutation,
) -> SceneGraphDeltas {
    let safe_kind = mutation.kind.replace('/', ".").replace(' ', "_");
    let commit_id = format!("{document_uri}:{index}");
    let node_id = format!("node:{safe_kind}:{index}");
    let edge_id = format!("edge:{safe_kind}:{index}");
    SceneGraphDeltas {
        commit_id,
        nodes: vec![SceneGraphNode {
            nodeId: node_id.clone(),
            nodeType: map_mutation_kind_to_node_type(&mutation.kind).to_string(),
            attributes: json!({
                "kind": mutation.kind,
                "payload": mutation.payload,
                "index": index
            }),
        }],
        edges: vec![SceneGraphEdge {
            edgeId: edge_id,
            fromNodeId: format!("doc:{document_uri}"),
            toNodeId: node_id,
            edgeType: "applies_mutation".to_string(),
            attributes: json!({ "kind": mutation.kind }),
        }],
    }
}

/// Normalize KiCad export format strings to MIME types.
pub fn export_content_type(format: &str) -> Option<&'static str> {
    match format.to_ascii_lowercase().as_str() {
        "step" | "stp" => Some("model/step"),
        "svg" => Some("image/svg+xml"),
        "gerber" => Some("application/gerber"),
        "netlist" => Some("text/plain"),
        _ => None,
    }
}

/// Trace payload written to stderr by sidecars when host OSS is active.
pub fn host_trace_event(event: &str, detail: Value) {
    let payload = json!({ "event": event, "detail": detail, "adapter": "hnf-kicad" });
    if let Ok(line) = serde_json::to_string(&payload) {
        eprintln!("HCP_HOST_OSS:{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn mutation_mapping_produces_stable_scene_delta() {
        let mutation = Mutation {
            kind: "schematic.addSymbol".to_string(),
            payload: json!({"refdes": "R1"}),
        };
        let delta = map_mutation_to_scene_delta("hcp://doc/schematic.kicad_sch", 3, &mutation);
        assert_eq!(delta.commit_id, "hcp://doc/schematic.kicad_sch:3");
        assert_eq!(delta.nodes[0].nodeType, "kicad.schematic.element");
    }
}
