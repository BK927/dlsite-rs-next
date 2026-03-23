//! Mock tests for Play Library v3 API adapter
//!
//! These tests focus on the PlayLibraryAdapter which requires authentication
//! (cookie-store feature) and accesses the user's purchased library.
//!
//! IMPORTANT: These tests require the `cookie-store` feature flag.
//!
//! NOTE: The PlayClient.library() method is not yet exposed. These tests
//! verify the adapter functionality directly and the model types.

#![cfg(feature = "cookie-store")]

mod common;

use dlsite_rs_next::adapters::play_library::{LibraryCount, LibraryEntry};
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

// Basic count response
const COUNT_RESPONSE_BASIC: &str = r#"{"count": 42, "has_more": true}"#;
const COUNT_RESPONSE_EMPTY: &str = r#"{"count": 0, "has_more": false}"#;

// Basic sales/library entries response
const SALES_RESPONSE_PAGE1: &str = r#"[
  {"workno":"RJ403038","work_name":"Test Work 1","maker_id":"RG62982","maker_name":"Test Maker","purchase_date":"2022-07-20","is_downloadable":true,"is_play_available":true,"file_size":"500MB","image_main":"//img.dlsite.jp/test1.jpg","image_thumb":"//img.dlsite.jp/test1_thumb.jpg"},
  {"workno":"RJ01017217","work_name":"Test Work 2","maker_id":"RG24350","maker_name":"Other Maker","purchase_date":"2023-02-15","is_downloadable":true,"is_play_available":true,"file_size":"750MB","image_main":"//img.dlsite.jp/test2.jpg","image_thumb":"//img.dlsite.jp/test2_thumb.jpg"}
]"#;

const SALES_RESPONSE_PAGE2: &str = r#"[
  {"workno":"RJ291224","work_name":"Test Work 3","maker_id":"RG51654","maker_name":"Third Maker","purchase_date":"2021-03-10","is_downloadable":true,"is_play_available":true,"file_size":"350MB","image_main":"//img.dlsite.jp/test3.jpg","image_thumb":"//img.dlsite.jp/test3_thumb.jpg"}
]"#;

// =============================================================================
// HTTP Request Tests using get_raw
// These tests verify the client can handle Play API responses
// =============================================================================

#[tokio::test]
async fn test_get_count_endpoint_success() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/content/count"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(COUNT_RESPONSE_BASIC, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/count", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let count: LibraryCount = serde_json::from_str(&body).unwrap();

    assert_eq!(count.count, 42);
    assert_eq!(count.has_more, Some(true));
}

#[tokio::test]
async fn test_get_count_endpoint_empty() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/content/count"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(COUNT_RESPONSE_EMPTY, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/count", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let count: LibraryCount = serde_json::from_str(&body).unwrap();

    assert_eq!(count.count, 0);
    assert_eq!(count.has_more, Some(false));
}

#[tokio::test]
async fn test_get_sales_endpoint_success() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/content/sales"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(SALES_RESPONSE_PAGE1, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/sales", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let entries: Vec<LibraryEntry> = serde_json::from_str(&body).unwrap();

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].workno, "RJ403038");
    assert_eq!(entries[0].work_name, "Test Work 1");
    assert_eq!(entries[1].workno, "RJ01017217");
}

#[tokio::test]
async fn test_get_sales_endpoint_with_pagination() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/content/sales"))
        .and(query_param("last", "12345"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(SALES_RESPONSE_PAGE2, "application/json"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/sales?last=12345", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let entries: Vec<LibraryEntry> = serde_json::from_str(&body).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].workno, "RJ291224");
}

// =============================================================================
// Authentication Error Tests (401/403)
// Note: get_raw() returns the response body regardless of status code.
// Auth error handling is done at higher abstraction levels.
// These tests verify the mock server setup and that get_raw doesn't crash.
// =============================================================================

#[tokio::test]
async fn test_get_count_401_returns_body() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/content/count"))
        .respond_with(ResponseTemplate::new(401).set_body_raw("Unauthorized", "text/plain"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/count", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();

    // get_raw returns the body even for 401 (status checking is done elsewhere)
    assert_eq!(body, "Unauthorized");
}

#[tokio::test]
async fn test_get_count_403_returns_body() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/content/count"))
        .respond_with(ResponseTemplate::new(403).set_body_raw("Forbidden", "text/plain"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/count", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();

    // get_raw returns the body even for 403 (status checking is done elsewhere)
    assert_eq!(body, "Forbidden");
}

// =============================================================================
// Retry Logic Tests
// =============================================================================

#[tokio::test]
async fn test_get_count_429_with_retry() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // Mount success mock first (wiremock uses LIFO, so this will be checked after failure mock)
    Mock::given(method("GET"))
        .and(path("/api/v3/content/count"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(COUNT_RESPONSE_BASIC, "application/json"),
        )
        .mount(&mock_server)
        .await;

    // Mount failure mock second - it will be checked first due to LIFO
    Mock::given(method("GET"))
        .and(path("/api/v3/content/count"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/count", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let count: LibraryCount = serde_json::from_str(&body).unwrap();

    assert_eq!(count.count, 42);
}

#[tokio::test]
async fn test_get_sales_500_with_retry() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // Mount success mock first (wiremock uses LIFO)
    Mock::given(method("GET"))
        .and(path("/api/v3/content/sales"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(SALES_RESPONSE_PAGE1, "application/json"),
        )
        .mount(&mock_server)
        .await;

    // Mount failure mock second - it will be checked first due to LIFO
    Mock::given(method("GET"))
        .and(path("/api/v3/content/sales"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/api/v3/content/sales", mock_server.uri());
    let body = client.get_raw(&url).await.unwrap();
    let entries: Vec<LibraryEntry> = serde_json::from_str(&body).unwrap();

    assert_eq!(entries.len(), 2);
}

// =============================================================================
// Model Parsing Tests
// =============================================================================

#[test]
fn test_library_entry_parsing_full() {
    let json = r#"{"workno":"RJ403038","work_name":"Test Work","maker_id":"RG62982","maker_name":"Test Maker","purchase_date":"2022-07-20","is_downloadable":true,"is_play_available":true,"file_size":"500MB","image_main":"//img.dlsite.jp/test.jpg","image_thumb":"//img.dlsite.jp/test_thumb.jpg"}"#;

    let entry: LibraryEntry = serde_json::from_str(json).unwrap();

    assert_eq!(entry.workno, "RJ403038");
    assert_eq!(entry.work_name, "Test Work");
    assert_eq!(entry.maker_id, Some("RG62982".to_string()));
    assert_eq!(entry.maker_name, Some("Test Maker".to_string()));
    assert_eq!(entry.purchase_date, Some("2022-07-20".to_string()));
    assert_eq!(entry.is_downloadable, Some(true));
    assert_eq!(entry.is_play_available, Some(true));
    assert_eq!(entry.file_size, Some("500MB".to_string()));
    assert_eq!(
        entry.image_main,
        Some("//img.dlsite.jp/test.jpg".to_string())
    );
    assert_eq!(
        entry.image_thumb,
        Some("//img.dlsite.jp/test_thumb.jpg".to_string())
    );
}

#[test]
fn test_library_entry_parsing_minimal() {
    // Test with only required fields
    let json = r#"{"workno":"RJ123456","work_name":"Minimal Work"}"#;

    let entry: LibraryEntry = serde_json::from_str(json).unwrap();

    assert_eq!(entry.workno, "RJ123456");
    assert_eq!(entry.work_name, "Minimal Work");
    assert!(entry.maker_id.is_none());
    assert!(entry.maker_name.is_none());
    assert!(entry.purchase_date.is_none());
    assert!(entry.is_downloadable.is_none());
    assert!(entry.is_play_available.is_none());
    assert!(entry.file_size.is_none());
}

#[test]
fn test_library_entry_parsing_partial() {
    let json = r#"{"workno":"RJ789012","work_name":"Partial Work","maker_name":"Some Circle","is_downloadable":false}"#;

    let entry: LibraryEntry = serde_json::from_str(json).unwrap();

    assert_eq!(entry.workno, "RJ789012");
    assert_eq!(entry.work_name, "Partial Work");
    assert!(entry.maker_id.is_none());
    assert_eq!(entry.maker_name, Some("Some Circle".to_string()));
    assert_eq!(entry.is_downloadable, Some(false));
}

#[test]
fn test_library_count_parsing_full() {
    let json = r#"{"count": 42, "has_more": true}"#;

    let count: LibraryCount = serde_json::from_str(json).unwrap();

    assert_eq!(count.count, 42);
    assert_eq!(count.has_more, Some(true));
}

#[test]
fn test_library_count_parsing_without_has_more() {
    let json = r#"{"count": 10}"#;

    let count: LibraryCount = serde_json::from_str(json).unwrap();

    assert_eq!(count.count, 10);
    assert!(count.has_more.is_none());
}

#[test]
fn test_library_count_parsing_zero() {
    let json = r#"{"count": 0, "has_more": false}"#;

    let count: LibraryCount = serde_json::from_str(json).unwrap();

    assert_eq!(count.count, 0);
    assert_eq!(count.has_more, Some(false));
}

#[test]
fn test_library_entries_array_parsing() {
    let json = r#"[
        {"workno":"RJ001","work_name":"Work 1"},
        {"workno":"RJ002","work_name":"Work 2"},
        {"workno":"RJ003","work_name":"Work 3"}
    ]"#;

    let entries: Vec<LibraryEntry> = serde_json::from_str(json).unwrap();

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].workno, "RJ001");
    assert_eq!(entries[1].workno, "RJ002");
    assert_eq!(entries[2].workno, "RJ003");
}
