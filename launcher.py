#!/usr/bin/env python3
"""
FastDataBroker Launcher
Starts dashboard server and optionally the broker itself
"""

import subprocess
import sys
import time
import os
import webbrowser
from pathlib import Path

def print_banner():
    """Print startup banner"""
    print("\n" + "="*70)
    print("⚡ FastDataBroker Startup Manager".center(70))
    print("="*70)

def start_dashboard():
    """Start dashboard server"""
    print("\n📊 Starting Dashboard Server...\n")
    
    try:
        # Start dashboard server in background
        if sys.platform == "win32":
            proc = subprocess.Popen(
                [sys.executable, "dashboard_server.py"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                creationflags=subprocess.CREATE_NEW_CONSOLE
            )
        else:
            proc = subprocess.Popen(
                [sys.executable, "dashboard_server.py"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                preexec_fn=os.setsid
            )
        
        # Wait for server to start
        time.sleep(2)
        
        print("✓ Dashboard server started")
        print("✓ Open your browser: http://localhost:8080")
        print("✓ Login with: admin / admin (or your credentials)")
        
        # Try to open in browser
        try:
            webbrowser.open("http://localhost:8080")
            print("✓ Opened browser automatically")
        except:
            print("  (Manual: http://localhost:8080)")
        
        return proc
    except Exception as e:
        print(f"✗ Failed to start dashboard: {e}")
        return None

def main():
    """Main launcher"""
    print_banner()
    
    # Start dashboard
    dashboard_proc = start_dashboard()
    if not dashboard_proc:
        sys.exit(1)
    
    # Instructions
    print("\n" + "-"*70)
    print("📖 Next Steps:")
    print("  1. Login to dashboard: http://localhost:8080")
    print("  2. Configure settings in Dashboard → Settings tab")
    print("  3. Review configuration options:")
    print("     ⚙️  App (environment)")
    print("     🖥️  Server (port, connections)")
    print("     📝 Logging (level, output, file path)")
    print("     🔐 Certificates (TLS setup)")
    print("     ✨ Features (multi-tenancy, auth, etc.)")
    print("\n📚 For detailed guide: See STARTUP_GUIDE.md")
    print("-"*70)
    
    # Ask to start broker
    print("\nStart FastDataBroker server now?")
    print("  Note: Make sure to configure settings first in dashboard")
    response = input("\nStart broker? (y/n) [y]: ").strip().lower()
    
    if response != 'n':
        print("\n🚀 Starting FastDataBroker...\n")
        try:
            subprocess.run(["cargo", "run", "--release"], check=False)
        except FileNotFoundError:
            print("✗ Cargo not found")
            print("  Install Rust: https://rustup.rs/")
            sys.exit(1)
        except KeyboardInterrupt:
            print("\n✓ Broker stopped")
    else:
        print("\n💡 To start broker later, run: cargo run --release")
        print("✓ Dashboard continues running at http://localhost:8080")
        
        try:
            # Keep launcher running so dashboard stays accessible
            print("\nPress Ctrl+C to exit")
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\n✓ Launcher stopped")
    
    sys.exit(0)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n✓ Launcher stopped")
        sys.exit(0)
    except Exception as e:
        print(f"\n✗ Error: {e}")
        sys.exit(1)
