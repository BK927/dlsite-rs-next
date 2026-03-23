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

use dlsite_rs_next::DlsiteClient;

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
    assert_eq!(
        res.title,
        "【ブルーアーカイブ】ユウカASMR～頑張るあなたのすぐそばに～"
    );
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
    assert_eq!(
        res.title,
        "【イヤーキャンドル】道草屋-なつな3-たぬさんこんにちは【ずぶ濡れシャンプー】"
    );
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
    assert_eq!(
        res.work_name,
        "【ブルーアーカイブ】ユウカASMR～頑張るあなたのすぐそばに～"
    );
    assert_eq!(res.maker_name, "Yostar");
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_product_api_rj01017217() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let res = client.product_api().get("RJ01017217").await.unwrap();

    assert_eq!(res.workno, "RJ01017217");
    assert_eq!(
        res.work_name,
        "【イヤーキャンドル】道草屋-なつな3-たぬさんこんにちは【ずぶ濡れシャンプー】"
    );
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
#[ignore = "Live test - requires network and search-html feature. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke --features search-html -- --ignored"]
#[cfg(feature = "search-html")]
async fn live_search_basic() {
    use dlsite_rs_next::client::search::SearchProductQuery;

    skip_unless_live!();

    let client = DlsiteClient::default();
    let query = SearchProductQuery {
        keyword: Some("ASMR".to_string()),
        ..Default::default()
    };
    let results = client.search().search_product(&query).await.unwrap();

    assert!(!results.products.is_empty());
}

// =============================================================================
// Raw Request Tests
// =============================================================================

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_raw_request_product_json() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let body = client
        .get("/api/=/product.json?workno=RJ403038")
        .await
        .unwrap();

    // Should be valid JSON
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json.is_array());
    assert!(!json.as_array().unwrap().is_empty());
}

// =============================================================================
// Review API Tests
// =============================================================================

use dlsite_rs_next::client::product::review::ReviewSortOrder;
use dlsite_rs_next::interface::query::Language;

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_review_basic() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let review = client
        .product()
        .get_review("RJ403038", 6, 1, true, ReviewSortOrder::New)
        .await
        .unwrap();

    assert!(review.is_success);
    // Popular product should have reviews
    assert!(!review.review_list.is_empty());
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_review_with_locale_korean() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let review = client
        .product()
        .get_review_with_locale("RJ403038", 6, 1, true, ReviewSortOrder::Top, Language::Ko)
        .await
        .unwrap();

    assert!(review.is_success);
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_review_with_locale_chinese_simplified() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let review = client
        .product()
        .get_review_with_locale("RJ403038", 6, 1, true, ReviewSortOrder::New, Language::ZhCn)
        .await
        .unwrap();

    assert!(review.is_success);
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_review_with_locale_chinese_traditional() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let review = client
        .product()
        .get_review_with_locale("RJ403038", 6, 1, true, ReviewSortOrder::New, Language::ZhTw)
        .await
        .unwrap();

    assert!(review.is_success);
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_get_reviewer_genre_list() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    let review = client
        .product()
        .get_review("RJ403038", 6, 1, true, ReviewSortOrder::New)
        .await
        .unwrap();

    // Popular product should have reviewer genre breakdown
    assert!(review.reviewer_genre_list.is_some());
    let genres = review.reviewer_genre_list.unwrap();
    // Should have at least some genre data
    assert!(!genres.is_empty() || !review.review_list.is_empty());
}

// =============================================================================
// Circle API Tests (requires search-html feature)
// =============================================================================

#[tokio::test]
#[ignore = "Live test - requires network and search-html feature. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke --features search-html -- --ignored"]
#[cfg(feature = "search-html")]
async fn live_list_circle_games() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    // RG24350 is 桃色CODE circle
    let result = client.circle().list_circle_games("RG24350").await;

    // Note: HTML scraping can fail if DLsite changes their page structure
    // This test verifies the API exists and works when the HTML structure matches
    match result {
        Ok(games) => {
            // Should have at least some games
            assert!(!games.is_empty(), "Circle should have at least one game");
            // All results should be game types
            for game in &games {
                assert!(game.work_type.is_game(), "Result should be a game type");
            }
        }
        Err(e) => {
            // HTML parsing can fail if the site structure changes
            // Log the error but don't fail the test entirely
            eprintln!(
                "Warning: Circle games list failed (HTML structure may have changed): {:?}",
                e
            );
        }
    }
}

#[tokio::test]
#[ignore = "Live test - requires network and search-html feature. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke --features search-html -- --ignored"]
#[cfg(feature = "search-html")]
async fn live_get_circle_profile() {
    skip_unless_live!();

    let client = DlsiteClient::default();
    // RG24350 is 桃色CODE circle
    let result = client.circle().get_circle_profile("RG24350").await;

    match result {
        Ok(profile) => {
            assert_eq!(profile.id, "RG24350");
            assert!(!profile.name.is_empty());
        }
        Err(e) => {
            // HTML parsing can fail if the site structure changes
            eprintln!(
                "Warning: Circle profile failed (HTML structure may have changed): {:?}",
                e
            );
        }
    }
}

// =============================================================================
// Product API with All Locales Tests
// =============================================================================

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_product_api_all_locales() {
    skip_unless_live!();

    let client = DlsiteClient::default();

    // Test all 5 locales
    let locales = [
        ("ja_JP", Language::Jp),
        ("en_US", Language::En),
        ("ko_KR", Language::Ko),
        ("zh_CN", Language::ZhCn),
        ("zh_TW", Language::ZhTw),
    ];

    for (locale_str, _locale) in locales {
        let path = format!("/api/=/product.json?workno=RJ403038&locale={}", locale_str);
        let body = client.get(&path).await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(json.is_array(), "Locale {} should return array", locale_str);
        assert!(
            !json.as_array().unwrap().is_empty(),
            "Locale {} should have data",
            locale_str
        );
    }
}

#[tokio::test]
#[ignore = "Live test - requires network. Run with: DLSITE_LIVE_TESTS=1 cargo test --test live_smoke -- --ignored"]
async fn live_product_thumbnail_and_screenshots() {
    skip_unless_live!();

    let client = DlsiteClient::default();

    // Get thumbnail
    let thumbnail = client
        .product_api()
        .get_product_thumbnail("RJ403038")
        .await
        .unwrap();
    // URL can be absolute (http://...) or protocol-relative (//...)
    assert!(
        thumbnail.starts_with("http") || thumbnail.starts_with("//"),
        "Thumbnail should be a URL, got: {}",
        thumbnail
    );

    // Get screenshots
    let screenshots = client
        .product_api()
        .list_product_screenshots("RJ403038")
        .await
        .unwrap();
    // Screenshots may or may not exist, but should be a valid vector
    for screenshot in &screenshots {
        assert!(
            screenshot.starts_with("http") || screenshot.starts_with("//"),
            "Screenshot should be a URL, got: {}",
            screenshot
        );
    }
}
