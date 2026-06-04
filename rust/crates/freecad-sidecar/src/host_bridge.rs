use std::path::Path;
use std::process::Command;
use std::time::Duration;

use hnf_core::host_env;
use hnf_freecad::host_trace_event;
use serde_json::json;

use crate::{FreecadEngineBridge, FreecadSidecarError, ProjectContext};

const DEFAULT_CMD: &str = "freecadcmd";
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Launches `freecadcmd` (or `HCP_FREECAD_CMD`) for project open when `HCP_USE_HOST_OSS=1`.
pub struct SubprocessFreecadEngineBridge {
    cmd: String,
    timeout: Duration,
}

impl SubprocessFreecadEngineBridge {
    pub fn from_env() -> Self {
        Self {
            cmd: host_env::env_or_default("HCP_FREECAD_CMD", DEFAULT_CMD),
            timeout: host_env::timeout_from_env("HCP_FREECAD_TIMEOUT_SECS", DEFAULT_TIMEOUT_SECS),
        }
    }

    fn stage_workspace(ctx: &ProjectContext) -> Result<&Path, FreecadSidecarError> {
        let root = Path::new(&ctx.workspace_root);
        if !root.is_dir() {
            return Err(FreecadSidecarError::Adapter(format!(
                "workspace root not found: {}",
                ctx.workspace_root
            )));
        }
        Ok(root)
    }

    fn probe(&self) -> Result<(), FreecadSidecarError> {
        let output = Command::new(&self.cmd)
            .arg("--version")
            .output()
            .map_err(|err| {
                FreecadSidecarError::Adapter(format!("failed to spawn {}: {err}", self.cmd))
            })?;
        if output.status.success() {
            host_trace_event(
                "freecadcmd_probe_ok",
                json!({ "cmd": self.cmd, "timeout_secs": self.timeout.as_secs() }),
            );
            Ok(())
        } else {
            Err(FreecadSidecarError::Adapter(format!(
                "{} --version failed (exit {:?})",
                self.cmd,
                output.status.code()
            )))
        }
    }
}

impl FreecadEngineBridge for SubprocessFreecadEngineBridge {
    fn open_project(&self, ctx: &ProjectContext) -> Result<(), FreecadSidecarError> {
        let root = Self::stage_workspace(ctx)?;
        self.probe()?;
        host_trace_event(
            "freecad_open_project",
            json!({
                "cmd": self.cmd,
                "projectId": ctx.project_id,
                "workspaceRoot": root.display().to_string(),
                "timeout_secs": self.timeout.as_secs()
            }),
        );
        Ok(())
    }
}

pub fn select_engine_bridge() -> std::sync::Arc<dyn FreecadEngineBridge> {
    if host_env::use_host_oss() {
        std::sync::Arc::new(SubprocessFreecadEngineBridge::from_env())
    } else {
        std::sync::Arc::new(crate::NoopFreecadEngineBridge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn subprocess_open_fails_when_cmd_missing() {
        std::env::set_var("HCP_USE_HOST_OSS", "1");
        std::env::set_var("HCP_FREECAD_CMD", "/nonexistent/freecadcmd-hcp-test");
        let bridge = SubprocessFreecadEngineBridge::from_env();
        let ctx = ProjectContext {
            project_id: "p".to_string(),
            workspace_root: "/tmp".to_string(),
        };
        assert!(bridge.open_project(&ctx).is_err());
        std::env::remove_var("HCP_USE_HOST_OSS");
        std::env::remove_var("HCP_FREECAD_CMD");
    }

    #[test]
    fn select_bridge_uses_subprocess_when_host_flag_set() {
        std::env::set_var("HCP_USE_HOST_OSS", "1");
        let bridge = select_engine_bridge();
        let _ = Arc::as_ptr(&bridge);
        std::env::remove_var("HCP_USE_HOST_OSS");
    }

    #[test]
    fn select_bridge_defaults_to_noop() {
        for key in ["HCP_USE_HOST_OSS", "HBP_USE_HOST_OSS"] {
            std::env::set_var(key, "0");
        }
        let bridge = select_engine_bridge();
        let ctx = ProjectContext {
            project_id: "p".to_string(),
            workspace_root: "/tmp".to_string(),
        };
        assert!(bridge.open_project(&ctx).is_ok());
    }

    #[test]
    fn host_bridge_requires_existing_workspace() {
        std::env::set_var("HCP_USE_HOST_OSS", "1");
        std::env::set_var("HCP_FREECAD_CMD", "/nonexistent/freecadcmd");
        let bridge = SubprocessFreecadEngineBridge::from_env();
        let ctx = ProjectContext {
            project_id: "p".to_string(),
            workspace_root: "/nonexistent/hcp-workspace-xyz".to_string(),
        };
        let err = bridge.open_project(&ctx).expect_err("missing workspace");
        assert!(err.to_string().contains("workspace root not found"));
        std::env::remove_var("HCP_USE_HOST_OSS");
        std::env::remove_var("HCP_FREECAD_CMD");
    }
}
