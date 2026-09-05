#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Full setup, re-setup, and registration helper for the cve-rag MCP server.

.DESCRIPTION
    Idempotent end-to-end script for a new machine or a clean re-install.
    Runs every step in order; skips steps that are already done.

    Steps
    -----
    1. Preflight  — docker, cargo, pwsh version checks
    2. HelixDB    — start cve-rag-helix container (port 47474)
    3. Build      — cargo build --release
    4. Image      — docker build cve-rag-mcp:0.1.0
    5. Register   — docker mcp catalog + profile
    6. Seed       — optional initial CVE backfill (--Seed)

.PARAMETER Profile
    Docker MCP profile to register the server in. Default: "profile".

.PARAMETER Seed
    When supplied, runs an initial CVE backfill after registration.

.PARAMETER SeedLimit
    Number of CVEs to backfill when --Seed is used. Default: 500.

.PARAMETER SkipBuild
    Skip cargo build (use existing binary).

.PARAMETER SkipImage
    Skip docker image build (use existing image).

.EXAMPLE
    # Full setup on a new machine, seed 500 CVEs:
    pwsh cve-rag/scripts/setup.ps1 -Seed

    # Re-register only (binary and image already exist):
    pwsh cve-rag/scripts/setup.ps1 -SkipBuild -SkipImage

    # Custom profile, no seed:
    pwsh cve-rag/scripts/setup.ps1 -Profile my-team
#>
param(
    [string] $Profile    = "profile",
    [switch] $Seed,
    [int]    $SeedLimit  = 500,
    [switch] $SkipBuild,
    [switch] $SkipImage
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ─── Helpers ──────────────────────────────────────────────────────────────────
function Step([string]$msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Ok([string]$msg)   { Write-Host "    [ok] $msg" -ForegroundColor Green }
function Warn([string]$msg) { Write-Host "    [warn] $msg" -ForegroundColor Yellow }
function Fail([string]$msg) { Write-Host "`n[FAIL] $msg" -ForegroundColor Red; exit 1 }

# ─── Paths ────────────────────────────────────────────────────────────────────
$repoRoot    = Split-Path $PSScriptRoot -Parent          # cve-rag/
$dockerDir   = Join-Path $repoRoot "docker"
$composeFile = Join-Path $dockerDir "compose.yml"
$serverYaml  = Join-Path $dockerDir "cve-rag-server.yaml"
$dockerfile  = Join-Path $repoRoot "Dockerfile"
$binary      = Join-Path $repoRoot "target" "release" "cve-rag-mcp.exe"

$catalogName = "cve-rag-tools:latest"
$serverRef   = "catalog://$catalogName/cve-rag"
$imageName   = "cve-rag-mcp:0.1.0"
$helixPort   = 47474
$helixUrl    = "http://localhost:$helixPort"

# ─── Step 1: Preflight ────────────────────────────────────────────────────────
Step "Preflight checks"

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Fail "docker not found. Install Docker Desktop: https://www.docker.com/products/docker-desktop"
}
Ok "docker $(docker --version)"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Fail "cargo not found. Install Rust: https://rustup.rs"
}
Ok "cargo $(cargo --version)"

if ($PSVersionTable.PSVersion.Major -lt 7) {
    Fail "PowerShell 7+ required. Install: https://aka.ms/pscore6"
}
Ok "pwsh $($PSVersionTable.PSVersion)"

# ─── Step 2: HelixDB ─────────────────────────────────────────────────────────
Step "Starting HelixDB (cve-rag-helix on port $helixPort)"

Push-Location $repoRoot
try {
    docker compose -f $composeFile --project-name cve-rag up -d helix 2>&1 | Out-Null
} finally { Pop-Location }

# Wait up to 30 s for /healthz
$deadline = (Get-Date).AddSeconds(30)
$ready    = $false
while ((Get-Date) -lt $deadline) {
    try {
        $resp = Invoke-WebRequest -Uri "$helixUrl/healthz" -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
        if (($resp.Content | ConvertFrom-Json).ready -eq $true) { $ready = $true; break }
    } catch {}
    Start-Sleep -Seconds 2
}
if (-not $ready) { Fail "HelixDB did not become healthy within 30 s. Check: docker logs cve-rag-helix" }
Ok "HelixDB healthy at $helixUrl"

# ─── Step 3: Build binary ─────────────────────────────────────────────────────
if ($SkipBuild) {
    if (-not (Test-Path $binary)) { Fail "--SkipBuild set but binary not found at $binary" }
    Warn "Skipping cargo build (--SkipBuild)."
} else {
    Step "Building release binary  (cargo build --release)"
    Push-Location $repoRoot
    try {
        cargo build --release --bin cve-rag-mcp
        if ($LASTEXITCODE -ne 0) { Fail "cargo build failed." }
    } finally { Pop-Location }
    Ok "Binary: $binary"
}

# ─── Step 4: Docker image ─────────────────────────────────────────────────────
if ($SkipImage) {
    $img = docker images $imageName --format "{{.Repository}}:{{.Tag}}" 2>$null
    if (-not $img) { Fail "--SkipImage set but image '$imageName' not found locally." }
    Warn "Skipping docker build (--SkipImage)."
} else {
    Step "Building Docker image  ($imageName)"
    Push-Location $repoRoot
    try {
        docker build -t $imageName -f $dockerfile .
        if ($LASTEXITCODE -ne 0) { Fail "docker build failed." }
    } finally { Pop-Location }
    Ok "Image $imageName built."
}

# ─── Step 5: Register with Docker MCP ─────────────────────────────────────────
Step "Registering MCP catalog '$catalogName'"

$serverYamlUri = "file://" + ($serverYaml -replace "\\", "/")

$existing = docker mcp catalog list 2>&1 | Select-String $catalogName
if ($existing) {
    Warn "Removing stale catalog '$catalogName' ..."
    docker mcp catalog remove $catalogName | Out-Null
}

docker mcp catalog create $catalogName --title "CVE & Security Knowledge RAG" --server $serverYamlUri
if ($LASTEXITCODE -ne 0) { Fail "docker mcp catalog create failed." }
Ok "Catalog created."

Step "Adding server to Docker MCP profile '$Profile'"
docker mcp profile server add $Profile --server $serverRef
if ($LASTEXITCODE -ne 0) { Fail "docker mcp profile server add failed." }
Ok "Server added to profile '$Profile'."

# ─── Step 6: Optional seed ────────────────────────────────────────────────────
if ($Seed) {
    Step "Seeding CVE database (backfill --limit $SeedLimit) ..."
    $env:HELIX_URL = $helixUrl
    Push-Location $repoRoot
    try {
        & $binary backfill --limit $SeedLimit
        if ($LASTEXITCODE -ne 0) { Warn "backfill exited with code $LASTEXITCODE — check output above." }
    } finally { Pop-Location }
    Ok "Seed complete."
}

# ─── Done ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host " cve-rag is ready." -ForegroundColor Green
Write-Host ""
Write-Host " HelixDB  : $helixUrl  (container: cve-rag-helix)"
Write-Host " Binary   : $binary"
Write-Host " Image    : $imageName"
Write-Host " Profile  : $Profile"
Write-Host ""
Write-Host " Useful commands:"
Write-Host "   Status :  `$env:HELIX_URL='$helixUrl'; $binary status"
Write-Host "   Sync   :  `$env:HELIX_URL='$helixUrl'; $binary ingest --limit 50"
Write-Host "   Search :  `$env:HELIX_URL='$helixUrl'; $binary search `"<query>`""
Write-Host ""
Write-Host " Re-run this script anytime — it is fully idempotent."
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
