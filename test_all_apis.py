#!/usr/bin/env python3
"""Comprehensive API tests for current FastDataBroker admin-api contract.

Covered routes:
- GET /health
- GET /health/detailed
- GET /openapi.json
- CRUD + limits/usage/secrets under /api/v1/tenants
"""

import os
import sys
import time
import json
from typing import Dict, Any, Optional, Tuple

import requests


class ApiTester:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()
        self.session.headers.update(
            {
                "Content-Type": "application/json",
                "X-API-Key": os.getenv("ADMIN_API_KEY", "admin-key-default-change-me"),
            }
        )
        self.passed = 0
        self.failed = 0

    def _url(self, path: str) -> str:
        return f"{self.base_url}{path}"

    def _record(self, ok: bool, name: str, detail: str = "") -> None:
        status = "PASS" if ok else "FAIL"
        line = f"[{status}] {name}"
        if detail:
            line += f" - {detail}"
        print(line)

        if ok:
            self.passed += 1
        else:
            self.failed += 1

    def request(
        self,
        name: str,
        method: str,
        path: str,
        expected_status: int,
        payload: Optional[Dict[str, Any]] = None,
    ) -> Tuple[bool, Optional[requests.Response]]:
        try:
            start = time.time()
            response = self.session.request(
                method=method,
                url=self._url(path),
                json=payload,
                timeout=8,
            )
            elapsed_ms = (time.time() - start) * 1000
            ok = response.status_code == expected_status
            detail = f"status={response.status_code}, expected={expected_status}, {elapsed_ms:.1f}ms"
            if not ok:
                body = response.text.strip().replace("\n", " ")
                detail += f", body={body[:200]}"
            self._record(ok, name, detail)
            return ok, response
        except Exception as ex:
            self._record(False, name, f"exception={ex}")
            return False, None

    def run(self) -> bool:
        print(f"Running API tests against: {self.base_url}")

        ok, _ = self.request("health", "GET", "/health", 200)
        if not ok:
            self._summary()
            return False

        self.request("health detailed", "GET", "/health/detailed", 200)
        self.request("openapi", "GET", "/openapi.json", 200)

        self.request("list tenants before create", "GET", "/api/v1/tenants", 200)

        create_payload = {
            "name": "API Test Tenant",
            "email": "apitest@example.com",
            "rate_limit_rps": 1200,
            "max_connections": 55,
            "max_message_size": 2097152,
            "retention_days": 21,
        }
        ok, create_resp = self.request(
            "create tenant",
            "POST",
            "/api/v1/tenants",
            201,
            create_payload,
        )

        if not ok or create_resp is None:
            self._summary()
            return False

        tenant = create_resp.json()
        tenant_id = tenant.get("tenant_id")
        if not tenant_id:
            self._record(False, "tenant id returned", "missing tenant_id")
            self._summary()
            return False
        self._record(True, "tenant id returned", tenant_id)

        self.request("get tenant", "GET", f"/api/v1/tenants/{tenant_id}", 200)

        update_payload = {
            "name": "API Test Tenant Updated",
            "email": "apitest-updated@example.com",
            "status": "active",
        }
        self.request("update tenant", "PUT", f"/api/v1/tenants/{tenant_id}", 200, update_payload)

        self.request("get tenant usage", "GET", f"/api/v1/tenants/{tenant_id}/usage", 200)
        self.request("get tenant limits", "GET", f"/api/v1/tenants/{tenant_id}/limits", 200)

        limits_payload = {
            "rate_limit_rps": 999,
            "max_connections": 44,
            "max_message_size": 1572864,
            "retention_days": 10,
        }
        self.request(
            "update tenant limits",
            "PUT",
            f"/api/v1/tenants/{tenant_id}/limits",
            200,
            limits_payload,
        )
        self.request(
            "reset tenant limits",
            "POST",
            f"/api/v1/tenants/{tenant_id}/limits/reset",
            200,
        )

        self.request("list tenant secrets", "GET", f"/api/v1/tenants/{tenant_id}/secrets", 200)

        secret_payload = {
            "secret_key": "test_secret_key",
            "secret_value": "test_secret_value",
        }
        ok, create_secret_resp = self.request(
            "create tenant secret",
            "POST",
            f"/api/v1/tenants/{tenant_id}/secrets",
            201,
            secret_payload,
        )

        secret_id = None
        if ok and create_secret_resp is not None:
            body = create_secret_resp.json()
            secret_id = body.get("secret_id")
            if secret_id:
                self._record(True, "secret id returned", secret_id)
            else:
                self._record(False, "secret id returned", "missing secret_id")

        update_secret_payload = {
            "secret_key": "test_secret_key",
            "secret_value": "test_secret_value_updated",
        }
        self.request(
            "update tenant secret",
            "PUT",
            f"/api/v1/tenants/{tenant_id}/secrets",
            200,
            update_secret_payload,
        )

        if secret_id:
            self.request(
                "delete tenant secret",
                "DELETE",
                f"/api/v1/tenants/{tenant_id}/secrets/{secret_id}",
                204,
            )

        self.request("delete tenant", "DELETE", f"/api/v1/tenants/{tenant_id}", 204)
        self.request("list tenants after delete", "GET", "/api/v1/tenants", 200)

        return self._summary()

    def _summary(self) -> bool:
        total = self.passed + self.failed
        print("\n" + "=" * 64)
        print("API TEST SUMMARY")
        print("=" * 64)
        print(f"Passed: {self.passed}")
        print(f"Failed: {self.failed}")
        print(f"Total : {total}")
        success = self.failed == 0
        print("Result: PASS" if success else "Result: FAIL")
        print("=" * 64)
        return success


def main() -> int:
    base_url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
    tester = ApiTester(base_url)
    return 0 if tester.run() else 1


if __name__ == "__main__":
    sys.exit(main())
