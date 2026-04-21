use crate::error::{AdminApiError, AdminResult};
use reqwest;

/// Broker API client for communication with FastDataBroker
pub struct BrokerClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl BrokerClient {
    /// Create a new broker client
    pub fn new(base_url: String) -> Self {
        BrokerClient {
            base_url,
            http_client: reqwest::Client::new(),
        }
    }

    /// Check broker health
    pub async fn health_check(&self) -> AdminResult<bool> {
        match self.http_client
            .get(&format!("{}/health", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Get broker info
    pub async fn get_broker_info(&self) -> AdminResult<serde_json::Value> {
        let response = self.http_client
            .get(&format!("{}/info", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))?;

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))
    }

    /// Get broker statistics
    pub async fn get_stats(&self) -> AdminResult<serde_json::Value> {
        let response = self.http_client
            .get(&format!("{}/stats", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))?;

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))
    }

    /// Get cluster status
    pub async fn get_cluster_status(&self) -> AdminResult<serde_json::Value> {
        let response = self.http_client
            .get(&format!("{}/cluster/status", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))?;

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))
    }

    /// Verify tenant API key with broker
    pub async fn verify_tenant_key(&self, tenant_id: &str, api_key: &str) -> AdminResult<bool> {
        let response = self.http_client
            .post(&format!("{}/auth/verify", self.base_url))
            .json(&serde_json::json!({
                "tenant_id": tenant_id,
                "api_key": api_key
            }))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))?;

        Ok(response.status().is_success())
    }

    /// Get tenant statistics from broker
    pub async fn get_tenant_stats(&self, tenant_id: &str) -> AdminResult<serde_json::Value> {
        let response = self.http_client
            .get(&format!("{}/tenants/{}/stats", self.base_url, tenant_id))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))?;

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AdminApiError::BrokerError(e.to_string()))
    }
}
