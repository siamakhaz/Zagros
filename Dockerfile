FROM rust:1.95-bookworm@sha256:6258907abe69656e41cd992e0b705cdcfabcbbe3db374f92ed2d47121282d4a1 AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# Build both MCP binaries in one pass so layer caching is shared.
RUN cargo build --release --locked --bin zagros-mcp --bin zagros-mcp-http

FROM debian:bookworm-slim@sha256:abd67ffcfa541b485a3dff59865ab629aa048a6c613e639d36e7456b0b229241
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates=20250419~deb12u1 curl=7.88.1-10+deb12u15 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 zagros \
    && mkdir -p /data \
    && chown zagros:zagros /data

COPY --from=builder --chown=root:root /src/target/release/zagros-mcp      /usr/local/bin/zagros-mcp
COPY --from=builder --chown=root:root /src/target/release/zagros-mcp-http /usr/local/bin/zagros-mcp-http
RUN chmod 755 /usr/local/bin/zagros-mcp /usr/local/bin/zagros-mcp-http

ENV ZAGROS_DATA_DIR=/data

# Default to the HTTP server.  Override with --entrypoint when the
# Docker Desktop MCP Toolkit needs the stdio binary.
EXPOSE 8789
VOLUME ["/data"]
USER zagros
ENTRYPOINT ["/usr/local/bin/zagros-mcp-http"]
