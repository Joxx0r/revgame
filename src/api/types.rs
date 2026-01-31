use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// API error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Player model from RevBackend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub skill_rating: i32,
}

/// Authentication response containing tokens and player info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub player: Player,
}

/// Token refresh response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
}

/// Game session model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub max_players: i32,
    pub status: SessionStatus,
    #[serde(default)]
    pub players: Vec<Player>,
}

/// Session status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Waiting,
    InProgress,
    Finished,
}

/// Matchmaking status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakingStatus {
    pub status: MatchmakingState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

/// Matchmaking state enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchmakingState {
    Queued,
    Matched,
}

/// Error response from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Message response for simple operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message: String,
}

// Request types

/// Register request
#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Login request
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Refresh token request
#[derive(Debug, Clone, Serialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Logout request
#[derive(Debug, Clone, Serialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

/// Create session request
#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub max_players: i32,
}
