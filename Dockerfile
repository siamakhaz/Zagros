FROM rust:1.95-bookworm@sha256:6258907abe69656e41cd992e0b705cdcfabcbbe3db374f92ed2d47121282d4a1 AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# Build CLI + both MCP binaries in one pass so the daily refresh job can use the same trusted ingestion code.
RUN cargo build --release --locked --bin zagros --bin zagros-mcp --bin zagros-mcp-http

FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates=20250419~deb12u1 curl=7.88.1-10+deb12u15 libpcre2-8-0=10.42-1+deb12u1 \
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
      amd64) SUPERCRONIC_SHA256="a53ae236602c7338aba3fbaff40bda6300eae3b9fedb8261eb06cfe3724430c1" ;; \
      arm64) SUPERCRONIC_SHA256="02aa0cb229ba09050cba6638059dadb9eedc2276632ea43d6a57a2f8c1629dd5" ;; \
      *) echo "unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac \
    && curl -fsSL -o /usr/local/bin/supercronic "https://github.com/aptible/supercronic/releases/download/v0.2.49/supercronic-linux-$TARGETARCH" \
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
