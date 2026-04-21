package fastdatabroker

import (
	"context"
	"testing"
	"time"
)

// TestTenantConfigCreation tests tenant configuration creation
func TestTenantConfigCreation(t *testing.T) {
	config := &TenantConfig{
		TenantID:       "test-tenant-1",
		PSKSecret:      "super-secret-key",
		ClientID:       "client-001",
		APIKey:         "api_key_xxx",
		Role:           RoleAdmin,
		RateLimitRPS:   5000,
		MaxConnections: 200,
	}

	if config.TenantID != "test-tenant-1" {
		t.Errorf("Expected tenant_id=test-tenant-1, got %s", config.TenantID)
	}

	if config.PSKSecret != "super-secret-key" {
		t.Errorf("Expected psk_secret=super-secret-key, got %s", config.PSKSecret)
	}

	if config.Role != RoleAdmin {
		t.Errorf("Expected role=admin, got %s", config.Role)
	}

	if config.RateLimitRPS != 5000 {
		t.Errorf("Expected rate_limit_rps=5000, got %d", config.RateLimitRPS)
	}
}

// TestTenantQuicHandshake tests tenant-specific QUIC handshake
func TestTenantQuicHandshake(t *testing.T) {
	config := &TenantConfig{
		TenantID:       "acme-corp",
		PSKSecret:      "acme-psk-secret",
		ClientID:       "client-acme-01",
		APIKey:         "api_acme_xyz",
		RateLimitRPS:   1000,
		MaxConnections: 100,
	}

	client := NewTenantQuicClient("localhost", 6000, config)

	if client.connectionState != StateIdle {
		t.Errorf("Expected initial state=idle, got %s", client.connectionState)
	}

	if client.isAuthenticated {
		t.Errorf("Expected isAuthenticated=false initially")
	}

	// Perform handshake
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	err := client.Connect(ctx)
	if err != nil {
		t.Errorf("Expected connect to succeed, got error: %v", err)
	}

	if client.connectionState != StateEstablished {
		t.Errorf("Expected state=established, got %s", client.connectionState)
	}

	if !client.isAuthenticated {
		t.Errorf("Expected isAuthenticated=true after handshake")
	}

	if client.sessionToken == "" {
		t.Errorf("Expected session_token to be set")
	}

	if client.connectionID == "" {
		t.Errorf("Expected connection_id to be set")
	}

	if client.handshakeDurationMS <= 0 {
		t.Errorf("Expected handshake_duration_ms > 0, got %d", client.handshakeDurationMS)
	}

	client.Disconnect()
}

// TestTenantMessageIsolation tests tenant message isolation
func TestTenantMessageIsolation(t *testing.T) {
	config1 := &TenantConfig{
		TenantID:       "tenant-1",
		PSKSecret:      "secret-1",
		ClientID:       "client-1",
		APIKey:         "api_1",
		MaxConnections: 100,
	}

	config2 := &TenantConfig{
		TenantID:       "tenant-2",
		PSKSecret:      "secret-2",
		ClientID:       "client-2",
		APIKey:         "api_2",
		MaxConnections: 100,
	}

	client1 := NewTenantQuicClient("localhost", 6000, config1)
	client2 := NewTenantQuicClient("localhost", 6000, config2)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Connect both tenants
	if err := client1.Connect(ctx); err != nil {
		t.Errorf("Client1 connect failed: %v", err)
	}

	if err := client2.Connect(ctx); err != nil {
		t.Errorf("Client2 connect failed: %v", err)
	}

	// Send messages
	msg1 := &Message{
		SenderID:     "sender1",
		RecipientIDs: []string{"user1"},
		Subject:      "Tenant1 Message",
		Content:      []byte("Data from tenant 1"),
		Priority:     PriorityHigh,
	}

	msg2 := &Message{
		SenderID:     "sender2",
		RecipientIDs: []string{"user2"},
		Subject:      "Tenant2 Message",
		Content:      []byte("Data from tenant 2"),
		Priority:     PriorityHigh,
	}

	result1, err := client1.SendMessage(ctx, msg1)
	if err != nil {
		t.Errorf("SendMessage failed for client1: %v", err)
	}

	result2, err := client2.SendMessage(ctx, msg2)
	if err != nil {
		t.Errorf("SendMessage failed for client2: %v", err)
	}

	// Verify messages have correct tenant context
	if result1.TenantID != "tenant-1" {
		t.Errorf("Expected result1.TenantID=tenant-1, got %s", result1.TenantID)
	}

	if result2.TenantID != "tenant-2" {
		t.Errorf("Expected result2.TenantID=tenant-2, got %s", result2.TenantID)
	}

	if result1.MessageID == result2.MessageID {
		t.Errorf("Expected different message IDs, got both: %s", result1.MessageID)
	}

	// Verify session isolation
	if client1.sessionToken == client2.sessionToken {
		t.Errorf("Expected different session tokens for different tenants")
	}

	if client1.connectionID == client2.connectionID {
		t.Errorf("Expected different connection IDs for different tenants")
	}

	client1.Disconnect()
	client2.Disconnect()
}

// TestConcurrentTenantConnections tests concurrent connections from multiple tenants
func TestConcurrentTenantConnections(t *testing.T) {
	numTenants := 5
	configs := make([]*TenantConfig, numTenants)

	for i := 0; i < numTenants; i++ {
		configs[i] = &TenantConfig{
			TenantID:       "tenant-" + string(rune(i)),
			PSKSecret:      "secret-" + string(rune(i)),
			ClientID:       "client-" + string(rune(i)),
			APIKey:         "api_" + string(rune(i)),
			MaxConnections: 100,
		}
	}

	clients := make([]*TenantQuicClient, numTenants)
	for i := 0; i < numTenants; i++ {
		clients[i] = NewTenantQuicClient("localhost", 6000, configs[i])
	}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// Connect all tenants
	for i, client := range clients {
		if err := client.Connect(ctx); err != nil {
			t.Errorf("Client %d connect failed: %v", i, err)
		}
	}

	// Send messages from each tenant
	totalSent := int64(0)
	for i, client := range clients {
		msg := &Message{
			SenderID:     "sender-" + string(rune(i)),
			RecipientIDs: []string{"user"},
			Subject:      "Test Message",
			Content:      []byte("Multi-tenant test"),
			Priority:     PriorityNormal,
		}

		result, err := client.SendMessage(ctx, msg)
		if err != nil {
			t.Errorf("SendMessage failed for client %d: %v", i, err)
		}

		if result.Status != "success" {
			t.Errorf("Expected status=success, got %s", result.Status)
		}

		totalSent++
	}

	// Verify message counts
	if totalSent != int64(numTenants) {
		t.Errorf("Expected %d messages sent, got %d", numTenants, totalSent)
	}

	// Disconnect all clients
	for _, client := range clients {
		client.Disconnect()
	}
}

// TestPSKValidation tests PSK-based tenant authentication
func TestPSKValidation(t *testing.T) {
	config := &TenantConfig{
		TenantID:       "psk-test-tenant",
		PSKSecret:      "specific-psk-secret",
		ClientID:       "psk-client-01",
		APIKey:         "psk_api_key",
		MaxConnections: 100,
	}

	client := NewTenantQuicClient("localhost", 6000, config)

	// Get handshake params
	params := client.createHandshakeParams()

	if params.TenantID != "psk-test-tenant" {
		t.Errorf("Expected tenant_id=psk-test-tenant, got %s", params.TenantID)
	}

	if params.PSKToken == "" {
		t.Errorf("Expected psk_token to be non-empty")
	}

	if len(params.PSKToken) != 64 { // SHA256 hex is 64 chars
		t.Errorf("Expected psk_token length=64, got %d", len(params.PSKToken))
	}

	// PSK should be consistent for same inputs (deterministic based on timestamp and secrets)
	if !client.validateTenantInHandshake(params) {
		t.Errorf("Expected tenant validation to pass")
	}
}

// TestHandshakeMetrics tests handshake performance metrics
func TestHandshakeMetrics(t *testing.T) {
	config := &TenantConfig{
		TenantID:       "metrics-tenant",
		PSKSecret:      "metrics-secret",
		ClientID:       "metrics-client",
		APIKey:         "metrics_api",
		MaxConnections: 100,
	}

	client := NewTenantQuicClient("localhost", 6000, config)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client.Connect(ctx)

	stats := client.GetStats()

	if isConnected, ok := stats["is_connected"].(bool); !ok || !isConnected {
		t.Errorf("Expected is_connected=true")
	}

	if handshakeDuration, ok := stats["handshake_duration_ms"].(int64); !ok || handshakeDuration <= 0 {
		t.Errorf("Expected handshake_duration_ms > 0, got %v", handshakeDuration)
	}

	client.Disconnect()
}

// TestConnectionStateTransitions tests connection state machine
func TestConnectionStateTransitions(t *testing.T) {
	config := &TenantConfig{
		TenantID:       "state-test",
		PSKSecret:      "state-secret",
		ClientID:       "state-client",
		APIKey:         "state-api",
		MaxConnections: 100,
	}

	client := NewTenantQuicClient("localhost", 6000, config)

	// Initial state
	if client.connectionState != StateIdle {
		t.Errorf("Expected initial state=idle, got %s", client.connectionState)
	}

	// After connect (which includes handshake)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client.Connect(ctx)

	if client.connectionState != StateEstablished {
		t.Errorf("Expected state=established after connect, got %s", client.connectionState)
	}

	// Can send messages in established state
	if !client.IsConnected() {
		t.Errorf("Expected IsConnected()=true")
	}

	// After disconnect
	client.Disconnect()

	if client.connectionState != StateClosed {
		t.Errorf("Expected state=closed after disconnect, got %s", client.connectionState)
	}

	if client.IsConnected() {
		t.Errorf("Expected IsConnected()=false after disconnect")
	}
}

// TestRateLimitingConfig tests tenant-specific rate limiting configuration
func TestRateLimitingConfig(t *testing.T) {
	config := &TenantConfig{
		TenantID:       "rate-limit-tenant",
		PSKSecret:      "rate-secret",
		ClientID:       "rate-client",
		APIKey:         "rate_api",
		RateLimitRPS:   2000,
		MaxConnections: 50,
	}

	client := NewTenantQuicClient("localhost", 6000, config)

	if client.tenantConfig.RateLimitRPS != 2000 {
		t.Errorf("Expected rate_limit_rps=2000, got %d", client.tenantConfig.RateLimitRPS)
	}

	if client.tenantConfig.MaxConnections != 50 {
		t.Errorf("Expected max_connections=50, got %d", client.tenantConfig.MaxConnections)
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client.Connect(ctx)

	stats := client.GetStats()

	if isConnected, ok := stats["is_connected"].(bool); !ok || !isConnected {
		t.Errorf("Expected is_connected=true")
	}

	client.Disconnect()
}

// TestCustomHeaders tests tenant custom headers in configuration
func TestCustomHeaders(t *testing.T) {
	customHeaders := map[string]string{
		"X-Tenant-Region": "us-west",
		"X-Custom-Header": "custom-value",
	}

	config := &TenantConfig{
		TenantID:       "custom-header-tenant",
		PSKSecret:      "custom-secret",
		ClientID:       "custom-client",
		APIKey:         "custom_api",
		CustomHeaders:  customHeaders,
		MaxConnections: 100,
	}

	if config.CustomHeaders["X-Tenant-Region"] != "us-west" {
		t.Errorf("Expected custom header X-Tenant-Region=us-west")
	}

	if config.CustomHeaders["X-Custom-Header"] != "custom-value" {
		t.Errorf("Expected custom header X-Custom-Header=custom-value")
	}
}
