//! Harness-only mutation → scene-graph mapping for Phase 1 built-environment tools.

use hnf_core::SceneGraphDeltas;
use serde_json::{json, Value};
use sidecar_protocol::{Mutation, SceneGraphEdge, SceneGraphNode};

pub const PHASE1_TOOLS: &[&str] = &[
    "blenderbim",
    "freecad_bim",
    "openstudio",
    "qgis",
    "grass",
    "openscad",
    "opensees",
    "code_aster",
    "calculix",
    "librecad",
];

pub fn domain_for_tool(tool: &str) -> &'static str {
    match tool {
        "blenderbim" | "freecad_bim" => "bim",
        "openstudio" => "energy_building",
        "qgis" | "grass" => "geospatial",
        "openscad" => "mechanical",
        "opensees" | "code_aster" | "calculix" => "structural",
        "librecad" => "layout",
        _ => "unknown",
    }
}

pub fn node_type_prefix(tool: &str) -> &'static str {
    match tool {
        "blenderbim" => "blenderbim.bim",
        "freecad_bim" => "freecad.bim",
        "openstudio" => "openstudio.energy",
        "qgis" => "qgis.geospatial",
        "grass" => "grass.geospatial",
        "openscad" => "openscad.mechanical",
        "opensees" => "opensees.structural",
        "code_aster" => "code_aster.structural",
        "calculix" => "calculix.structural",
        "librecad" => "librecad.layout",
        _ => "phase1.mutation",
    }
}

pub fn map_mutation_to_scene_delta(
    tool: &str,
    document_uri: &str,
    index: usize,
    mutation: &Mutation,
) -> SceneGraphDeltas {
    let safe_kind = mutation.kind.replace('/', ".").replace(' ', "_");
    let commit_id = format!("{document_uri}:{tool}:{index}");
    let node_id = format!("{tool}:node:{safe_kind}:{index}");
    let edge_id = format!("{tool}:edge:{safe_kind}:{index}");
    SceneGraphDeltas {
        commit_id,
        nodes: vec![SceneGraphNode {
            nodeId: node_id.clone(),
            nodeType: node_type_prefix(tool).to_string(),
            attributes: json!({
                "tool": tool,
                "domain": domain_for_tool(tool),
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
            attributes: json!({ "tool": tool, "kind": mutation.kind }),
        }],
    }
}

pub fn fingerprint_deltas(deltas: &SceneGraphDeltas) -> String {
    format!(
        "{}|{}|{}|{}",
        deltas.commit_id,
        deltas.nodes.len(),
        deltas
            .nodes
            .first()
            .map(|n| n.nodeType.as_str())
            .unwrap_or(""),
        deltas.edges.len()
    )
}

pub fn host_trace_event(tool: &str, event: &str, detail: Value) {
    let payload = json!({ "event": event, "detail": detail, "adapter": format!("hnf-{tool}") });
    if let Ok(line) = serde_json::to_string(&payload) {
        eprintln!("HCP_HOST_OSS:{line}");
    }
}

pub fn host_tool_gate_enabled(tool: &str) -> bool {
    let key = format!(
        "HB_BRIDGE_HOST_{}",
        tool.replace('-', "_").to_ascii_uppercase()
    );
    std::env::var(&key)
        .map(|v| !v.is_empty() && v != "0" && v != "false")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn phase1_tools_cover_ten_entries() {
        assert_eq!(PHASE1_TOOLS.len(), 10);
    }

    #[test]
    fn blenderbim_stub_mapping_is_stable() {
        let mutation = Mutation {
            kind: "bim/element/upsert".to_string(),
            payload: json!({"ifc_class": "IfcWall", "name": "Wall-1"}),
        };
        let d1 = map_mutation_to_scene_delta("blenderbim", "hbp://site/building", 0, &mutation);
        let d2 = map_mutation_to_scene_delta("blenderbim", "hbp://site/building", 0, &mutation);
        assert_eq!(fingerprint_deltas(&d1), fingerprint_deltas(&d2));
        assert_eq!(domain_for_tool("blenderbim"), "bim");
    }

    #[test]
    fn openstudio_maps_energy_domain() {
        let mutation = Mutation {
            kind: "energy/zone/upsert".to_string(),
            payload: json!({"name": "Zone-A"}),
        };
        let deltas = map_mutation_to_scene_delta("openstudio", "hbp://energy/model", 0, &mutation);
        assert!(deltas.nodes[0].nodeType.starts_with("openstudio"));
        assert_eq!(domain_for_tool("openstudio"), "energy_building");
    }
}
