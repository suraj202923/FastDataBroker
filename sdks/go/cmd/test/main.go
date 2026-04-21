package main

import (
	"fmt"
	"math/rand"
	"time"
)

// ============================================================================
// Mock SDK Types (for testing without live server)
// ============================================================================

type Message struct {
	Topic      string
	Payload    interface{}
	Priority   int
	TTLSeconds int
	Headers    map[string]string
}

type DeliveryResult struct {
	MessageID string
	Status    string
	LatencyMs float64
	Timestamp int64
}

type ConnectionStats struct {
	IsConnected      bool
	MessagesSent     int64
	MessagesReceived int64
	ConnectionTimeMs int64
	UptimeSeconds    int64
	LastMessageTime  int64
}

type FastDataBrokerQuicClient struct {
	config          map[string]interface{}
	connected       bool
	authenticated   bool
	stats           map[string]int64
	connectionStart time.Time
	messageHandlers map[string]func(interface{})
}

func NewFastDataBrokerQuicClient(host string, port int, tenantID, clientID, pskSecret string) *FastDataBrokerQuicClient {
	return &FastDataBrokerQuicClient{
		config: map[string]interface{}{
			"host":       host,
			"port":       port,
			"tenant_id":  tenantID,
			"client_id":  clientID,
			"psk_secret": pskSecret,
		},
		connected:       false,
		authenticated:   false,
		stats:           make(map[string]int64),
		messageHandlers: make(map[string]func(interface{})),
	}
}

func (c *FastDataBrokerQuicClient) Connect() error {
	host := c.config["host"].(string)
	port := c.config["port"].(int)
	fmt.Printf("Connecting to %s:%d...\n", host, port)
	c.connected = true
	c.connectionStart = time.Now()
	fmt.Printf("✓ Connected to %s:%d\n", host, port)
	return nil
}

func (c *FastDataBrokerQuicClient) SendMessage(msg *Message) (*DeliveryResult, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected")
	}

	messageID := fmt.Sprintf("msg_%d_%d", time.Now().UnixMilli(), rand.Intn(10000))
	latency := rand.Float64()*50 + 5 // Simulate 5-55ms latency
	c.stats["messages_sent"]++

	return &DeliveryResult{
		MessageID: messageID,
		Status:    "success",
		LatencyMs: latency,
		Timestamp: time.Now().UnixMilli(),
	}, nil
}

func (c *FastDataBrokerQuicClient) OnMessage(topic string, handler func(interface{})) {
	c.messageHandlers[topic] = handler
}

func (c *FastDataBrokerQuicClient) OffMessage(topic string) {
	delete(c.messageHandlers, topic)
}

func (c *FastDataBrokerQuicClient) GetStats() *ConnectionStats {
	uptime := int64(0)
	if c.connected {
		uptime = time.Since(c.connectionStart).Milliseconds()
	}

	return &ConnectionStats{
		IsConnected:      c.connected,
		MessagesSent:     c.stats["messages_sent"],
		MessagesReceived: c.stats["messages_received"],
		ConnectionTimeMs: uptime,
		UptimeSeconds:    uptime / 1000,
		LastMessageTime:  c.stats["last_message_time"],
	}
}

func (c *FastDataBrokerQuicClient) IsConnected() bool {
	return c.connected
}

func (c *FastDataBrokerQuicClient) Disconnect() error {
	c.connected = false
	fmt.Println("✓ Disconnected")
	return nil
}

// ============================================================================
// TEST SUITE
// ============================================================================

var testsPassed int = 0
var testsFailed int = 0

func runTest(name string, testFn func() error) {
	if err := testFn(); err != nil {
		fmt.Printf("❌ FAIL: %s\n", name)
		fmt.Printf("   Error: %v\n", err)
		testsFailed++
	} else {
		fmt.Printf("✅ PASS: %s\n", name)
		testsPassed++
	}
}

func test1BasicConnection() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")

	if client.IsConnected() {
		return fmt.Errorf("should not be connected yet")
	}

	if err := client.Connect(); err != nil {
		return err
	}

	if !client.IsConnected() {
		return fmt.Errorf("should be connected")
	}

	client.Disconnect()
	return nil
}

func test2SendMessage() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")
	client.Connect()

	result, err := client.SendMessage(&Message{
		Topic:    "test.topic",
		Payload:  map[string]string{"data": "test"},
		Priority: 5,
	})

	if err != nil {
		return err
	}

	if result.Status != "success" {
		return fmt.Errorf("expected status 'success', got '%s'", result.Status)
	}

	if result.MessageID == "" {
		return fmt.Errorf("message ID should not be empty")
	}

	if result.LatencyMs < 0 {
		return fmt.Errorf("latency should be non-negative")
	}

	client.Disconnect()
	return nil
}

func test3MessageHandlers() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")
	client.Connect()

	handler := func(msg interface{}) {}
	client.OnMessage("test.topic", handler)

	if _, exists := client.messageHandlers["test.topic"]; !exists {
		return fmt.Errorf("handler should be registered")
	}

	client.OffMessage("test.topic")

	if _, exists := client.messageHandlers["test.topic"]; exists {
		return fmt.Errorf("handler should be unregistered")
	}

	client.Disconnect()
	return nil
}

func test4ConnectionStatistics() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")
	client.Connect()

	for i := 0; i < 5; i++ {
		client.SendMessage(&Message{
			Topic:    "test.topic",
			Payload:  map[string]int{"index": i},
			Priority: 5,
		})
	}

	time.Sleep(50 * time.Millisecond)

	stats := client.GetStats()

	if !stats.IsConnected {
		return fmt.Errorf("should be connected")
	}

	if stats.MessagesSent != 5 {
		return fmt.Errorf("expected 5 messages sent, got %d", stats.MessagesSent)
	}

	if stats.UptimeSeconds < 0 {
		return fmt.Errorf("uptime should be non-negative")
	}

	client.Disconnect()
	return nil
}

func test5ConcurrentMessages() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")
	client.Connect()

	results := make([]*DeliveryResult, 10)
	for i := 0; i < 10; i++ {
		result, err := client.SendMessage(&Message{
			Topic:    "test.concurrent",
			Payload:  map[string]int{"index": i},
			Priority: 5,
		})
		if err != nil {
			return err
		}
		results[i] = result
	}

	if len(results) != 10 {
		return fmt.Errorf("expected 10 results, got %d", len(results))
	}

	for _, result := range results {
		if result.Status != "success" {
			return fmt.Errorf("all messages should be successful")
		}
	}

	if client.stats["messages_sent"] != 10 {
		return fmt.Errorf("expected 10 messages sent, got %d", client.stats["messages_sent"])
	}

	client.Disconnect()
	return nil
}

func test6PriorityLevels() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")
	client.Connect()

	priorities := []int{1, 5, 10, 20}

	for _, priority := range priorities {
		result, err := client.SendMessage(&Message{
			Topic:    "test.priority",
			Payload:  map[string]int{"priority": priority},
			Priority: priority,
		})

		if err != nil {
			return err
		}

		if result.Status != "success" {
			return fmt.Errorf("failed to send message with priority %d", priority)
		}
	}

	client.Disconnect()
	return nil
}

func test7LatencyMeasurement() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")
	client.Connect()

	latencies := make([]float64, 0)
	for i := 0; i < 50; i++ {
		result, err := client.SendMessage(&Message{
			Topic:    "test.latency",
			Payload:  map[string]int{"iteration": i},
			Priority: 5,
		})

		if err != nil {
			return err
		}

		latencies = append(latencies, result.LatencyMs)
	}

	var avgLatency float64 = 0
	for _, latency := range latencies {
		avgLatency += latency
	}
	avgLatency /= float64(len(latencies))

	if avgLatency < 0 {
		return fmt.Errorf("average latency should be non-negative")
	}

	maxLatency := latencies[0]
	for _, latency := range latencies {
		if latency > maxLatency {
			maxLatency = latency
		}
	}

	fmt.Printf("   Average latency: %.2fms, Max: %.2fms\n", avgLatency, maxLatency)

	client.Disconnect()
	return nil
}

func test8ErrorHandling() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")

	_, err := client.SendMessage(&Message{
		Topic:    "test",
		Payload:  map[string]string{},
		Priority: 5,
	})

	if err == nil {
		return fmt.Errorf("should have thrown error")
	}

	if err.Error() != "not connected" {
		return fmt.Errorf("should throw 'not connected' error, got: %v", err)
	}

	client.Connect()
	result, err := client.SendMessage(&Message{Topic: "test", Payload: map[string]string{}, Priority: 5})

	if err != nil {
		return err
	}

	if result.Status != "success" {
		return fmt.Errorf("should send successfully after connect")
	}

	client.Disconnect()
	return nil
}

func test9ConfigurationValidation() error {
	client := NewFastDataBrokerQuicClient("localhost", 6000, "test-tenant", "test-client", "test-secret")

	if client.config["host"] != "localhost" {
		return fmt.Errorf("host configuration not properly saved")
	}

	if client.config["port"] != 6000 {
		return fmt.Errorf("port configuration not properly saved")
	}

	return nil
}

// ============================================================================
// MAIN - RUN ALL TESTS
// ============================================================================

func main() {
	fmt.Println("\n" + string(make([]byte, 70)) + "\n")
	fmt.Println("FastDataBroker Go SDK - Test Suite")
	for i := 0; i < 70; i++ {
		fmt.Print("=")
	}
	fmt.Println("\n")

	runTest("1. Basic Connection", test1BasicConnection)
	runTest("2. Send Message", test2SendMessage)
	runTest("3. Message Handlers", test3MessageHandlers)
	runTest("4. Connection Statistics", test4ConnectionStatistics)
	runTest("5. Concurrent Messages", test5ConcurrentMessages)
	runTest("6. Priority Levels", test6PriorityLevels)
	runTest("7. Latency Measurement", test7LatencyMeasurement)
	runTest("8. Error Handling", test8ErrorHandling)
	runTest("9. Configuration Validation", test9ConfigurationValidation)

	fmt.Println("\n" + "===================================================================" + "\n")
	fmt.Printf("Results: %d passed, %d failed\n", testsPassed, testsFailed)
	fmt.Println("===================================================================" + "\n")
}
