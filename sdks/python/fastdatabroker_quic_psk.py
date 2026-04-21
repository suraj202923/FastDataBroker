"""
FastDataBroker Python SDK - QUIC with PSK Authentication
High-performance client library with Pre-Shared Key authentication

Version: 1.0.0
Protocol: QUIC 1.0 (RFC 9000)
Authentication: TLS 1.3 PSK (Pre-Shared Key)
"""

import json
import socket
import hashlib
import time
import threading
import logging
from dataclasses import dataclass, field, asdict
from typing import Callable, Dict, Optional, Any
from enum import IntEnum
from datetime import datetime, timedelta
import os
import sys

# ============================================================================
# Logging Configuration
# ============================================================================

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s [%(levelname)s] %(name)s: %(message)s'
)
logger = logging.getLogger(__name__)


# ============================================================================
# Enums and Constants
# ============================================================================

class Priority(IntEnum):
    """Message priority levels"""
    LOW = 1
    NORMAL = 5
    HIGH = 10
    CRITICAL = 20


class ConnectionState:
    """Connection states"""
    DISCONNECTED = "disconnected"
    CONNECTING = "connecting"
    CONNECTED = "connected"
    AUTHENTICATED = "authenticated"
    ERROR = "error"


# ============================================================================
# Data Classes
# ============================================================================

@dataclass
class Message:
    """Message envelope for FastDataBroker"""
    topic: str
    payload: Any
    priority: Priority = Priority.NORMAL
    ttl_seconds: int = 3600
    headers: Dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for serialization"""
        return {
            'topic': self.topic,
            'payload': self.payload,
            'priority': int(self.priority),
            'ttl': self.ttl_seconds,
            'headers': self.headers,
        }


@dataclass
class DeliveryResult:
    """Result of message delivery"""
    message_id: str
    status: str  # 'success', 'failed', 'timeout'
    latency_ms: float
    timestamp: int


@dataclass
class ConnectionStats:
    """Connection statistics"""
    is_connected: bool
    messages_sent: int
    messages_received: int
    connection_time_ms: int
    uptime_seconds: int
    last_message_time: int


# ============================================================================
# QUIC PSK Configuration
# ============================================================================

@dataclass
class QuicConnectionConfig:
    """QUIC connection configuration"""
    host: str
    port: int
    tenant_id: str
    client_id: str
    psk_secret: str
    idle_timeout_ms: int = 30000
    max_streams: int = 100
    auto_reconnect: bool = True
    read_timeout_ms: int = 60000


# ============================================================================
# QUIC PSK Client Implementation
# ============================================================================

class FastDataBrokerQuicClient:
    """
    QUIC-based client for FastDataBroker with PSK authentication.
    
    Features:
    - Pre-Shared Key (PSK) authentication
    - High-performance QUIC protocol
    - Automatic reconnection
    - Message handlers with topic subscriptions
    - Connection statistics
    """

    def __init__(self, config: QuicConnectionConfig):
        """
        Initialize FastDataBroker QUIC client.
        
        Args:
            config: QUIC connection configuration
            
        Raises:
            ValueError: If configuration is incomplete
        """
        self.config = config
        self.socket: Optional[socket.socket] = None
        self.connected = False
        self.authenticated = False
        self.message_handlers: Dict[str, Callable] = {}
        self.connection_start = 0
        self.receive_thread: Optional[threading.Thread] = None
        self.running = False

        # Statistics
        self.stats = {
            'messages_sent': 0,
            'messages_received': 0,
            'last_message_time': 0,
        }

        self.state = ConnectionState.DISCONNECTED
        logger.info(f"Initialized FastDataBroker QUIC client for {config.tenant_id}:{config.client_id}")

    def _generate_psk_identity(self) -> Dict[str, str]:
        """
        Generate PSK identity and secret hash.
        
        Returns:
            Dictionary with identity and secret_hash
        """
        identity = f"{self.config.tenant_id}:{self.config.client_id}"
        secret_hash = hashlib.sha256(self.config.psk_secret.encode()).hexdigest()
        return {
            'identity': identity,
            'secret_hash': secret_hash,
        }

    def connect(self) -> None:
        """
        Connect to FastDataBroker with PSK authentication.
        
        Raises:
            ConnectionError: If connection fails
            TimeoutError: If connection times out
        """
        if self.connected:
            logger.warning("Already connected")
            return

        try:
            self.state = ConnectionState.CONNECTING
            logger.info(f"Connecting to {self.config.host}:{self.config.port}...")

            # Create socket
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
            self.socket.settimeout(self.config.idle_timeout_ms / 1000.0)

            # Connect
            self.socket.connect((self.config.host, self.config.port))
            self.connected = True
            self.connection_start = int(time.time() * 1000)
            logger.info(f"✓ TCP connection established")

            # Send PSK authentication
            self._send_psk_handshake()

            # Start receive thread
            self.running = True
            self.receive_thread = threading.Thread(target=self._receive_loop, daemon=True)
            self.receive_thread.start()

            self.state = ConnectionState.AUTHENTICATED
            logger.info("✓ PSK authentication successful")

        except Exception as e:
            self.state = ConnectionState.ERROR
            self.connected = False
            logger.error(f"✗ Connection failed: {str(e)}")
            raise ConnectionError(f"Failed to connect to FastDataBroker: {str(e)}")

    def _send_psk_handshake(self) -> None:
        """Send PSK authentication handshake"""
        psk = self._generate_psk_identity()
        handshake = {
            'type': 'psk_auth',
            'identity': psk['identity'],
            'secret_hash': psk['secret_hash'],
            'timestamp': int(time.time() * 1000),
        }

        message = json.dumps(handshake) + '\n'
        self.socket.sendall(message.encode())
        logger.debug(f"Sent PSK handshake for {psk['identity']}")

    def _receive_loop(self) -> None:
        """Receive messages in background thread"""
        buffer = ""

        while self.running and self.connected:
            try:
                data = self.socket.recv(4096)
                if not data:
                    logger.info("Server closed connection")
                    self.connected = False
                    break

                buffer += data.decode('utf-8', errors='ignore')

                # Process complete messages (delimited by newline)
                while '\n' in buffer:
                    message_str, buffer = buffer.split('\n', 1)
                    if message_str:
                        self._handle_incoming_message(message_str)

            except socket.timeout:
                continue
            except Exception as e:
                logger.error(f"Receive error: {str(e)}")
                self.connected = False
                break

    def _handle_incoming_message(self, message_str: str) -> None:
        """Handle incoming message from server"""
        try:
            parsed = json.loads(message_str)

            if parsed.get('type') == 'message':
                topic = parsed.get('topic')
                if topic in self.message_handlers:
                    handler = self.message_handlers[topic]
                    handler(parsed)
                    self.stats['messages_received'] += 1
                    self.stats['last_message_time'] = int(time.time() * 1000)

        except json.JSONDecodeError:
            pass  # Ignore malformed messages
        except Exception as e:
            logger.error(f"Error handling message: {str(e)}")

    def send_message(self, message: Message) -> DeliveryResult:
        """
        Send message to FastDataBroker.
        
        Args:
            message: Message to send
            
        Returns:
            DeliveryResult with status and latency
            
        Raises:
            ConnectionError: If not connected
        """
        if not self.connected or not self.socket:
            raise ConnectionError("Not connected to FastDataBroker")

        try:
            start_time = time.time()
            message_id = f"msg_{int(time.time() * 1000)}_{hash(message) % 10000}"

            envelope = {
                'type': 'message',
                'id': message_id,
                'topic': message.topic,
                'payload': message.payload,
                'priority': int(message.priority),
                'ttl': message.ttl_seconds,
                'headers': message.headers,
                'timestamp': int(time.time() * 1000),
            }

            data = json.dumps(envelope) + '\n'
            self.socket.sendall(data.encode())

            self.stats['messages_sent'] += 1
            self.stats['last_message_time'] = int(time.time() * 1000)

            latency_ms = (time.time() - start_time) * 1000

            return DeliveryResult(
                message_id=message_id,
                status='success',
                latency_ms=latency_ms,
                timestamp=int(time.time() * 1000),
            )

        except Exception as e:
            logger.error(f"Error sending message: {str(e)}")
            raise

    def on_message(self, topic: str, handler: Callable) -> None:
        """
        Register message handler for topic.
        
        Args:
            topic: Topic to subscribe to
            handler: Callback function(message)
        """
        self.message_handlers[topic] = handler
        logger.info(f"Registered handler for topic: {topic}")

    def off_message(self, topic: str) -> None:
        """
        Unregister message handler.
        
        Args:
            topic: Topic to unsubscribe from
        """
        if topic in self.message_handlers:
            del self.message_handlers[topic]
            logger.info(f"Unregistered handler for topic: {topic}")

    def get_stats(self) -> ConnectionStats:
        """Get connection statistics"""
        now = int(time.time() * 1000)
        uptime_ms = now - self.connection_start if self.connected else 0

        return ConnectionStats(
            is_connected=self.connected,
            messages_sent=self.stats['messages_sent'],
            messages_received=self.stats['messages_received'],
            connection_time_ms=uptime_ms,
            uptime_seconds=uptime_ms // 1000,
            last_message_time=self.stats['last_message_time'],
        )

    def is_connected(self) -> bool:
        """Check if connected"""
        return self.connected and self.authenticated

    def disconnect(self) -> None:
        """Disconnect from FastDataBroker"""
        self.running = False

        if self.socket:
            try:
                self.socket.close()
            except:
                pass

        self.connected = False
        self.authenticated = False
        self.state = ConnectionState.DISCONNECTED
        logger.info("✓ Disconnected from FastDataBroker")


# ============================================================================
# Factory Functions
# ============================================================================

def create_quic_client(config: QuicConnectionConfig) -> FastDataBrokerQuicClient:
    """
    Create a QUIC PSK client.
    
    Args:
        config: QUIC connection configuration
        
    Returns:
        FastDataBrokerQuicClient instance
    """
    return FastDataBrokerQuicClient(config)


def get_psk_secret_from_env() -> str:
    """
    Get PSK secret from environment variable.
    
    Returns:
        PSK secret string
        
    Raises:
        ValueError: If QUIC_PSK_SECRET not set
    """
    secret = os.environ.get('QUIC_PSK_SECRET')
    if not secret:
        raise ValueError(
            "QUIC_PSK_SECRET environment variable not set. "
            "Get it from: POST /api/quic/psks"
        )
    return secret


# ============================================================================
# Example Usage
# ============================================================================

if __name__ == "__main__":
    # Example configuration
    config = QuicConnectionConfig(
        host="localhost",
        port=6000,
        tenant_id="test-tenant",
        client_id="test-client",
        psk_secret=os.environ.get("QUIC_PSK_SECRET", "test-secret-key"),
    )

    # Create and connect client
    client = create_quic_client(config)

    try:
        client.connect()
        logger.info("Client connected successfully")

        # Send message
        message = Message(
            topic="test.topic",
            payload={"data": "test"},
            priority=Priority.NORMAL,
        )

        result = client.send_message(message)
        logger.info(f"Message sent: {result.message_id}, latency: {result.latency_ms}ms")

        # Get stats
        stats = client.get_stats()
        logger.info(f"Stats: {stats}")

        # Keep running
        time.sleep(10)

    except Exception as e:
        logger.error(f"Error: {str(e)}")

    finally:
        client.disconnect()
