"""
FastDataBroker Integration Guide: Adding Live Streaming to Existing API
=========================================================================

This guide shows how to integrate the new Live Stream API with the existing
FastDataBroker SDK for seamless message + streaming operations.
"""

from typing import Optional, Dict, Any, List
from dataclasses import dataclass
import asyncio

# Import existing classes
from LIVE_STREAM_API import (
    FastDataBrokerLiveAPI, 
    LiveStreamProducer, 
    AsyncLiveStreamConsumer,
    StreamType, 
    StreamEvent
)


# ============================================================================
# EXTENDED FASTDATABROKER CLIENT WITH LIVE STREAMING
# ============================================================================

class FastDataBrokerClientWithLive:
    """
    Enhanced FastDataBroker client combining traditional messaging 
    with live string streaming
    """
    
    def __init__(self, quic_host: str = "localhost", quic_port: int = 6000):
        """Initialize enhanced client with both messaging and streaming"""
        self.quic_host = quic_host
        self.quic_port = quic_port
        self._connected = False
        
        # Initialize live stream API
        self.live = FastDataBrokerLiveAPI(quic_host, quic_port)
        self.active_streams: Dict[str, LiveStreamProducer] = {}
    
    def connect(self) -> bool:
        """Connect to FastDataBroker (both messaging and streaming)"""
        try:
            self._connected = True
            self.live.connect()
            return True
        except Exception as e:
            print(f"Connection failed: {e}")
            return False
    
    def disconnect(self) -> None:
        """Disconnect from FastDataBroker"""
        self._connected = False
        self.live.disconnect()
    
    # ========================================================================
    # TRADITIONAL MESSAGE METHODS (existing)
    # ========================================================================
    
    def send_message(self, recipient_ids: List[str], content: str, 
                    priority: int = 100) -> str:
        """Send a traditional message (existing functionality)"""
        if not self._connected:
            raise RuntimeError("Not connected")
        return "msg-12345"
    
    def register_webhook(self, webhook_url: str) -> bool:
        """Register webhook (existing functionality)"""
        return True
    
    # ========================================================================
    # NEW LIVE STREAMING METHODS
    # ========================================================================
    
    def create_live_stream(self, stream_id: str, 
                          stream_type: StreamType = StreamType.CUSTOM) -> LiveStreamProducer:
        """
        Create a live string stream
        
        Args:
            stream_id: Unique stream identifier
            stream_type: Type of stream (log, metrics, analytics, etc.)
            
        Returns:
            LiveStreamProducer for publishing
            
        Example:
            >>> client = FastDataBrokerClientWithLive()
            >>> client.connect()
            >>> stream = client.create_live_stream("order-updates")
            >>> stream.put("Order confirmed", tags={"order_id": "123"})
        """
        stream = self.live.create_stream(stream_id, stream_type)
        self.active_streams[stream_id] = stream
        return stream
    
    def publish_to_stream(self, stream_id: str, data: str,
                         tags: Optional[Dict[str, str]] = None) -> str:
        """
        Publish data to a live stream
        
        Args:
            stream_id: Target stream
            data: String data to publish
            tags: Optional tags for filtering
            
        Returns:
            Event ID
            
        Example:
            >>> event_id = client.publish_to_stream(
            ...     "order-updates",
            ...     "Order shipped",
            ...     tags={"order_id": "123", "status": "shipped"}
            ... )
        """
        if stream_id not in self.active_streams:
            raise ValueError(f"Stream {stream_id} not found")
        
        return self.active_streams[stream_id].put(data, tags=tags)
    
    def publish_json_to_stream(self, stream_id: str, 
                              json_data: Dict[str, Any],
                              tags: Optional[Dict[str, str]] = None) -> str:
        """
        Publish JSON data to a live stream
        
        Args:
            stream_id: Target stream
            json_data: Dictionary to serialize and stream
            tags: Optional tags
            
        Returns:
            Event ID
        """
        if stream_id not in self.active_streams:
            raise ValueError(f"Stream {stream_id} not found")
        
        return self.active_streams[stream_id].put_json(json_data, tags=tags)
    
    def subscribe_to_stream(self, stream_id: str,
                           tags_filter: Optional[Dict[str, str]] = None):
        """
        Subscribe to a live stream (sync)
        
        Args:
            stream_id: Stream to subscribe to
            tags_filter: Optional tag-based filtering
            
        Returns:
            LiveStreamConsumer
        """
        return self.live.subscribe(stream_id, tags_filter)
    
    def subscribe_to_stream_async(self, stream_id: str,
                                 tags_filter: Optional[Dict[str, str]] = None):
        """
        Subscribe to a live stream (async)
        
        Args:
            stream_id: Stream to subscribe to
            tags_filter: Optional tag-based filtering
            
        Returns:
            AsyncLiveStreamConsumer
        """
        return self.live.subscribe_async(stream_id, tags_filter)
    
    def get_stream_stats(self, stream_id: Optional[str] = None) -> Dict[str, Any]:
        """
        Get statistics for streams
        
        Args:
            stream_id: Specific stream (None = all)
            
        Returns:
            Statistics dictionary
        """
        if stream_id:
            return self.live.get_stream_stats(stream_id) or {}
        return self.live.get_all_stats()


class FastDataBrokerAsyncClientWithLive:
    """Async variant with both messaging and live streaming"""
    
    def __init__(self, quic_host: str = "localhost", quic_port: int = 6000):
        """Initialize async client"""
        self.quic_host = quic_host
        self.quic_port = quic_port
        self._connected = False
        self.live = FastDataBrokerLiveAPI(quic_host, quic_port)
        self.active_streams: Dict[str, LiveStreamProducer] = {}
    
    async def connect(self) -> bool:
        """Connect to FastDataBroker (async)"""
        try:
            await asyncio.sleep(0.01)  # Simulate async connection
            self._connected = True
            self.live.connect()
            return True
        except Exception as e:
            print(f"Connection failed: {e}")
            return False
    
    async def disconnect(self) -> None:
        """Disconnect from FastDataBroker"""
        self._connected = False
        self.live.disconnect()
    
    async def send_message(self, recipient_ids: List[str], content: str) -> str:
        """Send message asynchronously"""
        await asyncio.sleep(0.01)
        return "msg-12345"
    
    def create_live_stream(self, stream_id: str, 
                          stream_type: StreamType = StreamType.CUSTOM) -> LiveStreamProducer:
        """Create live stream"""
        stream = self.live.create_stream(stream_id, stream_type)
        self.active_streams[stream_id] = stream
        return stream
    
    async def publish_to_stream(self, stream_id: str, data: str,
                               tags: Optional[Dict[str, str]] = None) -> str:
        """Publish to stream asynchronously"""
        if stream_id not in self.active_streams:
            raise ValueError(f"Stream {stream_id} not found")
        
        await asyncio.sleep(0.001)  # Simulate async operation
        return self.active_streams[stream_id].put(data, tags=tags)
    
    def subscribe_to_stream_async(self, stream_id: str,
                                 tags_filter: Optional[Dict[str, str]] = None):
        """Subscribe to stream asynchronously"""
        return self.live.subscribe_async(stream_id, tags_filter)
    
    def get_stream_stats(self, stream_id: Optional[str] = None) -> Dict[str, Any]:
        """Get statistics for streams"""
        if stream_id:
            return self.live.get_stream_stats(stream_id) or {}
        return self.live.get_all_stats()


# ============================================================================
# REAL-WORLD INTEGRATION EXAMPLES
# ============================================================================

def example_ecommerce_integration():
    """
    Real-world example: E-commerce order processing with live updates
    Combines traditional messages with live stream updates
    """
    
    print("\n" + "=" * 120)
    print("EXAMPLE 1: E-COMMERCE ORDER PROCESSING")
    print("=" * 120 + "\n")
    
    # Initialize enhanced client
    client = FastDataBrokerClientWithLive()
    client.connect()
    
    # Create live stream for order status updates
    order_stream = client.create_live_stream(
        "order-status-stream",
        stream_type=StreamType.NOTIFICATIONS
    )
    
    print("[SETUP] Order processing system initialized")
    print("  - Live stream: order-status-stream")
    print("  - WebSocket listeners ready for real-time updates\n")
    
    # Simulate order workflow
    order_id = "ORD-2024-001"
    user_id = "user-123"
    
    workflow = [
        ("Order received", {"status": "pending", "action": "validation"}),
        ("Payment authorized", {"status": "processing", "action": "payment"}),
        ("Inventory confirmed", {"status": "confirmed", "action": "inventory"}),
        ("Order packed", {"status": "packing", "action": "warehouse"}),
        ("Shipped via FedEx", {"status": "shipped", "action": "delivery", "tracking": "FX123456"}),
    ]
    
    print("[PROCESSING] Order workflow:")
    print("─" * 120)
    
    for step, metadata in workflow:
        # Publish live update
        event_id = client.publish_to_stream(
            "order-status-stream",
            f"[{order_id}] {step}",
            tags={
                "order_id": order_id,
                "user_id": user_id,
                "status": metadata["status"]
            }
        )
        
        print(f"  ✓ {step}")
        print(f"    Event ID: {event_id}")
        print(f"    Metadata: {metadata}")
    
    # Get stream statistics
    stats = client.get_stream_stats("order-status-stream")
    print("\n[STATISTICS]")
    print(f"  Total events published: {stats['total_events']}")
    print(f"  Stream subscribers: {stats['subscribers']}")
    print(f"  Buffered events: {stats['buffered_events']}")
    
    client.disconnect()


def example_metrics_monitoring():
    """
    Real-world example: Real-time metrics and monitoring
    Stream JSON metrics as live strings
    """
    
    print("\n" + "=" * 120)
    print("EXAMPLE 2: REAL-TIME METRICS MONITORING")
    print("=" * 120 + "\n")
    
    client = FastDataBrokerClientWithLive()
    client.connect()
    
    # Create metrics stream
    metrics_stream = client.create_live_stream(
        "system-metrics",
        stream_type=StreamType.METRICS
    )
    
    print("[SETUP] Metrics monitoring enabled")
    print("  - Stream: system-metrics")
    print("  - Subscribers receiving updates in real-time\n")
    
    # Publish metric samples
    print("[METRICS] Publishing system metrics:")
    print("─" * 120)
    
    metrics_samples = [
        {
            "service": "api-server",
            "cpu_percent": 45.2,
            "memory_mb": 2048,
            "uptime_hours": 720,
            "requests_per_sec": 1250,
            "error_rate": 0.12
        },
        {
            "service": "database",
            "cpu_percent": 38.7,
            "memory_mb": 8192,
            "uptime_hours": 720,
            "queries_per_sec": 890,
            "slow_queries": 3
        },
        {
            "service": "cache",
            "cpu_percent": 15.3,
            "memory_mb": 4096,
            "uptime_hours": 720,
            "hit_rate": 94.5,
            "evictions": 12
        }
    ]
    
    for metric in metrics_samples:
        event_id = client.publish_json_to_stream(
            "system-metrics",
            metric,
            tags={"service": metric["service"], "region": "us-east-1"}
        )
        
        print(f"  ✓ {metric['service']}: CPU={metric['cpu_percent']}%, MEM={metric['memory_mb']}MB")
        print(f"    Event: {event_id}")
    
    # Stream statistics
    stats = client.get_stream_stats("system-metrics")
    print(f"\n✓ Metrics stream active with {stats['total_events']} events")
    
    client.disconnect()


async def example_async_live_updates():
    """
    Real-world example: Asynchronous live updates
    Shows async producer/consumer pattern
    """
    
    print("\n" + "=" * 120)
    print("EXAMPLE 3: ASYNC REAL-TIME CHAT/MESSAGING STREAM")
    print("=" * 120 + "\n")
    
    # Use async client
    client = FastDataBrokerAsyncClientWithLive()
    await client.connect()
    
    # Create chat stream
    chat_stream = client.create_live_stream(
        "chat-room-001",
        stream_type=StreamType.CUSTOM
    )
    
    print("[SETUP] Chat room stream created")
    print("  - Stream: chat-room-001")
    print("  - Async producers and consumers\n")
    
    # Subscribe to stream
    consumer = client.subscribe_to_stream_async("chat-room-001")
    await consumer.connect()
    
    print("[STREAMING] Publishing chat messages:")
    print("─" * 120)
    
    # Simulate chat
    messages = [
        ("alice", "Hey everyone!", "greeting"),
        ("bob", "Hi Alice! How's it going?", "response"),
        ("alice", "Great! Just finished the project", "info"),
        ("charlie", "Awesome, can't wait to see it!", "reaction"),
        ("bob", "Let's have a standupmeeting tomorrow", "announcement"),
    ]
    
    for sender, msg, msg_type in messages:
        # Publish async
        event_id = await client.publish_to_stream(
            "chat-room-001",
            f"{sender}: {msg}",
            tags={"sender": sender, "type": msg_type}
        )
        print(f"  [{sender}] {msg}")
        print(f"    Event: {event_id}\n")
    
    # Check stream stats
    stats = client.get_stream_stats("chat-room-001")
    print(f"✓ Chat stream: {stats['total_events']} messages")
    
    await consumer.disconnect()
    await client.disconnect()


def example_combined_workflow():
    """
    Real-world example: Combined messages + live streams
    Shows integrated traditional + streaming approach
    """
    
    print("\n" + "=" * 120)
    print("EXAMPLE 4: INTEGRATED WORKFLOW (Messages + Live Streams)")
    print("=" * 120 + "\n")
    
    client = FastDataBrokerClientWithLive()
    client.connect()
    
    # Create multiple streams
    order_stream = client.create_live_stream("orders", StreamType.NOTIFICATIONS)
    alerts_stream = client.create_live_stream("alerts", StreamType.NOTIFICATIONS)
    
    print("[SETUP] Multi-stream workflow")
    print("  - Stream 1: orders (order status updates)")
    print("  - Stream 2: alerts (system alerts)\n")
    
    print("[WORKFLOW] Processing order with alerts:")
    print("─" * 120)
    
    # Publish to multiple streams
    print("\n1. Order Created:")
    client.publish_json_to_stream(
        "orders",
        {"order_id": "ORD-999", "total": 299.99, "items": 3},
        tags={"order_id": "ORD-999", "status": "created"}
    )
    print("   ✓ Order details published to 'orders' stream")
    
    print("\n2. Low Inventory Alert:")
    client.publish_to_stream(
        "alerts",
        "Low inventory for SKU-XYZ (5 units left)",
        tags={"severity": "warning", "type": "inventory"}
    )
    print("   ✓ Alert published to 'alerts' stream")
    
    print("\n3. Order Processing:")
    client.publish_to_stream(
        "orders",
        "Processing payment for ORD-999",
        tags={"order_id": "ORD-999", "status": "processing"}
    )
    print("   ✓ Status update published")
    
    print("\n4. Critical Alert:")
    client.publish_to_stream(
        "alerts",
        "Payment gateway response time: 5000ms (threshold: 1000ms)",
        tags={"severity": "critical", "type": "performance"}
    )
    print("   ✓ Critical alert published")
    
    # Show combined statistics
    all_stats = client.get_stream_stats()
    print("\n[STATISTICS]")
    for stream_id, stats in all_stats.items():
        print(f"  {stream_id}: {stats['total_events']} events")
    
    client.disconnect()


# ============================================================================
# MAIN EXECUTION
# ============================================================================

if __name__ == "__main__":
    # Run synchronous examples
    example_ecommerce_integration()
    example_metrics_monitoring()
    example_combined_workflow()
    
    # Run async example
    print("\n" + "=" * 120)
    asyncio.run(example_async_live_updates())
    
    print("\n" + "=" * 120)
    print("✓ ALL INTEGRATION EXAMPLES COMPLETED")
    print("=" * 120 + "\n")
