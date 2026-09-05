// zagros-mcp — stdio MCP transport
//
// Thin wrapper around the shared CveMcpServer in zagros::mcp.
// Launched on-demand by Docker Desktop MCP Toolkit as a short-lived
// subprocess with one client connected over stdin/stdout.

use rmcp::{ServiceExt, transport::stdio};
use zagros::mcp::CveMcpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = CveMcpServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use zagros::mcp::valid_cve_id;

    #[test]
    fn validates_cve_ids() {
        assert!(valid_cve_id("CVE-2026-17061"));
        assert!(!valid_cve_id("CVE-26-1"));
        assert!(!valid_cve_id("../../secret"));
    }
}
