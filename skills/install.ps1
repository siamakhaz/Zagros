# zagros-skill standalone installer (Windows PowerShell 5.1+ / PowerShell 7).
# Installs the cybersecurity-expert skill + Zagros MCP wiring. No APM required.
#
# Quick install (run in the project directory, self-hosted Zagros at http://localhost:8789/mcp):
#   irm <release>/install.ps1 | iex
# With a remote (deployed) MCP:
#   $env:ZAGROS_MCP_URL = "https://mcp.example.com/mcp"; irm <release>/install.ps1 | iex
# Safer (download, verify hash, then run):
#   irm <release>/install.ps1 -OutFile install-zagros-skill.ps1
#   irm <release>/install.ps1.sha256 -OutFile install-zagros-skill.ps1.sha256
#   if ((Get-FileHash install-zagros-skill.ps1 -Algorithm SHA256).Hash -ne (Get-Content install-zagros-skill.ps1.sha256).Split()[0]) { throw 'hash mismatch' }
#   powershell -ExecutionPolicy Bypass -File install-zagros-skill.ps1 -UserScope
#
# Server layout expected under $BaseUrl (a versioned Release):
#   install.sh  install.ps1  zagros-skill.tar.gz  zagros-skill.tar.gz.sha256
[CmdletBinding()]
param(
  [string]$Url = "",
  [string]$Dir = (Get-Location).Path,
  [Alias('Global')][switch]$UserScope,
  [string]$McpUrl = "",
  [string]$McpName = "",
  [string]$Sha256 = "",
  [switch]$SkipVerify,
  [switch]$Force
)

$ErrorActionPreference = 'Stop'
# ----------------------------------------------------- EDIT ME (fork) ----
# Point at your published Release assets. Env override: ZAGROS_SKILL_BASE_URL
# (legacy CCSKILL_BASE_URL still honoured).
$BaseUrl = $env:ZAGROS_SKILL_BASE_URL
if (-not $BaseUrl) { $BaseUrl = $env:CCSKILL_BASE_URL }
if (-not $BaseUrl) { $BaseUrl = "https://github.com/<org>/zagros/releases/download/zagros-skill-v0.1.0" }
# ----------------------------------------------------------------------------
$Tarball = "zagros-skill.tar.gz"
$Skill = "cybersecurity-expert"
if (-not $McpName) { $McpName = $env:ZAGROS_MCP_NAME }
if (-not $McpName) { $McpName = $env:CCSKILL_MCP_NAME }
if (-not $McpName) { $McpName = "Zagros" }
if (-not $McpUrl) { $McpUrl = $env:ZAGROS_MCP_URL }
if (-not $McpUrl) { $McpUrl = $env:CCSKILL_MCP_URL }
if (-not $McpUrl) { $McpUrl = "http://localhost:8789/mcp" }

if ($UserScope) {
  $SkillDest = Join-Path $HOME ".agents\skills\$Skill"
  $OpencodeJson = Join-Path $HOME ".config\opencode\opencode.json"
} else {
  $SkillDest = Join-Path $Dir ".agents\skills\$Skill"
  $OpencodeJson = Join-Path $Dir "opencode.json"
}

$Tmp = Join-Path ([IO.Path]::GetTempPath()) ("zagros-skill-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $Tmp | Out-Null
try {
  # --- fetch tarball ---------------------------------------------------------
  if (-not $Url) { $Src = "$BaseUrl/$Tarball" } else { $Src = $Url }
  Write-Host "Fetching $Src ..."
  $Tgz = Join-Path $Tmp $Tarball
  if ($Src -match '^https?://') {
    Invoke-WebRequest -Uri $Src -OutFile $Tgz -UseBasicParsing
    if (-not $Sha256 -and -not $SkipVerify) {
      Write-Host "Fetching $Src.sha256 ..."
      $Sha256 = ((Invoke-WebRequest -Uri "$Src.sha256" -UseBasicParsing).Content -split '\s+')[0]
    }
  } elseif ($Src -match '^file://') {
    Copy-Item ([uri]$Src).LocalPath $Tgz
  } else {
    if (-not (Test-Path $Src)) { throw "Local file not found: $Src" }
    Copy-Item $Src $Tgz
  }

  if ($Sha256) {
    $Actual = (Get-FileHash $Tgz -Algorithm SHA256).Hash.ToLower()
    if ($Actual -ne $Sha256.ToLower()) { throw "SHA256 MISMATCH (expected $Sha256, got $Actual)" }
    Write-Host "Hash verified."
  } elseif ($SkipVerify) {
    Write-Host "Skipping hash verification (-SkipVerify)."
  } else {
    Write-Host "No hash available; pass -Sha256 or host $Tarball.sha256 (or -SkipVerify)."
  }

  # --- install skill files ---------------------------------------------------
  $Tar = Get-Command tar -ErrorAction SilentlyContinue
  if (-not $Tar) { throw "'tar' not found. Windows 10 1803+ ships tar.exe; otherwise install Git for Windows." }
  & $Tar.Source -xzf $Tgz -C $Tmp
  if (-not (Test-Path (Join-Path $Tmp "$Skill\SKILL.md"))) { throw "Tarball is missing $Skill/SKILL.md" }
  if (Test-Path $SkillDest) { Write-Host "Updating existing skill at $SkillDest" } else { Write-Host "Installing skill to $SkillDest" }
  $Parent = Split-Path $SkillDest -Parent
  if (-not (Test-Path $Parent)) { New-Item -ItemType Directory -Path $Parent | Out-Null }
  if (Test-Path $SkillDest) { Remove-Item $SkillDest -Recurse -Force }
  Copy-Item (Join-Path $Tmp $Skill) $SkillDest -Recurse -Force

  # --- wire Zagros MCP into opencode.json ------------------------------------
  $CfgDir = Split-Path $OpencodeJson -Parent
  if (-not (Test-Path $CfgDir)) { New-Item -ItemType Directory -Path $CfgDir | Out-Null }
  if (Test-Path $OpencodeJson) {
    $Raw = Get-Content $OpencodeJson -Raw
    try { $Data = $Raw | ConvertFrom-Json }
    catch {
      Write-Warning "$OpencodeJson is not plain JSON; leaving it untouched."
      Write-Host "ACTION NEEDED: add manually under `"mcp`": {`"$McpName`": {`"type`": `"remote`", `"enabled`": true, `"url`": `"$McpUrl`"}}"
      return
    }
    if ($null -eq $Data) { $Data = @{} }
    if ($null -eq $Data.mcp) { $Data | Add-Member -NotePropertyName mcp -NotePropertyValue @{} }
    $Exists = $Data.mcp.PSObject.Properties[$McpName]
    if ($Exists -and -not $Force) {
      Write-Host "MCP '$McpName' already present in $OpencodeJson; keeping it (use -Force to overwrite)."
    } else {
      Copy-Item $OpencodeJson "$OpencodeJson.bak" -Force
      Write-Host "Backed up $OpencodeJson -> $OpencodeJson.bak"
      if ($Exists) { $Data.mcp.PSObject.Properties.Remove($McpName) }
      $Data.mcp | Add-Member -NotePropertyName $McpName -NotePropertyValue @{ type = "remote"; enabled = $true; url = $McpUrl }
      ($Data | ConvertTo-Json -Depth 10) + "`n" | Set-Content $OpencodeJson -Encoding UTF8
      Write-Host "Wired MCP '$McpName' into $OpencodeJson"
    }
  } else {
    @{ mcp = @{ $McpName = @{ type = "remote"; enabled = $true; url = $McpUrl } } } |
      ConvertTo-Json -Depth 10 | Set-Content $OpencodeJson -Encoding UTF8
    Write-Host "Created $OpencodeJson with MCP '$McpName'"
  }

  Write-Host ""
  Write-Host "Done. Skill: $SkillDest"
  Write-Host "Next: restart opencode, then verify with:  opencode mcp list   (expect Zagros: connected)"
} finally {
  Remove-Item $Tmp -Recurse -Force -ErrorAction SilentlyContinue
}
