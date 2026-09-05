// cve-rag-mcp — stdio MCP transport
//
// Thin wrapper around the shared CveMcpServer in cve_rag::mcp.
// Launched on-demand by Docker Desktop MCP Toolkit as a short-lived
// subprocess with one client connected over stdin/stdout.

use cve_rag::mcp::CveMcpServer;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = CveMcpServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use cve_rag::mcp::valid_cve_id;

    #[test]
    fn validates_cve_ids() {
        assert!(valid_cve_id("CVE-2026-17061"));
        assert!(!valid_cve_id("CVE-26-1"));
        assert!(!valid_cve_id("../../secret"));
    }
}
