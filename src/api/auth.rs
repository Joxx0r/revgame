use super::client::ApiClient;
use super::types::{
    ApiError, AuthResponse, LoginRequest, LogoutRequest, MessageResponse, RefreshRequest,
    RefreshResponse, RegisterRequest,
};

impl ApiClient {
    /// Register a new player account
    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, ApiError> {
        let request = RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client()
            .post(self.url("/api/v1/auth/register"))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let auth: AuthResponse = response.json().await?;
            self.set_tokens(auth.access_token.clone(), auth.refresh_token.clone())
                .await;
            Ok(auth)
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// Login with email and password
    pub async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, ApiError> {
        let request = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };

        let response = self
            .client()
            .post(self.url("/api/v1/auth/login"))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let auth: AuthResponse = response.json().await?;
            self.set_tokens(auth.access_token.clone(), auth.refresh_token.clone())
                .await;
            Ok(auth)
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// Refresh the access token using the refresh token
    pub async fn refresh(&self) -> Result<String, ApiError> {
        let refresh_token = self
            .refresh_token()
            .await
            .ok_or_else(|| ApiError::Auth("No refresh token available".to_string()))?;

        let request = RefreshRequest { refresh_token };

        let response = self
            .client()
            .post(self.url("/api/v1/auth/refresh"))
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let refresh: RefreshResponse = response.json().await?;
            self.set_access_token(refresh.access_token.clone()).await;
            Ok(refresh.access_token)
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// Logout and invalidate the refresh token
    pub async fn logout(&self) -> Result<(), ApiError> {
        let refresh_token = match self.refresh_token().await {
            Some(token) => token,
            None => {
                self.clear_tokens().await;
                return Ok(());
            }
        };

        let request = LogoutRequest { refresh_token };

        let response = self
            .client()
            .post(self.url("/api/v1/auth/logout"))
            .json(&request)
            .send()
            .await?;

        self.clear_tokens().await;

        if response.status().is_success() {
            let _: MessageResponse = response.json().await?;
            Ok(())
        } else {
            // Even if logout fails on server, clear local tokens
            Err(Self::parse_error(response).await)
        }
    }
}
