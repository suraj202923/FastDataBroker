package multitenancy

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

const MultiTenancyVersion = "0.1.16"

// Priority represents message priority
type Priority uint8

const (
	PriorityDeferred Priority = 50
	PriorityNormal   Priority = 100
	PriorityHigh     Priority = 150
	PriorityUrgent   Priority = 200
	PriorityCritical Priority = 255
)

// NotificationChannel represents delivery channels
type NotificationChannel int

const (
	ChannelEmail NotificationChannel = iota
	ChannelWebSocket
	ChannelPush
	ChannelWebhook
)

// TenantConfig represents tenant-specific configuration
type TenantConfig struct {
	TenantID       string                 `json:"tenant_id"`
	TenantName     string                 `json:"tenant_name"`
	APIKeyPrefix   string                 `json:"api_key_prefix"`
	RateLimitRps   uint32                 `json:"rate_limit_rps"`
	MaxConnections uint32                 `json:"max_connections"`
	MaxMessageSize uint64                 `json:"max_message_size"`
	RetentionDays  uint32                 `json:"retention_days"`
	Enabled        bool                   `json:"enabled"`
	Metadata       map[string]interface{} `json:"metadata,omitempty"`
}

// Validate checks if tenant configuration is valid
func (tc *TenantConfig) Validate() error {
	if tc.TenantID == "" {
		return fmt.Errorf("TenantID cannot be empty")
	}
	if tc.APIKeyPrefix == "" || !strings.HasSuffix(tc.APIKeyPrefix, "_") {
		return fmt.Errorf("APIKeyPrefix must end with '_'")
	}
	if tc.RateLimitRps == 0 {
		return fmt.Errorf("RateLimitRps must be greater than 0")
	}
	if tc.MaxConnections == 0 {
		return fmt.Errorf("MaxConnections must be greater than 0")
	}
	return nil
}

// ServerConfig represents server configuration
type ServerConfig struct {
	BindAddress string `json:"bind_address"`
	Port        uint16 `json:"port"`
	EnableTLS   bool   `json:"enable_tls"`
	CertPath    string `json:"cert_path"`
	KeyPath     string `json:"key_path"`
}

// AppConfig represents application configuration
type AppConfig struct {
	Name        string `json:"name"`
	Version     string `json:"version"`
	Environment string `json:"environment"`
}

// AppSettings represents complete application settings
type AppSettings struct {
	App     AppConfig      `json:"app"`
	Server  ServerConfig   `json:"server"`
	Tenants []TenantConfig `json:"tenants"`
}

// LoadFromFile loads AppSettings from JSON file with environment overrides
func LoadFromFile(filePath string, environment string) (*AppSettings, error) {
	if _, err := os.Stat(filePath); os.IsNotExist(err) {
		return nil, fmt.Errorf("configuration file not found: %s", filePath)
	}

	data, err := os.ReadFile(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	var settings AppSettings
	if err := json.Unmarshal(data, &settings); err != nil {
		return nil, fmt.Errorf("failed to parse config file: %w", err)
	}

	// Try to load environment-specific config
	envFile := filepath.Join(
		filepath.Dir(filePath),
		strings.TrimSuffix(filepath.Base(filePath), filepath.Ext(filePath))+"."+environment+".json",
	)

	if _, err := os.Stat(envFile); err == nil {
		envData, err := os.ReadFile(envFile)
		if err == nil {
			var envSettings AppSettings
			if err := json.Unmarshal(envData, &envSettings); err == nil {
				if envSettings.App != (AppConfig{}) {
					settings.App = envSettings.App
				}
				if envSettings.Server != (ServerConfig{}) {
					settings.Server = envSettings.Server
				}
				if len(envSettings.Tenants) > 0 {
					for _, tenant := range envSettings.Tenants {
						found := false
						for _, existing := range settings.Tenants {
							if existing.TenantID == tenant.TenantID {
								found = true
								break
							}
						}
						if !found {
							settings.Tenants = append(settings.Tenants, tenant)
						}
					}
				}
			}
		}
	}

	return &settings, nil
}

// GetTenant retrieves tenant by ID
func (as *AppSettings) GetTenant(tenantID string) *TenantConfig {
	for i := range as.Tenants {
		if as.Tenants[i].TenantID == tenantID {
			return &as.Tenants[i]
		}
	}
	return nil
}

// GetTenantByAPIKey retrieves tenant by API key prefix
func (as *AppSettings) GetTenantByAPIKey(apiKey string) *TenantConfig {
	for i := range as.Tenants {
		if strings.HasPrefix(apiKey, as.Tenants[i].APIKeyPrefix) {
			return &as.Tenants[i]
		}
	}
	return nil
}

// PushPlatform represents push notification platforms
type PushPlatform int

const (
	Firebase PushPlatform = iota
	APNs
	FCM
	WebPush
)

// Message represents a FastDataBroker message
type Message struct {
	TenantID       string
	SenderID       string
	RecipientIDs   []string
	Subject        string
	Content        []byte
	Priority       Priority
	TTLSeconds     *int64
	Tags           map[string]string
	RequireConfirm bool
}

// DeliveryResult represents message delivery result
type DeliveryResult struct {
	MessageID         string
	TenantID          string
	Status            string
	DeliveredChannels int
	Details           map[string]interface{}
}

// WebSocketClientInfo represents a WebSocket client connection
type WebSocketClientInfo struct {
	ClientID    string
	UserID      string
	TenantID    string
	ConnectedAt time.Time
}

// WebhookConfig represents webhook configuration
type WebhookConfig struct {
	URL       string
	Headers   map[string]string
	Retries   int
	TimeoutMs int
	VerifySSL bool
}

// Client represents a multi-tenant FastDataBroker client
type Client struct {
	Host      string
	Port      int
	TenantID  string
	APIKey    string
	Settings  *AppSettings
	connected bool
	wsClients map[string]*WebSocketClientInfo
}

// NewClient creates a new FastDataBroker client
func NewClient(host string, port int) *Client {
	return &Client{
		Host:      host,
		Port:      port,
		connected: false,
		wsClients: make(map[string]*WebSocketClientInfo),
		Settings:  &AppSettings{},
	}
}

// NewClientWithTenant creates a new multi-tenant aware client
func NewClientWithTenant(tenantID string, apiKey string, host string, port int) (*Client, error) {
	if tenantID == "" {
		return nil, fmt.Errorf("TenantID cannot be empty")
	}
	if apiKey == "" {
		return nil, fmt.Errorf("APIKey cannot be empty")
	}

	return &Client{
		Host:      host,
		Port:      port,
		TenantID:  tenantID,
		APIKey:    apiKey,
		connected: false,
		wsClients: make(map[string]*WebSocketClientInfo),
		Settings:  &AppSettings{},
	}, nil
}

// NewClientFromSettings creates a client from AppSettings with tenant validation
func NewClientFromSettings(settings *AppSettings, tenantID string, apiKey string) (*Client, error) {
	if settings == nil {
		return nil, fmt.Errorf("settings cannot be nil")
	}
	if tenantID == "" {
		return nil, fmt.Errorf("TenantID cannot be empty")
	}
	if apiKey == "" {
		return nil, fmt.Errorf("APIKey cannot be empty")
	}

	// Validate tenant exists
	tenant := settings.GetTenant(tenantID)
	if tenant == nil {
		return nil, fmt.Errorf("tenant '%s' not found in configuration", tenantID)
	}

	// Validate API key prefix
	if !strings.HasPrefix(apiKey, tenant.APIKeyPrefix) {
		return nil, fmt.Errorf("API key does not match tenant prefix: %s", tenant.APIKeyPrefix)
	}

	return &Client{
		Host:      settings.Server.BindAddress,
		Port:      int(settings.Server.Port),
		TenantID:  tenantID,
		APIKey:    apiKey,
		Settings:  settings,
		connected: false,
		wsClients: make(map[string]*WebSocketClientInfo),
	}, nil
}

// Connect establishes connection with tenant context
func (c *Client) Connect() error {
	if c.TenantID == "" {
		return fmt.Errorf("TenantID must be set before connecting")
	}

	c.connected = true
	fmt.Printf("[TENANT: %s] Connected to FastDataBroker at %s:%d\n", c.TenantID, c.Host, c.Port)
	return nil
}

// SendMessage sends a message with tenant isolation
func (c *Client) SendMessage(msg *Message) (*DeliveryResult, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected. Call Connect first")
	}

	if msg == nil {
		return nil, fmt.Errorf("message cannot be nil")
	}

	if msg.TenantID == "" {
		msg.TenantID = c.TenantID
	}

	if msg.TenantID != c.TenantID {
		return nil, fmt.Errorf("message tenant does not match client tenant")
	}

	return &DeliveryResult{
		MessageID:         generateID(),
		TenantID:          c.TenantID,
		Status:            "success",
		DeliveredChannels: 1,
		Details:           make(map[string]interface{}),
	}, nil
}

// RegisterWebSocketClient registers a WebSocket client (tenant-isolated)
func (c *Client) RegisterWebSocketClient(clientID string, userID string) bool {
	if clientID == "" || userID == "" {
		return false
	}

	c.wsClients[clientID] = &WebSocketClientInfo{
		ClientID:    clientID,
		UserID:      userID,
		TenantID:    c.TenantID,
		ConnectedAt: time.Now().UTC(),
	}

	return true
}

// UnregisterWebSocketClient unregisters a WebSocket client
func (c *Client) UnregisterWebSocketClient(clientID string) bool {
	if _, exists := c.wsClients[clientID]; exists {
		delete(c.wsClients, clientID)
		return true
	}
	return false
}

// RegisterWebhook registers a webhook endpoint (tenant-isolated)
func (c *Client) RegisterWebhook(channel NotificationChannel, config *WebhookConfig) bool {
	if config == nil || config.URL == "" {
		return false
	}
	return true
}

// GenerateAPIKey generates API key for a client (tenant-aware)
func (c *Client) GenerateAPIKey(clientID string) (string, error) {
	if clientID == "" {
		return "", fmt.Errorf("ClientID cannot be empty")
	}

	tenant := c.Settings.GetTenant(c.TenantID)
	if tenant == nil {
		return "", fmt.Errorf("tenant '%s' not found", c.TenantID)
	}

	return tenant.APIKeyPrefix + generateID()[:16], nil
}

// GetTenantConfig returns current tenant configuration
func (c *Client) GetTenantConfig() *TenantConfig {
	return c.Settings.GetTenant(c.TenantID)
}

// Disconnect closes the connection
func (c *Client) Disconnect() {
	c.connected = false
	fmt.Printf("[TENANT: %s] Disconnected\n", c.TenantID)
}

// CreateClient helper function to load config and create client
func CreateClient(configPath string, tenantID string, apiKey string, environment string) (*Client, error) {
	settings, err := LoadFromFile(configPath, environment)
	if err != nil {
		return nil, err
	}

	return NewClientFromSettings(settings, tenantID, apiKey)
}

// Helper function to generate IDs
func generateID() string {
	return fmt.Sprintf("%d%d", time.Now().UnixNano(), time.Now().Nanosecond())
}
