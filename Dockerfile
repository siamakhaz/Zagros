FROM rust:1.95-bookworm@sha256:6258907abe69656e41cd992e0b705cdcfabcbbe3db374f92ed2d47121282d4a1 AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# Build both MCP binaries in one pass so layer caching is shared.
RUN cargo build --release --locked --bin cve-rag-mcp --bin cve-rag-mcp-http

FROM debian:bookworm-slim@sha256:abd67ffcfa541b485a3dff59865ab629aa048a6c613e639d36e7456b0b229241
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates=20250419~deb12u1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 cverag \
    && mkdir -p /data \
    && chown cverag:cverag /data

COPY --from=builder --chown=root:root /src/target/release/cve-rag-mcp      /usr/local/bin/cve-rag-mcp
COPY --from=builder --chown=root:root /src/target/release/cve-rag-mcp-http /usr/local/bin/cve-rag-mcp-http
RUN chmod 755 /usr/local/bin/cve-rag-mcp /usr/local/bin/cve-rag-mcp-http

ENV CVE_RAG_DATA_DIR=/data

# Default to the HTTP server.  Override with --entrypoint when the
# Docker Desktop MCP Toolkit needs the stdio binary.
EXPOSE 8789
VOLUME ["/data"]
USER cverag
ENTRYPOINT ["/usr/local/bin/cve-rag-mcp-http"]
