use super::client::ApiClient;
use super::types::{ApiError, MatchmakingStatus, MessageResponse};

impl ApiClient {
    /// Join the matchmaking queue
    pub async fn join_matchmaking_queue(&self) -> Result<(), ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .post(self.url("/api/v1/matchmaking/queue"))
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

    /// Leave the matchmaking queue
    pub async fn leave_matchmaking_queue(&self) -> Result<(), ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .delete(self.url("/api/v1/matchmaking/queue"))
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

    /// Get current matchmaking status
    pub async fn get_matchmaking_status(&self) -> Result<MatchmakingStatus, ApiError> {
        let token = self
            .access_token()
            .await
            .ok_or_else(|| ApiError::Auth("Not authenticated".to_string()))?;

        let response = self
            .client()
            .get(self.url("/api/v1/matchmaking/status"))
            .bearer_auth(&token)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(Self::parse_error(response).await)
        }
    }
}
