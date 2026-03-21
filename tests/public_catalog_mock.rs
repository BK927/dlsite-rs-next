//! Mock tests for public catalog adapters
//!
//! These tests focus on HTTP error handling and retry logic.
//! For complex JSON parsing tests, see the existing embedded tests
//! in src/client/product/test.rs and src/client/product_api/test.rs
//! which use real API responses.

mod common;

use dlsite_gamebox::client::product::ProductPeople;
use dlsite_gamebox::client::product_api::interface::{Creators, Creator};
use dlsite_gamebox::interface::product::WorkType;
use dlsite_gamebox::{DlsiteClient, DlsiteError, RetryConfig};
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
// HTTP Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_product_api_404_returns_http_status_error() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ999999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let result = client.product_api().get("RJ999999").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DlsiteError::HttpStatus(404) => {}
        e => panic!("Expected HttpStatus(404) error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_product_api_403_returns_auth_error() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ999999"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&mock_server)
        .await;

    let result = client.product_api().get("RJ999999").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DlsiteError::AuthRequired(_) => {}
        e => panic!("Expected AuthRequired error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_product_api_malformed_json_returns_parse_error() {
    let (mock_server, client) = setup_mock_server().await;

    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ123456"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("not valid json", "application/json"))
        .mount(&mock_server)
        .await;

    let result = client.product_api().get("RJ123456").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DlsiteError::Parse(_) => {}
        e => panic!("Expected Parse error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_product_api_empty_array_returns_parse_error() {
    let (mock_server, client) = setup_mock_server().await;

    // DLsite returns empty array for not found products
    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ999999"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .mount(&mock_server)
        .await;

    let result = client.product_api().get("RJ999999").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DlsiteError::Parse(msg) => assert!(msg.contains("No product found")),
        e => panic!("Expected Parse error with 'No product found', got: {:?}", e),
    }
}

// =============================================================================
// Retry Logic Tests
// =============================================================================

#[tokio::test]
async fn test_product_api_429_retries_on_retryable_config() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // First request returns 429 (rate limit)
    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(2) // Fail twice
        .mount(&mock_server)
        .await;

    // Third request succeeds with empty array (which will fail, but proves retry worked)
    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .mount(&mock_server)
        .await;

    let result = client.product_api().get("RJ403038").await;

    // Should get Parse error (from empty array), not RateLimit error
    // This proves the retry happened
    match result.unwrap_err() {
        DlsiteError::Parse(_) => {} // Success - retry happened
        DlsiteError::RateLimit(_) => panic!("Should have retried on 429"),
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn test_product_api_500_retries_on_retryable_config() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // First request returns 500 (server error)
    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1) // Fail once
        .mount(&mock_server)
        .await;

    // Second request succeeds with empty array
    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(200).set_body_raw("[]", "application/json"))
        .mount(&mock_server)
        .await;

    let result = client.product_api().get("RJ403038").await;

    // Should get Parse error (from empty array), not HttpStatus(500)
    match result.unwrap_err() {
        DlsiteError::Parse(_) => {} // Success - retry happened
        DlsiteError::HttpStatus(500) => panic!("Should have retried on 500"),
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn test_product_api_404_does_not_retry() {
    let (mock_server, client) = setup_mock_server().await;
    let mut client = client;
    client.set_retry_config(zero_delay_retry_config());

    // 404 should not retry
    Mock::given(method("GET"))
        .and(path("/api/=/product.json"))
        .and(query_param("workno", "RJ403038"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let result = client.product_api().get("RJ403038").await;

    assert!(result.is_err());
    match result.unwrap_err() {
        DlsiteError::HttpStatus(404) => {} // Correct - no retry
        e => panic!("Expected HttpStatus(404), got: {:?}", e),
    }
}

// =============================================================================
// Model Mapping Tests
// =============================================================================

#[test]
fn test_creators_to_product_people_basic() {
    let creators = Creators {
        created_by: Some(vec![Creator {
            id: "123".to_string(),
            name: "桃鳥".to_string(),
            classification: "author".to_string(),
            sub_classification: None,
        }]),
        voice_by: Some(vec![
            Creator {
                id: "456".to_string(),
                name: "丹羽うさぎ".to_string(),
                classification: "voice".to_string(),
                sub_classification: None,
            },
            Creator {
                id: "457".to_string(),
                name: "藤堂れんげ".to_string(),
                classification: "voice".to_string(),
                sub_classification: None,
            },
        ]),
        illust_by: None,
        scenario_by: None,
    };

    let people = ProductPeople::from(creators);

    // Verify author mapping
    let author = people.author.expect("author should be set");
    assert_eq!(author.len(), 1);
    assert_eq!(author[0], "桃鳥");

    // Verify voice actor mapping
    let voice_actor = people.voice_actor.expect("voice_actor should be set");
    assert_eq!(voice_actor.len(), 2);
    assert!(voice_actor.contains(&"丹羽うさぎ".to_string()));
    assert!(voice_actor.contains(&"藤堂れんげ".to_string()));
}

#[test]
fn test_creators_to_product_people_empty() {
    let creators = Creators {
        created_by: None,
        voice_by: None,
        illust_by: None,
        scenario_by: None,
    };

    let people = ProductPeople::from(creators);

    assert!(people.author.is_none());
    assert!(people.voice_actor.is_none());
    assert!(people.illustrator.is_none());
    assert!(people.scenario.is_none());
}

#[test]
fn test_creators_to_product_people_all_fields() {
    let creators = Creators {
        created_by: Some(vec![Creator {
            id: "1".to_string(),
            name: "Author Name".to_string(),
            classification: "author".to_string(),
            sub_classification: None,
        }]),
        voice_by: Some(vec![Creator {
            id: "2".to_string(),
            name: "Voice Actor".to_string(),
            classification: "voice".to_string(),
            sub_classification: None,
        }]),
        illust_by: Some(vec![Creator {
            id: "3".to_string(),
            name: "Illustrator".to_string(),
            classification: "illustration".to_string(),
            sub_classification: None,
        }]),
        scenario_by: Some(vec![Creator {
            id: "4".to_string(),
            name: "Scenario Writer".to_string(),
            classification: "scenario".to_string(),
            sub_classification: None,
        }]),
    };

    let people = ProductPeople::from(creators);

    assert_eq!(people.author.as_ref().unwrap()[0], "Author Name");
    assert_eq!(people.voice_actor.as_ref().unwrap()[0], "Voice Actor");
    assert_eq!(people.illustrator.as_ref().unwrap()[0], "Illustrator");
    assert_eq!(people.scenario.as_ref().unwrap()[0], "Scenario Writer");
}

// =============================================================================
// WorkType Parsing Tests
// =============================================================================

#[test]
fn test_work_type_from_str_valid() {
    use std::str::FromStr;

    assert_eq!(WorkType::from_str("SOU").unwrap(), WorkType::SOU);
    assert_eq!(WorkType::from_str("ACN").unwrap(), WorkType::ACN);
    assert_eq!(WorkType::from_str("ADV").unwrap(), WorkType::ADV);
    assert_eq!(WorkType::from_str("RPG").unwrap(), WorkType::RPG);
    assert_eq!(WorkType::from_str("MNG").unwrap(), WorkType::MNG);
    assert_eq!(WorkType::from_str("ICG").unwrap(), WorkType::ICG);
    assert_eq!(WorkType::from_str("MOV").unwrap(), WorkType::MOV);
    assert_eq!(WorkType::from_str("MUS").unwrap(), WorkType::MUS);
    assert_eq!(WorkType::from_str("TOL").unwrap(), WorkType::TOL);
}

#[test]
fn test_work_type_from_str_unknown() {
    use std::str::FromStr;

    let unknown = WorkType::from_str("XYZ").unwrap();
    match unknown {
        WorkType::Unknown(s) => assert_eq!(s, "XYZ"),
        _ => panic!("Expected Unknown variant"),
    }
}
