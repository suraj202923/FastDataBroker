#!/usr/bin/env python3
"""
FastDataBroker Configuration Deployment Validator
==================================================

Validates all configuration files before deployment:
1. JSON syntax validation
2. Required fields check
3. Tenant configuration validation
4. Cross-tenant conflict detection
5. API key prefix uniqueness
6. Deployment readiness report
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple

class ConfigValidator:
    def __init__(self):
        self.errors: List[str] = []
        self.warnings: List[str] = []
        self.success_checks: List[str] = []
        
    def print_header(self, title: str):
        print(f"\n{'='*80}")
        print(f"  {title}")
        print(f"{'='*80}\n")
    
    def print_result(self, passed: bool, message: str):
        status = "✅" if passed else "❌"
        print(f"{status} {message}")
    
    def validate_json_file(self, filepath: str) -> bool:
        """Validate JSON syntax"""
        try:
            with open(filepath, 'r') as f:
                json.load(f)
            self.success_checks.append(f"Valid JSON: {filepath}")
            self.print_result(True, f"Valid JSON: {filepath}")
            return True
        except json.JSONDecodeError as e:
            self.errors.append(f"Invalid JSON in {filepath}: {e}")
            self.print_result(False, f"Invalid JSON in {filepath}: {e}")
            return False
        except FileNotFoundError:
            self.warnings.append(f"File not found: {filepath}")
            self.print_result(False, f"File not found: {filepath}")
            return False
    
    def validate_required_fields(self, config: Dict) -> bool:
        """Check for required top-level fields"""
        required = ["app", "server", "logging", "features", "tenants"]
        missing = [f for f in required if f not in config]
        
        if missing:
            self.errors.append(f"Missing required fields: {missing}")
            self.print_result(False, f"Missing fields: {missing}")
            return False
        
        self.success_checks.append("All required fields present")
        self.print_result(True, "All required fields present")
        return True
    
    def validate_tenants(self, config: Dict, env: str) -> bool:
        """Validate tenant configurations"""
        if "tenants" not in config or not config["tenants"]:
            self.warnings.append(f"No tenants defined in {env}")
            self.print_result(False, f"No tenants defined in {env}")
            return False
        
        tenants = config["tenants"]
        api_prefixes = set()
        tenant_ids = set()
        all_valid = True
        
        print(f"\n  Validating {len(tenants)} tenant(s)...")
        
        for tenant in tenants:
            # Check required tenant fields
            required_fields = ["tenant_id", "tenant_name", "api_key_prefix", 
                             "rate_limit_rps", "max_connections", "max_message_size"]
            missing = [f for f in required_fields if f not in tenant]
            
            if missing:
                self.errors.append(f"Tenant {tenant.get('tenant_id', 'UNKNOWN')} missing fields: {missing}")
                self.print_result(False, f"Tenant {tenant.get('tenant_id')} has missing fields")
                all_valid = False
                continue
            
            tenant_id = tenant["tenant_id"]
            api_prefix = tenant["api_key_prefix"]
            
            # Check for duplicate tenant IDs
            if tenant_id in tenant_ids:
                self.errors.append(f"Duplicate tenant_id: {tenant_id}")
                self.print_result(False, f"Duplicate tenant_id: {tenant_id}")
                all_valid = False
            else:
                tenant_ids.add(tenant_id)
            
            # Check for duplicate API key prefixes
            if api_prefix in api_prefixes:
                self.errors.append(f"Duplicate API key prefix: {api_prefix}")
                self.print_result(False, f"Duplicate API key prefix: {api_prefix}")
                all_valid = False
            else:
                api_prefixes.add(api_prefix)
            
            # Validate API key prefix format (must end with _)
            if not api_prefix.endswith('_'):
                self.errors.append(f"API key prefix '{api_prefix}' must end with underscore (_)")
                self.print_result(False, f"Invalid API key prefix format: {api_prefix}")
                all_valid = False
            
            # Validate numeric fields
            if tenant["rate_limit_rps"] <= 0:
                self.errors.append(f"Tenant {tenant_id}: rate_limit_rps must be > 0")
                all_valid = False
            
            if tenant["max_connections"] <= 0:
                self.errors.append(f"Tenant {tenant_id}: max_connections must be > 0")
                all_valid = False
            
            if tenant["max_message_size"] <= 0:
                self.errors.append(f"Tenant {tenant_id}: max_message_size must be > 0")
                all_valid = False
            
            # Print tenant summary
            status = "✅" if tenant.get("enabled", True) else "⏸️"
            print(f"    {status} {tenant_id:30} - {tenant.get('tenant_name', 'UNKNOWN')}")
        
        if all_valid:
            self.success_checks.append(f"All {len(tenants)} tenants valid in {env}")
        
        return all_valid
    
    def validate_server_config(self, config: Dict, env: str) -> bool:
        """Validate server configuration"""
        server = config.get("server", {})
        required = ["bind_address", "port", "max_connections"]
        missing = [f for f in required if f not in server]
        
        if missing:
            self.errors.append(f"Server config missing fields: {missing}")
            self.print_result(False, f"Server config incomplete in {env}")
            return False
        
        port = server.get("port", 0)
        if port <= 0 or port > 65535:
            self.errors.append(f"Invalid port number: {port}")
            self.print_result(False, f"Invalid port: {port}")
            return False
        
        self.success_checks.append(f"Server config valid in {env}")
        self.print_result(True, f"Server config valid (port {port})")
        return True
    
    def validate_all_configs(self) -> Tuple[bool, Dict]:
        """Validate all configuration files"""
        self.print_header("FastDataBroker Configuration Deployment Validator")
        
        configs_to_validate = {
            "production": "appsettings.production.json",
            "staging": "appsettings.staging.json",
            "development": "appsettings.development.json"
        }
        
        all_valid = True
        results = {}
        
        for env, filepath in configs_to_validate.items():
            print(f"\n[{env.upper()}] Validating {filepath}...")
            print("-" * 80)
            
            if not self.validate_json_file(filepath):
                all_valid = False
                results[env] = False
                continue
            
            # Load and validate
            try:
                with open(filepath, 'r') as f:
                    config = json.load(f)
                
                # Run all validations
                checks = [
                    (self.validate_required_fields(config), "Required fields"),
                    (self.validate_server_config(config, env), "Server config"),
                    (self.validate_tenants(config, env), "Tenant configs"),
                ]
                
                env_valid = all(check[0] for check in checks)
                results[env] = env_valid
                
                if not env_valid:
                    all_valid = False
                    
            except Exception as e:
                self.errors.append(f"Error validating {env}: {str(e)}")
                all_valid = False
                results[env] = False
        
        return all_valid, results
    
    def print_summary(self, all_valid: bool, results: Dict):
        """Print validation summary"""
        self.print_header("VALIDATION SUMMARY")
        
        print(f"Overall Status: {'✅ PASSED' if all_valid else '❌ FAILED'}\n")
        
        print("Configuration Status:")
        print("-" * 80)
        for env, passed in results.items():
            status = "✅" if passed else "❌"
            print(f"  {status} {env:15} {'READY FOR DEPLOYMENT' if passed else 'ISSUES FOUND'}")
        
        if self.success_checks:
            print(f"\n✅ Success Checks ({len(self.success_checks)}):")
            for check in self.success_checks[:5]:
                print(f"   ✓ {check}")
            if len(self.success_checks) > 5:
                print(f"   ... and {len(self.success_checks) - 5} more")
        
        if self.warnings:
            print(f"\n⚠️  Warnings ({len(self.warnings)}):")
            for warning in self.warnings:
                print(f"   ⚠️  {warning}")
        
        if self.errors:
            print(f"\n❌ Errors ({len(self.errors)}):")
            for error in self.errors:
                print(f"   ✗ {error}")
        
        print("\n" + "=" * 80)
        
        if all_valid:
            print("✅ All configurations are valid and ready for deployment!")
        else:
            print("❌ Configuration validation failed. Please fix the errors before deploying.")
        
        print("=" * 80 + "\n")
    
    def generate_deployment_report(self, all_valid: bool):
        """Generate deployment readiness report"""
        if not all_valid:
            return
        
        self.print_header("DEPLOYMENT READINESS REPORT")
        
        report = """
✅ DEPLOYMENT CHECKLIST
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[✅] Configuration Syntax
     • All JSON files valid
     • No parsing errors

[✅] Required Fields
     • All mandatory fields present
     • No missing configuration

[✅] Tenant Configuration
     • All tenants properly configured
     • No duplicate tenant IDs
     • No duplicate API key prefixes
     • All API key prefixes end with underscore

[✅] Server Configuration
     • Server config valid
     • Port ranges valid
     • Resource limits reasonable

[✅] Environment Overrides
     • Production configuration loaded
     • Staging configuration available
     • Development configuration available

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📋 RECOMMENDED NEXT STEPS:

1. ✓ Review tenant configurations
2. ✓ Validate SSL certificates exist
3. ✓ Verify database connectivity
4. ✓ Test multi-tenant isolation
5. ✓ Run performance benchmarks
6. ✓ Deploy to staging environment
7. ✓ Run integration tests
8. ✓ Deploy to production

✅ DEPLOYMENT STATUS: READY ✅

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"""
        print(report)

def main():
    validator = ConfigValidator()
    all_valid, results = validator.validate_all_configs()
    validator.print_summary(all_valid, results)
    
    if all_valid:
        validator.generate_deployment_report(all_valid)
        sys.exit(0)
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()
