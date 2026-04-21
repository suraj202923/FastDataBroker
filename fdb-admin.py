#!/usr/bin/env python3
"""
FastDataBroker Tenant Admin CLI
================================

Command-line interface for tenant management:
  - fdb-admin tenant list
  - fdb-admin tenant add
  - fdb-admin tenant info
  - fdb-admin tenant update
  - fdb-admin tenant enable/disable
  - fdb-admin tenant remove
  - fdb-admin tenant generate-key
  - fdb-admin config validate
"""

import argparse
import json
import sys
import uuid
from pathlib import Path
from typing import Optional, Dict

class TenantAdmin:
    def __init__(self, config_file: str = "appsettings.production.json"):
        self.config_file = config_file
        self.config = self.load_config()
    
    def load_config(self) -> Dict:
        """Load configuration from JSON file"""
        try:
            with open(self.config_file, 'r') as f:
                return json.load(f)
        except FileNotFoundError:
            print(f"❌ Config file not found: {self.config_file}")
            sys.exit(1)
        except json.JSONDecodeError as e:
            print(f"❌ Invalid JSON in {self.config_file}: {e}")
            sys.exit(1)
    
    def save_config(self):
        """Save configuration back to file"""
        try:
            with open(self.config_file, 'w') as f:
                json.dump(self.config, f, indent=2)
            print(f"✅ Configuration saved to {self.config_file}")
        except Exception as e:
            print(f"❌ Failed to save configuration: {e}")
            sys.exit(1)
    
    def list_tenants(self):
        """List all tenants"""
        tenants = self.config.get("tenants", [])
        
        if not tenants:
            print("No tenants found.")
            return
        
        print(f"\n{'TENANT ID':<20} {'NAME':<30} {'STATUS':<10} {'RPS LIMIT':<12}")
        print("─" * 72)
        
        for tenant in tenants:
            status = "✅ Active" if tenant.get("enabled", True) else "❌ Disabled"
            tenant_id = tenant.get("tenant_id", "UNKNOWN")
            name = tenant.get("tenant_name", "UNKNOWN")[:29]
            rps = tenant.get("rate_limit_rps", 0)
            
            print(f"{tenant_id:<20} {name:<30} {status:<10} {rps:<12,}")
        
        print(f"\nTotal: {len(tenants)} tenant(s)")
    
    def add_tenant(self, tenant_id: str, tenant_name: str, api_key_prefix: str,
                  rate_limit: int = 1000, max_connections: int = 100):
        """Add a new tenant"""
        
        # Validate inputs
        if not tenant_id or not tenant_name or not api_key_prefix:
            print("❌ tenant_id, tenant_name, and api_key_prefix are required")
            sys.exit(1)
        
        if not api_key_prefix.endswith('_'):
            print("❌ api_key_prefix must end with underscore (_)")
            sys.exit(1)
        
        # Check for duplicates
        for tenant in self.config.get("tenants", []):
            if tenant["tenant_id"] == tenant_id:
                print(f"❌ Tenant '{tenant_id}' already exists")
                sys.exit(1)
            if tenant["api_key_prefix"] == api_key_prefix:
                print(f"❌ API key prefix '{api_key_prefix}' already in use")
                sys.exit(1)
        
        # Create new tenant
        new_tenant = {
            "tenant_id": tenant_id,
            "tenant_name": tenant_name,
            "description": f"New tenant: {tenant_name}",
            "api_key_prefix": api_key_prefix,
            "rate_limit_rps": rate_limit,
            "max_connections": max_connections,
            "max_message_size": 10485760,  # 10MB default
            "retention_days": 30,
            "features": {
                "priority_queue": True,
                "scheduled_messages": True,
                "routing": True,
                "webhooks": True,
                "clustering": False,
                "metrics": True,
                "persistence": True
            },
            "metadata": {
                "owner": "admin",
                "tier": "standard",
                "region": "us-east-1"
            },
            "enabled": True
        }
        
        self.config["tenants"].append(new_tenant)
        self.save_config()
        
        print(f"✅ Tenant '{tenant_id}' created successfully!")
        print(f"   Name: {tenant_name}")
        print(f"   API Key Prefix: {api_key_prefix}")
        print(f"   Rate Limit: {rate_limit} RPS")
    
    def get_tenant_info(self, tenant_id: str):
        """Get detailed tenant information"""
        tenant = None
        for t in self.config.get("tenants", []):
            if t["tenant_id"] == tenant_id:
                tenant = t
                break
        
        if not tenant:
            print(f"❌ Tenant '{tenant_id}' not found")
            sys.exit(1)
        
        status = "✅ Active" if tenant.get("enabled", True) else "❌ Disabled"
        
        print(f"\n{'─'*60}")
        print(f"TENANT CONFIGURATION: {tenant_id}")
        print(f"{'─'*60}")
        print(f"\nBasic Info:")
        print(f"  ID:          {tenant.get('tenant_id')}")
        print(f"  Name:        {tenant.get('tenant_name')}")
        print(f"  Status:      {status}")
        print(f"  Description: {tenant.get('description', 'N/A')}")
        
        print(f"\nLimits:")
        print(f"  API Key Prefix:     {tenant.get('api_key_prefix')}")
        print(f"  Rate Limit:         {tenant.get('rate_limit_rps'):,} RPS")
        print(f"  Max Connections:    {tenant.get('max_connections'):,}")
        print(f"  Max Message Size:   {tenant.get('max_message_size'):,} bytes")
        print(f"  Retention:          {tenant.get('retention_days')} days")
        
        features = tenant.get("features", {})
        print(f"\nFeatures:")
        for feature, enabled in features.items():
            status = "✅" if enabled else "❌"
            print(f"  {status} {feature:20} {enabled}")
        
        metadata = tenant.get("metadata", {})
        if metadata:
            print(f"\nMetadata:")
            for key, value in metadata.items():
                print(f"  {key:20} {value}")
        
        print(f"\n{'─'*60}\n")
    
    def toggle_tenant(self, tenant_id: str, enable: bool):
        """Enable or disable a tenant"""
        for tenant in self.config.get("tenants", []):
            if tenant["tenant_id"] == tenant_id:
                tenant["enabled"] = enable
                self.save_config()
                action = "enabled" if enable else "disabled"
                print(f"✅ Tenant '{tenant_id}' {action}")
                return
        
        print(f"❌ Tenant '{tenant_id}' not found")
        sys.exit(1)
    
    def remove_tenant(self, tenant_id: str, force: bool = False):
        """Remove a tenant from configuration"""
        if not force:
            response = input(f"⚠️  Are you sure you want to remove tenant '{tenant_id}'? (yes/no): ")
            if response.lower() != "yes":
                print("Cancelled.")
                return
        
        initial_count = len(self.config.get("tenants", []))
        self.config["tenants"] = [t for t in self.config.get("tenants", [])
                                  if t["tenant_id"] != tenant_id]
        
        if len(self.config["tenants"]) < initial_count:
            self.save_config()
            print(f"✅ Tenant '{tenant_id}' removed")
        else:
            print(f"❌ Tenant '{tenant_id}' not found")
            sys.exit(1)
    
    def generate_api_key(self, tenant_id: str):
        """Generate a new API key for a tenant"""
        tenant = None
        for t in self.config.get("tenants", []):
            if t["tenant_id"] == tenant_id:
                tenant = t
                break
        
        if not tenant:
            print(f"❌ Tenant '{tenant_id}' not found")
            sys.exit(1)
        
        prefix = tenant.get("api_key_prefix")
        api_key = f"{prefix}{uuid.uuid4().hex}"
        
        print(f"\n✅ Generated API Key for '{tenant_id}':")
        print(f"\n   {api_key}")
        print(f"\n⚠️  Save this key securely. You won't be able to see it again.")
        print(f"\nKey Details:")
        print(f"  Prefix:     {prefix}")
        print(f"  Tenant:     {tenant_id}")
        print(f"  Rate Limit: {tenant.get('rate_limit_rps')} RPS")
    
    def update_tenant(self, tenant_id: str, **kwargs):
        """Update tenant configuration"""
        tenant = None
        tenant_idx = None
        
        for idx, t in enumerate(self.config.get("tenants", [])):
            if t["tenant_id"] == tenant_id:
                tenant = t
                tenant_idx = idx
                break
        
        if tenant is None:
            print(f"❌ Tenant '{tenant_id}' not found")
            sys.exit(1)
        
        # Update allowed fields
        allowed_fields = ["tenant_name", "description", "rate_limit_rps", 
                         "max_connections", "max_message_size", "retention_days"]
        
        for key, value in kwargs.items():
            if key in allowed_fields and value is not None:
                tenant[key] = value
                print(f"✅ Updated {key}: {value}")
        
        self.config["tenants"][tenant_idx] = tenant
        self.save_config()

def main():
    parser = argparse.ArgumentParser(
        description="FastDataBroker Tenant Administration Tool"
    )
    
    parser.add_argument(
        "--config",
        default="appsettings.production.json",
        help="Configuration file path (default: appsettings.production.json)"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Command to execute")
    
    # List command
    subparsers.add_parser("list", help="List all tenants")
    
    # Add command
    add_parser = subparsers.add_parser("add", help="Add a new tenant")
    add_parser.add_argument("tenant_id", help="Unique tenant ID")
    add_parser.add_argument("tenant_name", help="Human-readable tenant name")
    add_parser.add_argument("api_key_prefix", help="API key prefix (must end with _)")
    add_parser.add_argument("--rate-limit", type=int, default=1000, help="Rate limit (RPS)")
    add_parser.add_argument("--max-connections", type=int, default=100, help="Max connections")
    
    # Info command
    info_parser = subparsers.add_parser("info", help="Get tenant information")
    info_parser.add_argument("tenant_id", help="Tenant ID")
    
    # Enable command
    enable_parser = subparsers.add_parser("enable", help="Enable a tenant")
    enable_parser.add_argument("tenant_id", help="Tenant ID")
    
    # Disable command
    disable_parser = subparsers.add_parser("disable", help="Disable a tenant")
    disable_parser.add_argument("tenant_id", help="Tenant ID")
    
    # Remove command
    remove_parser = subparsers.add_parser("remove", help="Remove a tenant")
    remove_parser.add_argument("tenant_id", help="Tenant ID")
    remove_parser.add_argument("--force", action="store_true", help="Skip confirmation")
    
    # Generate key command
    key_parser = subparsers.add_parser("generate-key", help="Generate API key")
    key_parser.add_argument("tenant_id", help="Tenant ID")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        sys.exit(1)
    
    admin = TenantAdmin(args.config)
    
    if args.command == "list":
        admin.list_tenants()
    
    elif args.command == "add":
        admin.add_tenant(
            args.tenant_id,
            args.tenant_name,
            args.api_key_prefix,
            args.rate_limit,
            args.max_connections
        )
    
    elif args.command == "info":
        admin.get_tenant_info(args.tenant_id)
    
    elif args.command == "enable":
        admin.toggle_tenant(args.tenant_id, True)
    
    elif args.command == "disable":
        admin.toggle_tenant(args.tenant_id, False)
    
    elif args.command == "remove":
        admin.remove_tenant(args.tenant_id, args.force)
    
    elif args.command == "generate-key":
        admin.generate_api_key(args.tenant_id)

if __name__ == "__main__":
    main()
