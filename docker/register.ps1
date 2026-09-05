#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Register the zagros MCP server with Docker Desktop.

.DESCRIPTION
    1. Creates (or recreates) the local catalog  zagros-tools:latest
       from docker/server.yaml using the absolute path of this script's
       own directory — so it works on any machine regardless of where
       the repo is cloned.
    2. Adds the server to the Docker MCP profile named "profile".

.PARAMETER Profile
    Name of the Docker MCP profile to add the server to.
    Defaults to "profile".

.EXAMPLE
    # From any directory:
    pwsh zagros/docker/register.ps1

    # With a custom profile name:
    pwsh zagros/docker/register.ps1 -Profile my-team
#>
param(
    [string]$Profile = "profile"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Paths ────────────────────────────────────────────────────────────────────
$scriptDir   = $PSScriptRoot                          # always the docker/ folder
$serverYaml  = Join-Path $scriptDir "server.yaml"
$catalogName = "zagros-tools:latest"
$serverRef   = "catalog://$catalogName/zagros"
$title       = "CVE & Security Knowledge RAG"

# Normalise path separators for the file:// URI (forward slashes on all platforms)
$serverYamlUri = "file://" + ($serverYaml -replace "\\", "/")

# ── Preflight ─────────────────────────────────────────────────────────────────
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Error "docker is not on PATH. Install Docker Desktop and try again."
    exit 1
}

if (-not (Test-Path $serverYaml)) {
    Write-Error "server.yaml not found at: $serverYaml"
    exit 1
}

# ── Step 1: (re)create catalog ────────────────────────────────────────────────
Write-Host ""
Write-Host "==> Registering catalog '$catalogName' ..."
Write-Host "    server.yaml : $serverYaml"

# Remove any stale copy so create is idempotent
$existing = docker mcp catalog list 2>&1 | Select-String $catalogName
if ($existing) {
    Write-Host "    Removing stale catalog ..."
    docker mcp catalog remove $catalogName | Out-Null
}

docker mcp catalog create $catalogName `
    --title $title `
    --server $serverYamlUri

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to create catalog."
    exit 1
}

Write-Host "    Catalog created."

# ── Step 2: add server to profile ─────────────────────────────────────────────
Write-Host ""
Write-Host "==> Adding server to Docker MCP profile '$Profile' ..."

docker mcp profile server add $Profile --server $serverRef

if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to add server to profile '$Profile'."
    exit 1
}

Write-Host ""
Write-Host "Done. zagros is registered in Docker Desktop."
Write-Host "Open Docker Desktop -> MCP Toolkit to confirm."
