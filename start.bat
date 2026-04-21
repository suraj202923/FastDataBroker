@echo off
REM FastDataBroker Launcher for Windows
REM Starts dashboard with login and configuration manager

setlocal enabledelayedexpansion

echo.
echo Checking prerequisites...

REM Check Python
python --version >nul 2>&1
if errorlevel 1 (
    echo Error: Python is not installed or not in PATH
    echo Download from: https://www.python.org/
    pause
    exit /b 1
)

REM Check Cargo (optional - only needed if starting broker)
cargo --version >nul 2>&1
if errorlevel 1 (
    echo Warning: Cargo not found (needed to run broker)
    echo Install Rust from: https://rustup.rs/
    echo.
)

REM Start launcher
echo Starting FastDataBroker Launcher...
python launcher.py

if errorlevel 1 (
    echo.
    echo Error: Launcher failed
    pause
    exit /b 1
)

endlocal
