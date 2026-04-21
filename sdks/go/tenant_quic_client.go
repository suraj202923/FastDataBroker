package fastdatabroker

import (
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"math/rand"
	"sync"
	"time"
)

// TenantRole defines tenant authorization levels
type TenantRole string

const (
	RoleAdmin   TenantRole = "admin"
	RoleUser    TenantRole = "user"
	RoleService TenantRole = "service"
)

// ConnectionState represents QUIC connection state
type ConnectionState string

const (
	StateIdle        ConnectionState = "idle"
	StateHandshake   ConnectionState = "handshake"
	StateEstablished ConnectionState = "established"
	StateClosing     ConnectionState = "closing"
	StateClosed      ConnectionState = "closed"
)

// TenantConfig holds tenant-specific configuration
type TenantConfig struct {
	TenantID        string
	PSKSecret       string
	ClientID        string
	APIKey          string
	Role            TenantRole
	RateLimitRPS    int
	MaxConnections  int
	CustomHeaders   map[string]string
}

// QuicHandshakeParams holds tenant-specific QUIC handshake parameters
type QuicHandshakeParams struct {
	TenantID            string
	ClientID            string
	TimestampMS         int64
	RandomNonce         string
	PSKToken            string
	InitialMaxStreams   int
	IdleTimeoutMS       int
	SessionToken        string
	ConnectionID        string
}

// TenantQuicClient represents a QUIC client with tenant-specific handshake
type TenantQuicClient struct {
	host                  string
	port                  int
	tenantConfig          *TenantConfig
	connectionState       ConnectionState
	isAuthenticated       bool
	handshakeStartTime    int64
	handshakeDurationMS   int64
	connectionStart       int64
	stats                 map[string]int64
	messageHandlers       map[string]func(*Message)
	connectionID          string
	sessionToken          string
	mu                    sync.RWMutex
}

// NewTenantQuicClient creates a new QUIC client with tenant configuration
func NewTenantQuicClient(host string, port int, tenantConfig *TenantConfig) *TenantQuicClient {
	return &TenantQuicClient{
		host:            host,
		port:            port,
		tenantConfig:    tenantConfig,
		connectionState: StateIdle,
		isAuthenticated: false,
		stats: map[string]int64{
			"messages_sent":      0,
			"messages_received":  0,
			"last_message_time":  0,
			"handshake_attempts": 0,
		},
		messageHandlers: make(map[string]func(*Message)),
	}
}

// generatePSKToken generates a tenant-specific PSK token
func (c *TenantQuicClient) generatePSKToken() string {
	message := fmt.Sprintf("%s:%s:%d",
		c.tenantConfig.TenantID,
		c.tenantConfig.ClientID,
		time.Now().UnixMilli(),
	)

	h := hmac.New(sha256.New, []byte(c.tenantConfig.PSKSecret))
	h.Write([]byte(message))
	return hex.EncodeToString(h.Sum(nil))
}

// createHandshakeParams creates tenant-specific QUIC handshake parameters
func (c *TenantQuicClient) createHandshakeParams() *QuicHandshakeParams {
	timestampMS := time.Now().UnixMilli()
	randomNonce := fmt.Sprintf("%x", sha256.Sum256([]byte(fmt.Sprintf("%d%d", rand.Int63(), timestampMS))))[:32]
	pskToken := c.generatePSKToken()

	return &QuicHandshakeParams{
		TenantID:          c.tenantConfig.TenantID,
		ClientID:          c.tenantConfig.ClientID,
		TimestampMS:       timestampMS,
		RandomNonce:       randomNonce,
		PSKToken:          pskToken,
		InitialMaxStreams: c.tenantConfig.MaxConnections,
		IdleTimeoutMS:     30000,
	}
}

// performTenantQuicHandshake performs the tenant-specific QUIC handshake
func (c *TenantQuicClient) performTenantQuicHandshake(ctx context.Context) error {
	c.mu.Lock()
	c.handshakeStartTime = time.Now().UnixMilli()
	c.connectionState = StateHandshake
	c.mu.Unlock()

	// Create handshake parameters
	params := c.createHandshakeParams()

	// Validate tenant during handshake
	if !c.validateTenantInHandshake(params) {
		return fmt.Errorf("tenant validation failed during handshake")
	}

	// Generate session token and connection ID
	c.sessionToken = c.generateSessionToken(params)
	c.connectionID = c.generateConnectionID(params)

	// Calculate handshake duration
	c.mu.Lock()
	c.handshakeDurationMS = time.Now().UnixMilli() - c.handshakeStartTime
	c.isAuthenticated = true
	c.mu.Unlock()

	return nil
}

// validateTenantInHandshake validates the tenant during QUIC handshake
func (c *TenantQuicClient) validateTenantInHandshake(params *QuicHandshakeParams) bool {
	// Verify tenant ID matches
	if params.TenantID != c.tenantConfig.TenantID {
		return false
	}

	// Verify timestamp is recent (within 60 seconds)
	currentTime := time.Now().UnixMilli()
	if currentTime-params.TimestampMS > 60000 {
		return false
	}

	return true
}

// generateSessionToken generates a post-handshake session token
func (c *TenantQuicClient) generateSessionToken(params *QuicHandshakeParams) string {
	sessionData := fmt.Sprintf("%s:%s:%s:%d",
		params.TenantID,
		params.ClientID,
		params.PSKToken,
		time.Now().UnixMilli(),
	)
	return fmt.Sprintf("%x", sha256.Sum256([]byte(sessionData)))
}

// generateConnectionID generates a unique connection ID for the tenant session
func (c *TenantQuicClient) generateConnectionID(params *QuicHandshakeParams) string {
	connData := fmt.Sprintf("%s:%s:%d:%s",
		params.TenantID,
		params.ClientID,
		params.TimestampMS,
		params.RandomNonce,
	)
	return fmt.Sprintf("%x", sha256.Sum256([]byte(connData)))[:16]
}

// Connect establishes a connection with tenant-specific QUIC handshake
func (c *TenantQuicClient) Connect(ctx context.Context) error {
	c.mu.RLock()
	if c.connectionState == StateEstablished {
		c.mu.RUnlock()
		return nil
	}
	c.mu.RUnlock()

	fmt.Printf("Initiating tenant-specific QUIC handshake for tenant: %s\n", c.tenantConfig.TenantID)

	// Increment handshake attempts
	c.mu.Lock()
	c.stats["handshake_attempts"]++
	c.mu.Unlock()

	// Perform handshake
	if err := c.performTenantQuicHandshake(ctx); err != nil {
		c.mu.Lock()
		c.connectionState = StateClosed
		c.mu.Unlock()
		return err
	}

	// Connection established
	c.mu.Lock()
	c.connectionState = StateEstablished
	c.connectionStart = time.Now().UnixMilli()
	c.mu.Unlock()

	fmt.Printf("✓ Connected to %s:%d\n", c.host, c.port)
	fmt.Printf("  Tenant: %s\n", c.tenantConfig.TenantID)
	fmt.Printf("  Handshake Duration: %dms\n", c.handshakeDurationMS)
	fmt.Printf("  Session Token: %s...\n", c.sessionToken[:16])
	fmt.Printf("  Connection ID: %s\n", c.connectionID)

	return nil
}

// SendMessage sends a message through the tenant-specific QUIC connection
func (c *TenantQuicClient) SendMessage(ctx context.Context, msg *Message) (*DeliveryResult, error) {
	c.mu.RLock()
	if c.connectionState != StateEstablished {
		c.mu.RUnlock()
		return nil, fmt.Errorf("connection not established (state: %s)", c.connectionState)
	}
	if !c.isAuthenticated {
		c.mu.RUnlock()
		return nil, fmt.Errorf("tenant authentication failed")
	}
	c.mu.RUnlock()

	// Add tenant context
	msg.TenantID = c.tenantConfig.TenantID

	// Simulate message sending
	messageID := fmt.Sprintf("msg_%d_%d", time.Now().UnixMilli(), rand.Intn(10000))
	latency := float64((time.Now().UnixNano() % 50000000) + 5000000) / 1000000

	c.mu.Lock()
	c.stats["messages_sent"]++
	c.stats["last_message_time"] = time.Now().UnixMilli()
	c.mu.Unlock()

	return &DeliveryResult{
		MessageID: messageID,
		Status:    "success",
		LatencyMS: latency,
		Timestamp: time.Now().UnixMilli(),
		TenantID:  c.tenantConfig.TenantID,
	}, nil
}

// OnMessage registers a message handler for a topic
func (c *TenantQuicClient) OnMessage(topic string, handler func(*Message)) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.messageHandlers[topic] = handler
}

// OffMessage unregisters a message handler
func (c *TenantQuicClient) OffMessage(topic string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	delete(c.messageHandlers, topic)
}

// GetStats returns current connection statistics
func (c *TenantQuicClient) GetStats() map[string]interface{} {
	c.mu.RLock()
	defer c.mu.RUnlock()

	uptime := int64(0)
	if c.connectionState == StateEstablished {
		uptime = (time.Now().UnixMilli() - c.connectionStart) / 1000
	}

	return map[string]interface{}{
		"is_connected":         c.connectionState == StateEstablished && c.isAuthenticated,
		"messages_sent":        c.stats["messages_sent"],
		"messages_received":    c.stats["messages_received"],
		"uptime_seconds":       uptime,
		"handshake_duration_ms": c.handshakeDurationMS,
		"tenant_id":            c.tenantConfig.TenantID,
		"connection_id":        c.connectionID,
		"session_token":        c.sessionToken[:16] + "...",
	}
}

// IsConnected checks if the connection is established and authenticated
func (c *TenantQuicClient) IsConnected() bool {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.connectionState == StateEstablished && c.isAuthenticated
}

// Disconnect closes the connection
func (c *TenantQuicClient) Disconnect() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.connectionState != StateClosed {
		c.connectionState = StateClosing
		c.connectionState = StateClosed
		c.isAuthenticated = false
		fmt.Printf("✓ Disconnected from %s:%d (Tenant: %s)\n", c.host, c.port, c.tenantConfig.TenantID)
	}

	return nil
}
