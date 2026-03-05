use fcp_regex::mcp::server::RegexServer;
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let server = RegexServer::new();
    let service = server.serve(stdio()).await.inspect_err(|e| {
        eprintln!("MCP server error: {e}");
    })?;

    service.waiting().await?;

    Ok(())
}
