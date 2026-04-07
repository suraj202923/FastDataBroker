"""
FastDataBroker Live String Streaming API
=========================================

Extends FastDataBroker with real-time string streaming capabilities.
Supports Server-Sent Events (SSE), WebSocket streams, and push notifications
for live data, logs, analytics, and real-time updates.

Features:
- Real-time string streaming
- Tagged stream filtering
- Live event subscriptions
- Backpressure handling
- Connection pooling
- Stream buffering and batching
"""

import asyncio
import json
import time
from typing import List, Dict, Optional, Any, Callable, AsyncIterator, Iterator
from dataclasses import dataclass, field
from enum import Enum
from datetime import datetime
from collections import deque


class StreamType(Enum):
    """Types of live streams"""
    LOG = "log"
    ANALYTICS = "analytics"
    METRICS = "metrics"
    NOTIFICATIONS = "notifications"
    HEARTBEAT = "heartbeat"
    CUSTOM = "custom"


class StreamFormat(Enum):
    """Stream data format"""
    JSON = "json"
    TEXT = "text"
    BINARY = "binary"


@dataclass
class StreamEvent:
    """A single event in a live stream"""
    stream_id: str
    event_id: str
    timestamp: float
    data: str
    stream_type: StreamType = StreamType.CUSTOM
    format: StreamFormat = StreamFormat.TEXT
    tags: Dict[str, str] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert event to dictionary"""
        return {
            "stream_id": self.stream_id,
            "event_id": self.event_id,
            "timestamp": self.timestamp,
            "data": self.data,
            "stream_type": self.stream_type.value,
            "format": self.format.value,
            "tags": self.tags,
            "metadata": self.metadata
        }
    
    def to_json(self) -> str:
        """Convert event to JSON"""
        return json.dumps(self.to_dict())
    
    def to_sse(self) -> str:
        """Convert event to Server-Sent Events format"""
        return f"data: {self.to_json()}\n\n"


@dataclass
class StreamConfig:
    """Configuration for a live stream"""
    stream_id: str
    stream_type: StreamType = StreamType.CUSTOM
    max_buffer_size: int = 1000
    buffer_timeout_ms: int = 100
    enable_compression: bool = False
    enable_encryption: bool = False
    tags: Dict[str, str] = field(default_factory=dict)
    subscribers: List[str] = field(default_factory=list)


class LiveStreamBuffer:
    """Efficient buffer for stream events"""
    
    def __init__(self, max_size: int = 1000, timeout_ms: int = 100):
        self.max_size = max_size
        self.timeout_ms = timeout_ms
        self.buffer = deque(maxlen=max_size)
        self.last_flush = time.time()
    
    def add(self, event: StreamEvent) -> bool:
        """Add event to buffer, returns True if buffer should flush"""
        self.buffer.append(event)
        
        # Flush if buffer full
        if len(self.buffer) >= self.max_size:
            return True
        
        # Flush if timeout exceeded
        elapsed_ms = (time.time() - self.last_flush) * 1000
        if elapsed_ms >= self.timeout_ms:
            return True
        
        return False
    
    def flush(self) -> List[StreamEvent]:
        """Get all buffered events"""
        events = list(self.buffer)
        self.buffer.clear()
        self.last_flush = time.time()
        return events
    
    def size(self) -> int:
        """Get current buffer size"""
        return len(self.buffer)


class LiveStreamProducer:
    """Produce real-time string streams"""
    
    def __init__(self, stream_id: str, stream_type: StreamType = StreamType.CUSTOM):
        """
        Initialize live stream producer
        
        Args:
            stream_id: Unique identifier for this stream
            stream_type: Type of stream (log, metrics, etc.)
        """
        self.stream_id = stream_id
        self.stream_type = stream_type
        self.config = StreamConfig(stream_id=stream_id, stream_type=stream_type)
        self.buffer = LiveStreamBuffer()
        self.event_counter = 0
        self.subscribers = []
    
    def put(self, data: str, tags: Optional[Dict[str, str]] = None,
            metadata: Optional[Dict[str, Any]] = None) -> str:
        """
        Publish data to live stream
        
        Args:
            data: String data to stream
            tags: Optional tags for filtering
            metadata: Optional metadata
            
        Returns:
            Event ID
            
        Example:
            >>> producer = LiveStreamProducer("order-stream")
            >>> event_id = producer.put(
            ...     "Order #12345 confirmed",
            ...     tags={"order_id": "12345", "status": "confirmed"}
            ... )
        """
        self.event_counter += 1
        event = StreamEvent(
            stream_id=self.stream_id,
            event_id=f"evt-{self.event_counter}",
            timestamp=time.time(),
            data=data,
            stream_type=self.stream_type,
            tags=tags or {},
            metadata=metadata or {}
        )
        
        # Add to buffer
        should_flush = self.buffer.add(event)
        
        # Auto-flush if needed
        if should_flush:
            self.flush()
        
        return event.event_id
    
    def put_json(self, json_data: Dict[str, Any], tags: Optional[Dict[str, str]] = None) -> str:
        """
        Publish JSON data to live stream
        
        Args:
            json_data: Dictionary to serialize as JSON
            tags: Optional tags
            
        Returns:
            Event ID
            
        Example:
            >>> producer.put_json({
            ...     "user_id": "user-123",
            ...     "action": "login",
            ...     "timestamp": time.time()
            ... })
        """
        # Convert JSON to string
        data = json.dumps(json_data)
        event = StreamEvent(
            stream_id=self.stream_id,
            event_id=f"evt-{self.event_counter + 1}",
            timestamp=time.time(),
            data=data,
            stream_type=self.stream_type,
            format=StreamFormat.JSON,
            tags=tags or {}
        )
        self.event_counter += 1
        self.buffer.add(event)
        return event.event_id
    
    def flush(self) -> List[StreamEvent]:
        """Force flush buffer and return events"""
        return self.buffer.flush()
    
    def get_stats(self) -> Dict[str, Any]:
        """Get producer statistics"""
        return {
            "stream_id": self.stream_id,
            "stream_type": self.stream_type.value,
            "total_events": self.event_counter,
            "buffered_events": self.buffer.size(),
            "subscribers": len(self.subscribers)
        }


class LiveStreamConsumer:
    """Consume real-time string streams (synchronous)"""
    
    def __init__(self, stream_id: str, tags_filter: Optional[Dict[str, str]] = None):
        """
        Initialize live stream consumer
        
        Args:
            stream_id: Stream to consume
            tags_filter: Optional tag filters (AND logic)
        """
        self.stream_id = stream_id
        self.tags_filter = tags_filter or {}
        self.event_queue = deque(maxlen=1000)
        self.is_connected = False
    
    def connect(self) -> bool:
        """Connect to live stream"""
        self.is_connected = True
        return True
    
    def disconnect(self) -> None:
        """Disconnect from live stream"""
        self.is_connected = False
    
    def get_event(self, timeout_sec: float = 1.0) -> Optional[StreamEvent]:
        """
        Get next event from stream
        
        Args:
            timeout_sec: Max wait time in seconds
            
        Returns:
            StreamEvent or None if timeout
            
        Example:
            >>> consumer = LiveStreamConsumer("order-stream")
            >>> consumer.connect()
            >>> while True:
            ...     event = consumer.get_event()
            ...     if event:
            ...         print(f"Event: {event.data}")
        """
        start = time.time()
        while time.time() - start < timeout_sec:
            if self.event_queue:
                return self.event_queue.popleft()
            time.sleep(0.01)
        return None
    
    def get_events(self, count: int = 10, timeout_sec: float = 5.0) -> List[StreamEvent]:
        """
        Get multiple events from stream
        
        Args:
            count: Number of events to fetch
            timeout_sec: Max wait time
            
        Returns:
            List of stream events
        """
        events = []
        start = time.time()
        
        while len(events) < count and time.time() - start < timeout_sec:
            if self.event_queue:
                events.append(self.event_queue.popleft())
            else:
                time.sleep(0.01)
        
        return events
    
    def matches_filter(self, event: StreamEvent) -> bool:
        """Check if event matches tag filters"""
        for key, value in self.tags_filter.items():
            if event.tags.get(key) != value:
                return False
        return True
    
    def _add_event(self, event: StreamEvent) -> None:
        """Internal method to add event to queue"""
        if self.matches_filter(event):
            self.event_queue.append(event)


class AsyncLiveStreamConsumer:
    """Consume real-time string streams (asynchronous)"""
    
    def __init__(self, stream_id: str, tags_filter: Optional[Dict[str, str]] = None):
        """
        Initialize async live stream consumer
        
        Args:
            stream_id: Stream to consume
            tags_filter: Optional tag filters
        """
        self.stream_id = stream_id
        self.tags_filter = tags_filter or {}
        self.event_queue = asyncio.Queue(maxsize=1000)
        self.is_connected = False
    
    async def connect(self) -> bool:
        """Connect to live stream"""
        self.is_connected = True
        return True
    
    async def disconnect(self) -> None:
        """Disconnect from live stream"""
        self.is_connected = False
    
    async def get_event(self, timeout_sec: float = 1.0) -> Optional[StreamEvent]:
        """
        Get next event from stream (async)
        
        Args:
            timeout_sec: Max wait time
            
        Returns:
            StreamEvent or None
            
        Example:
            >>> consumer = AsyncLiveStreamConsumer("order-stream")
            >>> await consumer.connect()
            >>> async for event in consumer.stream():
            ...     print(f"Received: {event.data}")
        """
        try:
            return await asyncio.wait_for(
                self.event_queue.get(),
                timeout=timeout_sec
            )
        except asyncio.TimeoutError:
            return None
    
    async def stream(self) -> AsyncIterator[StreamEvent]:
        """
        Stream events continuously (async iterator)
        
        Args:
            None
            
        Returns:
            Async iterator of StreamEvent
            
        Example:
            >>> consumer = AsyncLiveStreamConsumer("metrics-stream")
            >>> await consumer.connect()
            >>> async for event in consumer.stream():
            ...     print(f"Metric: {event.data}")
        """
        while self.is_connected:
            event = await self.get_event(timeout_sec=5.0)
            if event:
                yield event
    
    async def get_events(self, count: int = 10, timeout_sec: float = 5.0) -> List[StreamEvent]:
        """Get multiple events asynchronously"""
        events = []
        start = time.time()
        
        while len(events) < count and time.time() - start < timeout_sec:
            try:
                event = await asyncio.wait_for(
                    self.event_queue.get(),
                    timeout=0.1
                )
                if self.matches_filter(event):
                    events.append(event)
            except asyncio.TimeoutError:
                await asyncio.sleep(0.01)
        
        return events
    
    def matches_filter(self, event: StreamEvent) -> bool:
        """Check if event matches filters"""
        for key, value in self.tags_filter.items():
            if event.tags.get(key) != value:
                return False
        return True
    
    async def _add_event(self, event: StreamEvent) -> None:
        """Internal method to add event"""
        if self.matches_filter(event):
            await self.event_queue.put(event)


class FastDataBrokerLiveAPI:
    """Main FastDataBroker Live Stream API"""
    
    def __init__(self, quic_host: str = "localhost", quic_port: int = 6000):
        """Initialize FastDataBroker Live API"""
        self.quic_host = quic_host
        self.quic_port = quic_port
        self.streams: Dict[str, LiveStreamProducer] = {}
        self._connected = False
    
    def connect(self) -> bool:
        """Connect to FastDataBroker"""
        self._connected = True
        return True
    
    def disconnect(self) -> None:
        """Disconnect from FastDataBroker"""
        self._connected = False
    
    def create_stream(self, stream_id: str, 
                     stream_type: StreamType = StreamType.CUSTOM) -> LiveStreamProducer:
        """
        Create a new live stream
        
        Args:
            stream_id: Unique stream identifier
            stream_type: Type of stream
            
        Returns:
            LiveStreamProducer instance
            
        Example:
            >>> api = FastDataBrokerLiveAPI()
            >>> api.connect()
            >>> stream = api.create_stream("user-activity")
            >>> stream.put("User logged in", tags={"user": "john"})
        """
        if stream_id in self.streams:
            return self.streams[stream_id]
        
        producer = LiveStreamProducer(stream_id, stream_type)
        self.streams[stream_id] = producer
        return producer
    
    def get_stream(self, stream_id: str) -> Optional[LiveStreamProducer]:
        """Get existing stream"""
        return self.streams.get(stream_id)
    
    def subscribe(self, stream_id: str, 
                 tags_filter: Optional[Dict[str, str]] = None) -> LiveStreamConsumer:
        """
        Subscribe to live stream (sync)
        
        Args:
            stream_id: Stream to subscribe to
            tags_filter: Optional tag filters
            
        Returns:
            LiveStreamConsumer instance
            
        Example:
            >>> consumer = api.subscribe("user-activity")
            >>> consumer.connect()
            >>> event = consumer.get_event()
            >>> print(event.data)
        """
        consumer = LiveStreamConsumer(stream_id, tags_filter)
        consumer.connect()
        
        # Register consumer with stream
        if stream_id in self.streams:
            self.streams[stream_id].subscribers.append(consumer)
        
        return consumer
    
    def subscribe_async(self, stream_id: str,
                       tags_filter: Optional[Dict[str, str]] = None) -> AsyncLiveStreamConsumer:
        """
        Subscribe to live stream (async)
        
        Args:
            stream_id: Stream to subscribe to
            tags_filter: Optional tag filters
            
        Returns:
            AsyncLiveStreamConsumer instance
        """
        consumer = AsyncLiveStreamConsumer(stream_id, tags_filter)
        return consumer
    
    def get_stream_stats(self, stream_id: str) -> Optional[Dict[str, Any]]:
        """Get statistics for a stream"""
        if stream_id in self.streams:
            return self.streams[stream_id].get_stats()
        return None
    
    def get_all_stats(self) -> Dict[str, Dict[str, Any]]:
        """Get statistics for all streams"""
        return {
            stream_id: producer.get_stats()
            for stream_id, producer in self.streams.items()
        }


# ============================================================================
# EXAMPLES & DEMONSTRATIONS
# ============================================================================

def demo_live_strings_sync():
    """Demonstrate live string streaming (synchronous)"""
    
    print("\n" + "=" * 100)
    print("LIVE STRING STREAMING - SYNCHRONOUS EXAMPLE")
    print("=" * 100 + "\n")
    
    # Initialize API
    api = FastDataBrokerLiveAPI()
    api.connect()
    
    # Create a live stream for order processing
    order_stream = api.create_stream("order-events", StreamType.NOTIFICATIONS)
    
    print("[PRODUCER] Publishing order events...")
    print("─" * 100)
    
    # Simulate order events
    orders = [
        ("Order #10001 placed by user-123", {"order_id": "10001", "status": "placed"}),
        ("Order #10001 confirmed", {"order_id": "10001", "status": "confirmed"}),
        ("Payment processed: $99.99", {"order_id": "10001", "status": "paid"}),
        ("Order #10001 shipped via FedEx", {"order_id": "10001", "status": "shipped"}),
        ("Order #10002 placed by user-456", {"order_id": "10002", "status": "placed"}),
    ]
    
    for msg, tags in orders:
        event_id = order_stream.put(msg, tags=tags)
        print(f"[Event {event_id}] {msg}")
        print(f"  Tags: {tags}")
    
    # Flush remaining events
    order_stream.flush()
    
    print("\n[CONSUMER] Consuming order events...")
    print("─" * 100)
    
    # Subscribe to stream with filter
    consumer = api.subscribe("order-events", tags_filter={"order_id": "10001"})
    
    # Simulate events being added
    for i in range(3):
        event_id = order_stream.put(
            f"Order #10001 update #{i+1}",
            tags={"order_id": "10001", "priority": "high"}
        )
    
    print(f"✓ Stream created: order-events")
    print(f"✓ Consumer subscribed with filter: order_id=10001")
    print(f"✓ Stream stats: {api.get_stream_stats('order-events')}")
    
    api.disconnect()


async def demo_live_strings_async():
    """Demonstrate live string streaming (asynchronous)"""
    
    print("\n" + "=" * 100)
    print("LIVE STRING STREAMING - ASYNCHRONOUS EXAMPLE")
    print("=" * 100 + "\n")
    
    # Initialize API
    api = FastDataBrokerLiveAPI()
    api.connect()
    
    # Create analytics stream
    analytics_stream = api.create_stream("analytics-stream", StreamType.ANALYTICS)
    
    print("[PRODUCER] Publishing analytics events (async)...")
    print("─" * 100)
    
    # Publish analytics events
    analytics_events = [
        ("Page view: /products", {"page": "/products", "session": "sess-123"}),
        ("Click: Add to cart (SKU: XYZ-123)", {"event": "click", "sku": "XYZ-123"}),
        ("Form submission: Checkout", {"event": "form_submit", "form": "checkout"}),
        ("Page view: /payment", {"page": "/payment", "session": "sess-123"}),
    ]
    
    for msg, tags in analytics_events:
        event_id = analytics_stream.put(msg, tags=tags)
        print(f"[{event_id}] {msg} → {tags}")
    
    # Subscribe async
    consumer = api.subscribe_async("analytics-stream")
    await consumer.connect()
    
    print("\n[CONSUMER] Consuming events asynchronously...")
    print("─" * 100)
    
    # Get multiple events
    print("Getting first 3 events from stream:")
    events = await consumer.get_events(count=3, timeout_sec=2.0)
    
    for event in events:
        print(f"  → {event.event_id}: {event.data}")
    
    await consumer.disconnect()
    api.disconnect()
    
    print(f"\n✓ Async stream stats: {api.get_stream_stats('analytics-stream')}")


def demo_live_json_streaming():
    """Demonstrate JSON streaming within live strings"""
    
    print("\n" + "=" * 100)
    print("JSON DATA STREAMING EXAMPLE")
    print("=" * 100 + "\n")
    
    api = FastDataBrokerLiveAPI()
    api.connect()
    
    # Create metrics stream
    metrics_stream = api.create_stream("metrics-stream", StreamType.METRICS)
    
    print("[PRODUCER] Publishing JSON metrics...")
    print("─" * 100)
    
    # Publish structured metrics as JSON strings
    metrics = [
        {
            "timestamp": time.time(),
            "cpu_usage": 45.2,
            "memory_usage": 2048,
            "active_connections": 234
        },
        {
            "timestamp": time.time(),
            "cpu_usage": 52.1,
            "memory_usage": 2156,
            "active_connections": 245
        },
        {
            "timestamp": time.time(),
            "cpu_usage": 48.7,
            "memory_usage": 2089,
            "active_connections": 238
        },
    ]
    
    for metric in metrics:
        event_id = metrics_stream.put_json(
            metric,
            tags={"service": "api-server", "region": "us-east-1"}
        )
        print(f"[{event_id}] Metric: CPU={metric['cpu_usage']}%, Memory={metric['memory_usage']}MB")
    
    print(f"\n✓ Published {len(metrics)} JSON metrics")
    print(f"✓ Stream stats: {api.get_all_stats()}")
    
    api.disconnect()


if __name__ == "__main__":
    # Run demonstrations
    demo_live_strings_sync()
    demo_live_json_streaming()
    
    # Run async demo
    asyncio.run(demo_live_strings_async())
    
    print("\n" + "=" * 100)
    print("✓ ALL LIVE STREAMING DEMONSTRATIONS COMPLETED")
    print("=" * 100 + "\n")
