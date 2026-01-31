use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{ApiError, ErrorResponse};

/// HTTP client wrapper for RevBackend API
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    tokens: Arc<RwLock<TokenState>>,
}

/// Token state for authenticated requests
#[derive(Default)]
pub struct TokenState {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            tokens: Arc::new(RwLock::new(TokenState::default())),
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Set the access token
    pub async fn set_access_token(&self, token: String) {
        let mut tokens = self.tokens.write().await;
        tokens.access_token = Some(token);
    }

    /// Set the refresh token
    pub async fn set_refresh_token(&self, token: String) {
        let mut tokens = self.tokens.write().await;
        tokens.refresh_token = Some(token);
    }

    /// Set both tokens at once
    pub async fn set_tokens(&self, access_token: String, refresh_token: String) {
        let mut tokens = self.tokens.write().await;
        tokens.access_token = Some(access_token);
        tokens.refresh_token = Some(refresh_token);
    }

    /// Get the current access token
    pub async fn access_token(&self) -> Option<String> {
        let tokens = self.tokens.read().await;
        tokens.access_token.clone()
    }

    /// Get the current refresh token
    pub async fn refresh_token(&self) -> Option<String> {
        let tokens = self.tokens.read().await;
        tokens.refresh_token.clone()
    }

    /// Clear all tokens (logout)
    pub async fn clear_tokens(&self) {
        let mut tokens = self.tokens.write().await;
        tokens.access_token = None;
        tokens.refresh_token = None;
    }

    /// Check if we have an access token
    pub async fn is_authenticated(&self) -> bool {
        let tokens = self.tokens.read().await;
        tokens.access_token.is_some()
    }

    /// Build the full URL for an endpoint
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Parse an error response from the API
    pub async fn parse_error(response: reqwest::Response) -> ApiError {
        let status = response.status();

        let error_msg = match response.json::<ErrorResponse>().await {
            Ok(err) => err.error,
            Err(_) => "Unknown error".to_string(),
        };

        match status.as_u16() {
            401 => ApiError::Auth(error_msg),
            404 => ApiError::NotFound(error_msg),
            409 => ApiError::Conflict(error_msg),
            500..=599 => ApiError::Server(error_msg),
            _ => ApiError::Unknown(error_msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = ApiClient::new("http://localhost:8080");
        assert_eq!(client.base_url(), "http://localhost:8080");
        assert!(!client.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_token_management() {
        let client = ApiClient::new("http://localhost:8080");

        client
            .set_tokens("access123".to_string(), "refresh456".to_string())
            .await;

        assert!(client.is_authenticated().await);
        assert_eq!(client.access_token().await, Some("access123".to_string()));
        assert_eq!(client.refresh_token().await, Some("refresh456".to_string()));

        client.clear_tokens().await;
        assert!(!client.is_authenticated().await);
    }
}
