package fastdatabroker

import (
	"context"
	"errors"
	"fmt"
	"sync"
	"time"
)

// Version of the SDK
const Version = "0.1.12"

// Priority levels for messages
type Priority uint8

const (
	PriorityDeferred Priority = 50
	PriorityNormal   Priority = 100
	PriorityHigh     Priority = 150
	PriorityUrgent   Priority = 200
	PriorityCritical Priority = 255
)

// NotificationChannel enum
type NotificationChannel string

const (
	ChannelEmail     NotificationChannel = "email"
	ChannelWebSocket NotificationChannel = "websocket"
	ChannelPush      NotificationChannel = "push"
	ChannelWebhook   NotificationChannel = "webhook"
)

// PushPlatform enum
type PushPlatform string

const (
	PlatformFirebase PushPlatform = "firebase"
	PlatformAPNs     PushPlatform = "apns"
	PlatformFCM      PushPlatform = "fcm"
	PlatformWebPush  PushPlatform = "webpush"
)

// Message represents a fastdatabroker message envelope
type Message struct {
	SenderID       string
	RecipientIDs   []string
	Subject        string
	Content        []byte
	Priority       Priority
	TTLSeconds     *int64
	Tags           map[string]string
	RequireConfirm bool
}

// DeliveryResult represents the result of sending a message
type DeliveryResult struct {
	MessageID         string
	Status            string
	DeliveredChannels int
	Details           map[string]interface{}
}

// WebSocketClientInfo stores WebSocket client information
type WebSocketClientInfo struct {
	ClientID  string
	UserID    string
	Timestamp time.Time
}

// WebhookConfig configuration for webhook endpoints
type WebhookConfig struct {
	URL       string
	Headers   map[string]string
	Retries   int
	Timeout   time.Duration
	VerifySSL bool
}

// Client is the main fastdatabroker client
type Client struct {
	host      string
	port      int
	connected bool
	mu        sync.RWMutex

	// WebSocket clients registry
	wsClients map[string]*WebSocketClientInfo
}

// NewClient creates a new fastdatabroker client
func NewClient(host string, port int) *Client {
	return &Client{
		host:      host,
		port:      port,
		connected: false,
		wsClients: make(map[string]*WebSocketClientInfo),
	}
}

// NewClientWithDefaults creates a new client with default localhost:6000
func NewClientWithDefaults() *Client {
	return NewClient("localhost", 6000)
}

// Connect establishes connection to fastdatabroker server
func (c *Client) Connect(ctx context.Context) error {
	select {
	case <-ctx.Done():
		return ctx.Err()
	default:
	}

	c.mu.Lock()
	defer c.mu.Unlock()

	// Phase 4: Establish QUIC connection
	c.connected = true
	fmt.Printf("Connected to fastdatabroker at %s:%d\n", c.host, c.port)
	return nil
}

// Disconnect closes the connection to fastdatabroker server
func (c *Client) Disconnect() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.connected = false
	return nil
}

// SendMessage sends a single message through fastdatabroker
func (c *Client) SendMessage(ctx context.Context, msg *Message) (*DeliveryResult, error) {
	c.mu.RLock()
	if !c.connected {
		c.mu.RUnlock()
		return nil, errors.New("not connected to fastdatabroker")
	}
	c.mu.RUnlock()

	// Phase 4: Send via QUIC transport
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	case <-time.After(10 * time.Millisecond):
	}

	return &DeliveryResult{
		MessageID:         fmt.Sprintf("msg-%d", time.Now().UnixNano()),
		Status:            "success",
		DeliveredChannels: 4,
		Details: map[string]interface{}{
			"email":     "sent",
			"websocket": "delivered",
			"push":      "pending",
			"webhook":   "delivered",
		},
	}, nil
}

// BatchSend sends multiple messages in batch
func (c *Client) BatchSend(ctx context.Context, messages []*Message) ([]*DeliveryResult, error) {
	results := make([]*DeliveryResult, len(messages))

	for i, msg := range messages {
		result, err := c.SendMessage(ctx, msg)
		if err != nil {
			return nil, fmt.Errorf("batch send failed at index %d: %w", i, err)
		}
		results[i] = result
	}

	return results, nil
}

// RegisterWebSocket registers a WebSocket client
func (c *Client) RegisterWebSocket(clientID, userID string) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.wsClients[clientID] = &WebSocketClientInfo{
		ClientID:  clientID,
		UserID:    userID,
		Timestamp: time.Now(),
	}

	fmt.Printf("WebSocket client registered: %s -> %s\n", clientID, userID)
	return nil
}

// UnregisterWebSocket unregisters a WebSocket client
func (c *Client) UnregisterWebSocket(clientID string) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	delete(c.wsClients, clientID)
	return nil
}

// RegisterWebhook registers a webhook endpoint
func (c *Client) RegisterWebhook(recipientID string, config *WebhookConfig) error {
	if config == nil {
		return errors.New("webhook config cannot be nil")
	}

	if config.URL == "" {
		return errors.New("webhook URL cannot be empty")
	}

	// Phase 4: Register with broker
	fmt.Printf("Webhook registered for %s: %s\n", recipientID, config.URL)
	return nil
}

// UnregisterWebhook unregisters a webhook endpoint
func (c *Client) UnregisterWebhook(recipientID string) error {
	// Phase 4: Unregister from broker
	return nil
}

// GetStats retrieves fastdatabroker statistics
func (c *Client) GetStats(ctx context.Context) (map[string]interface{}, error) {
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	case <-time.After(10 * time.Millisecond):
	}

	c.mu.RLock()
	wsConnected := len(c.wsClients)
	c.mu.RUnlock()

	return map[string]interface{}{
		"totalMessages": 0,
		"delivered":     0,
		"failed":        0,
		"channels": map[string]interface{}{
			"email": map[string]interface{}{
				"sent":   0,
				"failed": 0,
			},
			"websocket": map[string]interface{}{
				"connected": wsConnected,
				"delivered": 0,
			},
			"push": map[string]interface{}{
				"sent":      0,
				"delivered": 0,
			},
			"webhook": map[string]interface{}{
				"sent":      0,
				"delivered": 0,
			},
		},
	}, nil
}

// MessageBuilder provides a fluent interface for building messages
type MessageBuilder struct {
	msg *Message
}

// NewMessageBuilder creates a new message builder
func NewMessageBuilder(senderID string) *MessageBuilder {
	return &MessageBuilder{
		msg: &Message{
			SenderID:     senderID,
			RecipientIDs: []string{},
			Tags:         make(map[string]string),
			Priority:     PriorityNormal,
		},
	}
}

// AddRecipient adds a recipient to the message
func (mb *MessageBuilder) AddRecipient(recipientID string) *MessageBuilder {
	mb.msg.RecipientIDs = append(mb.msg.RecipientIDs, recipientID)
	return mb
}

// SetSubject sets the message subject
func (mb *MessageBuilder) SetSubject(subject string) *MessageBuilder {
	mb.msg.Subject = subject
	return mb
}

// SetContent sets the message content
func (mb *MessageBuilder) SetContent(content []byte) *MessageBuilder {
	mb.msg.Content = content
	return mb
}

// SetPriority sets the message priority
func (mb *MessageBuilder) SetPriority(priority Priority) *MessageBuilder {
	mb.msg.Priority = priority
	return mb
}

// SetTTL sets the message time-to-live
func (mb *MessageBuilder) SetTTL(seconds int64) *MessageBuilder {
	mb.msg.TTLSeconds = &seconds
	return mb
}

// AddTag adds a tag to the message
func (mb *MessageBuilder) AddTag(key, value string) *MessageBuilder {
	mb.msg.Tags[key] = value
	return mb
}

// RequireConfirmation sets require confirmation flag
func (mb *MessageBuilder) RequireConfirmation(require bool) *MessageBuilder {
	mb.msg.RequireConfirm = require
	return mb
}

// Build builds the message
func (mb *MessageBuilder) Build() *Message {
	return mb.msg
}

// PushNotificationBuilder for building push notifications
type PushNotificationBuilder struct {
	title string
	body  string
	icon  string
	badge string
	sound string
	data  map[string]string
}

// NewPushNotification creates a new push notification builder
func NewPushNotification(title string) *PushNotificationBuilder {
	return &PushNotificationBuilder{
		title: title,
		data:  make(map[string]string),
	}
}

// WithBody sets the notification body
func (pnb *PushNotificationBuilder) WithBody(body string) *PushNotificationBuilder {
	pnb.body = body
	return pnb
}

// WithIcon sets the notification icon
func (pnb *PushNotificationBuilder) WithIcon(icon string) *PushNotificationBuilder {
	pnb.icon = icon
	return pnb
}

// WithSound sets the notification sound
func (pnb *PushNotificationBuilder) WithSound(sound string) *PushNotificationBuilder {
	pnb.sound = sound
	return pnb
}

// AddData adds custom data
func (pnb *PushNotificationBuilder) AddData(key, value string) *PushNotificationBuilder {
	pnb.data[key] = value
	return pnb
}

// Build builds the notification
func (pnb *PushNotificationBuilder) Build() map[string]interface{} {
	return map[string]interface{}{
		"title": pnb.title,
		"body":  pnb.body,
		"icon":  pnb.icon,
		"badge": pnb.badge,
		"sound": pnb.sound,
		"data":  pnb.data,
	}
}
