// cve-rag-mcp-http — Streamable HTTP MCP transport
//
// Persistent HTTP server that exposes the same four MCP tools as the
// stdio binary (cve-rag-mcp), but over a long-lived TCP port so that
// OpenCode and other remote clients can connect without Docker Desktop
// MCP Toolkit.
//
// Environment variables
// ---------------------
//   MCP_BIND_ADDR      Bind address.  Default: 0.0.0.0:8789
//   MCP_ALLOWED_HOSTS  Comma-separated Host header values the server
//                      accepts.  Default: "localhost,127.0.0.1"
//                      Set to "*" to disable host validation (not
//                      recommended for public deployments).
//
// The HELIX_URL variable (default http://localhost:47474) is read by
// the shared db::client() in cve_rag::db.

use axum::{Router, routing::any};
use cve_rag::mcp::CveMcpServer;
use rmcp::transport::streamable_http_server::tower::{
    StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── configuration ──────────────────────────────────────────────────────
    let bind_addr = std::env::var("MCP_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8789".to_string());

    let allowed_hosts_raw = std::env::var("MCP_ALLOWED_HOSTS")
        .unwrap_or_else(|_| "localhost,127.0.0.1".to_string());

    // ── rmcp HTTP service ──────────────────────────────────────────────────
    let session_manager = Arc::new(LocalSessionManager::default());

    let mut config = StreamableHttpServerConfig::default();

    if allowed_hosts_raw.trim() == "*" {
        config = config.disable_allowed_hosts();
    } else {
        let hosts: Vec<String> = allowed_hosts_raw
            .split(',')
            .map(|h| h.trim().to_string())
            .filter(|h| !h.is_empty())
            .collect();
        config = config.with_allowed_hosts(hosts);
    }

    let mcp_service = StreamableHttpService::new(
        || Ok(CveMcpServer::new()),
        session_manager,
        config,
    );

    // ── axum router ────────────────────────────────────────────────────────
    let app = Router::new()
        .route("/mcp", any(move |req| {
            let svc = mcp_service.clone();
            async move { svc.handle(req).await }
        }))
        .route("/health", axum::routing::get(health));

    // ── listen ─────────────────────────────────────────────────────────────
    let listener = TcpListener::bind(&bind_addr).await?;
    eprintln!("[cve-rag-mcp-http] listening on http://{bind_addr}/mcp");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
