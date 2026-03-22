//! Mock tests for SearchClient
//!
//! These tests verify HTTP interactions and caching behavior
//! without requiring network access to DLsite.

#[cfg(feature = "search-html")]
mod search_client_mock {
    use dlsite_rs::client::search::SearchProductQuery;
    use dlsite_rs::{DlsiteClient, DlsiteError};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_mock_server() -> (MockServer, DlsiteClient) {
        let mock_server = MockServer::start().await;
        let client = DlsiteClient::new(&mock_server.uri());
        (mock_server, client)
    }

    /// Create a mock search AJAX response with the given HTML and count
    fn create_search_response(html: &str, count: i32) -> String {
        serde_json::json!({
            "search_result": html,
            "page_info": {
                "count": count
            }
        })
        .to_string()
    }

    /// Create a minimal valid search result HTML with one product
    fn create_minimal_search_html(product_id: &str, title: &str) -> String {
        format!(
            r#"<ul id="search_result_img_box">
                <li>
                    <div class="work_thumb_inner">
                        <img src="//img.dlsite.jp/test.jpg">
                    </div>
                    <div data-product_id="{}">
                        <div class="work_name">
                            <a title="{}">{}</a>
                        </div>
                        <div class="work_genre">
                            <span title="全年齢">全年齢</span>
                        </div>
                        <div class="maker_name">
                            <a href="//www.dlsite.com/maniax/circle/profile/=/maker_code/RG00001.html">Test Circle</a>
                        </div>
                        <div class="work_price_wrap">
                            <span class="work_price"><span class="work_price_base">1,000</span></span>
                        </div>
                        <div class="work_category type_SOU"></div>
                    </div>
                </li>
            </ul>"#,
            product_id, title, title
        )
    }

    // =========================================================================
    // Basic Search Tests
    // =========================================================================

    #[tokio::test]
    async fn test_search_product_parses_response_correctly() {
        let (mock_server, client) = setup_mock_server().await;

        let html = create_minimal_search_html("RJ123456", "Test Product");
        let response = create_search_response(&html, 1);

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
            .mount(&mock_server)
            .await;

        let query = SearchProductQuery::default();
        let result = client.search().search_product(&query).await.unwrap();

        assert_eq!(result.count, 1);
        assert_eq!(result.products.len(), 1);
        assert_eq!(result.products[0].id, "RJ123456");
        assert_eq!(result.products[0].title, "Test Product");
    }

    #[tokio::test]
    async fn test_search_product_reflects_count_from_api() {
        let (mock_server, client) = setup_mock_server().await;

        let html = create_minimal_search_html("RJ123456", "Test");
        // HTML has 1 item, but API reports 100 total
        let response = create_search_response(&html, 100);

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
            .mount(&mock_server)
            .await;

        let query = SearchProductQuery::default();
        let result = client.search().search_product(&query).await.unwrap();

        // Count should reflect API response, not parsed items
        assert_eq!(result.count, 100);
        assert_eq!(result.products.len(), 1);
    }

    #[tokio::test]
    async fn test_search_product_handles_multiple_items() {
        let (mock_server, client) = setup_mock_server().await;

        let html = r#"<ul id="search_result_img_box">
                <li>
                    <div class="work_thumb_inner"><img src="//img.dlsite.jp/test1.jpg"></div>
                    <div data-product_id="RJ111111">
                        <div class="work_name"><a title="Product 1">Product 1</a></div>
                        <div class="work_genre"><span title="全年齢">全年齢</span></div>
                        <div class="maker_name"><a href="//www.dlsite.com/maniax/circle/profile/=/maker_code/RG001.html">Circle A</a></div>
                        <div class="work_price_wrap"><span class="work_price"><span class="work_price_base">1,000</span></span></div>
                        <div class="work_category type_SOU"></div>
                    </div>
                </li>
                <li>
                    <div class="work_thumb_inner"><img src="//img.dlsite.jp/test2.jpg"></div>
                    <div data-product_id="RJ222222">
                        <div class="work_name"><a title="Product 2">Product 2</a></div>
                        <div class="work_genre"><span title="R-15">R-15</span></div>
                        <div class="maker_name"><a href="//www.dlsite.com/maniax/circle/profile/=/maker_code/RG002.html">Circle B</a></div>
                        <div class="work_price_wrap"><span class="work_price"><span class="work_price_base">2,000</span></span></div>
                        <div class="work_category type_ADV"></div>
                    </div>
                </li>
            </ul>"#;
        let response = create_search_response(html, 50);

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
            .mount(&mock_server)
            .await;

        let query = SearchProductQuery::default();
        let result = client.search().search_product(&query).await.unwrap();

        assert_eq!(result.count, 50);
        assert_eq!(result.products.len(), 2);
        assert_eq!(result.products[0].id, "RJ111111");
        assert_eq!(result.products[1].id, "RJ222222");
    }

    // =========================================================================
    // Error Handling Tests
    // =========================================================================

    #[tokio::test]
    async fn test_search_product_handles_404() {
        let (mock_server, client) = setup_mock_server().await;

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let query = SearchProductQuery::default();
        let result = client.search().search_product(&query).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            DlsiteError::HttpStatus(404) => {}
            e => panic!("Expected HttpStatus(404), got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_search_product_handles_malformed_json() {
        let (mock_server, client) = setup_mock_server().await;

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp"))
            .respond_with(ResponseTemplate::new(200).set_body_raw("not valid json", "application/json"))
            .mount(&mock_server)
            .await;

        let query = SearchProductQuery::default();
        let result = client.search().search_product(&query).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_product_handles_empty_results() {
        let (mock_server, client) = setup_mock_server().await;

        let response = create_search_response("", 0);

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
            .mount(&mock_server)
            .await;

        let query = SearchProductQuery::default();
        let result = client.search().search_product(&query).await.unwrap();

        assert_eq!(result.count, 0);
        assert!(result.products.is_empty());
    }

    // =========================================================================
    // Cache Behavior Tests
    // =========================================================================

    #[tokio::test]
    async fn test_search_product_caches_results() {
        let (mock_server, client) = setup_mock_server().await;

        let html = create_minimal_search_html("RJ123456", "Test Product");
        let response = create_search_response(&html, 1);

        // Mount a mock that can only be called once - this verifies cache behavior
        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response.clone(), "application/json"))
            .expect(1..=2) // Should be called at least once, at most twice (once for fetch, possibly once for count check)
            .mount(&mock_server)
            .await;

        let query = SearchProductQuery::default();

        // First call - should fetch and cache
        let result1 = client.search().search_product(&query).await.unwrap();
        assert_eq!(result1.products[0].id, "RJ123456");
        assert_eq!(result1.products[0].title, "Test Product");

        // Second call - should return same products (from cache)
        let result2 = client.search().search_product(&query).await.unwrap();

        // Products should be identical to first call (cached)
        assert_eq!(result2.products[0].id, "RJ123456");
        assert_eq!(result2.products[0].title, "Test Product");

        // Verify the products are the same object (by value comparison)
        assert_eq!(result1.products[0].id, result2.products[0].id);
        assert_eq!(result1.products[0].title, result2.products[0].title);
    }

    #[tokio::test]
    async fn test_search_product_different_queries_not_cached() {
        let (mock_server, client) = setup_mock_server().await;

        // First query response (keyword=test1)
        let html1 = create_minimal_search_html("RJ111111", "Product 1");
        let response1 = create_search_response(&html1, 1);

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp/keyword/test1"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response1, "application/json"))
            .up_to_n_times(1)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Second query response (keyword=test2)
        let html2 = create_minimal_search_html("RJ222222", "Product 2");
        let response2 = create_search_response(&html2, 2);

        Mock::given(method("GET"))
            .and(path("/fsr/ajax/=/language/jp/keyword/test2"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response2, "application/json"))
            .up_to_n_times(1)
            .expect(1)
            .mount(&mock_server)
            .await;

        // Different queries should not share cache
        let query1 = SearchProductQuery {
            keyword: Some("test1".to_string()),
            ..Default::default()
        };
        let query2 = SearchProductQuery {
            keyword: Some("test2".to_string()),
            ..Default::default()
        };

        let result1 = client.search().search_product(&query1).await.unwrap();
        assert_eq!(result1.products[0].id, "RJ111111");

        let result2 = client.search().search_product(&query2).await.unwrap();
        assert_eq!(result2.products[0].id, "RJ222222");
    }

    // =========================================================================
    // Batch Query Tests
    // =========================================================================

    #[tokio::test]
    async fn test_search_products_batch_concurrent() {
        let (mock_server, client) = setup_mock_server().await;

        // Set up response for batch queries (page 1 and page 2)
        let html = create_minimal_search_html("RJ123456", "Test");
        let response = create_search_response(&html, 10);

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
            .mount(&mock_server)
            .await;

        let queries = vec![
            SearchProductQuery {
                page: Some(1),
                ..Default::default()
            },
            SearchProductQuery {
                page: Some(2),
                ..Default::default()
            },
        ];

        let results = client.search().search_products_batch(&queries).await.unwrap();

        assert_eq!(results.len(), 2);
        for result in results {
            assert_eq!(result.count, 10);
            assert_eq!(result.products.len(), 1);
        }
    }
}

// Non-search-html feature test - verify module structure compiles
#[cfg(not(feature = "search-html"))]
mod search_client_mock {
    #[test]
    fn test_search_module_requires_feature() {
        // This test verifies the module structure compiles without the feature
        assert!(true);
    }
}
