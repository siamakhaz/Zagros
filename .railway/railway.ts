// Railway deployment blueprint. Applying this changes Railway resources: review plan first.
// No public domains are configured. Operator must add authenticated ingress.
import { defineRailway, github, project, service, volume } from "railway/iac";

export default defineRailway(() => {
  const helixData = volume("zagros-helix-data", { region: "us-west2", sizeMB: 1024 });
  const mcpData = volume("zagros-mcp-data", { region: "us-west2", sizeMB: 1024 });

  const helix = service("zagros-helix", {
    source: github("siamakhaz/Zagros", { branch: "main" }),
    env: {
      RAILWAY_DOCKERFILE_PATH: "docker/Dockerfile.helix.railway",
      HELIX_HTTP_ADDR: "0.0.0.0:8080",
      HELIX_GRPC_ADDR: "0.0.0.0:8081",
      HELIX_DATA_DIR: "/data",
    },
    healthcheck: "/healthz",
    volumeMounts: { "/data": helixData },
  });

  const mcp = service("zagros-mcp", {
    source: github("siamakhaz/Zagros", { branch: "main" }),
    env: {
      RAILWAY_DOCKERFILE_PATH: "Dockerfile.railway",
      HELIX_URL: "http://zagros-helix.railway.internal:8080",
      MCP_BIND_ADDR: "0.0.0.0:8789",
      // Internal-only by default. Configure approved public hostnames separately.
      MCP_ALLOWED_HOSTS: "localhost,127.0.0.1,zagros-mcp.railway.internal",
      ZAGROS_DATA_DIR: "/data",
    },
    healthcheck: "/health",
    volumeMounts: { "/data": mcpData },
  });

  return project("zagros", { resources: [helixData, mcpData, helix, mcp] });
});
