package multitenancy

import (
	"strings"
	"testing"
)

// ============== Tenant Configuration Tests ==============

func TestTenantConfigCreation(t *testing.T) {
	tenant := &TenantConfig{
		TenantID:       "acme-corp",
		TenantName:     "ACME Corporation",
		APIKeyPrefix:   "acme_",
		RateLimitRps:   1000,
		MaxConnections: 100,
		MaxMessageSize: 1048576,
		RetentionDays:  30,
		Enabled:        true,
	}

	if tenant.TenantID != "acme-corp" {
		t.Errorf("Expected TenantID 'acme-corp', got '%s'", tenant.TenantID)
	}
	if tenant.APIKeyPrefix != "acme_" {
		t.Errorf("Expected APIKeyPrefix 'acme_', got '%s'", tenant.APIKeyPrefix)
	}
	if tenant.RateLimitRps != 1000 {
		t.Errorf("Expected RateLimitRps 1000, got %d", tenant.RateLimitRps)
	}
}

func TestTenantValidation_Success(t *testing.T) {
	tenant := &TenantConfig{
		TenantID:       "acme-corp",
		APIKeyPrefix:   "acme_",
		RateLimitRps:   1000,
		MaxConnections: 100,
	}

	if err := tenant.Validate(); err != nil {
		t.Errorf("Validation should succeed, but got error: %v", err)
	}
}

func TestTenantValidation_EmptyID(t *testing.T) {
	tenant := &TenantConfig{
		TenantID:       "",
		APIKeyPrefix:   "acme_",
		RateLimitRps:   1000,
		MaxConnections: 100,
	}

	if err := tenant.Validate(); err == nil {
		t.Error("Expected validation error for empty TenantID")
	}
}

func TestTenantValidation_BadPrefix(t *testing.T) {
	tenant := &TenantConfig{
		TenantID:       "acme-corp",
		APIKeyPrefix:   "acme", // Missing underscore
		RateLimitRps:   1000,
		MaxConnections: 100,
	}

	if err := tenant.Validate(); err == nil {
		t.Error("Expected validation error for bad APIKeyPrefix")
	}
}

func TestTenantValidation_ZeroRateLimit(t *testing.T) {
	tenant := &TenantConfig{
		TenantID:       "acme-corp",
		APIKeyPrefix:   "acme_",
		RateLimitRps:   0,
		MaxConnections: 100,
	}

	if err := tenant.Validate(); err == nil {
		t.Error("Expected validation error for zero RateLimitRps")
	}
}

func TestTenantValidation_ZeroConnections(t *testing.T) {
	tenant := &TenantConfig{
		TenantID:       "acme-corp",
		APIKeyPrefix:   "acme_",
		RateLimitRps:   1000,
		MaxConnections: 0,
	}

	if err := tenant.Validate(); err == nil {
		t.Error("Expected validation error for zero MaxConnections")
	}
}

// ============== AppSettings Tests ==============

func TestAppSettingsCreation(t *testing.T) {
	settings := &AppSettings{}

	if len(settings.Tenants) != 0 {
		t.Errorf("Expected empty tenants, got %d", len(settings.Tenants))
	}
}

func TestAppSettingsGetTenant(t *testing.T) {
	settings := &AppSettings{
		Tenants: []TenantConfig{
			{
				TenantID:       "acme-corp",
				APIKeyPrefix:   "acme_",
				RateLimitRps:   1000,
				MaxConnections: 100,
			},
		},
	}

	tenant := settings.GetTenant("acme-corp")
	if tenant == nil {
		t.Error("Expected to find tenant 'acme-corp'")
	} else if tenant.TenantID != "acme-corp" {
		t.Errorf("Expected TenantID 'acme-corp', got '%s'", tenant.TenantID)
	}
}

func TestAppSettingsGetTenant_NotFound(t *testing.T) {
	settings := &AppSettings{Tenants: []TenantConfig{}}

	tenant := settings.GetTenant("nonexistent")
	if tenant != nil {
		t.Error("Expected nil for nonexistent tenant")
	}
}

func TestAppSettingsGetTenantByAPIKey(t *testing.T) {
	settings := &AppSettings{
		Tenants: []TenantConfig{
			{
				TenantID:       "acme-corp",
				APIKeyPrefix:   "acme_",
				RateLimitRps:   1000,
				MaxConnections: 100,
			},
		},
	}

	tenant := settings.GetTenantByAPIKey("acme_550e8400e29b41d4a716446655440000")
	if tenant == nil {
		t.Error("Expected to find tenant by API key prefix")
	} else if tenant.TenantID != "acme-corp" {
		t.Errorf("Expected TenantID 'acme-corp', got '%s'", tenant.TenantID)
	}
}

func TestMultipleTenantIsolation(t *testing.T) {
	settings := &AppSettings{
		Tenants: []TenantConfig{
			{
				TenantID:       "acme-corp",
				APIKeyPrefix:   "acme_",
				RateLimitRps:   1000,
				MaxConnections: 100,
			},
			{
				TenantID:       "startup-xyz",
				APIKeyPrefix:   "xyz_",
				RateLimitRps:   100,
				MaxConnections: 10,
			},
		},
	}

	t1 := settings.GetTenant("acme-corp")
	t2 := settings.GetTenant("startup-xyz")

	if t1 == nil || t2 == nil {
		t.Fatal("Expected to find both tenants")
	}

	if t1.RateLimitRps != 1000 {
		t.Errorf("Expected ACME rate limit 1000, got %d", t1.RateLimitRps)
	}

	if t2.RateLimitRps != 100 {
		t.Errorf("Expected Startup rate limit 100, got %d", t2.RateLimitRps)
	}

	if t1.MaxConnections != 100 {
		t.Errorf("Expected ACME max connections 100, got %d", t1.MaxConnections)
	}

	if t2.MaxConnections != 10 {
		t.Errorf("Expected Startup max connections 10, got %d", t2.MaxConnections)
	}
}

// ============== Client Tests ==============

func TestClientCreation_WithTenant(t *testing.T) {
	client, err := NewClientWithTenant("acme-corp", "acme_key", "localhost", 6379)

	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if client.TenantID != "acme-corp" {
		t.Errorf("Expected TenantID 'acme-corp', got '%s'", client.TenantID)
	}
}

func TestClientCreation_EmptyTenantID(t *testing.T) {
	_, err := NewClientWithTenant("", "key", "localhost", 6379)

	if err == nil {
		t.Error("Expected error for empty TenantID")
	}
}

func TestClientCreation_EmptyAPIKey(t *testing.T) {
	_, err := NewClientWithTenant("acme-corp", "", "localhost", 6379)

	if err == nil {
		t.Error("Expected error for empty APIKey")
	}
}

func TestClientCreation_FromSettings_Success(t *testing.T) {
	settings := &AppSettings{
		Server: ServerConfig{
			BindAddress: "localhost",
			Port:        6379,
		},
		Tenants: []TenantConfig{
			{
				TenantID:       "acme-corp",
				APIKeyPrefix:   "acme_",
				RateLimitRps:   1000,
				MaxConnections: 100,
			},
		},
	}

	client, err := NewClientFromSettings(settings, "acme-corp", "acme_valid_key")

	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if client.TenantID != "acme-corp" {
		t.Errorf("Expected TenantID 'acme-corp', got '%s'", client.TenantID)
	}
}

func TestClientCreation_FromSettings_APIKeyMismatch(t *testing.T) {
	settings := &AppSettings{
		Tenants: []TenantConfig{
			{
				TenantID:       "acme-corp",
				APIKeyPrefix:   "acme_",
				RateLimitRps:   1000,
				MaxConnections: 100,
			},
		},
	}

	_, err := NewClientFromSettings(settings, "acme-corp", "xyz_invalid_key")

	if err == nil {
		t.Error("Expected error for API key prefix mismatch")
	}
}

func TestClientCreation_FromSettings_TenantNotFound(t *testing.T) {
	settings := &AppSettings{Tenants: []TenantConfig{}}

	_, err := NewClientFromSettings(settings, "nonexistent", "any_key")

	if err == nil {
		t.Error("Expected error for nonexistent tenant")
	}
}

func TestMessageSending_TenantIsolation(t *testing.T) {
	client := &Client{
		TenantID:  "acme-corp",
		connected: true,
		wsClients: make(map[string]*WebSocketClientInfo),
	}

	message := &Message{
		TenantID:     "acme-corp",
		SenderID:     "user1",
		RecipientIDs: []string{"user2"},
		Subject:      "Test",
		Content:      []byte("content"),
	}

	result, err := client.SendMessage(message)

	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if result.TenantID != "acme-corp" {
		t.Errorf("Expected TenantID 'acme-corp', got '%s'", result.TenantID)
	}
}

func TestAPIKeyGeneration(t *testing.T) {
	settings := &AppSettings{
		Tenants: []TenantConfig{
			{
				TenantID:       "acme-corp",
				APIKeyPrefix:   "acme_",
				RateLimitRps:   1000,
				MaxConnections: 100,
			},
		},
	}

	client, _ := NewClientFromSettings(settings, "acme-corp", "acme_existing_key")

	newKey, err := client.GenerateAPIKey("client-1")

	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if !strings.HasPrefix(newKey, "acme_") {
		t.Errorf("Expected key with prefix 'acme_', got '%s'", newKey)
	}
}
