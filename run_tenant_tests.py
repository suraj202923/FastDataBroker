#!/usr/bin/env python3
"""
FastDataBroker SDK Tenant Tests Runner
Runs all tenant-specific tests across all SDKs
"""

import subprocess
import sys
import os
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple

class TenantTestRunner:
    """Runs tenant-specific tests for all SDKs"""
    
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.sdks_dir = workspace_root / "sdks"
        self.results: Dict[str, Dict] = {}
        self.start_time = datetime.now()
    
    def run_python_tenant_tests(self) -> bool:
        """Run Python SDK tenant tests"""
        print("\n" + "="*70)
        print("🐍 PYTHON SDK - TENANT TESTS")
        print("="*70)
        
        python_dir = self.sdks_dir / "python"
        if not python_dir.exists():
            print("❌ Python SDK directory not found")
            return False
        
        try:
            # Run pytest for Python SDK tenant tests
            cmd = [sys.executable, "-m", "pytest", "test_sdk.py", "-v", "--tb=short"]
            
            os.chdir(python_dir)
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
            
            passed = result.returncode == 0
            self.results["python"] = {
                "status": "✅ PASSED" if passed else "❌ FAILED",
                "output": result.stdout,
                "errors": result.stderr,
                "passed": passed
            }
            
            if passed:
                print("✅ Python SDK - All tenant tests PASSED")
            else:
                print("❌ Python SDK - Some tests FAILED")
                print(result.stdout)
                if result.stderr:
                    print("Errors:", result.stderr)
            
            return passed
        except FileNotFoundError:
            print("⚠️  pytest not found - skipping Python tests")
            self.results["python"] = {"status": "⚠️ SKIPPED", "passed": False}
            return False
        except Exception as e:
            print(f"❌ Error running Python tests: {e}")
            self.results["python"] = {"status": "❌ ERROR", "passed": False}
            return False
    
    def run_go_tenant_tests(self) -> bool:
        """Run Go SDK tenant tests"""
        print("\n" + "="*70)
        print("🐹 GO SDK - TENANT TESTS")
        print("="*70)
        
        go_dir = self.sdks_dir / "go"
        if not go_dir.exists():
            print("❌ Go SDK directory not found")
            return False
        
        try:
            # Run multitenancy tests
            cmd = ["go", "test", "-v", "-run", "Tenant", "./..."]
            
            os.chdir(go_dir)
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
            
            passed = result.returncode == 0
            self.results["go"] = {
                "status": "✅ PASSED" if passed else "❌ FAILED",
                "output": result.stdout,
                "errors": result.stderr,
                "passed": passed
            }
            
            if passed:
                print("✅ Go SDK - All tenant tests PASSED")
            else:
                print("❌ Go SDK - Some tests FAILED")
                print(result.stdout[-500:] if len(result.stdout) > 500 else result.stdout)
            
            return passed
        except FileNotFoundError:
            print("⚠️  Go compiler not found - skipping Go tests")
            self.results["go"] = {"status": "⚠️ SKIPPED", "passed": False}
            return False
        except Exception as e:
            print(f"❌ Error running Go tests: {e}")
            self.results["go"] = {"status": "❌ ERROR", "passed": False}
            return False
    
    def run_java_tenant_tests(self) -> bool:
        """Run Java SDK tenant tests"""
        print("\n" + "="*70)
        print("☕ JAVA SDK - TENANT TESTS")
        print("="*70)
        
        java_dir = self.sdks_dir / "java"
        if not java_dir.exists():
            print("❌ Java SDK directory not found")
            return False
        
        try:
            # Run Maven tests for multitenancy
            cmd = ["mvn", "test", "-Dtest=*MultiTenant*", "-DfailIfNoTests=false"]
            
            os.chdir(java_dir)
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
            
            passed = "BUILD SUCCESS" in result.stdout
            self.results["java"] = {
                "status": "✅ PASSED" if passed else "❌ FAILED",
                "output": result.stdout,
                "errors": result.stderr,
                "passed": passed
            }
            
            if passed:
                print("✅ Java SDK - All tenant tests PASSED")
            else:
                print("❌ Java SDK - Some tests FAILED")
                # Print last 500 chars of output
                print(result.stdout[-500:] if len(result.stdout) > 500 else result.stdout)
            
            return passed
        except FileNotFoundError:
            print("⚠️  Maven not found - skipping Java tests")
            self.results["java"] = {"status": "⚠️ SKIPPED", "passed": False}
            return False
        except Exception as e:
            print(f"❌ Error running Java tests: {e}")
            self.results["java"] = {"status": "❌ ERROR", "passed": False}
            return False
    
    def run_csharp_tenant_tests(self) -> bool:
        """Run C# SDK tenant tests"""
        print("\n" + "="*70)
        print("🔵 C# SDK - TENANT TESTS")
        print("="*70)
        
        csharp_dir = self.sdks_dir / "csharp"
        if not csharp_dir.exists():
            print("❌ C# SDK directory not found")
            return False
        
        try:
            # Run dotnet tests
            cmd = ["dotnet", "test", "--filter", "MultiTenant", "-v", "normal"]
            
            os.chdir(csharp_dir)
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
            
            passed = result.returncode == 0
            self.results["csharp"] = {
                "status": "✅ PASSED" if passed else "❌ FAILED",
                "output": result.stdout,
                "errors": result.stderr,
                "passed": passed
            }
            
            if passed:
                print("✅ C# SDK - All tenant tests PASSED")
            else:
                print("❌ C# SDK - Some tests FAILED")
                print(result.stdout[-500:] if len(result.stdout) > 500 else result.stdout)
            
            return passed
        except FileNotFoundError:
            print("⚠️  .NET SDK not found - skipping C# tests")
            self.results["csharp"] = {"status": "⚠️ SKIPPED", "passed": False}
            return False
        except Exception as e:
            print(f"❌ Error running C# tests: {e}")
            self.results["csharp"] = {"status": "❌ ERROR", "passed": False}
            return False
    
    def run_javascript_tenant_tests(self) -> bool:
        """Run JavaScript SDK tenant tests"""
        print("\n" + "="*70)
        print("📜 JAVASCRIPT SDK - TENANT TESTS")
        print("="*70)
        
        js_dir = self.sdks_dir / "javascript"
        if not js_dir.exists():
            print("❌ JavaScript SDK directory not found")
            return False
        
        try:
            # Check if npm dependencies are installed
            node_modules = js_dir / "node_modules"
            if not node_modules.exists():
                print("⚠️  Installing npm dependencies...")
                install_cmd = ["npm", "install"]
                os.chdir(js_dir)
                subprocess.run(install_cmd, capture_output=True, timeout=120)
            
            # Run Jest with multitenancy filter
            cmd = ["npm", "test", "--", "--testNamePattern='MultiTenant|Tenant'"]
            
            os.chdir(js_dir)
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=120, shell=True)
            
            passed = result.returncode == 0
            self.results["javascript"] = {
                "status": "✅ PASSED" if passed else "❌ FAILED",
                "output": result.stdout,
                "errors": result.stderr,
                "passed": passed
            }
            
            if passed:
                print("✅ JavaScript SDK - All tenant tests PASSED")
            else:
                print("❌ JavaScript SDK - Some tests FAILED")
                print(result.stdout[-500:] if len(result.stdout) > 500 else result.stdout)
            
            return passed
        except FileNotFoundError:
            print("⚠️  Node.js/npm not found - skipping JavaScript tests")
            self.results["javascript"] = {"status": "⚠️ SKIPPED", "passed": False}
            return False
        except Exception as e:
            print(f"❌ Error running JavaScript tests: {e}")
            self.results["javascript"] = {"status": "❌ ERROR", "passed": False}
            return False
    
    def run_all_tests(self) -> None:
        """Run all SDK tenant tests"""
        print("\n" + "="*70)
        print("🧪 FASTDATABROKER - ALL SDK TENANT TESTS")
        print("="*70)
        print(f"Start Time: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print()
        
        # Run tests for each SDK
        python_passed = self.run_python_tenant_tests()
        go_passed = self.run_go_tenant_tests()
        java_passed = self.run_java_tenant_tests()
        csharp_passed = self.run_csharp_tenant_tests()
        javascript_passed = self.run_javascript_tenant_tests()
        
        # Print summary
        self.print_summary()
    
    def print_summary(self) -> None:
        """Print test execution summary"""
        end_time = datetime.now()
        duration = (end_time - self.start_time).total_seconds()
        
        print("\n" + "="*70)
        print("📊 TEST EXECUTION SUMMARY")
        print("="*70)
        print()
        
        passed_count = sum(1 for r in self.results.values() if r.get("passed", False))
        total_count = len(self.results)
        
        print("Results by SDK:")
        print()
        for sdk, result in self.results.items():
            status = result.get("status", "❓ UNKNOWN")
            print(f"  {sdk.upper():12} {status}")
        
        print()
        print("-" * 70)
        print(f"Total: {passed_count}/{total_count} SDKs passed tenant-specific tests")
        print(f"Duration: {duration:.1f} seconds")
        print()
        
        if passed_count == total_count:
            print("✅ ALL TENANT TESTS PASSED!")
        else:
            print(f"⚠️  {total_count - passed_count} SDK test suite(s) need attention")
        
        print()
        print("="*70)

def main():
    """Main entry point"""
    workspace_root = Path.cwd()
    
    runner = TenantTestRunner(workspace_root)
    runner.run_all_tests()

if __name__ == "__main__":
    main()
