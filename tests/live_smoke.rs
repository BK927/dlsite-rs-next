//! Live smoke tests for DLsite API
//!
//! These tests make real HTTP requests to DLsite servers.
//! They are disabled by default and must be explicitly enabled.
//!
//! # Running Live Tests
//!
//! Set the `DLSITE_LIVE_TESTS` environment variable:
//!
//! ```bash
//! DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored
//! ```
//!
//! # Warning
//!
//! These tests require network access and may be slow or flaky.
//! They are intended for manual verification, not CI.

mod common;

use dlsite_gamebox::{DlsiteClient, DlsiteError};

/// Check if live tests are enabled via environment variable
fn live_tests_enabled() -> bool {
    std::env::var("DLSITE_LIVE_TESTS")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

/// Skip test if live tests are not enabled
macro_rules! skip_unless_live {
    () => {
        if !live_tests_enabled() {
            println!("Skipping live test. Set DLSITE_LIVE_TESTS=1 to run.");
            return;
        }
    };
}

// =============================================================================
// Product Client Tests (HTML scraping based)
// =============================================================================

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_product_rj403038() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let res = client.product().get_all("RJ403038").await.unwrap();

    assert_eq!(res.id, "RJ403038");
    assert_eq!(res.title, "【ブルーアーカイブ】ユウカASMR～頑張るあなたのすぐそばに～");
    assert_eq!(res.circle_name.as_deref(), Some("Yostar"));
    assert_eq!(res.circle_id.as_deref(), Some("RG62982"));
    assert!(res.sale_count.unwrap() > 50000);
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_product_rj01017217() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let res = client.product().get_all("RJ01017217").await.unwrap();

    assert_eq!(res.id, "RJ01017217");
    assert_eq!(res.title, "【イヤーキャンドル】道草屋-なつな3-たぬさんこんにちは【ずぶ濡れシャンプー】");
    assert_eq!(res.circle_name.as_deref(), Some("桃色CODE"));
    assert_eq!(res.circle_id.as_deref(), Some("RG24350"));
}

// =============================================================================
// Product API Client Tests (JSON API based)
// =============================================================================

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_product_api_rj403038() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let res = client.product_api().get("RJ403038").await.unwrap();

    assert_eq!(res.workno, "RJ403038");
    assert_eq!(res.work_name, "【ブルーアーカイブ】ユウカASMR～頑張るあなたのすぐそばに～");
    assert_eq!(res.maker_name, "Yostar");
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_product_api_rj01017217() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let res = client.product_api().get("RJ01017217").await.unwrap();

    assert_eq!(res.workno, "RJ01017217");
    assert_eq!(res.work_name, "【イヤーキャンドル】道草屋-なつな3-たぬさんこんにちは【ずぶ濡れシャンプー】");
    assert_eq!(res.maker_name, "桃色CODE");
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_product_api_vj01000513() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let res = client.product_api().get("VJ01000513").await.unwrap();

    assert_eq!(res.workno, "VJ01000513");
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_product_api_not_found() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let result = client.product_api().get("RJ999999999").await;

    // Should return an error for non-existent product
    assert!(result.is_err());
}

// =============================================================================
// Search Tests (HTML based, requires scraper feature)
// =============================================================================

#[tokio::test]
#[ignore = "Live test - requires network and scraper feature. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke --features scraper -- --ignored"]
#[cfg(feature = "scraper")]
async fn live_search_basic() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let results = client.search().query("ASMR").execute().await.unwrap();

    assert!(!results.is_empty());
}

// =============================================================================
// Raw Request Tests
// =============================================================================

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_raw_request_product_json() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let body = client.get("/api/=/product.json?workno=RJ403038").await.unwrap();

    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json.is_array());
    assert!(!json.as_array().unwrap().is_empty());
}
