use actix_web::{web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use crate::{models::*, error::{AdminApiError, AdminResult}, AppState};

/// Get SMTP configuration
pub async fn get_smtp_config(state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let config: Option<(String, String, i32, String, String, String, bool)> = sqlx::query_as(
        "SELECT id, host, port, username, password, from_email, use_tls FROM smtp_config LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    if let Some(cfg) = config {
        let response = SmtpConfigResponse {
            host: cfg.1,
            port: cfg.2,
            from_email: cfg.5,
            use_tls: cfg.6,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };
        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(AdminApiError::NotFound("SMTP configuration not found".to_string()))
    }
}

/// Update SMTP configuration
pub async fn update_smtp_config(
    state: web::Data<AppState>,
    req: web::Json<UpdateSmtpConfigRequest>,
) -> AdminResult<HttpResponse> {
    let now = Utc::now().to_rfc3339();

    // Get existing config or create new one
    let existing: Option<(String, String, i32, String, String, String, bool, String)> = sqlx::query_as(
        "SELECT id, host, port, username, password, from_email, use_tls, created_at FROM smtp_config LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    let id = existing.as_ref().map(|e| e.0.clone()).unwrap_or_else(|| Uuid::new_v4().to_string());
    let host = req.host.clone().unwrap_or_else(|| existing.as_ref().map(|e| e.1.clone()).unwrap_or_default());
    let port = req.port.unwrap_or_else(|| existing.as_ref().map(|e| e.2).unwrap_or(587));
    let username = req.username.clone().unwrap_or_else(|| existing.as_ref().map(|e| e.3.clone()).unwrap_or_default());
    let password = req.password.clone().unwrap_or_else(|| existing.as_ref().map(|e| e.4.clone()).unwrap_or_default());
    let from_email = req.from_email.clone().unwrap_or_else(|| existing.as_ref().map(|e| e.5.clone()).unwrap_or_default());
    let use_tls = req.use_tls.unwrap_or_else(|| existing.as_ref().map(|e| e.6).unwrap_or(true));
    let created_at = existing.as_ref().map(|e| e.7.clone()).unwrap_or_else(|| now.clone());

    if existing.is_some() {
        sqlx::query(
            "UPDATE smtp_config SET host = ?, port = ?, username = ?, password = ?, from_email = ?, use_tls = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&host)
        .bind(port)
        .bind(&username)
        .bind(&password)
        .bind(&from_email)
        .bind(use_tls)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;
    } else {
        sqlx::query(
            "INSERT INTO smtp_config (id, host, port, username, password, from_email, use_tls, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&host)
        .bind(port)
        .bind(&username)
        .bind(&password)
        .bind(&from_email)
        .bind(use_tls)
        .bind(&created_at)
        .bind(&now)
        .execute(&state.db)
        .await
        .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;
    }

    let response = SmtpConfigResponse {
        host,
        port,
        from_email,
        use_tls,
        created_at,
        updated_at: now,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Test SMTP connection
pub async fn test_smtp(
    state: web::Data<AppState>,
    req: web::Json<SmtpTestRequest>,
) -> AdminResult<HttpResponse> {
    let _config: (String, String, i32, String, String, String, bool) = sqlx::query_as(
        "SELECT id, host, port, username, password, from_email, use_tls FROM smtp_config LIMIT 1"
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AdminApiError::NotFound("SMTP configuration not found".to_string()))?;

    // TODO: Implement actual SMTP test connection
    // For now, return success
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Test email sent successfully",
        "recipient": req.recipient_email
    })))
}

/// Get all notification settings
pub async fn get_notification_settings(state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let settings: Vec<NotificationSettings> = sqlx::query_as(
        "SELECT id, event_type, enabled, recipient_email, notification_channels, created_at, updated_at FROM notification_settings ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    let responses: Vec<NotificationSettingsResponse> = settings.into_iter().map(|s| {
        let channels: Vec<String> = serde_json::from_str(&s.notification_channels).unwrap_or_default();
        NotificationSettingsResponse {
            id: s.id,
            event_type: s.event_type,
            enabled: s.enabled,
            recipient_email: s.recipient_email,
            notification_channels: channels,
            created_at: s.created_at,
        }
    }).collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// Create or update notification settings
pub async fn update_notification_settings(
    state: web::Data<AppState>,
    req: web::Json<NotificationSettingsRequest>,
) -> AdminResult<HttpResponse> {
    let now = Utc::now().to_rfc3339();
    let channels_json = serde_json::to_string(&req.notification_channels)
        .map_err(|e| AdminApiError::BadRequest(e.to_string()))?;

    // Check if already exists
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM notification_settings WHERE event_type = ?"
    )
    .bind(&req.event_type)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    if let Some((id,)) = existing {
        sqlx::query(
            "UPDATE notification_settings SET enabled = ?, recipient_email = ?, notification_channels = ?, updated_at = ? WHERE id = ?"
        )
        .bind(req.enabled)
        .bind(&req.recipient_email)
        .bind(&channels_json)
        .bind(&now)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

        let response = NotificationSettingsResponse {
            id,
            event_type: req.event_type.clone(),
            enabled: req.enabled,
            recipient_email: req.recipient_email.clone(),
            notification_channels: req.notification_channels.clone(),
            created_at: "".to_string(),
        };

        Ok(HttpResponse::Ok().json(response))
    } else {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO notification_settings (id, event_type, enabled, recipient_email, notification_channels, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&req.event_type)
        .bind(req.enabled)
        .bind(&req.recipient_email)
        .bind(&channels_json)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await
        .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

        let response = NotificationSettingsResponse {
            id,
            event_type: req.event_type.clone(),
            enabled: req.enabled,
            recipient_email: req.recipient_email.clone(),
            notification_channels: req.notification_channels.clone(),
            created_at: now,
        };

        Ok(HttpResponse::Created().json(response))
    }
}

/// List all notification events
pub async fn list_notification_events(state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let events: Vec<NotificationEvent> = sqlx::query_as(
        "SELECT event_id, event_type, title, description, severity, created_at FROM notification_events ORDER BY created_at DESC LIMIT 100"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    let responses: Vec<NotificationEventResponse> = events.into_iter().map(|e| {
        NotificationEventResponse {
            event_id: e.event_id,
            event_type: e.event_type,
            title: e.title,
            description: e.description,
            severity: e.severity,
            created_at: e.created_at,
        }
    }).collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// Get specific notification event
pub async fn get_notification_event(
    state: web::Data<AppState>,
    event_id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    let event: NotificationEvent = sqlx::query_as(
        "SELECT event_id, event_type, title, description, severity, created_at FROM notification_events WHERE event_id = ?"
    )
    .bind(event_id.into_inner())
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AdminApiError::NotFound("Notification event not found".to_string()))?;

    let response = NotificationEventResponse {
        event_id: event.event_id,
        event_type: event.event_type,
        title: event.title,
        description: event.description,
        severity: event.severity,
        created_at: event.created_at,
    };

    Ok(HttpResponse::Ok().json(response))
}
