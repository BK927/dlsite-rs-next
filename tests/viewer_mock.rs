//! Mock tests for Viewer API adapter
//!
//! These tests focus on the ViewerAdapter which requires authentication
//! (cookie-store feature flag) and handles viewer session tokens.
//!
//! IMPORTANT: These tests require the `cookie-store` feature flag.

#![cfg(feature = "cookie-store")]

mod common;

use dlsite_rs_next::adapters::viewer::{ManifestToken, ViewerSession};
use dlsite_rs_next::{DlsiteClient, RetryConfig};
use std::time::Duration;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_mock_server() -> (MockServer, DlsiteClient) {
    let mock_server = MockServer::start().await;
    let client = DlsiteClient::new(&mock_server.uri());
    (mock_server, client)
}

fn zero_delay_retry_config() -> RetryConfig {
    RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(0),
        max_delay: Duration::from_millis(0),
        backoff_multiplier: 1.0,
    }
}

// =============================================================================
// Response Fixtures
// =============================================================================

const MANIFEST_TOKEN_SUCCESS: &str = r#"{
    "workno": "RJ403038",
    "manifest_url": "https://play.dl.dlsite.com/manifest/RJ403038.m3u8?token=abc123",
    "token": "view_token_xyz",
    "expires_at": "2024-01-15T12:00:00Z"
}"#;

const MANIFEST_TOKEN_UNAVAILABLE: &str = r#"{
    "workno": "RJ999999",
    "error": "This work is not available for streaming"
}"#;

const MANIFEST_TOKEN_MINIMAL: &str = r#"{
    "workno": "RJ123456"
}"#;

// =============================================================================
// HTTP Request Tests using get_raw
// These tests verify the client can handle Viewer API responses
// =============================================================================

#[tokio::test]
async fn test_get_manifest_token_success() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(MANIFEST_TOKEN_SUCCESS, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let url = format!(
        "{}/api/v3/download/sign/cookie?workno=RJ403038",
        mock_server.uri()
    );
    let body = client.get_raw(&url).await.unwrap();
    let token: ManifestToken = serde_json::from_str(&body).unwrap();

    assert_eq!(token.workno, Some("RJ403038".to_string()));
    assert_eq!(
        token.manifest_url,
        Some("https://play.dl.dlsite.com/manifest/RJ403038.m3u8?token=abc123".to_string())
    );
    assert_eq!(token.token, Some("view_token_xyz".to_string()));
    assert_eq!(token.expires_at, Some("2024-01-15T12:00:00Z".to_string()));
    assert!(token.error.is_none());
}

#[tokio::test]
async fn test_get_manifest_token_unavailable() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ999999"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(MANIFEST_TOKEN_UNAVAILABLE, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let url = format!(
        "{}/api/v3/download/sign/cookie?workno=RJ999999",
        mock_server.uri()
    );
    let body = client.get_raw(&url).await.unwrap();
    let token: ManifestToken = serde_json::from_str(&body).unwrap();

    assert_eq!(token.workno, Some("RJ999999".to_string()));
    assert_eq!(
        token.error,
        Some("This work is not available for streaming".to_string())
    );
    assert!(token.manifest_url.is_none());
    assert!(token.token.is_none());
}

#[tokio::test]
async fn test_get_manifest_token_minimal() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ123456"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(MANIFEST_TOKEN_MINIMAL, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let url = format!(
        "{}/api/v3/download/sign/cookie?workno=RJ123456",
        mock_server.uri()
    );
    let body = client.get_raw(&url).await.unwrap();
    let token: ManifestToken = serde_json::from_str(&body).unwrap();

    assert_eq!(token.workno, Some("RJ123456".to_string()));
    assert!(token.manifest_url.is_none());
    assert!(token.token.is_none());
    assert!(token.expires_at.is_none());
    assert!(token.error.is_none());
}

// =============================================================================
// Authentication Error Tests
// Note: get_raw() returns the response body regardless of status code.
// =============================================================================

#[tokio::test]
async fn test_get_manifest_token_401_returns_body() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(401).set_body_raw("Unauthorized", "text/plain"))
        .mount(&mock_server)
        .await;

    let url = format!(
        "{}/api/v3/download/sign/cookie?workno=RJ403038",
        mock_server.uri()
    );
    let body = client.get_raw(&url).await.unwrap();

    assert_eq!(body, "Unauthorized");
}

#[tokio::test]
async fn test_get_manifest_token_403_returns_body() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(403).set_body_raw("Forbidden", "text/plain"))
        .mount(&mock_server)
        .await;

    let url = format!(
        "{}/api/v3/download/sign/cookie?workno=RJ403038",
        mock_server.uri()
    );
    let body = client.get_raw(&url).await.unwrap();

    assert_eq!(body, "Forbidden");
}

// =============================================================================
// Retry Logic Tests
// =============================================================================

#[tokio::test]
async fn test_get_manifest_token_429_with_retry() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // Mount success mock first (wiremock uses LIFO)
    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(MANIFEST_TOKEN_SUCCESS, "application/json"),
        )
        .mount(&mock_server)
        .await;

    // Mount failure mock second - it will be checked first due to LIFO
    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let url = format!(
        "{}/api/v3/download/sign/cookie?workno=RJ403038",
        mock_server.uri()
    );
    let body = client.get_raw(&url).await.unwrap();
    let token: ManifestToken = serde_json::from_str(&body).unwrap();

    assert_eq!(token.workno, Some("RJ403038".to_string()));
}

#[tokio::test]
async fn test_get_manifest_token_500_with_retry() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // Mount success mock first (wiremock uses LIFO)
    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(MANIFEST_TOKEN_SUCCESS, "application/json"),
        )
        .mount(&mock_server)
        .await;

    // Mount failure mock second - it will be checked first due to LIFO
    Mock::given(method("GET"))
        .and(path("/api/v3/download/sign/cookie"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let url = format!(
        "{}/api/v3/download/sign/cookie?workno=RJ403038",
        mock_server.uri()
    );
    let body = client.get_raw(&url).await.unwrap();
    let token: ManifestToken = serde_json::from_str(&body).unwrap();

    assert_eq!(token.workno, Some("RJ403038".to_string()));
}

// =============================================================================
// Model Parsing Tests
// =============================================================================

#[test]
fn test_manifest_token_parsing_full() {
    let json = r#"{
        "workno": "RJ403038",
        "manifest_url": "https://example.com/manifest.m3u8",
        "token": "token123",
        "expires_at": "2024-01-15T12:00:00Z"
    }"#;

    let token: ManifestToken = serde_json::from_str(json).unwrap();

    assert_eq!(token.workno, Some("RJ403038".to_string()));
    assert_eq!(
        token.manifest_url,
        Some("https://example.com/manifest.m3u8".to_string())
    );
    assert_eq!(token.token, Some("token123".to_string()));
    assert_eq!(token.expires_at, Some("2024-01-15T12:00:00Z".to_string()));
    assert!(token.error.is_none());
}

#[test]
fn test_manifest_token_parsing_minimal() {
    let json = r#"{}"#;

    let token: ManifestToken = serde_json::from_str(json).unwrap();

    assert!(token.workno.is_none());
    assert!(token.manifest_url.is_none());
    assert!(token.token.is_none());
    assert!(token.expires_at.is_none());
    assert!(token.error.is_none());
}

#[test]
fn test_manifest_token_parsing_with_error() {
    let json = r#"{
        "workno": "RJ999999",
        "error": "Streaming not available"
    }"#;

    let token: ManifestToken = serde_json::from_str(json).unwrap();

    assert_eq!(token.workno, Some("RJ999999".to_string()));
    assert_eq!(token.error, Some("Streaming not available".to_string()));
    assert!(token.manifest_url.is_none());
}

#[test]
fn test_viewer_session_parsing_full() {
    let json = r#"{
        "workno": "RJ403038",
        "token": "session_token_abc",
        "viewer_url": "https://play.dlsite.com/viewer/RJ403038",
        "expires_at": "2024-01-15T13:00:00Z",
        "is_valid": true
    }"#;

    let session: ViewerSession = serde_json::from_str(json).unwrap();

    assert_eq!(session.workno, "RJ403038");
    assert_eq!(session.token, Some("session_token_abc".to_string()));
    assert_eq!(
        session.viewer_url,
        Some("https://play.dlsite.com/viewer/RJ403038".to_string())
    );
    assert_eq!(session.expires_at, Some("2024-01-15T13:00:00Z".to_string()));
    assert_eq!(session.is_valid, Some(true));
    assert!(session.error.is_none());
}

#[test]
fn test_viewer_session_parsing_minimal() {
    let json = r#"{"workno": "RJ123456"}"#;

    let session: ViewerSession = serde_json::from_str(json).unwrap();

    assert_eq!(session.workno, "RJ123456");
    assert!(session.token.is_none());
    assert!(session.viewer_url.is_none());
    assert!(session.expires_at.is_none());
    assert!(session.is_valid.is_none());
    assert!(session.error.is_none());
}

#[test]
fn test_viewer_session_parsing_with_error() {
    let json = r#"{
        "workno": "RJ999999",
        "is_valid": false,
        "error": "Session creation failed"
    }"#;

    let session: ViewerSession = serde_json::from_str(json).unwrap();

    assert_eq!(session.workno, "RJ999999");
    assert_eq!(session.is_valid, Some(false));
    assert_eq!(session.error, Some("Session creation failed".to_string()));
    assert!(session.token.is_none());
}
