"""
Load testing suite for FastDataBroker using Locust
Requirements: locust>=2.15.0

Run with:
    locust -f load_tests/locustfile.py --host http://localhost:6380 -c 1000 -r 100
"""

from locust import HttpUser, task, between, TaskSet, events
from datetime import datetime
import json
import random

class FastDataBrokerTaskSet(TaskSet):
    """Task set for FastDataBroker load testing"""
    
    @task(5)
    def send_message(self):
        """Send a message (5x frequency)"""
        payload = {
            "sender": f"load-test-{random.randint(1, 100)}",
            "recipients": [f"user{random.randint(1, 1000)}@test.com"],
            "subject": f"Test message {random.randint(1, 10000)}",
            "content": f"This is a load test message with random data: {random.random()}"
        }
        
        with self.client.post(
            "/api/send",
            json=payload,
            catch_response=True
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Status code: {response.status_code}")

    @task(3)
    def check_health(self):
        """Check service health (3x frequency)"""
        with self.client.get("/health", catch_response=True) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Health check failed: {response.status_code}")

    @task(2)
    def get_metrics(self):
        """Get metrics (2x frequency)"""
        with self.client.get("/metrics", catch_response=True) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Metrics fetch failed: {response.status_code}")

    @task(1)
    def get_status(self):
        """Get service status (1x frequency)"""
        with self.client.get("/status", catch_response=True) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Status check failed: {response.status_code}")


class FastDataBrokerUser(HttpUser):
    """FastDataBroker load test user"""
    
    tasks = [FastDataBrokerTaskSet]
    wait_time = between(0.5, 2)  # Wait 0.5-2 seconds between tasks
    
    def on_start(self):
        """Called when a user starts"""
        self.user_id = random.randint(1, 1000000)
        print(f"User {self.user_id} started")


class FastFastDataBrokerUser(HttpUser):
    """Fast user with minimal wait time"""
    
    tasks = [FastDataBrokerTaskSet]
    wait_time = between(0.1, 0.5)
    
    def on_start(self):
        self.user_id = random.randint(1, 1000000)


# Event handlers for statistics
@events.test_start.add_listener
def on_test_start(environment, **kwargs):
    """Called when test starts"""
    print(f"\n{'='*60}")
    print(f"FastDataBroker Load Test Started: {datetime.now()}")
    print(f"{'='*60}\n")


@events.test_stop.add_listener
def on_test_stop(environment, **kwargs):
    """Called when test stops"""
    print(f"\n{'='*60}")
    print(f"FastDataBroker Load Test Completed: {datetime.now()}")
    print(f"{'='*60}\n")
    
    # Print statistics
    print("Final Statistics:")
    for name, stats in environment.stats.items():
        if stats.num_requests > 0:
            print(f"  {name}: {stats.num_requests} requests, "
                  f"Avg response time: {stats.avg_response_time:.0f}ms")


@events.request.add_listener
def on_request(request_type, name, response_time, response_length, 
               response, context, exception, **kwargs):
    """Called for each request"""
    if exception:
        print(f"ERROR in {name}: {exception}")
    if response and response.status_code >= 500:
        print(f"Server error ({response.status_code}) in {name}")
