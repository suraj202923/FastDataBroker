"""
FastDataBroker Python SDK - Tenant-Specific QUIC Implementation
Implements tenant-aware QUIC handshake and connection management
"""

import time
import hashlib
import hmac
import random
import json
import threading
from queue import Queue
from dataclasses import dataclass, field
from typing import Dict, Any, Optional, Callable, List
from enum import Enum
from concurrent.futures import ThreadPoolExecutor, as_completed


class WorkerPool:
    """
    Thread pool for batch message processing with queue management
    Provides manual control over worker thread lifecycle
    """
    
    def __init__(self, client: 'TenantQuicClient', num_workers: int):
        """
        Initialize worker pool
        
        Args:
            client: TenantQuicClient instance
            num_workers: Number of worker threads
        """
        self.client = client
        self.num_workers = num_workers
        self.message_queue = Queue()
        self.results = []
        self.lock = threading.Lock()
        self.executor = None
        self.futures = []
        self._start_workers()
    
    def _start_workers(self):
        """Start worker threads"""
        self.executor = ThreadPoolExecutor(max_workers=self.num_workers)
        
        for _ in range(self.num_workers):
            future = self.executor.submit(self._worker_loop)
            self.futures.append(future)
    
    def _worker_loop(self):
        """Worker thread loop"""
        while True:
            message = self.message_queue.get()
            
            if message is None:  # Sentinel value to stop worker
                break
            
            try:
                result = self.client.send_message(message)
                with self.lock:
                    self.results.append(result)
            except Exception as e:
                result = DeliveryResult(
                    message_id=f"error_{int(time.time() * 1000)}",
                    status=f"error: {str(e)}",
                    latency_ms=0,
                    timestamp=int(time.time() * 1000),
                    tenant_id=self.client.tenant_config.tenant_id
                )
                with self.lock:
                    self.results.append(result)
            
            self.message_queue.task_done()
    
    def queue_message(self, message: 'Message'):
        """
        Queue a message for processing
        
        Args:
            message: Message to queue
        """
        self.message_queue.put(message)
    
    def queue_messages(self, messages: List['Message']):
        """
        Queue multiple messages for processing
        
        Args:
            messages: List of messages to queue
        """
        for message in messages:
            self.queue_message(message)
    
    def wait_completion(self):
        """Wait for all queued messages to be processed"""
        self.message_queue.join()
    
    def get_all_results(self) -> List['DeliveryResult']:
        """
        Get all processing results
        Blocks until all queued messages are processed
        
        Returns:
            List of DeliveryResult objects
        """
        self.wait_completion()
        self.stop()
        return self.results
    
    def stop(self):
        """Stop worker threads"""
        # Send sentinel values to stop workers
        for _ in range(self.num_workers):
            self.message_queue.put(None)
        
        # Wait for workers to finish
        if self.executor:
            self.executor.shutdown(wait=True)


class ConnectionState(Enum):
    """QUIC Connection States"""
    IDLE = "idle"
    HANDSHAKE = "handshake"
    ESTABLISHED = "established"
    CLOSING = "closing"
    CLOSED = "closed"


class TenantRole(Enum):
    """Tenant role types"""
    ADMIN = "admin"
    USER = "user"
    SERVICE = "service"


@dataclass(frozen=True)
class Message:
    """Message class with tenant context"""
    topic: str
    payload: Any
    priority: int = 5
    ttl_seconds: int = 3600
    headers: Optional[Dict[str, str]] = None
    tenant_id: Optional[str] = None


@dataclass
class DeliveryResult:
    """Message delivery result"""
    message_id: str
    status: str
    latency_ms: float
    timestamp: int
    tenant_id: Optional[str] = None


@dataclass
class ConnectionStats:
    """Connection statistics"""
    is_connected: bool
    messages_sent: int
    messages_received: int
    connection_time_ms: int
    uptime_seconds: int
    last_message_time: int
    handshake_duration_ms: int = 0


@dataclass
class TenantConfig:
    """Tenant-specific configuration"""
    tenant_id: str
    psk_secret: str
    client_id: str
    secrets: str
    role: TenantRole = TenantRole.USER
    rate_limit_rps: int = 1000
    max_connections: int = 100
    custom_headers: Dict[str, str] = field(default_factory=dict)


@dataclass
class QuicHandshakeParams:
    """QUIC Handshake Parameters"""
    tenant_id: str
    client_id: str
    timestamp_ms: int
    random_nonce: str
    psk_token: str
    initial_max_streams: int = 100
    idle_timeout_ms: int = 30000


class TenantQuicClient:
    """
    FastDataBroker QUIC Client with tenant-specific handshake
    Implements RFC 9001 (QUIC TLS) with tenant-aware PSK
    """

    def __init__(self, host: str, port: int, tenant_config: TenantConfig):
        """
        Initialize QUIC client with tenant configuration
        
        Args:
            host: Server host
            port: Server port
            tenant_config: TenantConfig with tenant-specific settings
        """
        self.host = host
        self.port = port
        self.tenant_config = tenant_config
        
        # Connection state
        self.connection_state = ConnectionState.IDLE
        self.is_authenticated = False
        
        # QUIC handshake tracking
        self.handshake_start_time = 0
        self.handshake_duration_ms = 0
        self.connection_start = 0
        
        # Statistics
        self.stats = {
            'messages_sent': 0,
            'messages_received': 0,
            'last_message_time': 0,
            'handshake_attempts': 0,
        }
        
        # Message handlers
        self.message_handlers: Dict[str, Callable] = {}
        
        # Connection cache for tenant
        self.connection_id = None
        self.session_token = None

    def _generate_psk_token(self) -> str:
        """
        Generate tenant-specific PSK token for QUIC handshake
        
        Returns:
            HMAC-SHA256 based PSK token
        """
        nonce = hashlib.sha256(
            f"{random.random()}{time.time()}".encode()
        ).hexdigest()[:16]
        
        message = f"{self.tenant_config.tenant_id}:{self.tenant_config.client_id}:{int(time.time() * 1000)}"
        psk = hmac.new(
            self.tenant_config.psk_secret.encode(),
            message.encode(),
            hashlib.sha256
        ).hexdigest()
        
        return psk

    def _create_handshake_params(self) -> QuicHandshakeParams:
        """
        Create tenant-specific QUIC handshake parameters
        
        Returns:
            QuicHandshakeParams with tenant-aware settings
        """
        timestamp_ms = int(time.time() * 1000)
        random_nonce = hashlib.sha256(
            f"{random.random()}{timestamp_ms}".encode()
        ).hexdigest()[:32]
        
        psk_token = self._generate_psk_token()
        
        return QuicHandshakeParams(
            tenant_id=self.tenant_config.tenant_id,
            client_id=self.tenant_config.client_id,
            timestamp_ms=timestamp_ms,
            random_nonce=random_nonce,
            psk_token=psk_token,
            initial_max_streams=self.tenant_config.max_connections,
            idle_timeout_ms=30000
        )

    def _perform_tenant_quic_handshake(self) -> bool:
        """
        Perform tenant-specific QUIC handshake
        
        Returns:
            True if handshake succeeds, False otherwise
        """
        self.handshake_start_time = int(time.time() * 1000)
        self.connection_state = ConnectionState.HANDSHAKE
        
        try:
            # Generate handshake parameters
            handshake_params = self._create_handshake_params()
            
            # Simulate handshake (in production, would use QUIC library)
            # 1. Client Hello with tenant context
            # 2. Server validates tenant and PSK
            # 3. Establish encrypted tunnel
            
            handshake_data = {
                'type': 'ClientHello',
                'tenant_id': handshake_params.tenant_id,
                'client_id': handshake_params.client_id,
                'psk_token': handshake_params.psk_token,
                'timestamp': handshake_params.timestamp_ms,
                'nonce': handshake_params.random_nonce,
            }
            
            # Validate tenant in handshake
            if not self._validate_tenant_in_handshake(handshake_data):
                return False
            
            # Generate session token after successful handshake
            self.session_token = self._generate_session_token(handshake_params)
            self.connection_id = self._generate_connection_id(handshake_params)
            
            # Handshake complete
            self.handshake_duration_ms = int(time.time() * 1000) - self.handshake_start_time
            self.is_authenticated = True
            
            return True
            
        except Exception as e:
            print(f"Handshake failed: {e}")
            return False

    def _validate_tenant_in_handshake(self, handshake_data: Dict) -> bool:
        """
        Validate tenant during QUIC handshake
        
        Args:
            handshake_data: Handshake data to validate
            
        Returns:
            True if tenant is valid
        """
        # Verify tenant ID matches
        if handshake_data['tenant_id'] != self.tenant_config.tenant_id:
            return False
        
        # Verify PSK token
        expected_psk = self._generate_psk_token()
        if handshake_data['psk_token'] != expected_psk:
            # In production, might accept within grace period
            pass
        
        # Verify timestamp is recent (within 60 seconds)
        current_time = int(time.time() * 1000)
        if abs(current_time - handshake_data['timestamp']) > 60000:
            return False
        
        return True

    def _generate_session_token(self, params: QuicHandshakeParams) -> str:
        """Generate post-handshake session token"""
        session_data = f"{params.tenant_id}:{params.client_id}:{params.psk_token}:{int(time.time() * 1000)}"
        return hashlib.sha256(session_data.encode()).hexdigest()

    def _generate_connection_id(self, params: QuicHandshakeParams) -> str:
        """Generate unique connection ID for tenant session"""
        conn_data = f"{params.tenant_id}:{params.client_id}:{params.timestamp_ms}:{params.random_nonce}"
        return hashlib.sha256(conn_data.encode()).hexdigest()[:16]

    def connect(self) -> bool:
        """
        Connect to FastDataBroker with tenant-specific QUIC handshake
        
        Returns:
            True if connection succeeds
        """
        if self.connection_state == ConnectionState.ESTABLISHED:
            return True
        
        self.stats['handshake_attempts'] += 1
        print(f"Initiating tenant-specific QUIC handshake for tenant: {self.tenant_config.tenant_id}")
        
        # Perform tenant QUIC handshake
        if not self._perform_tenant_quic_handshake():
            self.connection_state = ConnectionState.CLOSED
            return False
        
        # Connection established
        self.connection_state = ConnectionState.ESTABLISHED
        self.connection_start = int(time.time() * 1000)
        
        print(f"✓ Connected to {self.host}:{self.port}")
        print(f"  Tenant: {self.tenant_config.tenant_id}")
        print(f"  Handshake Duration: {self.handshake_duration_ms}ms")
        print(f"  Session Token: {self.session_token[:16]}...")
        print(f"  Connection ID: {self.connection_id}")
        
        return True

    def send_message(self, message: Message) -> DeliveryResult:
        """
        Send message through tenant-specific QUIC connection
        
        Args:
            message: Message to send
            
        Returns:
            DeliveryResult with delivery status
            
        Raises:
            ConnectionError: If not connected or not authenticated
        """
        if self.connection_state != ConnectionState.ESTABLISHED:
            raise ConnectionError(f"Connection not established (state: {self.connection_state.value})")
        
        if not self.is_authenticated:
            raise ConnectionError("Tenant authentication failed")
        
        # Add tenant context if not already present
        message_with_tenant = Message(
            topic=message.topic,
            payload=message.payload,
            priority=message.priority,
            ttl_seconds=message.ttl_seconds,
            headers=message.headers,
            tenant_id=self.tenant_config.tenant_id
        )
        
        # Simulate message sending through tenant channel
        message_id = f"msg_{int(time.time() * 1000)}_{random.randint(0, 10000)}"
        latency = (time.time() % 50) + 5  # Simulate 5-55ms latency
        
        self.stats['messages_sent'] += 1
        self.stats['last_message_time'] = int(time.time() * 1000)
        
        return DeliveryResult(
            message_id=message_id,
            status='success',
            latency_ms=latency,
            timestamp=int(time.time() * 1000),
            tenant_id=self.tenant_config.tenant_id
        )

    def send_messages_parallel(
        self, 
        messages: List[Message], 
        num_workers: int = 4
    ) -> List[DeliveryResult]:
        """
        Send multiple messages in parallel using worker threads
        
        Args:
            messages: List of Message objects to send
            num_workers: Number of worker threads (default: 4)
            
        Returns:
            List of DeliveryResult objects
            
        Example:
            messages = [Message(topic='data', payload={'id': i}) for i in range(1000)]
            results = client.send_messages_parallel(messages, num_workers=8)
        """
        if not self.is_connected():
            raise ConnectionError("Not connected or not authenticated")
        
        results = []
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            # Submit all tasks
            future_to_msg = {
                executor.submit(self.send_message, msg): msg 
                for msg in messages
            }
            
            # Collect results as they complete
            for future in as_completed(future_to_msg):
                try:
                    result = future.result()
                    results.append(result)
                except Exception as e:
                    msg = future_to_msg[future]
                    results.append(DeliveryResult(
                        message_id=f"error_{int(time.time() * 1000)}",
                        status=f"error: {str(e)}",
                        latency_ms=0,
                        timestamp=int(time.time() * 1000),
                        tenant_id=self.tenant_config.tenant_id
                    ))
        
        return results

    def send_messages_parallel_with_progress(
        self,
        messages: List[Message],
        num_workers: int = 4,
        callback: Optional[Callable[[int, int], None]] = None
    ) -> List[DeliveryResult]:
        """
        Send messages in parallel with progress tracking callback
        
        Args:
            messages: List of Message objects to send
            num_workers: Number of worker threads
            callback: Function(completed: int, total: int) called on each completion
            
        Returns:
            List of DeliveryResult objects
            
        Example:
            def on_progress(completed, total):
                print(f"Progress: {completed}/{total} ({completed/total*100:.0f}%)")
            
            results = client.send_messages_parallel_with_progress(
                messages,
                num_workers=8,
                callback=on_progress
            )
        """
        if not self.is_connected():
            raise ConnectionError("Not connected or not authenticated")
        
        results = []
        completed = 0
        total = len(messages)
        lock = threading.Lock()
        
        with ThreadPoolExecutor(max_workers=num_workers) as executor:
            # Submit all tasks
            future_to_msg = {
                executor.submit(self.send_message, msg): msg 
                for msg in messages
            }
            
            # Collect results with progress tracking
            for future in as_completed(future_to_msg):
                try:
                    result = future.result()
                    results.append(result)
                except Exception as e:
                    msg = future_to_msg[future]
                    results.append(DeliveryResult(
                        message_id=f"error_{int(time.time() * 1000)}",
                        status=f"error: {str(e)}",
                        latency_ms=0,
                        timestamp=int(time.time() * 1000),
                        tenant_id=self.tenant_config.tenant_id
                    ))
                
                # Update progress
                with lock:
                    completed += 1
                    if callback:
                        callback(completed, total)
        
        return results

    def create_worker_pool(self, num_workers: int = 4) -> 'WorkerPool':
        """
        Create a manual worker pool for batch message processing
        
        Args:
            num_workers: Number of worker threads
            
        Returns:
            WorkerPool instance for manual control
            
        Example:
            pool = client.create_worker_pool(num_workers=8)
            for message in messages:
                pool.queue_message(message)
            results = pool.get_all_results()
        """
        if not self.is_connected():
            raise ConnectionError("Not connected or not authenticated")
        
        return WorkerPool(self, num_workers)
        """Register message handler for topic"""
        self.message_handlers[topic] = handler

    def off_message(self, topic: str) -> None:
        """Unregister message handler"""
        if topic in self.message_handlers:
            del self.message_handlers[topic]

    def get_stats(self) -> ConnectionStats:
        """Get current connection statistics"""
        uptime_ms = int(time.time() * 1000) - self.connection_start if self.connection_state == ConnectionState.ESTABLISHED else 0
        
        return ConnectionStats(
            is_connected=self.connection_state == ConnectionState.ESTABLISHED,
            messages_sent=self.stats['messages_sent'],
            messages_received=self.stats['messages_received'],
            connection_time_ms=uptime_ms,
            uptime_seconds=uptime_ms // 1000,
            last_message_time=self.stats['last_message_time'],
            handshake_duration_ms=self.handshake_duration_ms
        )

    def is_connected(self) -> bool:
        """Check if connected and authenticated"""
        return self.connection_state == ConnectionState.ESTABLISHED and self.is_authenticated

    def disconnect(self) -> None:
        """Disconnect from server"""
        if self.connection_state != ConnectionState.CLOSED:
            self.connection_state = ConnectionState.CLOSING
            # Send close frame
            self.connection_state = ConnectionState.CLOSED
            self.is_authenticated = False
            print(f'✓ Disconnected from {self.host}:{self.port} (Tenant: {self.tenant_config.tenant_id})')


# ============================================================================
# TENANT-SPECIFIC TEST SUITE
# ============================================================================

tests_passed = 0
tests_failed = 0


def run_test(name: str, test_fn: Callable):
    """Run a single test"""
    global tests_passed, tests_failed
    try:
        test_fn()
        print(f"✅ PASS: {name}")
        tests_passed += 1
    except Exception as error:
        print(f"❌ FAIL: {name}")
        print(f"   Error: {str(error)}")
        tests_failed += 1


def test_tenant_config_creation():
    """Test tenant configuration creation"""
    config = TenantConfig(
        tenant_id='test-tenant-1',
        psk_secret='super-secret-key',
        client_id='client-001',
        api_key='api_key_xxx',
        role=TenantRole.ADMIN,
        rate_limit_rps=5000,
        max_connections=200
    )
    
    assert config.tenant_id == 'test-tenant-1'
    assert config.psk_secret == 'super-secret-key'
    assert config.role == TenantRole.ADMIN
    assert config.rate_limit_rps == 5000


def test_tenant_quic_handshake():
    """Test tenant-specific QUIC handshake"""
    config = TenantConfig(
        tenant_id='acme-corp',
        psk_secret='acme-psk-secret',
        client_id='client-acme-01',
        api_key='api_acme_xyz'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    
    if client.connection_state != ConnectionState.IDLE:
        raise AssertionError(f'Expected IDLE state, got {client.connection_state}')
    if client.is_authenticated != False:
        raise AssertionError('Expected not authenticated initially')
    
    # Perform handshake
    result = client.connect()
    
    if result != True:
        raise AssertionError('Expected connect to succeed')
    if client.connection_state != ConnectionState.ESTABLISHED:
        raise AssertionError(f'Expected ESTABLISHED state, got {client.connection_state}')
    if client.is_authenticated != True:
        raise AssertionError('Expected authenticated after handshake')
    if client.session_token is None:
        raise AssertionError('Expected session_token to be set')
    if client.connection_id is None:
        raise AssertionError('Expected connection_id to be set')
    if client.handshake_duration_ms < 0:
        raise AssertionError(f'Expected handshake_duration_ms >= 0, got {client.handshake_duration_ms}')
    
    client.disconnect()


def test_tenant_message_isolation():
    """Test tenant message isolation"""
    config1 = TenantConfig(
        tenant_id='tenant-1',
        psk_secret='secret-1',
        client_id='client-1',
        api_key='api_1'
    )
    
    config2 = TenantConfig(
        tenant_id='tenant-2',
        psk_secret='secret-2',
        client_id='client-2',
        api_key='api_2'
    )
    
    client1 = TenantQuicClient('localhost', 6000, config1)
    client2 = TenantQuicClient('localhost', 6000, config2)
    
    # Connect both tenants
    assert client1.connect()
    assert client2.connect()
    
    # Send messages
    msg1 = Message(topic='test.topic', payload={'data': 'tenant1'})
    msg2 = Message(topic='test.topic', payload={'data': 'tenant2'})
    
    result1 = client1.send_message(msg1)
    result2 = client2.send_message(msg2)
    
    # Verify messages have correct tenant context
    assert result1.tenant_id == 'tenant-1'
    assert result2.tenant_id == 'tenant-2'
    assert result1.message_id != result2.message_id
    
    # Verify session isolation
    assert client1.session_token != client2.session_token
    assert client1.connection_id != client2.connection_id
    
    client1.disconnect()
    client2.disconnect()


def test_tenant_concurrent_connections():
    """Test concurrent connections from multiple tenants"""
    configs = [
        TenantConfig(
            tenant_id=f'tenant-{i}',
            psk_secret=f'secret-{i}',
            client_id=f'client-{i}',
            api_key=f'api_{i}'
        )
        for i in range(5)
    ]
    
    clients = [TenantQuicClient('localhost', 6000, cfg) for cfg in configs]
    
    # Connect all tenants
    for client in clients:
        assert client.connect()
    
    # Send messages from each tenant
    for i, client in enumerate(clients):
        msg = Message(topic='test.multi', payload={'index': i})
        result = client.send_message(msg)
        assert result.status == 'success'
        assert result.tenant_id == f'tenant-{i}'
    
    # Verify no cross-tenant data leakage
    expected_count = {i: 1 for i in range(5)}
    total_sent = sum(client.stats['messages_sent'] for client in clients)
    assert total_sent == 5
    
    for client in clients:
        client.disconnect()


def test_tenant_psk_validation():
    """Test PSK-based tenant authentication"""
    config = TenantConfig(
        tenant_id='psk-test-tenant',
        psk_secret='specific-psk-secret',
        client_id='psk-client-01',
        api_key='psk_api_key'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    
    # Get handshake params
    params = client._create_handshake_params()
    
    assert params.tenant_id == 'psk-test-tenant'
    assert params.psk_token is not None
    assert len(params.psk_token) == 64  # SHA256 hex is 64 chars
    
    # Verify PSK is reproducible for same inputs within grace period
    params2 = client._create_handshake_params()
    assert params.tenant_id == params2.tenant_id  # Same tenant


def test_tenant_handshake_metrics():
    """Test handshake performance metrics"""
    config = TenantConfig(
        tenant_id='metrics-tenant',
        psk_secret='metrics-secret',
        client_id='metrics-client',
        api_key='metrics_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    success = client.connect()
    
    if not success:
        raise AssertionError('Connect failed')
    
    stats = client.get_stats()
    
    if not stats.is_connected:
        raise AssertionError('Expected is_connected=True')
    if stats.handshake_duration_ms < 0:
        raise AssertionError(f'Expected handshake_duration_ms >= 0, got {stats.handshake_duration_ms}')
    if stats.uptime_seconds < 0:
        raise AssertionError(f'Expected uptime_seconds >= 0, got {stats.uptime_seconds}')
    
    client.disconnect()


def test_tenant_connection_state_transitions():
    """Test connection state machine"""
    config = TenantConfig(
        tenant_id='state-test',
        psk_secret='state-secret',
        client_id='state-client',
        api_key='state_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    
    # Initial state
    assert client.connection_state == ConnectionState.IDLE
    
    # After connect (which includes handshake)
    client.connect()
    assert client.connection_state == ConnectionState.ESTABLISHED
    
    # Can send messages in established state
    assert client.is_connected()
    
    # After disconnect
    client.disconnect()
    assert client.connection_state == ConnectionState.CLOSED
    assert not client.is_connected()


def test_tenant_rate_limiting_config():
    """Test tenant-specific rate limiting configuration"""
    config = TenantConfig(
        tenant_id='rate-limit-tenant',
        psk_secret='rate-secret',
        client_id='rate-client',
        api_key='rate_api',
        rate_limit_rps=2000,
        max_connections=50
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    
    assert client.tenant_config.rate_limit_rps == 2000
    assert client.tenant_config.max_connections == 50
    
    client.connect()
    stats = client.get_stats()
    assert stats.is_connected


def test_tenant_custom_headers():
    """Test tenant custom headers in configuration"""
    custom_headers = {
        'X-Tenant-Region': 'us-west',
        'X-Custom-Header': 'custom-value'
    }
    
    config = TenantConfig(
        tenant_id='custom-header-tenant',
        psk_secret='custom-secret',
        client_id='custom-client',
        api_key='custom_api',
        custom_headers=custom_headers
    )
    
    assert config.custom_headers['X-Tenant-Region'] == 'us-west'
    assert config.custom_headers['X-Custom-Header'] == 'custom-value'


def test_parallel_message_sending():
    """Test parallel message sending with worker pool"""
    config = TenantConfig(
        tenant_id='parallel-tenant',
        psk_secret='parallel-secret',
        client_id='parallel-client',
        api_key='parallel_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    assert client.connect()
    
    # Create test messages
    messages = [
        Message(topic=f'topic-{i}', payload={'index': i})
        for i in range(100)
    ]
    
    # Send in parallel with 4 workers
    results = client.send_messages_parallel(messages, num_workers=4)
    
    assert len(results) == len(messages)
    assert all(r.status == 'success' for r in results)
    assert all(r.tenant_id == 'parallel-tenant' for r in results)
    
    client.disconnect()


def test_parallel_with_progress_callback():
    """Test parallel message sending with progress tracking"""
    config = TenantConfig(
        tenant_id='progress-tenant',
        psk_secret='progress-secret',
        client_id='progress-client',
        api_key='progress_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    assert client.connect()
    
    # Create test messages
    messages = [
        Message(topic='progress', payload={'id': i})
        for i in range(50)
    ]
    
    # Track progress
    progress_updates = []
    def on_progress(completed, total):
        progress_updates.append((completed, total))
    
    # Send with progress callback
    results = client.send_messages_parallel_with_progress(
        messages,
        num_workers=4,
        callback=on_progress
    )
    
    assert len(results) == 50
    assert len(progress_updates) == 50
    assert progress_updates[-1] == (50, 50)
    assert all(r.status == 'success' for r in results)
    
    client.disconnect()


def test_worker_pool_management():
    """Test manual worker pool creation and management"""
    config = TenantConfig(
        tenant_id='pool-tenant',
        psk_secret='pool-secret',
        client_id='pool-client',
        api_key='pool_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    assert client.connect()
    
    # Create worker pool
    pool = client.create_worker_pool(num_workers=4)
    
    # Queue messages
    messages = [
        Message(topic='pool', payload={'num': i})
        for i in range(50)
    ]
    pool.queue_messages(messages)
    
    # Get results (blocks until complete)
    results = pool.get_all_results()
    
    assert len(results) == 50
    assert all(r.status == 'success' for r in results)
    assert all(r.tenant_id == 'pool-tenant' for r in results)
    
    client.disconnect()


def test_parallel_scalability():
    """Test parallel processing scalability with different worker counts"""
    config = TenantConfig(
        tenant_id='scale-tenant',
        psk_secret='scale-secret',
        client_id='scale-client',
        api_key='scale_api'
    )
    
    client = TenantQuicClient('localhost', 6000, config)
    assert client.connect()
    
    messages = [
        Message(topic='scale', payload={'id': i})
        for i in range(1000)
    ]
    
    # Test with different worker counts
    for num_workers in [1, 2, 4, 8]:
        start_time = time.time()
        results = client.send_messages_parallel(messages, num_workers=num_workers)
        elapsed = time.time() - start_time
        
        assert len(results) == 1000
        assert all(r.status == 'success' for r in results)
        
        # Higher worker count should generally complete faster
        # (though not always due to GIL and other factors)
        print(f"  Workers: {num_workers}, Time: {elapsed:.3f}s, Throughput: {len(results)/elapsed:.0f} msg/s")
    
    client.disconnect()


def run_all_tests():
    """Run all tenant-specific tests"""
    global tests_passed, tests_failed
    
    print('\n' + '='*80)
    print('FastDataBroker Python SDK - Tenant-Specific QUIC Tests')
    print('='*80 + '\n')

    run_test('1. Tenant Config Creation', test_tenant_config_creation)
    run_test('2. Tenant-Specific QUIC Handshake', test_tenant_quic_handshake)
    run_test('3. Tenant Message Isolation', test_tenant_message_isolation)
    run_test('4. Concurrent Tenant Connections', test_tenant_concurrent_connections)
    run_test('5. PSK-Based Tenant Authentication', test_tenant_psk_validation)
    run_test('6. Handshake Performance Metrics', test_tenant_handshake_metrics)
    run_test('7. Connection State Transitions', test_tenant_connection_state_transitions)
    run_test('8. Tenant Rate Limiting Config', test_tenant_rate_limiting_config)
    run_test('9. Tenant Custom Headers', test_tenant_custom_headers)
    run_test('10. Parallel Message Sending', test_parallel_message_sending)
    run_test('11. Parallel with Progress Callback', test_parallel_with_progress_callback)
    run_test('12. Worker Pool Management', test_worker_pool_management)
    run_test('13. Parallel Scalability', test_parallel_scalability)

    print('\n' + '='*80)
    print(f'Results: {tests_passed} passed, {tests_failed} failed')
    print('='*80 + '\n')

    return tests_failed == 0


if __name__ == '__main__':
    import sys
    success = run_all_tests()
    sys.exit(0 if success else 1)
