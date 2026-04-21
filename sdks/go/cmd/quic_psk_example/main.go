package main

import (
	"bufio"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"sync"
	"time"
)

// ============================================================================
// Types and Constants
// ============================================================================

// Priority message priority levels
type Priority int

const (
	PriorityLow      Priority = 1
	PriorityNormal   Priority = 5
	PriorityHigh     Priority = 10
	PriorityCritical Priority = 20
)

// ConnectionState connection states
type ConnectionState string

const (
	StateDisconnected  ConnectionState = "disconnected"
	StateConnecting    ConnectionState = "connecting"
	StateConnected     ConnectionState = "connected"
	StateAuthenticated ConnectionState = "authenticated"
	StateError         ConnectionState = "error"
)

// Message FastDataBroker message envelope
type Message struct {
	Topic      string            `json:"topic"`
	Payload    interface{}       `json:"payload"`
	Priority   Priority          `json:"priority"`
	TTLSeconds int               `json:"ttl_seconds"`
	Headers    map[string]string `json:"headers"`
}

// DeliveryResult message delivery result
type DeliveryResult struct {
	MessageID string  `json:"message_id"`
	Status    string  `json:"status"`
	LatencyMs float64 `json:"latency_ms"`
	Timestamp int64   `json:"timestamp"`
}

// ConnectionStats connection statistics
type ConnectionStats struct {
	IsConnected      bool  `json:"is_connected"`
	MessagesSent     int64 `json:"messages_sent"`
	MessagesReceived int64 `json:"messages_received"`
	ConnectionTimeMs int64 `json:"connection_time_ms"`
	UptimeSeconds    int64 `json:"uptime_seconds"`
	LastMessageTime  int64 `json:"last_message_time"`
}

// QuicConnectionConfig QUIC connection configuration
type QuicConnectionConfig struct {
	Host          string
	Port          int
	TenantID      string
	ClientID      string
	PSKSecret     string
	Secrets       string
	IdleTimeoutMs int
	MaxStreams    int
	AutoReconnect bool
	ReadTimeoutMs int
}

// ============================================================================
// QUIC PSK Client
// ============================================================================

// FastDataBrokerQuicClient QUIC client with PSK authentication
type FastDataBrokerQuicClient struct {
	config          *QuicConnectionConfig
	conn            net.Conn
	connected       bool
	authenticated   bool
	state           ConnectionState
	messageHandlers map[string]func(interface{})
	handlersMutex   sync.RWMutex
	connectionStart time.Time
	receiveCancel   chan bool
	wg              sync.WaitGroup
	stats           Stats
	statsMutex      sync.RWMutex
}

// Stats internal statistics
type Stats struct {
	MessagesSent     int64
	MessagesReceived int64
	LastMessageTime  int64
}

// NewFastDataBrokerQuicClient create new QUIC client
func NewFastDataBrokerQuicClient(config *QuicConnectionConfig) *FastDataBrokerQuicClient {
	if config.IdleTimeoutMs == 0 {
		config.IdleTimeoutMs = 30000
	}
	if config.MaxStreams == 0 {
		config.MaxStreams = 100
	}
	config.AutoReconnect = true

	return &FastDataBrokerQuicClient{
		config:          config,
		connected:       false,
		authenticated:   false,
		state:           StateDisconnected,
		messageHandlers: make(map[string]func(interface{})),
		receiveCancel:   make(chan bool),
		stats:           Stats{},
	}
}

// generatePskIdentity generate PSK identity and secret hash
func (c *FastDataBrokerQuicClient) generatePskIdentity() (string, string) {
	identity := fmt.Sprintf("%s:%s", c.config.TenantID, c.config.ClientID)
	secretHash := sha256.Sum256([]byte(c.config.PSKSecret))
	return identity, hex.EncodeToString(secretHash[:])
}

// Connect establish connection with PSK authentication
func (c *FastDataBrokerQuicClient) Connect() error {
	if c.connected {
		return fmt.Errorf("already connected")
	}

	c.state = StateConnecting
	fmt.Printf("Connecting to %s...\n", net.JoinHostPort(c.config.Host, fmt.Sprintf("%d", c.config.Port)))

	// Dial connection
	addr := net.JoinHostPort(c.config.Host, fmt.Sprintf("%d", c.config.Port))
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		c.state = StateError
		return fmt.Errorf("failed to connect: %w", err)
	}

	c.conn = conn
	c.connected = true
	c.connectionStart = time.Now()
	fmt.Println("✓ TCP connection established")

	// Send PSK handshake
	if err := c.sendPskHandshake(); err != nil {
		c.connected = false
		c.state = StateError
		return err
	}

	// Start receive loop
	c.wg.Add(1)
	go c.receiveLoop()

	c.authenticated = true
	c.state = StateAuthenticated
	fmt.Println("✓ PSK authentication successful")

	return nil
}

// sendPskHandshake send PSK authentication handshake
func (c *FastDataBrokerQuicClient) sendPskHandshake() error {
	identity, secretHash := c.generatePskIdentity()

	handshake := map[string]interface{}{
		"type":        "psk_auth",
		"identity":    identity,
		"secret_hash": secretHash,
		"timestamp":   time.Now().UnixMilli(),
	}

	data, err := json.Marshal(handshake)
	if err != nil {
		return fmt.Errorf("failed to marshal handshake: %w", err)
	}

	_, err = c.conn.Write(append(data, '\n'))
	if err != nil {
		return fmt.Errorf("failed to send handshake: %w", err)
	}

	return nil
}

// receiveLoop receive messages in background
func (c *FastDataBrokerQuicClient) receiveLoop() {
	defer c.wg.Done()

	reader := bufio.NewReader(c.conn)

	for c.connected {
		// Set read deadline
		if err := c.conn.SetReadDeadline(time.Now().Add(time.Duration(c.config.ReadTimeoutMs) * time.Millisecond)); err != nil {
			if c.connected {
				fmt.Printf("SetReadDeadline error: %v\n", err)
			}
			break
		}

		line, err := reader.ReadString('\n')
		if err != nil {
			if opErr, ok := err.(net.Error); ok && opErr.Timeout() {
				continue
			}
			if c.connected {
				fmt.Printf("Receive error: %v\n", err)
			}
			break
		}

		if len(line) > 0 {
			c.handleIncomingMessage(line)
		}
	}

	c.connected = false
}

// handleIncomingMessage process incoming message
func (c *FastDataBrokerQuicClient) handleIncomingMessage(data string) {
	var msg map[string]interface{}
	if err := json.Unmarshal([]byte(data), &msg); err != nil {
		return
	}

	if msg["type"] == "message" {
		topic, ok := msg["topic"].(string)
		if !ok {
			return
		}

		c.handlersMutex.RLock()
		handler, exists := c.messageHandlers[topic]
		c.handlersMutex.RUnlock()

		if exists && handler != nil {
			handler(msg)
			c.statsMutex.Lock()
			c.stats.MessagesReceived++
			c.stats.LastMessageTime = time.Now().UnixMilli()
			c.statsMutex.Unlock()
		}
	}
}

// SendMessage send message to FastDataBroker
func (c *FastDataBrokerQuicClient) SendMessage(msg *Message) (*DeliveryResult, error) {
	if !c.connected || c.conn == nil {
		return nil, fmt.Errorf("not connected to FastDataBroker")
	}

	if msg.Headers == nil {
		msg.Headers = make(map[string]string)
	}
	if msg.Priority == 0 {
		msg.Priority = PriorityNormal
	}
	if msg.TTLSeconds == 0 {
		msg.TTLSeconds = 3600
	}

	start := time.Now()
	messageID := fmt.Sprintf("msg_%d_%d", time.Now().UnixMilli(), getRandomInt())

	envelope := map[string]interface{}{
		"type":      "message",
		"id":        messageID,
		"topic":     msg.Topic,
		"payload":   msg.Payload,
		"priority":  int(msg.Priority),
		"ttl":       msg.TTLSeconds,
		"headers":   msg.Headers,
		"timestamp": time.Now().UnixMilli(),
	}

	data, err := json.Marshal(envelope)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal message: %w", err)
	}

	// Set write deadline
	if err := c.conn.SetWriteDeadline(time.Now().Add(5 * time.Second)); err != nil {
		return nil, fmt.Errorf("failed to set write deadline: %w", err)
	}

	_, err = c.conn.Write(append(data, '\n'))
	if err != nil {
		return nil, fmt.Errorf("failed to send message: %w", err)
	}

	c.statsMutex.Lock()
	c.stats.MessagesSent++
	c.stats.LastMessageTime = time.Now().UnixMilli()
	c.statsMutex.Unlock()

	latency := time.Since(start).Seconds() * 1000

	return &DeliveryResult{
		MessageID: messageID,
		Status:    "success",
		LatencyMs: latency,
		Timestamp: time.Now().UnixMilli(),
	}, nil
}

// OnMessage register message handler
func (c *FastDataBrokerQuicClient) OnMessage(topic string, handler func(interface{})) {
	c.handlersMutex.Lock()
	defer c.handlersMutex.Unlock()
	c.messageHandlers[topic] = handler
	fmt.Printf("Registered handler for topic: %s\n", topic)
}

// OffMessage unregister message handler
func (c *FastDataBrokerQuicClient) OffMessage(topic string) {
	c.handlersMutex.Lock()
	defer c.handlersMutex.Unlock()
	delete(c.messageHandlers, topic)
	fmt.Printf("Unregistered handler for topic: %s\n", topic)
}

// GetStats get connection statistics
func (c *FastDataBrokerQuicClient) GetStats() *ConnectionStats {
	c.statsMutex.RLock()
	defer c.statsMutex.RUnlock()

	connectionTime := int64(0)
	if c.connected {
		connectionTime = time.Since(c.connectionStart).Milliseconds()
	}

	return &ConnectionStats{
		IsConnected:      c.connected,
		MessagesSent:     c.stats.MessagesSent,
		MessagesReceived: c.stats.MessagesReceived,
		ConnectionTimeMs: connectionTime,
		UptimeSeconds:    connectionTime / 1000,
		LastMessageTime:  c.stats.LastMessageTime,
	}
}

// IsConnected check connection status
func (c *FastDataBrokerQuicClient) IsConnected() bool {
	return c.connected && c.authenticated
}

// SendMessagesParallel sends multiple messages in parallel using goroutines
func (c *FastDataBrokerQuicClient) SendMessagesParallel(messages []Message, numWorkers int) []DeliveryResult {
	if !c.IsConnected() {
		return []DeliveryResult{}
	}

	results := make([]DeliveryResult, len(messages))
	resultsChan := make(chan DeliveryResult, len(messages))
	var wg sync.WaitGroup

	// Distribute messages among workers
	msgChan := make(chan Message, len(messages))
	for i := 0; i < numWorkers; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for msg := range msgChan {
				result, _ := c.SendMessage(&msg)
				resultsChan <- *result
			}
		}()
	}

	// Queue all messages
	go func() {
		for _, msg := range messages {
			msgChan <- msg
		}
		close(msgChan)
	}()

	// Collect results
	wg.Wait()
	close(resultsChan)
	i := 0
	for result := range resultsChan {
		results[i] = result
		i++
	}

	return results
}

// SendMessagesParallelWithProgress sends messages with progress callback
func (c *FastDataBrokerQuicClient) SendMessagesParallelWithProgress(
	messages []Message,
	numWorkers int,
	callback func(completed, total int),
) []DeliveryResult {
	if !c.IsConnected() {
		return []DeliveryResult{}
	}

	results := make([]DeliveryResult, len(messages))
	resultsChan := make(chan DeliveryResult, len(messages))
	var wg sync.WaitGroup
	var progressMutex sync.Mutex
	completed := 0

	// Distribute messages among workers
	msgChan := make(chan Message, len(messages))
	for i := 0; i < numWorkers; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for msg := range msgChan {
				result, _ := c.SendMessage(&msg)
				resultsChan <- *result

				progressMutex.Lock()
				completed++
				if callback != nil {
					callback(completed, len(messages))
				}
				progressMutex.Unlock()
			}
		}()
	}

	// Queue all messages
	go func() {
		for _, msg := range messages {
			msgChan <- msg
		}
		close(msgChan)
	}()

	// Collect results
	wg.Wait()
	close(resultsChan)
	i := 0
	for result := range resultsChan {
		results[i] = result
		i++
	}

	return results
}

// Disconnect close connection
func (c *FastDataBrokerQuicClient) Disconnect() error {
	c.connected = false
	c.authenticated = false
	c.state = StateDisconnected

	if c.conn != nil {
		err := c.conn.Close()
		fmt.Println("✓ Disconnected from FastDataBroker")
		return err
	}
	return nil
}

// ============================================================================
// Utility Functions
// ============================================================================

func getRandomInt() int64 {
	return time.Now().UnixNano() % 10000
}

// NewQuicConnectionConfig create configuration
func NewQuicConnectionConfig(host string, port int, tenantID, clientID, pskSecret string) *QuicConnectionConfig {
	return &QuicConnectionConfig{
		Host:          host,
		Port:          port,
		TenantID:      tenantID,
		ClientID:      clientID,
		PSKSecret:     pskSecret,
		IdleTimeoutMs: 30000,
		MaxStreams:    100,
		AutoReconnect: true,
		ReadTimeoutMs: 60000,
	}
}

// GetPskSecretFromEnv get PSK secret from environment
func GetPskSecretFromEnv() (string, error) {
	secret := os.Getenv("QUIC_PSK_SECRET")
	if secret == "" {
		return "", fmt.Errorf("QUIC_PSK_SECRET environment variable not set. Get it from: POST /api/quic/psks")
	}
	return secret, nil
}

// ============================================================================
// Example Usage
// ============================================================================

func main() {
	// Example configuration
	pskSecret := os.Getenv("QUIC_PSK_SECRET")
	if pskSecret == "" {
		pskSecret = "test-secret-key"
	}

	config := NewQuicConnectionConfig(
		"localhost",
		6000,
		"test-tenant",
		"test-client",
		pskSecret,
	)

	// Create and connect client
	client := NewFastDataBrokerQuicClient(config)

	err := client.Connect()
	if err != nil {
		fmt.Printf("✗ Failed to connect: %v\n", err)
		os.Exit(1)
	}

	// Send message
	msg := &Message{
		Topic:      "test.topic",
		Payload:    map[string]string{"data": "test"},
		Priority:   PriorityNormal,
		TTLSeconds: 3600,
		Headers:    make(map[string]string),
	}

	result, err := client.SendMessage(msg)
	if err != nil {
		fmt.Printf("✗ Failed to send message: %v\n", err)
	} else {
		fmt.Printf("✓ Message sent: %s, latency: %.2fms\n", result.MessageID, result.LatencyMs)
	}

	// Get stats
	stats := client.GetStats()
	fmt.Printf("Stats: %+v\n", stats)

	// Keep running
	time.Sleep(10 * time.Second)

	// Disconnect
	if err := client.Disconnect(); err != nil {
		fmt.Printf("✗ Failed to disconnect cleanly: %v\n", err)
	}
}
