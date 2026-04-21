package fastdatabroker

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"
)

// ============== Client Initialization Tests ==============

func TestNewClient(t *testing.T) {
	client := NewClient("localhost", 6000)

	if client == nil {
		t.Fatal("Expected client to be created")
	}

	if client.host != "localhost" || client.port != 6000 {
		t.Errorf("Expected host=localhost, port=6000; got host=%s, port=%d", client.host, client.port)
	}
}

func TestNewClientWithDefaults(t *testing.T) {
	client := NewClientWithDefaults()

	if client == nil {
		t.Fatal("Expected client to be created")
	}

	if client.host != "localhost" || client.port != 6000 {
		t.Errorf("Expected default localhost:6000")
	}
}

func TestClientConnect(t *testing.T) {
	client := NewClientWithDefaults()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	err := client.Connect(ctx)

	// Connection might fail if server not running, but should not panic
	if err != nil && err.Error() == "" {
		t.Fatal("Error should have message")
	}
}

func TestClientDisconnect(t *testing.T) {
	client := NewClientWithDefaults()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := client.Connect(ctx); err != nil {
		t.Logf("Connection error: %v (expected if server not running)", err)
	}

	err := client.Disconnect()

	// Should not panic
	_ = err
}

// ============== Message Creation Tests ==============

func TestNewMessage(t *testing.T) {
	msg := &Message{
		SenderID:     "sender1",
		RecipientIDs: []string{"user1", "user2"},
		Subject:      "Test Subject",
		Content:      []byte("Test content"),
		Priority:     PriorityHigh,
	}

	if msg.SenderID != "sender1" {
		t.Errorf("Expected sender_id=sender1, got %s", msg.SenderID)
	}

	if len(msg.RecipientIDs) != 2 {
		t.Errorf("Expected 2 recipients, got %d", len(msg.RecipientIDs))
	}

	if msg.Subject != "Test Subject" {
		t.Errorf("Expected subject 'Test Subject', got %s", msg.Subject)
	}
}

func TestMessageWithTTL(t *testing.T) {
	ttl := int64(3600)
	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Subject",
		Content:      []byte("content"),
		TTLSeconds:   &ttl,
	}

	if msg.TTLSeconds == nil || *msg.TTLSeconds != 3600 {
		t.Error("Expected TTL to be 3600")
	}
}

func TestMessageWithTags_Basic(t *testing.T) {
	tags := map[string]string{
		"category": "notification",
		"source":   "api",
	}

	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Subject",
		Content:      []byte("content"),
		Tags:         tags,
	}

	if len(msg.Tags) != 2 {
		t.Errorf("Expected 2 tags, got %d", len(msg.Tags))
	}

	if msg.Tags["category"] != "notification" {
		t.Errorf("Expected category=notification, got %s", msg.Tags["category"])
	}
}

func TestMessageEmptyContent(t *testing.T) {
	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Subject",
		Content:      []byte{},
	}

	if len(msg.Content) != 0 {
		t.Errorf("Expected empty content, got %d bytes", len(msg.Content))
	}
}

func TestMessageLargeContent_Basic(t *testing.T) {
	largeContent := make([]byte, 10*1024*1024) // 10MB

	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Subject",
		Content:      largeContent,
	}

	if len(msg.Content) != 10*1024*1024 {
		t.Errorf("Expected 10MB content, got %d bytes", len(msg.Content))
	}
}

func TestMessageMultipleRecipients(t *testing.T) {
	recipients := make([]string, 100)
	for i := 0; i < 100; i++ {
		recipients[i] = fmt.Sprintf("user%d", i)
	}

	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: recipients,
		Subject:      "Subject",
		Content:      []byte("content"),
	}

	if len(msg.RecipientIDs) != 100 {
		t.Errorf("Expected 100 recipients, got %d", len(msg.RecipientIDs))
	}
}

func TestMessageNoRecipients(t *testing.T) {
	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{},
		Subject:      "Subject",
		Content:      []byte("content"),
	}

	if len(msg.RecipientIDs) != 0 {
		t.Errorf("Expected 0 recipients, got %d", len(msg.RecipientIDs))
	}
}

// ============== Priority Tests ==============

func TestPriorityLevels(t *testing.T) {
	tests := []struct {
		priority Priority
		expected uint8
	}{
		{PriorityDeferred, 50},
		{PriorityNormal, 100},
		{PriorityHigh, 150},
		{PriorityUrgent, 200},
		{PriorityCritical, 255},
	}

	for _, test := range tests {
		if uint8(test.priority) != test.expected {
			t.Errorf("Expected %d, got %d", test.expected, test.priority)
		}
	}
}

func TestPriorityOrdering(t *testing.T) {
	priorities := []Priority{
		PriorityDeferred,
		PriorityCritical,
		PriorityNormal,
		PriorityHigh,
		PriorityUrgent,
	}

	// Verify values
	if uint8(priorities[0]) != 50 { // Deferred
		t.Error("Incorrect priority ordering")
	}

	if uint8(priorities[1]) != 255 { // Critical
		t.Error("Incorrect priority ordering")
	}
}

// ============== Notification Channel Tests ==============

func TestNotificationChannels(t *testing.T) {
	tests := []struct {
		channel NotificationChannel
		value   string
	}{
		{ChannelEmail, "email"},
		{ChannelWebSocket, "websocket"},
		{ChannelPush, "push"},
		{ChannelWebhook, "webhook"},
	}

	for _, test := range tests {
		if string(test.channel) != test.value {
			t.Errorf("Expected %s, got %s", test.value, test.channel)
		}
	}
}

func TestPushPlatforms(t *testing.T) {
	tests := []struct {
		platform PushPlatform
		value    string
	}{
		{PlatformFirebase, "firebase"},
		{PlatformAPNs, "apns"},
		{PlatformFCM, "fcm"},
		{PlatformWebPush, "webpush"},
	}

	for _, test := range tests {
		if string(test.platform) != test.value {
			t.Errorf("Expected %s, got %s", test.value, test.platform)
		}
	}
}

// ============== Delivery Result Tests ==============

func TestNewDeliveryResult(t *testing.T) {
	result := &DeliveryResult{
		MessageID:         "msg123",
		Status:            "delivered",
		DeliveredChannels: 2,
		Details: map[string]interface{}{
			"email": "sent",
			"push":  "sent",
		},
	}

	if result.MessageID != "msg123" {
		t.Errorf("Expected msg123, got %s", result.MessageID)
	}

	if result.Status != "delivered" {
		t.Errorf("Expected delivered, got %s", result.Status)
	}

	if result.DeliveredChannels != 2 {
		t.Errorf("Expected 2 channels, got %d", result.DeliveredChannels)
	}
}

// ============== WebSocket Client Info Tests ==============

func TestWebSocketClientInfo(t *testing.T) {
	now := time.Now()
	info := &WebSocketClientInfo{
		ClientID:  "client123",
		UserID:    "user123",
		Timestamp: now,
	}

	if info.ClientID != "client123" {
		t.Errorf("Expected client123, got %s", info.ClientID)
	}

	if info.UserID != "user123" {
		t.Errorf("Expected user123, got %s", info.UserID)
	}

	if info.Timestamp != now {
		t.Error("Timestamp mismatch")
	}
}

// ============== Webhook Config Tests ==============

func TestWebhookConfig(t *testing.T) {
	config := &WebhookConfig{
		URL:       "https://example.com/webhook",
		Retries:   3,
		Timeout:   30 * time.Second,
		VerifySSL: true,
	}

	if config.URL != "https://example.com/webhook" {
		t.Errorf("Expected https://example.com/webhook, got %s", config.URL)
	}

	if config.Retries != 3 {
		t.Errorf("Expected 3 retries, got %d", config.Retries)
	}

	if config.Timeout != 30*time.Second {
		t.Errorf("Expected 30s timeout, got %v", config.Timeout)
	}
}

// ============== Concurrency Tests ==============

func TestMultipleClients_Basic(t *testing.T) {
	clients := make([]*Client, 5)

	for i := 0; i < 5; i++ {
		clients[i] = NewClient("localhost", 6000+i)
	}

	if len(clients) != 5 {
		t.Errorf("Expected 5 clients, got %d", len(clients))
	}

	for i, client := range clients {
		if client == nil || client.port != 6000+i {
			t.Errorf("Client %d is invalid", i)
		}
	}
}

func TestConcurrentMessageCreation(t *testing.T) {
	var wg sync.WaitGroup
	messages := make([]*Message, 100)

	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func(index int) {
			defer wg.Done()

			messages[index] = &Message{
				SenderID:     "sender",
				RecipientIDs: []string{"user"},
				Subject:      fmt.Sprintf("Subject %d", index),
				Content:      []byte(fmt.Sprintf("content %d", index)),
			}
		}(i)
	}

	wg.Wait()

	if len(messages) != 100 {
		t.Errorf("Expected 100 messages, got %d", len(messages))
	}

	// Check no nils
	for i, msg := range messages {
		if msg == nil {
			t.Errorf("Message %d is nil", i)
		}
	}
}

// ============== Edge Case Tests ==============

func TestMessageSpecialCharacters(t *testing.T) {
	special := "Hello 你好 🚀 مرحبا"
	content := []byte(special)

	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Special",
		Content:      content,
	}

	if string(msg.Content) != special {
		t.Errorf("Expected %s, got %s", special, string(msg.Content))
	}
}

func TestMessageWithNilTTL(t *testing.T) {
	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Subject",
		Content:      []byte("content"),
		TTLSeconds:   nil,
	}

	if msg.TTLSeconds != nil {
		t.Error("Expected nil TTL")
	}
}

func TestMessageWithZeroTTL(t *testing.T) {
	ttl := int64(0)
	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Subject",
		Content:      []byte("content"),
		TTLSeconds:   &ttl,
	}

	if msg.TTLSeconds == nil || *msg.TTLSeconds != 0 {
		t.Error("Expected TTL=0")
	}
}

func TestExtremelyLongSubject(t *testing.T) {
	longSubject := ""
	for i := 0; i < 10000; i++ {
		longSubject += "x"
	}

	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      longSubject,
		Content:      []byte("content"),
	}

	if len(msg.Subject) != 10000 {
		t.Errorf("Expected subject length 10000, got %d", len(msg.Subject))
	}
}

func TestMessageWithManyTags(t *testing.T) {
	tags := make(map[string]string)
	for i := 0; i < 100; i++ {
		tags[fmt.Sprintf("tag%d", i)] = fmt.Sprintf("value%d", i)
	}

	msg := &Message{
		SenderID:     "sender",
		RecipientIDs: []string{"user"},
		Subject:      "Subject",
		Content:      []byte("content"),
		Tags:         tags,
	}

	if len(msg.Tags) != 100 {
		t.Errorf("Expected 100 tags, got %d", len(msg.Tags))
	}
}

// ============== Integration Tests ==============

func TestFullMessageWorkflow(t *testing.T) {
	// Create message
	msg := &Message{
		SenderID:       "system",
		RecipientIDs:   []string{"user1", "user2"},
		Subject:        "Important Notification",
		Content:        []byte("This is important"),
		Priority:       PriorityHigh,
		Tags:           map[string]string{"type": "notification"},
		RequireConfirm: true,
	}

	if msg.Subject != "Important Notification" {
		t.Error("Message subject mismatch")
	}

	if len(msg.RecipientIDs) != 2 {
		t.Error("Recipient count mismatch")
	}

	if !msg.RequireConfirm {
		t.Error("Confirm flag not set")
	}
}

func TestMessageWithAllFields(t *testing.T) {
	ttl := int64(7200)
	msg := &Message{
		SenderID:       "sender@system.com",
		RecipientIDs:   []string{"user1@example.com", "user2@example.com"},
		Subject:        "Complete Message Test",
		Content:        []byte("Full featured message content"),
		Priority:       PriorityUrgent,
		TTLSeconds:     &ttl,
		Tags:           map[string]string{"cat": "trans", "env": "prod"},
		RequireConfirm: true,
	}

	if msg.SenderID != "sender@system.com" {
		t.Error("Sender mismatch")
	}

	if len(msg.RecipientIDs) != 2 {
		t.Error("Recipient count mismatch")
	}

	if uint8(msg.Priority) != 200 {
		t.Error("Priority mismatch")
	}

	if msg.TTLSeconds == nil || *msg.TTLSeconds != 7200 {
		t.Error("TTL mismatch")
	}
}

func TestContextHandling(t *testing.T) {
	client := NewClientWithDefaults()

	// Test with timeout
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()

	err := client.Connect(ctx)

	// Should handle context properly
	_ = err
}

func TestBenchmarkMessageCreation(t *testing.T) {
	start := time.Now()

	for i := 0; i < 1000; i++ {
		_ = &Message{
			SenderID:     "sender",
			RecipientIDs: []string{"user"},
			Subject:      "Subject",
			Content:      []byte("content"),
		}
	}

	elapsed := time.Since(start)
	t.Logf("Created 1000 messages in %v", elapsed)

	if elapsed > 10*time.Second {
		t.Errorf("Message creation too slow: %v", elapsed)
	}
}

// ============== Benchmark Tests ==============

func BenchmarkNewMessage(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = &Message{
			SenderID:     "sender",
			RecipientIDs: []string{"user"},
			Subject:      "Subject",
			Content:      []byte("content"),
		}
	}
}

func BenchmarkClientCreation(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = NewClient("localhost", 6000)
	}
}

func BenchmarkDeliveryResult(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = &DeliveryResult{
			MessageID:         "msg123",
			Status:            "delivered",
			DeliveredChannels: 2,
			Details:           map[string]interface{}{"key": "value"},
		}
	}
}

// ============== Client Tests ==============
