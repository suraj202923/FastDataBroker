#!/bin/bash
# FastDataBroker Launcher for Linux/macOS
# Starts dashboard with login and configuration manager

echo ""
echo "Checking prerequisites..."

# Check Python 3
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is not installed"
    echo ""
    echo "Install Python 3:"
    echo "  macOS:  brew install python3"
    echo "  Ubuntu: sudo apt-get install python3"
    echo "  Fedora: sudo dnf install python3"
    exit 1
fi

# Check Cargo (optional - only needed if starting broker)
if ! command -v cargo &> /dev/null; then
    echo "Warning: Cargo not found (needed to run broker)"
    echo "Install Rust from: https://rustup.rs/"
    echo ""
fi

# Start launcher
echo "Starting FastDataBroker Launcher..."
echo ""

python3 launcher.py
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo ""
    echo "Error: Launcher failed (exit code: $EXIT_CODE)"
    exit 1
fi

exit 0
