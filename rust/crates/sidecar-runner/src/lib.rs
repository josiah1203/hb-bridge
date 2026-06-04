use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::net::TcpStream;
use std::sync::Arc;

use serde_json::Value;
use sidecar_protocol::{
    method, Capabilities, HcpPingResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, SidecarInfo,
    HCP_RPC_PROTOCOL_V0, JSONRPC_VERSION,
};
use thiserror::Error;

pub type Handler = Arc<dyn Fn(Value) -> Result<Value, JsonRpcError> + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub sidecar_name: String,
    pub sidecar_version: String,
    pub capabilities: Capabilities,
}

#[derive(Clone)]
pub struct SidecarRunner {
    config: RunnerConfig,
    handlers: HashMap<String, Handler>,
}

impl SidecarRunner {
    pub fn new(config: RunnerConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, method_name: impl Into<String>, handler: Handler) {
        self.handlers.insert(method_name.into(), handler);
    }

    pub fn handle_request_line(&self, input: &str) -> Result<Option<String>, RunnerError> {
        if input.trim().is_empty() {
            return Ok(None);
        }

        let request: JsonRpcRequest = serde_json::from_str(input)?;
        if request.jsonrpc != JSONRPC_VERSION {
            let response = JsonRpcResponse::error(
                request.id,
                JsonRpcError {
                    code: -32600,
                    message: "Invalid Request: unsupported jsonrpc version".to_string(),
                    data: None,
                },
            );
            return Ok(Some(serde_json::to_string(&response)?));
        }

        if request.method == method::PING {
            let response = JsonRpcResponse::success(request.id, self.ping_result());
            return Ok(Some(serde_json::to_string(&response)?));
        }

        if let Some(handler) = self.handlers.get(&request.method) {
            match handler(request.params) {
                Ok(result) => Ok(Some(serde_json::to_string(&JsonRpcResponse::success(
                    request.id, result,
                ))?)),
                Err(err) => Ok(Some(serde_json::to_string(&JsonRpcResponse::error(
                    request.id, err,
                ))?)),
            }
        } else {
            Ok(Some(serde_json::to_string(&JsonRpcResponse::error(
                request.id,
                JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                    data: None,
                },
            ))?))
        }
    }

    pub fn run_stdio<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<(), RunnerError> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }

            if let Some(response) = self.handle_request_line(line.trim_end())? {
                writer.write_all(response.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
        }
        Ok(())
    }

    pub fn run_tcp_stream(&self, stream: &mut TcpStream) -> Result<(), RunnerError> {
        let mut reader = std::io::BufReader::new(stream.try_clone()?);
        self.run_stdio(&mut reader, stream)
    }

    fn ping_result(&self) -> HcpPingResult {
        HcpPingResult {
            protocol: HCP_RPC_PROTOCOL_V0.to_string(),
            sidecar: SidecarInfo {
                name: self.config.sidecar_name.clone(),
                version: self.config.sidecar_version.clone(),
            },
            capabilities: self.config.capabilities.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::{Cursor, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    use serde_json::json;
    use sidecar_protocol::JsonRpcId;

    use super::*;

    fn test_runner() -> SidecarRunner {
        SidecarRunner::new(RunnerConfig {
            sidecar_name: "test-sidecar".to_string(),
            sidecar_version: "0.1.0".to_string(),
            capabilities: Capabilities {
                supportsSceneGraphWrites: Some(true),
                supportsRoundtripExport: Some(false),
                extra: BTreeMap::new(),
            },
        })
    }

    #[test]
    fn responds_to_ping_with_capabilities() {
        let runner = test_runner();
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "hcp/ping",
            "params": {}
        })
        .to_string();

        let response_line = runner
            .handle_request_line(&request)
            .expect("runner response")
            .expect("must return response");
        let response: JsonRpcResponse = serde_json::from_str(&response_line).expect("parse response");

        assert_eq!(response.id, JsonRpcId::Number(1));
        let result = response.result.expect("must contain result");
        assert_eq!(result["protocol"], "hcp.rpc.v0");
        assert_eq!(result["sidecar"]["name"], "test-sidecar");
        assert_eq!(result["capabilities"]["supportsSceneGraphWrites"], true);
    }

    #[test]
    fn stdio_loop_processes_multiple_requests() {
        let runner = test_runner();
        let input = format!(
            "{}\n{}\n",
            json!({
                "jsonrpc": "2.0",
                "id": "a",
                "method": "hcp/ping",
                "params": {}
            }),
            json!({
                "jsonrpc": "2.0",
                "id": "b",
                "method": "unknown/method",
                "params": {}
            })
        );

        let mut reader = Cursor::new(input.into_bytes());
        let mut writer: Vec<u8> = Vec::new();
        runner.run_stdio(&mut reader, &mut writer).expect("run stdio");

        let output = String::from_utf8(writer).expect("utf8 output");
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"protocol\":\"hcp.rpc.v0\""));
        assert!(lines[1].contains("Method not found"));
    }

    #[test]
    fn tcp_stream_handles_ping_request() {
        let runner = test_runner();
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let server_runner = runner.clone();

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept stream");
            server_runner
                .run_tcp_stream(&mut stream)
                .expect("run tcp stream");
        });

        let mut client = TcpStream::connect(addr).expect("connect client");
        let request = json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "hcp/ping",
            "params": {}
        })
        .to_string();
        client
            .write_all(format!("{request}\n").as_bytes())
            .expect("write request");
        client.shutdown(std::net::Shutdown::Write).expect("shutdown write");

        let mut raw_response = String::new();
        client.read_to_string(&mut raw_response).expect("read response");
        server.join().expect("server join");

        assert!(raw_response.contains("\"id\":7"));
        assert!(raw_response.contains("\"protocol\":\"hcp.rpc.v0\""));
    }
}
