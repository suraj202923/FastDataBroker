#!/usr/bin/env pwsh
# FastDataBroker Admin Dashboard - Quick Start Script (PowerShell)
# Run with: powershell -ExecutionPolicy Bypass -File start-dashboard.ps1

function Write-Header {
    Write-Host ""
    Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║                                                               ║" -ForegroundColor Cyan
    Write-Host "║     ⚡ FastDataBroker Admin Dashboard - Quick Start           ║" -ForegroundColor Cyan
    Write-Host "║                                                               ║" -ForegroundColor Cyan
    Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "❌ ERROR: $Message" -ForegroundColor Red
}

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ️ $Message" -ForegroundColor Yellow
}

# Main script
Write-Header

# Check if Node.js is installed
$NodePath = Get-Command node -ErrorAction SilentlyContinue
if (-not $NodePath) {
    Write-Error-Custom "Node.js is not installed or not in PATH"
    Write-Host ""
    Write-Host "Please install Node.js from: https://nodejs.org/"
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

$nodeVersion = & node --version
Write-Success "Node.js detected: $nodeVersion"

# Check if npm is installed
$NpmPath = Get-Command npm -ErrorAction SilentlyContinue
if (-not $NpmPath) {
    Write-Error-Custom "npm is not installed"
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

$npmVersion = & npm --version
Write-Success "npm detected: $npmVersion"
Write-Host ""

# Set working directory
$workDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
Set-Location $workDir

Write-Info "Working directory: $($PWD.Path)"
Write-Host ""

# Check if required files exist
if (-not (Test-Path "package.json")) {
    Write-Error-Custom "package.json not found in $($PWD.Path)"
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Success "package.json found"

if (-not (Test-Path "admin-dashboard.html")) {
    Write-Error-Custom "admin-dashboard.html not found"
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Success "admin-dashboard.html found"

if (-not (Test-Path "dashboard-server.js")) {
    Write-Error-Custom "dashboard-server.js not found"
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Success "dashboard-server.js found"
Write-Host ""

# Check and install dependencies
if (-not (Test-Path "node_modules")) {
    Write-Info "Installing dependencies..."
    Write-Host ""
    & npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Failed to install dependencies"
        Write-Host ""
        Read-Host "Press Enter to exit"
        exit 1
    }
    Write-Host ""
    Write-Success "Dependencies installed successfully"
} else {
    Write-Success "Dependencies already installed"
}

Write-Host ""
Write-Host "╔═══════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                    STARTING DASHBOARD SERVER                  ║" -ForegroundColor Cyan
Write-Host "╚═══════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""
Write-Info "Starting FastDataBroker Admin Dashboard Server..."
Write-Host ""
Write-Host "🌐 Server will start on: http://127.0.0.1:3000" -ForegroundColor Green
Write-Host ""
Write-Host "📝 Demo Credentials:" -ForegroundColor Cyan
Write-Host "   Username: admin"
Write-Host "   Password: admin"
Write-Host ""
Write-Host "⏳ Waiting 2 seconds before starting..."
Write-Host ""

Start-Sleep -Seconds 2

# Start the server
& npm start

Read-Host "Press Enter to exit"
