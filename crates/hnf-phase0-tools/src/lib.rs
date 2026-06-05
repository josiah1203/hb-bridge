//! Harness-only mutation → scene-graph mapping for Phase 0 tools without host installs.

use hnf_core::SceneGraphDeltas;
use serde_json::{json, Value};
use sidecar_protocol::{Mutation, SceneGraphEdge, SceneGraphNode};

pub const PHASE0_TOOLS: &[&str] = &[
    "klayout",
    "ngspice",
    "yosys",
    "verilator",
    "magic",
    "openroad",
];

pub fn node_type_prefix(tool: &str) -> &'static str {
    match tool {
        "klayout" => "klayout.layout",
        "ngspice" => "ngspice.circuit",
        "yosys" => "yosys.rtl",
        "verilator" => "verilator.tb",
        "magic" => "magic.layout",
        "openroad" => "openroad.pnr",
        _ => "phase0.mutation",
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

/// True when `HB_BRIDGE_HOST_<TOOL>` is set (e.g. `HB_BRIDGE_HOST_YOSYS=1`).
pub fn host_tool_gate_enabled(tool: &str) -> bool {
    let key = format!(
        "HB_BRIDGE_HOST_{}",
        tool.replace('-', "_").to_ascii_uppercase()
    );
    std::env::var(&key)
        .map(|v| !v.is_empty() && v != "0" && v != "false")
        .unwrap_or(false)
}

pub fn host_binary_available(program: &str) -> bool {
    std::process::Command::new("which")
        .arg(program)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn yosys_stub_mapping_is_stable() {
        let mutation = Mutation {
            kind: "rtl.module.upsert".to_string(),
            payload: json!({"top": "counter"}),
        };
        let a = map_mutation_to_scene_delta("yosys", "hbp://doc/top.v", 0, &mutation);
        let b = map_mutation_to_scene_delta("yosys", "hbp://doc/top.v", 0, &mutation);
        assert_eq!(fingerprint_deltas(&a), fingerprint_deltas(&b));
        assert_eq!(a.nodes[0].nodeType, "yosys.rtl");
    }

    #[test]
    fn ngspice_stub_mapping_uses_circuit_node_type() {
        let mutation = Mutation {
            kind: "sim.netlist.upsert".to_string(),
            payload: json!({"name": "rc-lowpass"}),
        };
        let deltas = map_mutation_to_scene_delta("ngspice", "hbp://doc/rc.cir", 1, &mutation);
        assert_eq!(deltas.nodes[0].nodeType, "ngspice.circuit");
    }
}
