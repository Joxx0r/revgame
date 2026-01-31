use uuid::Uuid;

use super::client::ApiClient;
use super::types::{ApiError, CreateSessionRequest, GameSession, MessageResponse};

impl ApiClient {
    /// Create a new game session
    pub async fn create_session(
        &self,
        name: &str,
        max_players: i32,
    ) -> Result<GameSession, ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let request = CreateSessionRequest {
            name: name.to_string(),
            max_players,
        };

        let response = self
            .client()
            .post(self.url("/api/v1/sessions"))
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// List all available game sessions
    pub async fn list_sessions(&self) -> Result<Vec<GameSession>, ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .get(self.url("/api/v1/sessions"))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// Get a specific game session by ID
    pub async fn get_session(&self, id: Uuid) -> Result<GameSession, ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .get(self.url(&format!("/api/v1/sessions/{}", id)))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// Join an existing game session
    pub async fn join_session(&self, id: Uuid) -> Result<GameSession, ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .post(self.url(&format!("/api/v1/sessions/{}/join", id)))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// Leave a game session
    pub async fn leave_session(&self, id: Uuid) -> Result<(), ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .post(self.url(&format!("/api/v1/sessions/{}/leave", id)))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() {
            let _: MessageResponse = response.json().await?;
            Ok(())
        } else {
            Err(Self::parse_error(response).await)
        }
    }

    /// Delete a game session (owner only)
    pub async fn delete_session(&self, id: Uuid) -> Result<(), ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .delete(self.url(&format!("/api/v1/sessions/{}", id)))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Self::parse_error(response).await)
        }
    }
}
