#!/usr/bin/env python3
"""
FastDataBroker Web Dashboard Server
Serves the HTML dashboard with configuration management
"""

import json
import os
import sys
import threading
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path
from typing import Dict, Any

class DashboardHandler(SimpleHTTPRequestHandler):
    """Custom HTTP handler for dashboard"""
    
    def do_GET(self):
        """Handle GET requests"""
        if self.path == "/" or self.path == "/dashboard":
            self.serve_dashboard()
        elif self.path.startswith("/api/"):
            self.handle_api_request()
        else:
            super().do_GET()
    
    def serve_dashboard(self):
        """Serve the dashboard HTML"""
        try:
            with open("dashboard.html", "r") as f:
                content = f.read()
            
            self.send_response(200)
            self.send_header("Content-type", "text/html")
            self.end_headers()
            self.wfile.write(content.encode())
        except FileNotFoundError:
            self.send_error(404, "Dashboard not found")
    
    def handle_api_request(self):
        """Handle API requests for configuration"""
        path = self.path.split("?")[0]
        
        if path == "/api/config":
            self.get_config()
        elif path == "/api/config/save":
            self.save_config()
        else:
            self.send_error(404, "API endpoint not found")
    
    def get_config(self):
        """Get current configuration"""
        try:
            if os.path.exists("appsettings.json"):
                with open("appsettings.json", "r") as f:
                    config = json.load(f)
            else:
                config = {}
            
            response = json.dumps({"success": True, "config": config})
            
            self.send_response(200)
            self.send_header("Content-type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(response.encode())
        except Exception as e:
            self.send_error(500, str(e))
    
    def save_config(self):
        """Save configuration"""
        self.send_error(405, "Use POST for saving")
    
    def log_message(self, format, *args):
        """Suppress default logging"""
        pass  # Comment out for debug

class DashboardServer:
    """Web server for dashboard"""
    
    def __init__(self, host: str = "localhost", port: int = 5001):
        self.host = host
        self.port = port
        self.server = None
        self.thread = None
    
    def start(self):
        """Start dashboard server"""
        try:
            self.server = HTTPServer((self.host, self.port), DashboardHandler)
            self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
            self.thread.start()
            
            print(f"\n✓ Dashboard running at http://{self.host}:{self.port}")
            print(f"  ➜ Login with: admin / admin")
            return True
        except Exception as e:
            print(f"✗ Failed to start dashboard: {e}")
            return False
    
    def stop(self):
        """Stop dashboard server"""
        if self.server:
            self.server.shutdown()

if __name__ == "__main__":
    server = DashboardServer(port=8080)
    server.start()
    
    try:
        # Keep server running
        import time
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("\n✓ Dashboard stopped")
        server.stop()
        sys.exit(0)
