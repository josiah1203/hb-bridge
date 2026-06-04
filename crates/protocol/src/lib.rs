use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const HCP_RPC_PROTOCOL_V0: &str = "hcp.rpc.v0";
pub const JSONRPC_VERSION: &str = "2.0";

pub mod method {
    pub const PING: &str = "hcp/ping";
    pub const PROJECT_OPEN: &str = "hcp/project/open";
    pub const DOCUMENT_APPLY_MUTATIONS: &str = "hcp/document/applyMutations";
    pub const DOCUMENT_EXPORT: &str = "hcp/document/export";
    pub const SCENEGRAPH_UPSERT_NODES: &str = "hcp/sceneGraph/upsertNodes";
    pub const SCENEGRAPH_UPSERT_EDGES: &str = "hcp/sceneGraph/upsertEdges";
    pub const SCENEGRAPH_CREATE_SNAPSHOT: &str = "hcp/sceneGraph/createSnapshot";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    String(String),
    Number(i64),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: JsonRpcId, result: impl Serialize) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(serde_json::to_value(result).expect("result must serialize")),
            error: None,
        }
    }

    pub fn error(id: JsonRpcId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SidecarInfo {
    pub name: String,
    pub version: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Capabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supportsSceneGraphWrites: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supportsRoundtripExport: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HcpPingResult {
    pub protocol: String,
    pub sidecar: SidecarInfo,
    pub capabilities: Capabilities,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectOpenParams {
    pub projectId: String,
    pub workspaceRoot: String,
    pub auth: ProjectAuth,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAuth {
    pub apiUrl: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OkResult {
    pub ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    pub kind: String,
    pub payload: Value,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyMutationsParams {
    pub documentUri: String,
    pub mutations: Vec<Mutation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyMutationsError {
    pub index: usize,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApplyMutationsResult {
    pub applied: usize,
    pub errors: Vec<ApplyMutationsError>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportParams {
    pub documentUri: String,
    pub format: String,
    pub outputDir: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportArtifact {
    pub path: String,
    pub contentType: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportResult {
    pub artifacts: Vec<ExportArtifact>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneGraphNode {
    pub nodeId: String,
    pub nodeType: String,
    pub attributes: Value,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneGraphEdge {
    pub edgeId: String,
    pub fromNodeId: String,
    pub toNodeId: String,
    pub edgeType: String,
    pub attributes: Value,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneGraphUpsertNodesParams {
    pub commitId: String,
    pub nodes: Vec<SceneGraphNode>,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneGraphUpsertEdgesParams {
    pub commitId: String,
    pub edges: Vec<SceneGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneGraphUpsertResult {
    pub upserted: usize,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneGraphCreateSnapshotParams {
    pub commitId: String,
    pub snapshotFormat: SnapshotFormat,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneGraphCreateSnapshotResult {
    pub snapshotId: String,
    pub artifactObjectId: String,
    pub artifactVersionNum: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapshotFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "glb")]
    Glb,
    #[serde(rename = "octree+json")]
    OctreeJson,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ping_result_round_trips_with_known_fields() {
        let ping = HcpPingResult {
            protocol: HCP_RPC_PROTOCOL_V0.to_string(),
            sidecar: SidecarInfo {
                name: "kicad".to_string(),
                version: "0.1.0".to_string(),
            },
            capabilities: Capabilities {
                supportsSceneGraphWrites: Some(true),
                supportsRoundtripExport: Some(false),
                extra: BTreeMap::new(),
            },
        };

        let encoded = serde_json::to_string(&ping).expect("encode ping");
        let decoded: HcpPingResult = serde_json::from_str(&encoded).expect("decode ping");
        assert_eq!(decoded, ping);
    }

    #[test]
    fn project_open_params_match_schema_shape() {
        let raw = r#"{
          "projectId":"proj-123",
          "workspaceRoot":"/tmp/workspace",
          "auth":{"apiUrl":"https://api.example","token":"abc"}
        }"#;
        let params: ProjectOpenParams = serde_json::from_str(raw).expect("parse params");
        assert_eq!(params.projectId, "proj-123");
        assert_eq!(params.auth.apiUrl, "https://api.example");
    }

    #[test]
    fn scenegraph_payloads_parse_from_schema_shape() {
        let node_payload = json!({
            "commitId": "c-1",
            "nodes": [{
                "nodeId": "n-1",
                "nodeType": "part",
                "attributes": {"label": "R1"}
            }]
        });
        let edge_payload = json!({
            "commitId": "c-1",
            "edges": [{
                "edgeId": "e-1",
                "fromNodeId": "n-1",
                "toNodeId": "n-2",
                "edgeType": "connects_to",
                "attributes": {"net": "VCC"}
            }]
        });
        let snapshot_payload = json!({
            "commitId": "c-1",
            "snapshotFormat": "octree+json"
        });

        let nodes: SceneGraphUpsertNodesParams =
            serde_json::from_value(node_payload).expect("parse upsertNodes params");
        let edges: SceneGraphUpsertEdgesParams =
            serde_json::from_value(edge_payload).expect("parse upsertEdges params");
        let snapshot: SceneGraphCreateSnapshotParams =
            serde_json::from_value(snapshot_payload).expect("parse createSnapshot params");

        assert_eq!(nodes.nodes.len(), 1);
        assert_eq!(edges.edges.len(), 1);
        assert_eq!(snapshot.snapshotFormat, SnapshotFormat::OctreeJson);
    }

    #[test]
    fn strict_schema_rejects_unknown_fields() {
        let raw = r#"{
          "projectId":"proj-123",
          "workspaceRoot":"/tmp/workspace",
          "auth":{"apiUrl":"https://api.example","token":"abc"},
          "unexpected":"field"
        }"#;
        let err = serde_json::from_str::<ProjectOpenParams>(raw).expect_err("must reject extras");
        assert!(err.to_string().contains("unknown field"));
    }
}
