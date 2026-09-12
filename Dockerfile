FROM rust:1.95-bookworm@sha256:6258907abe69656e41cd992e0b705cdcfabcbbe3db374f92ed2d47121282d4a1 AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# Build CLI + both MCP binaries in one pass so the daily refresh job can use the same trusted ingestion code.
RUN cargo build --release --locked --bin zagros --bin zagros-mcp --bin zagros-mcp-http

FROM debian:bookworm-slim@sha256:abd67ffcfa541b485a3dff59865ab629aa048a6c613e639d36e7456b0b229241
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates=20250419~deb12u1 curl=7.88.1-10+deb12u15 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 zagros \
    && mkdir -p /data \
    && chown zagros:zagros /data

COPY --from=builder --chown=root:root /src/target/release/zagros          /usr/local/bin/zagros
COPY --from=builder --chown=root:root /src/target/release/zagros-mcp      /usr/local/bin/zagros-mcp
COPY --from=builder --chown=root:root /src/target/release/zagros-mcp-http /usr/local/bin/zagros-mcp-http
COPY --chown=root:root docker/zagros-refresh.sh /usr/local/bin/zagros-refresh
COPY --chown=root:root docker/mcp-entrypoint.sh /usr/local/bin/mcp-entrypoint
RUN chmod 755 /usr/local/bin/zagros /usr/local/bin/zagros-mcp /usr/local/bin/zagros-mcp-http /usr/local/bin/zagros-refresh /usr/local/bin/mcp-entrypoint

ARG TARGETARCH
RUN case "$TARGETARCH" in \
      amd64) SUPERCRONIC_SHA256="88c1b66b94c486f972fdd1a4d1f901e3e75ff04f749cddd60c5db573e3a33c6c" ;; \
      arm64) SUPERCRONIC_SHA256="50ae8755e04fa72812d0a1bc47a112a856811cc91cce7b6c875c378a850788bc" ;; \
      *) echo "unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && curl -fsSL -o /usr/local/bin/supercronic "https://github.com/aptible/supercronic/releases/download/v0.2.48/supercronic-linux-$TARGETARCH" \
    && echo "$SUPERCRONIC_SHA256  /usr/local/bin/supercronic" | sha256sum -c - \
    && chmod 755 /usr/local/bin/supercronic

ENV ZAGROS_DATA_DIR=/data \
    ZAGROS_AUTO_REFRESH=true \
    ZAGROS_DAILY_CVE_LIMIT=1000

# Default to the HTTP server.  Override with --entrypoint when the
# Docker Desktop MCP Toolkit needs the stdio binary.
EXPOSE 8789
VOLUME ["/data"]
USER zagros
ENTRYPOINT ["/usr/local/bin/mcp-entrypoint"]
