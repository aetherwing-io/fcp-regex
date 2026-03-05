use fcp_regex::mcp::server::RegexServer;
use rmcp::{transport::stdio, ServiceExt};

mod bridge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("fcp-regex starting");

    let server = RegexServer::new();

    // Spawn slipstream bridge in background (silent no-op if daemon not running)
    tokio::spawn(bridge::connect(server.clone()));

    let service = server.serve(stdio()).await.inspect_err(|e| {
        eprintln!("MCP server error: {e}");
    })?;

    service.waiting().await?;

    Ok(())
}
