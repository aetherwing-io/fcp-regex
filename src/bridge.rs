// Slipstream bridge — connects FCP server to the Slipstream daemon via Unix socket.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use fcp_regex::domain::mutation;
use fcp_regex::domain::query;
use fcp_regex::fcpcore::parsed_op::parse_op;
use fcp_regex::mcp::server::RegexServer;

// ---------------------------------------------------------------------------
// JSON-RPC types (private to this module)
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct JsonRpcRequest {
    id: serde_json::Value,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(serde::Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// ---------------------------------------------------------------------------
// Socket discovery
// ---------------------------------------------------------------------------

fn discover_socket() -> Option<String> {
    if let Ok(path) = std::env::var("SLIPSTREAM_SOCKET") {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let path = format!("{xdg}/slipstream/daemon.sock");
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    #[cfg(unix)]
    {
        let uid = unsafe { libc::getuid() };
        let path = format!("/tmp/slipstream-{uid}/daemon.sock");
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Bridge entry point
// ---------------------------------------------------------------------------

/// Connect to the slipstream daemon and handle requests.
/// Silently returns on any failure. Call via `tokio::spawn`.
pub async fn connect(server: RegexServer) {
    let _ = run_bridge(server).await;
}

async fn run_bridge(server: RegexServer) -> Result<(), Box<dyn std::error::Error>> {
    let path = discover_socket().ok_or("no socket")?;
    let stream = UnixStream::connect(&path).await?;
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    // Send registration
    let register = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "fcp.register",
        "params": {
            "handler_name": "fcp-regex",
            "extensions": ["regex"],
            "capabilities": ["ops", "query", "session"]
        }
    });
    writer
        .write_all(format!("{register}\n").as_bytes())
        .await?;

    // Request loop
    while let Some(line) = lines.next_line().await? {
        let req: JsonRpcRequest = serde_json::from_str(&line)?;
        let response = handle_request(&server, req).await;
        let json = serde_json::to_string(&response)?;
        writer
            .write_all(format!("{json}\n").as_bytes())
            .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Request dispatch
// ---------------------------------------------------------------------------

async fn handle_request(server: &RegexServer, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = req.params.unwrap_or(serde_json::Value::Null);

    let text = match req.method.as_str() {
        "fcp.session" => {
            let action = params
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            server.handle_session(action).await
        }
        "fcp.ops" => {
            let ops: Vec<String> = params
                .get("ops")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let mut state = server.state().lock().await;
            let mut results = Vec::new();
            for op_str in &ops {
                let op = match parse_op(op_str) {
                    Ok(o) => o,
                    Err(e) => {
                        results.push(format!("ERROR: {}", e.error));
                        continue;
                    }
                };
                let (msg, event) = match op.verb.as_str() {
                    "define" => mutation::handle_define(&op, &mut state.registry),
                    "from" => mutation::handle_from(&op, &mut state.registry),
                    "compile" => mutation::handle_compile(&op, &state.registry),
                    "drop" => mutation::handle_drop(&op, &mut state.registry),
                    "rename" => mutation::handle_rename(&op, &mut state.registry),
                    _ => (format!("ERROR: unknown verb {:?}", op.verb), None),
                };
                if let Some(ev) = event {
                    state.event_log.append(ev);
                }
                results.push(msg);
            }
            results.join("\n")
        }
        "fcp.query" => {
            let q = params.get("q").and_then(|v| v.as_str()).unwrap_or("");
            let state = server.state().lock().await;
            query::handle_query(q, &state.registry)
        }
        _ => format!("unknown method: {}", req.method),
    };

    JsonRpcResponse {
        jsonrpc: "2.0",
        id: req.id,
        result: Some(serde_json::json!({"text": text})),
        error: None,
    }
}
