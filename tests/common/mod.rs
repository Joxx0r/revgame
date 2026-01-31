use revgame::api::ApiClient;
use uuid::Uuid;

/// Get the backend URL from environment or default
pub fn backend_url() -> String {
    std::env::var("REVBACKEND_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// Create a new API client for testing
pub fn create_test_client() -> ApiClient {
    ApiClient::new(backend_url())
}

/// Generate a unique test username
pub fn unique_username() -> String {
    format!("testuser_{}", Uuid::new_v4().to_string()[..8].to_string())
}

/// Generate a unique test email
pub fn unique_email() -> String {
    format!(
        "test_{}@example.com",
        Uuid::new_v4().to_string()[..8].to_string()
    )
}

/// Test password that meets requirements
pub fn test_password() -> String {
    "TestPassword123!".to_string()
}
