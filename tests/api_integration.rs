//! Integration tests for RevBackend API
//!
//! These tests require a running RevBackend instance.
//! Set REVBACKEND_URL environment variable to point to your backend.
//!
//! Run with: REVBACKEND_URL=http://localhost:8080 cargo test --test api_integration

mod common;

use common::*;
use revgame::api::{ApiError, MatchmakingState, SessionStatus};

/// Test user registration
#[tokio::test]
async fn test_register() {
    let client = create_test_client();
    let username = unique_username();
    let email = unique_email();
    let password = test_password();

    let result = client.register(&username, &email, &password).await;

    match result {
        Ok(auth) => {
            assert_eq!(auth.player.username, username);
            assert_eq!(auth.player.email, email);
            assert!(!auth.access_token.is_empty());
            assert!(!auth.refresh_token.is_empty());
            assert!(client.is_authenticated().await);
        }
        Err(ApiError::Request(e)) => {
            // Backend not running - skip test
            eprintln!("Skipping test - backend not available: {}", e);
        }
        Err(e) => panic!("Registration failed: {}", e),
    }
}

/// Test login with valid credentials
#[tokio::test]
async fn test_login() {
    let client = create_test_client();
    let username = unique_username();
    let email = unique_email();
    let password = test_password();

    // First register
    let register_result = client.register(&username, &email, &password).await;
    if let Err(ApiError::Request(_)) = register_result {
        eprintln!("Skipping test - backend not available");
        return;
    }
    register_result.expect("Registration should succeed");

    // Clear tokens and login again
    client.clear_tokens().await;
    assert!(!client.is_authenticated().await);

    let login_result = client.login(&email, &password).await;
    match login_result {
        Ok(auth) => {
            assert_eq!(auth.player.username, username);
            assert!(client.is_authenticated().await);
        }
        Err(e) => panic!("Login failed: {}", e),
    }
}

/// Test token refresh
#[tokio::test]
async fn test_token_refresh() {
    let client = create_test_client();
    let username = unique_username();
    let email = unique_email();
    let password = test_password();

    // Register and get tokens
    let register_result = client.register(&username, &email, &password).await;
    if let Err(ApiError::Request(_)) = register_result {
        eprintln!("Skipping test - backend not available");
        return;
    }
    register_result.expect("Registration should succeed");

    let old_access_token = client.access_token().await.unwrap();

    // Refresh token
    let refresh_result = client.refresh().await;
    match refresh_result {
        Ok(new_token) => {
            // New token should be different (usually)
            assert!(!new_token.is_empty());
            let current_token = client.access_token().await.unwrap();
            assert_eq!(new_token, current_token);
            // Note: tokens might be the same if refreshed quickly, so we just check validity
            let _ = old_access_token; // Suppress unused warning
        }
        Err(e) => panic!("Token refresh failed: {}", e),
    }
}

/// Test logout
#[tokio::test]
async fn test_logout() {
    let client = create_test_client();
    let username = unique_username();
    let email = unique_email();
    let password = test_password();

    // Register
    let register_result = client.register(&username, &email, &password).await;
    if let Err(ApiError::Request(_)) = register_result {
        eprintln!("Skipping test - backend not available");
        return;
    }
    register_result.expect("Registration should succeed");
    assert!(client.is_authenticated().await);

    // Logout
    let logout_result = client.logout().await;
    match logout_result {
        Ok(()) => {
            assert!(!client.is_authenticated().await);
        }
        Err(e) => panic!("Logout failed: {}", e),
    }
}

/// Test creating and listing sessions
#[tokio::test]
async fn test_create_and_list_sessions() {
    let client = create_test_client();
    let username = unique_username();
    let email = unique_email();
    let password = test_password();

    // Register
    let register_result = client.register(&username, &email, &password).await;
    if let Err(ApiError::Request(_)) = register_result {
        eprintln!("Skipping test - backend not available");
        return;
    }
    register_result.expect("Registration should succeed");

    // Create a session
    let session_name = format!("Test Session {}", unique_username());
    let create_result = client.create_session(&session_name, 4).await;
    let session = match create_result {
        Ok(s) => s,
        Err(e) => panic!("Create session failed: {}", e),
    };

    assert_eq!(session.name, session_name);
    assert_eq!(session.max_players, 4);
    assert_eq!(session.status, SessionStatus::Waiting);

    // List sessions
    let list_result = client.list_sessions().await;
    match list_result {
        Ok(sessions) => {
            assert!(sessions.iter().any(|s| s.id == session.id));
        }
        Err(e) => panic!("List sessions failed: {}", e),
    }

    // Get specific session
    let get_result = client.get_session(session.id).await;
    match get_result {
        Ok(s) => {
            assert_eq!(s.id, session.id);
            assert_eq!(s.name, session_name);
        }
        Err(e) => panic!("Get session failed: {}", e),
    }

    // Delete session
    let delete_result = client.delete_session(session.id).await;
    match delete_result {
        Ok(()) => {}
        Err(e) => panic!("Delete session failed: {}", e),
    }
}

/// Test joining and leaving sessions
#[tokio::test]
async fn test_join_and_leave_session() {
    // Create session owner
    let owner_client = create_test_client();
    let owner_username = unique_username();
    let owner_email = unique_email();

    let register_result = owner_client
        .register(&owner_username, &owner_email, &test_password())
        .await;
    if let Err(ApiError::Request(_)) = register_result {
        eprintln!("Skipping test - backend not available");
        return;
    }
    register_result.expect("Registration should succeed");

    // Create session
    let session = owner_client
        .create_session("Join Test Session", 4)
        .await
        .expect("Create session should succeed");

    // Create joiner
    let joiner_client = create_test_client();
    let joiner_username = unique_username();
    let joiner_email = unique_email();

    joiner_client
        .register(&joiner_username, &joiner_email, &test_password())
        .await
        .expect("Joiner registration should succeed");

    // Join session
    let join_result = joiner_client.join_session(session.id).await;
    match join_result {
        Ok(s) => {
            assert_eq!(s.id, session.id);
            assert!(s.players.iter().any(|p| p.username == joiner_username));
        }
        Err(e) => panic!("Join session failed: {}", e),
    }

    // Leave session
    let leave_result = joiner_client.leave_session(session.id).await;
    match leave_result {
        Ok(()) => {}
        Err(e) => panic!("Leave session failed: {}", e),
    }

    // Cleanup
    owner_client
        .delete_session(session.id)
        .await
        .expect("Delete should succeed");
}

/// Test matchmaking queue operations
#[tokio::test]
async fn test_matchmaking_queue() {
    let client = create_test_client();
    let username = unique_username();
    let email = unique_email();
    let password = test_password();

    // Register
    let register_result = client.register(&username, &email, &password).await;
    if let Err(ApiError::Request(_)) = register_result {
        eprintln!("Skipping test - backend not available");
        return;
    }
    register_result.expect("Registration should succeed");

    // Join matchmaking queue
    let join_result = client.join_matchmaking_queue().await;
    match join_result {
        Ok(()) => {}
        Err(e) => panic!("Join matchmaking queue failed: {}", e),
    }

    // Check status
    let status_result = client.get_matchmaking_status().await;
    match status_result {
        Ok(status) => {
            assert_eq!(status.status, MatchmakingState::Queued);
            assert!(status.position.is_some());
        }
        Err(e) => panic!("Get matchmaking status failed: {}", e),
    }

    // Leave queue
    let leave_result = client.leave_matchmaking_queue().await;
    match leave_result {
        Ok(()) => {}
        Err(e) => panic!("Leave matchmaking queue failed: {}", e),
    }

    // Verify no longer in queue
    let status_result = client.get_matchmaking_status().await;
    match status_result {
        Err(ApiError::NotFound(_)) => {
            // Expected - not in queue
        }
        Ok(_) => panic!("Should not be in queue after leaving"),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

/// Test that authentication is required for protected endpoints
#[tokio::test]
async fn test_auth_required() {
    let client = create_test_client();

    // Try to list sessions without auth
    let result = client.list_sessions().await;
    match result {
        Err(ApiError::Auth(_)) => {
            // Expected
        }
        Ok(_) => panic!("Should require authentication"),
        Err(ApiError::Request(_)) => {
            eprintln!("Skipping test - backend not available");
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
