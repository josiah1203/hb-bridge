mod host_binding;

use std::sync::{Arc, Mutex};

use hnf_core::SceneGraphDeltas;
pub use host_binding::{select_kicad_binding, SubprocessKiCadBinding};
pub use hnf_kicad::map_mutation_to_scene_delta;
use serde_json::{json, Value};
use sidecar_protocol::{
    method, ApplyMutationsError, ApplyMutationsParams, ApplyMutationsResult, Capabilities,
    ExportArtifact, ExportParams, ExportResult, JsonRpcError, Mutation, OkResult, ProjectOpenParams,
    SceneGraphUpsertEdgesParams, SceneGraphUpsertNodesParams,
    SceneGraphUpsertResult,
};
use sidecar_runner::{Handler, RunnerConfig, SidecarRunner};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContext {
    pub project_id: String,
    pub workspace_root: String,
    pub api_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutationOutcome {
    pub scene_deltas: SceneGraphDeltas,
}

pub trait KiCadBinding: Send + Sync {
    fn apply_mutation(
        &self,
        project: &ProjectContext,
        document_uri: &str,
        index: usize,
        mutation: &Mutation,
    ) -> Result<MutationOutcome, SidecarError>;

    fn export(
        &self,
        project: &ProjectContext,
        document_uri: &str,
        format: &str,
        output_dir: &str,
    ) -> Result<Vec<ExportArtifact>, SidecarError>;
}

pub trait SceneGraphClient: Send + Sync {
    fn upsert_nodes(
        &self,
        params: SceneGraphUpsertNodesParams,
    ) -> Result<SceneGraphUpsertResult, SidecarError>;
    fn upsert_edges(
        &self,
        params: SceneGraphUpsertEdgesParams,
    ) -> Result<SceneGraphUpsertResult, SidecarError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SidecarError {
    #[error("project is not open")]
    ProjectNotOpen,
    #[error("binding error: {0}")]
    Binding(String),
    #[error("scene graph error: {0}")]
    SceneGraph(String),
}

#[derive(Clone)]
pub struct KiCadSidecar {
    binding: Arc<dyn KiCadBinding>,
    scene_graph: Arc<dyn SceneGraphClient>,
    state: Arc<Mutex<Option<ProjectContext>>>,
}

impl KiCadSidecar {
    pub fn new(binding: Arc<dyn KiCadBinding>, scene_graph: Arc<dyn SceneGraphClient>) -> Self {
        Self {
            binding,
            scene_graph,
            state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn register_handlers(&self, runner: &mut SidecarRunner) {
        runner.register_handler(method::PROJECT_OPEN, self.project_open_handler());
        runner.register_handler(
            method::DOCUMENT_APPLY_MUTATIONS,
            self.apply_mutations_handler(),
        );
        runner.register_handler(method::DOCUMENT_EXPORT, self.export_handler());
    }

    pub fn handle_project_open(&self, params: ProjectOpenParams) -> Result<OkResult, SidecarError> {
        let mut guard = self.state.lock().expect("project state lock");
        *guard = Some(ProjectContext {
            project_id: params.projectId,
            workspace_root: params.workspaceRoot,
            api_url: params.auth.apiUrl,
        });
        Ok(OkResult { ok: true })
    }

    pub fn handle_apply_mutations(
        &self,
        params: ApplyMutationsParams,
    ) -> Result<ApplyMutationsResult, SidecarError> {
        Ok(self.apply_and_upsert(&new_commit_id(), &params))
    }

    pub fn handle_export(&self, params: ExportParams) -> Result<ExportResult, SidecarError> {
        let ctx = self.require_context()?;
        let artifacts = self
            .binding
            .export(&ctx, &params.documentUri, &params.format, &params.outputDir)
            .unwrap_or_default();
        Ok(ExportResult { artifacts })
    }

    fn project_open_handler(&self) -> Handler {
        let sidecar = self.clone();
        Arc::new(move |params: Value| {
            let parsed: ProjectOpenParams = parse_params(params)?;
            let out = sidecar.handle_project_open(parsed).map_err(error_to_jsonrpc)?;
            serde_json::to_value(out).map_err(internal_error)
        })
    }

    fn apply_mutations_handler(&self) -> Handler {
        let sidecar = self.clone();
        Arc::new(move |params: Value| {
            let parsed: ApplyMutationsParams = parse_params(params)?;
            let out = sidecar
                .handle_apply_mutations(parsed)
                .map_err(error_to_jsonrpc)?;
            serde_json::to_value(out).map_err(internal_error)
        })
    }

    fn export_handler(&self) -> Handler {
        let sidecar = self.clone();
        Arc::new(move |params: Value| {
            let parsed: ExportParams = parse_params(params)?;
            let out = sidecar.handle_export(parsed).map_err(error_to_jsonrpc)?;
            serde_json::to_value(out).map_err(internal_error)
        })
    }

    fn apply_and_upsert(&self, default_commit_id: &str, params: &ApplyMutationsParams) -> ApplyMutationsResult {
        let mut errors = Vec::new();
        let mut applied = 0usize;
        let ctx = match self.require_context() {
            Ok(v) => v,
            Err(err) => {
                return ApplyMutationsResult {
                    applied,
                    errors: vec![ApplyMutationsError {
                        index: 0,
                        code: "project_not_open".to_string(),
                        message: err.to_string(),
                    }],
                }
            }
        };

        for (index, mutation) in params.mutations.iter().enumerate() {
            let outcome = self
                .binding
                .apply_mutation(&ctx, &params.documentUri, index, mutation);
            match outcome {
                Ok(outcome) => {
                    let commit_id = if outcome.scene_deltas.commit_id.is_empty() {
                        default_commit_id.to_string()
                    } else {
                        outcome.scene_deltas.commit_id
                    };
                    let node_params = SceneGraphUpsertNodesParams {
                        commitId: commit_id.clone(),
                        nodes: outcome.scene_deltas.nodes,
                    };
                    let edge_params = SceneGraphUpsertEdgesParams {
                        commitId: commit_id,
                        edges: outcome.scene_deltas.edges,
                    };
                    let upsert = self
                        .scene_graph
                        .upsert_nodes(node_params)
                        .and_then(|_| self.scene_graph.upsert_edges(edge_params));
                    match upsert {
                        Ok(_) => applied += 1,
                        Err(err) => errors.push(ApplyMutationsError {
                            index,
                            code: "scene_graph_upsert_failed".to_string(),
                            message: err.to_string(),
                        }),
                    }
                }
                Err(err) => errors.push(ApplyMutationsError {
                    index,
                    code: "mutation_apply_failed".to_string(),
                    message: err.to_string(),
                }),
            }
        }

        ApplyMutationsResult { applied, errors }
    }

    fn require_context(&self) -> Result<ProjectContext, SidecarError> {
        let guard = self.state.lock().expect("project state lock");
        guard.clone().ok_or(SidecarError::ProjectNotOpen)
    }
}

fn parse_params<T: serde::de::DeserializeOwned>(params: Value) -> Result<T, JsonRpcError> {
    serde_json::from_value(params).map_err(invalid_params_error)
}

fn invalid_params_error(err: serde_json::Error) -> JsonRpcError {
    JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(json!({ "detail": err.to_string() })),
    }
}

fn internal_error(err: serde_json::Error) -> JsonRpcError {
    JsonRpcError {
        code: -32603,
        message: "Internal error".to_string(),
        data: Some(json!({ "detail": err.to_string() })),
    }
}

fn error_to_jsonrpc(err: SidecarError) -> JsonRpcError {
    JsonRpcError {
        code: -32000,
        message: err.to_string(),
        data: None,
    }
}

fn new_commit_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch");
    format!("kicad-{}", now.as_nanos())
}

/// Emits one JSON object per line on stderr prefixed with `HCP_TRACE:` (regression harness).
pub struct TracingSceneGraph;

impl SceneGraphClient for TracingSceneGraph {
    fn upsert_nodes(
        &self,
        params: SceneGraphUpsertNodesParams,
    ) -> Result<SceneGraphUpsertResult, SidecarError> {
        emit_scene_trace(sidecar_protocol::method::SCENEGRAPH_UPSERT_NODES, &params);
        let upserted = params.nodes.len();
        Ok(SceneGraphUpsertResult { upserted })
    }

    fn upsert_edges(
        &self,
        params: SceneGraphUpsertEdgesParams,
    ) -> Result<SceneGraphUpsertResult, SidecarError> {
        emit_scene_trace(sidecar_protocol::method::SCENEGRAPH_UPSERT_EDGES, &params);
        let upserted = params.edges.len();
        Ok(SceneGraphUpsertResult { upserted })
    }
}

fn emit_scene_trace(method: &str, params: &impl serde::Serialize) {
    let payload = json!({ "method": method, "params": params });
    if let Ok(line) = serde_json::to_string(&payload) {
        eprintln!("HCP_TRACE:{line}");
    }
}

pub fn build_stdio_runner() -> SidecarRunner {
    use std::collections::BTreeMap;

    let sidecar = KiCadSidecar::new(select_kicad_binding(), Arc::new(TracingSceneGraph));
    let mut runner = SidecarRunner::new(RunnerConfig {
        sidecar_name: "kicad".to_string(),
        sidecar_version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: Capabilities {
            supportsSceneGraphWrites: Some(true),
            supportsRoundtripExport: Some(true),
            extra: BTreeMap::new(),
        },
    });
    sidecar.register_handlers(&mut runner);
    runner
}

pub struct StubKiCadBinding;

impl KiCadBinding for StubKiCadBinding {
    fn apply_mutation(
        &self,
        _project: &ProjectContext,
        document_uri: &str,
        index: usize,
        mutation: &Mutation,
    ) -> Result<MutationOutcome, SidecarError> {
        Ok(MutationOutcome {
            scene_deltas: map_mutation_to_scene_delta(document_uri, index, mutation),
        })
    }

    fn export(
        &self,
        _project: &ProjectContext,
        _document_uri: &str,
        format: &str,
        output_dir: &str,
    ) -> Result<Vec<ExportArtifact>, SidecarError> {
        let content_type = match format {
            "step" => "model/step",
            "svg" => "image/svg+xml",
            other => {
                return Err(SidecarError::Binding(format!(
                    "unsupported export format: {other}"
                )))
            }
        };
        Ok(vec![ExportArtifact {
            path: format!("{output_dir}/kicad-export.{format}"),
            contentType: content_type.to_string(),
        }])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;
    use sidecar_protocol::{
        Capabilities, JsonRpcResponse, Mutation, ProjectAuth, ProjectOpenParams,
    };
    use sidecar_runner::{RunnerConfig, SidecarRunner};

    use super::*;

    #[derive(Default)]
    struct FakeBinding;

    impl KiCadBinding for FakeBinding {
        fn apply_mutation(
            &self,
            _project: &ProjectContext,
            document_uri: &str,
            index: usize,
            mutation: &Mutation,
        ) -> Result<MutationOutcome, SidecarError> {
            Ok(MutationOutcome {
                scene_deltas: map_mutation_to_scene_delta(document_uri, index, mutation),
            })
        }

        fn export(
            &self,
            _project: &ProjectContext,
            _document_uri: &str,
            _format: &str,
            _output_dir: &str,
        ) -> Result<Vec<ExportArtifact>, SidecarError> {
            Err(SidecarError::Binding("export unavailable".to_string()))
        }
    }

    #[derive(Default)]
    struct RecordingSceneGraph {
        nodes: Mutex<Vec<SceneGraphUpsertNodesParams>>,
        edges: Mutex<Vec<SceneGraphUpsertEdgesParams>>,
    }

    impl SceneGraphClient for RecordingSceneGraph {
        fn upsert_nodes(
            &self,
            params: SceneGraphUpsertNodesParams,
        ) -> Result<SceneGraphUpsertResult, SidecarError> {
            let upserted = params.nodes.len();
            self.nodes.lock().expect("nodes lock").push(params);
            Ok(SceneGraphUpsertResult { upserted })
        }

        fn upsert_edges(
            &self,
            params: SceneGraphUpsertEdgesParams,
        ) -> Result<SceneGraphUpsertResult, SidecarError> {
            let upserted = params.edges.len();
            self.edges.lock().expect("edges lock").push(params);
            Ok(SceneGraphUpsertResult { upserted })
        }
    }

    fn new_runner() -> SidecarRunner {
        SidecarRunner::new(RunnerConfig {
            sidecar_name: "kicad".to_string(),
            sidecar_version: "0.1.0".to_string(),
            capabilities: Capabilities {
                supportsSceneGraphWrites: Some(true),
                supportsRoundtripExport: Some(true),
                extra: BTreeMap::new(),
            },
        })
    }

    #[test]
    fn handler_requires_project_open_before_mutations() {
        let sidecar = KiCadSidecar::new(
            Arc::new(StubKiCadBinding),
            Arc::new(RecordingSceneGraph::default()),
        );
        let mut runner = new_runner();
        sidecar.register_handlers(&mut runner);
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method::DOCUMENT_APPLY_MUTATIONS,
            "params": {
                "documentUri":"hcp://doc/a",
                "mutations":[{"kind":"schematic.symbol.upsert","payload":{"ref":"R1"}}]
            }
        });
        let out = runner
            .handle_request_line(&req.to_string())
            .expect("runner output")
            .expect("has response");
        let parsed: JsonRpcResponse = serde_json::from_str(&out).expect("parse response");
        let result = parsed.result.expect("result present");
        assert_eq!(result["applied"], 0);
        assert_eq!(result["errors"][0]["code"], "project_not_open");
    }

    #[test]
    fn handlers_apply_mutations_and_emit_scene_upserts() {
        let scene = Arc::new(RecordingSceneGraph::default());
        let sidecar = KiCadSidecar::new(Arc::new(StubKiCadBinding), scene.clone());
        let mut runner = new_runner();
        sidecar.register_handlers(&mut runner);

        let open_req = json!({
            "jsonrpc":"2.0",
            "id": 1,
            "method": method::PROJECT_OPEN,
            "params": {
                "projectId":"p1",
                "workspaceRoot":"/tmp/work",
                "auth":{"apiUrl":"https://api.hcp.local","token":"redacted"}
            }
        });
        runner
            .handle_request_line(&open_req.to_string())
            .expect("open response");

        let mutate_req = json!({
            "jsonrpc":"2.0",
            "id": 2,
            "method": method::DOCUMENT_APPLY_MUTATIONS,
            "params": {
                "documentUri":"hcp://doc/board",
                "mutations":[
                    {"kind":"schematic.symbol.upsert","payload":{"ref":"R1"}},
                    {"kind":"pcb.track.upsert","payload":{"net":"GND"}}
                ]
            }
        });
        let out = runner
            .handle_request_line(&mutate_req.to_string())
            .expect("runner output")
            .expect("has response");
        let parsed: JsonRpcResponse = serde_json::from_str(&out).expect("parse response");
        let result = parsed.result.expect("result present");
        assert_eq!(result["applied"], 2);
        assert_eq!(result["errors"], json!([]));

        let nodes = scene.nodes.lock().expect("nodes calls");
        let edges = scene.edges.lock().expect("edges calls");
        assert_eq!(nodes.len(), 2);
        assert_eq!(edges.len(), 2);
        assert_eq!(nodes[0].nodes[0].nodeType, "kicad.schematic.element");
        assert_eq!(nodes[1].nodes[0].nodeType, "kicad.pcb.element");
    }

    #[test]
    fn export_handler_returns_best_effort_artifact() {
        let sidecar = KiCadSidecar::new(
            Arc::new(StubKiCadBinding),
            Arc::new(RecordingSceneGraph::default()),
        );
        let mut runner = new_runner();
        sidecar.register_handlers(&mut runner);

        let open_req = json!({
            "jsonrpc":"2.0",
            "id": 1,
            "method": method::PROJECT_OPEN,
            "params": {
                "projectId":"p1",
                "workspaceRoot":"/tmp/work",
                "auth":{"apiUrl":"https://api.hcp.local","token":"redacted"}
            }
        });
        runner
            .handle_request_line(&open_req.to_string())
            .expect("open response");

        let export_req = json!({
            "jsonrpc":"2.0",
            "id": 3,
            "method": method::DOCUMENT_EXPORT,
            "params": {
                "documentUri":"hcp://doc/board",
                "format":"step",
                "outputDir":"/tmp/out"
            }
        });
        let out = runner
            .handle_request_line(&export_req.to_string())
            .expect("runner output")
            .expect("has response");
        let parsed: JsonRpcResponse = serde_json::from_str(&out).expect("parse response");
        let result = parsed.result.expect("result present");
        assert_eq!(result["artifacts"][0]["contentType"], "model/step");
    }

    #[test]
    fn mutation_mapping_produces_stable_scene_delta() {
        let mutation = Mutation {
            kind: "schematic.addSymbol".to_string(),
            payload: json!({"refdes":"R1"}),
        };
        let delta = map_mutation_to_scene_delta("hcp://doc/schematic.kicad_sch", 3, &mutation);
        assert_eq!(delta.commit_id, "hcp://doc/schematic.kicad_sch:3");
        assert_eq!(delta.nodes[0].nodeType, "kicad.schematic.element");
        assert_eq!(delta.edges[0].edgeType, "applies_mutation");
    }
}
