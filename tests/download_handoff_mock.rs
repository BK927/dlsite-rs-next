//! Mock tests for Download handoff API adapter
//!
//! These tests focus on the DownloadAdapter which requires authentication
//! (cookie-store feature flag) and handles download URL retrieval.
//!
//! IMPORTANT: These tests require the `cookie-store` feature flag.
//!
//! NOTE: The current implementation parses JSON responses. Future versions
//! may handle 302 redirect semantics directly.

#![cfg(feature = "cookie-store")]

mod common;

use dlsite_gamebox::adapters::download::DownloadTarget;
use dlsite_gamebox::{DlsiteClient, RetryConfig};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};
use std::time::Duration;

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

const DOWNLOAD_SUCCESS_RESPONSE: &str = r#"{
    "workno": "RJ403038",
    "url": "https://example.com/download/RJ403038.zip?token=abc123",
    "filename": "RJ403038.zip",
    "filesize": 524288000,
    "expires_at": "2024-01-15T12:00:00Z",
    "is_available": true
}"#;

const DOWNLOAD_UNAVAILABLE_RESPONSE: &str = r#"{
    "workno": "RJ999999",
    "is_available": false,
    "error": "This work is not available for download"
}"#;

const DOWNLOAD_MINIMAL_RESPONSE: &str = r#"{
    "workno": "RJ123456"
}"#;

// =============================================================================
// HTTP Request Tests using get_raw
// These tests verify the client can handle Download API responses
// =============================================================================

#[tokio::test]
async fn test_get_download_endpoint_success() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            DOWNLOAD_SUCCESS_RESPONSE,
            "application/json",
        ))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ403038", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let target: DownloadTarget = serde_json::from_str(&body).unwrap();

    assert_eq!(target.workno, "RJ403038");
    assert_eq!(target.url, Some("https://example.com/download/RJ403038.zip?token=abc123".to_string()));
    assert_eq!(target.filename, Some("RJ403038.zip".to_string()));
    assert_eq!(target.filesize, Some(524288000));
    assert_eq!(target.is_available, Some(true));
}

#[tokio::test]
async fn test_get_download_endpoint_unavailable() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ999999"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            DOWNLOAD_UNAVAILABLE_RESPONSE,
            "application/json",
        ))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ999999", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let target: DownloadTarget = serde_json::from_str(&body).unwrap();

    assert_eq!(target.workno, "RJ999999");
    assert_eq!(target.is_available, Some(false));
    assert_eq!(target.error, Some("This work is not available for download".to_string()));
    assert!(target.url.is_none());
}

#[tokio::test]
async fn test_get_download_endpoint_minimal() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ123456"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            DOWNLOAD_MINIMAL_RESPONSE,
            "application/json",
        ))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ123456", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let target: DownloadTarget = serde_json::from_str(&body).unwrap();

    assert_eq!(target.workno, "RJ123456");
    assert!(target.url.is_none());
    assert!(target.filename.is_none());
    assert!(target.filesize.is_none());
    assert!(target.is_available.is_none());
}

// =============================================================================
// Authentication Error Tests
// Note: get_raw() returns the response body regardless of status code.
// =============================================================================

#[tokio::test]
async fn test_get_download_401_returns_body() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(401).set_body_raw("Unauthorized", "text/plain"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ403038", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();

    assert_eq!(body, "Unauthorized");
}

#[tokio::test]
async fn test_get_download_403_returns_body() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(403).set_body_raw("Forbidden", "text/plain"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ403038", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();

    assert_eq!(body, "Forbidden");
}

#[tokio::test]
async fn test_get_download_404_returns_body() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ999999"))
        .respond_with(ResponseTemplate::new(404).set_body_raw("Not Found", "text/plain"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ999999", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();

    assert_eq!(body, "Not Found");
}

// =============================================================================
// Retry Logic Tests
// =============================================================================

#[tokio::test]
async fn test_get_download_429_with_retry() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // Mount success mock first (wiremock uses LIFO)
    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            DOWNLOAD_SUCCESS_RESPONSE,
            "application/json",
        ))
        .mount(&mock_server)
        .await;

    // Mount failure mock second - it will be checked first due to LIFO
    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ403038", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let target: DownloadTarget = serde_json::from_str(&body).unwrap();

    assert_eq!(target.workno, "RJ403038");
}

#[tokio::test]
async fn test_get_download_500_with_retry() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // Mount success mock first (wiremock uses LIFO)
    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            DOWNLOAD_SUCCESS_RESPONSE,
            "application/json",
        ))
        .mount(&mock_server)
        .await;

    // Mount failure mock second - it will be checked first due to LIFO
    Mock::given(method("GET"))
        .and(path("/api/v3/download"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/download?workno=RJ403038", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let target: DownloadTarget = serde_json::from_str(&body).unwrap();

    assert_eq!(target.workno, "RJ403038");
}

// =============================================================================
// Model Parsing Tests
// =============================================================================

#[test]
fn test_download_target_parsing_full() {
    let json = r#"{
        "workno": "RJ403038",
        "url": "https://example.com/download/file.zip",
        "filename": "file.zip",
        "filesize": 12345678,
        "expires_at": "2024-01-15T12:00:00Z",
        "is_available": true
    }"#;

    let target: DownloadTarget = serde_json::from_str(json).unwrap();

    assert_eq!(target.workno, "RJ403038");
    assert_eq!(target.url, Some("https://example.com/download/file.zip".to_string()));
    assert_eq!(target.filename, Some("file.zip".to_string()));
    assert_eq!(target.filesize, Some(12345678));
    assert_eq!(target.expires_at, Some("2024-01-15T12:00:00Z".to_string()));
    assert_eq!(target.is_available, Some(true));
    assert!(target.error.is_none());
}

#[test]
fn test_download_target_parsing_minimal() {
    let json = r#"{"workno": "RJ123456"}"#;

    let target: DownloadTarget = serde_json::from_str(json).unwrap();

    assert_eq!(target.workno, "RJ123456");
    assert!(target.url.is_none());
    assert!(target.filename.is_none());
    assert!(target.filesize.is_none());
    assert!(target.expires_at.is_none());
    assert!(target.is_available.is_none());
    assert!(target.error.is_none());
}

#[test]
fn test_download_target_parsing_with_error() {
    let json = r#"{
        "workno": "RJ999999",
        "is_available": false,
        "error": "Download not available"
    }"#;

    let target: DownloadTarget = serde_json::from_str(json).unwrap();

    assert_eq!(target.workno, "RJ999999");
    assert_eq!(target.is_available, Some(false));
    assert_eq!(target.error, Some("Download not available".to_string()));
    assert!(target.url.is_none());
}

#[test]
fn test_download_target_parsing_partial() {
    let json = r#"{
        "workno": "RJ789012",
        "url": "https://example.com/file.zip",
        "is_available": true
    }"#;

    let target: DownloadTarget = serde_json::from_str(json).unwrap();

    assert_eq!(target.workno, "RJ789012");
    assert_eq!(target.url, Some("https://example.com/file.zip".to_string()));
    assert_eq!(target.is_available, Some(true));
    assert!(target.filename.is_none());
    assert!(target.filesize.is_none());
}
