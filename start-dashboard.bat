@echo off
REM FastDataBroker Admin Dashboard - Quick Start Script
REM Windows Batch Script

setlocal enabledelayedexpansion

echo.
echo ╔═══════════════════════════════════════════════════════════════╗
echo ║                                                               ║
echo ║     ⚡ FastDataBroker Admin Dashboard - Quick Start           ║
echo ║                                                               ║
echo ╚═══════════════════════════════════════════════════════════════╝
echo.

REM Check if Node.js is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ ERROR: Node.js is not installed or not in PATH
    echo.
    echo Please install Node.js from: https://nodejs.org/
    echo Or add Node.js to your PATH environment variable
    echo.
    pause
    exit /b 1
)

REM Check Node.js version
for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo ✓ Node.js detected: %NODE_VERSION%

REM Check if npm is installed
where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo ❌ ERROR: npm is not installed
    echo.
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('npm --version') do set NPM_VERSION=%%i
echo ✓ npm detected: %NPM_VERSION%
echo.

REM Get current directory
cd /d "%~dp0"
echo 📁 Working directory: %cd%
echo.

REM Check if package.json exists
if not exist package.json (
    echo ❌ ERROR: package.json not found in %cd%
    echo.
    pause
    exit /b 1
)

echo ✓ package.json found
echo.

REM Check if dashboard files exist
if not exist admin-dashboard.html (
    echo ❌ ERROR: admin-dashboard.html not found
    echo.
    pause
    exit /b 1
)
echo ✓ admin-dashboard.html found

if not exist dashboard-server.js (
    echo ❌ ERROR: dashboard-server.js not found
    echo.
    pause
    exit /b 1
)
echo ✓ dashboard-server.js found
echo.

REM Check if node_modules exists, if not install
if not exist node_modules (
    echo 📦 Installing dependencies...
    echo.
    call npm install
    if %errorlevel% neq 0 (
        echo ❌ ERROR: Failed to install dependencies
        echo.
        pause
        exit /b 1
    )
    echo.
    echo ✓ Dependencies installed successfully
) else (
    echo ✓ Dependencies already installed
)

echo.
echo ╔═══════════════════════════════════════════════════════════════╗
echo ║                    STARTING DASHBOARD SERVER                  ║
echo ╚═══════════════════════════════════════════════════════════════╝
echo.
echo 🚀 Starting FastDataBroker Admin Dashboard Server...
echo.
echo ⏳ Server starting on http://127.0.0.1:3000
echo.

timeout /t 2 /nobreak

REM Start the server
call npm start

pause
