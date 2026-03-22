# dlsite-rs

This is a library to get information about products on DLsite. Some information
is not available on the HTML page, so this library also makes requests to the
AJAX API.

## Performance

The library includes several performance optimizations that provide significant speedups:

| Optimization | Speedup | Use Case |
|--------------|---------|----------|
| Parallel Parsing | 3-4x | Large result sets (50+ items) |
| Result Caching | 10-100x | Repeated queries |
| Batch Queries | 2-3x | Multi-page queries |
| Streaming API | -50% memory | Large result processing |
| Selector Caching | 5-10% | All queries |
| **Combined** | **10-100x** | **Typical usage** |

See `docs/dlsite_endpoint_inventory.md` for the full endpoint coverage matrix.

## NOTE

- This library is still wip, and the API may change.
- Only the parts I needed are implemented, so there are many unimplemented
  parts. PR is welcome.
- Especially in the JSON API, the DLsite side may change the specification.
  Expect breaking changes due to such changes.

## Implemented features

- [ ] Get product information by scraping html and using ajax api for web.
  - [x] Basic information
  - [ ] Additional information
  - [x] Multi-language support via `Language` enum (`Jp`, `En`, `Ko`, `ZhCn`, `ZhTw`)
        (HTML scraping still uses Japanese page; locale applies to review and API calls)
- [x] Get product review (with locale support via `get_review_with_locale`)
- [x] Get product information using api (with locale support via `get_with_locale`)
- [x] Get product thumbnail (`ProductApiClient::get_product_thumbnail`)
- [x] Get product screenshots (`ProductApiClient::list_product_screenshots`)
- [x] Search product (all documented parameters now implemented)
- [x] Site abstraction via `Site` enum (`Maniax`, `Books`, `Soft`, `Pro`, `Appx`, `Comic`, `Home`, `Custom`)
- [x] Get circle info
  - [x] Get circle product list
  - [x] Get circle profile metadata (`get_circle_profile`)
  - [x] Get circle games only (`CircleClient::list_circle_games`)
  - [x] Resolve circle name to maker_id (`CircleClient::resolve_circle_name`)
  - [ ] Get circle sale list (needs network capture)
- [x] Work type helpers (`WorkType::is_game()`)
- [ ] Login and user related feature (stubs behind `cookie-store` feature)
- [ ] Get ranking (stub — endpoint URL needs network capture verification)

## Features

### Performance Optimizations
- **Parallel Parsing**: 3-4x faster search result parsing using rayon
- **Result Caching**: 10-100x faster repeated queries with LRU cache
- **Batch Queries**: 2-3x faster multi-page queries with concurrent requests
- **Streaming API**: 50% less memory usage for large result sets
- **Selector Caching**: 5-10% faster parsing with pre-compiled CSS selectors

### Reliability Features
- **Rate Limiting**: Automatic 2 requests/second to prevent IP bans
- **Retry Logic**: Automatic retry with exponential backoff for transient failures
- **Connection Pooling**: Configurable connection pool for better resource usage

## Example

### Basic Usage

- Get product by api

  ```rust,no_run
  use dlsite_gamebox::DlsiteClient;

  #[tokio::main]
  async fn main() {
      let client = DlsiteClient::default();
      let product = client.product_api().get("RJ01014447").await.unwrap();
      assert_eq!(product.creators.unwrap().voice_by.unwrap()[0].name, "佐倉綾音");
  }
  ```

- Get product thumbnail and screenshots

  ```rust,no_run
  use dlsite_gamebox::DlsiteClient;

  #[tokio::main]
  async fn main() {
      let client = DlsiteClient::default();

      // Get thumbnail URL
      let thumbnail = client.product_api().get_product_thumbnail("RJ01014447").await.unwrap();
      println!("Thumbnail: {}", thumbnail);

      // Get screenshot URLs
      let screenshots = client.product_api().list_product_screenshots("RJ01014447").await.unwrap();
      for url in screenshots {
          println!("Screenshot: {}", url);
      }
  }
  ```

- Search products (with automatic parallel parsing and caching)
  **Note: Requires `search-html` feature flag**

  ```rust,ignore
  use dlsite_gamebox::{DlsiteClient, client::search::SearchProductQuery, interface::query::SexCategory};

  #[tokio::main]
  async fn main() {
      let client = DlsiteClient::default();
      let query = SearchProductQuery {
          sex_category: Some(vec![SexCategory::Male]),
          keyword: Some("ASMR".to_string()),
          ..Default::default()
      };
      let results = client
          .search()
          .search_product(&query)
          .await
          .expect("Failed to search");
      println!("Found {} products", results.products.len());
  }
  ```

### Advanced Usage

- Batch query multiple pages concurrently
  **Note: Requires `search-html` feature flag**

  ```rust,ignore
  use dlsite_gamebox::{DlsiteClient, client::search::SearchProductQuery, interface::query::SexCategory};

  #[tokio::main]
  async fn main() {
      let client = DlsiteClient::default();
      let queries = vec![
          SearchProductQuery {
              sex_category: Some(vec![SexCategory::Male]),
              page: Some(1),
              ..Default::default()
          },
          SearchProductQuery {
              sex_category: Some(vec![SexCategory::Male]),
              page: Some(2),
              ..Default::default()
          },
      ];

      let results = client
          .search()
          .search_products_batch(&queries)
          .await
          .expect("Failed to search");

      for (i, result) in results.iter().enumerate() {
          println!("Page {}: {} products", i + 1, result.products.len());
      }
  }
  ```

- Stream large result sets with callback
  **Note: Requires `search-html` feature flag**

  ```rust,ignore
  use dlsite_gamebox::{DlsiteClient, client::search::SearchProductQuery, interface::query::SexCategory};

  #[tokio::main]
  async fn main() {
      let client = DlsiteClient::default();
      let query = SearchProductQuery {
          sex_category: Some(vec![SexCategory::Male]),
          ..Default::default()
      };

      let total = client
          .search()
          .search_product_stream(&query, |item| {
              println!("Processing: {} ({})", item.title, item.id);
          })
          .await
          .expect("Failed to search");

      println!("Total items: {}", total);
  }
  ```

- Get circle games and resolve circle name
  **Note: Requires `search-html` feature flag**

  ```rust,ignore
  use dlsite_gamebox::DlsiteClient;

  #[tokio::main]
  async fn main() {
      let client = DlsiteClient::default();

      // List all games from a circle (filters out non-game works)
      let games = client.circle().list_circle_games("RG24350").await.unwrap();
      for game in games {
          println!("{}: {}", game.id, game.title);
      }

      // Resolve circle name to maker_id
      if let Some(maker_id) = client.circle().resolve_circle_name("Circle Name").await.unwrap() {
          println!("Found maker ID: {}", maker_id);
      }
  }
  ```

- Custom client configuration

  ```rust,no_run
  use dlsite_gamebox::{DlsiteClient, RetryConfig};
  use std::time::Duration;

  #[tokio::main]
  async fn main() {
      let client = DlsiteClient::builder("https://www.dlsite.com/maniax")
          .pool_max_idle_per_host(20)  // Increase connection pool
          .timeout(Duration::from_secs(60))  // Increase timeout
          .cache(200, Duration::from_secs(7200))  // Larger cache, 2 hour TTL
          .retry_config(RetryConfig::new(
              5,  // Max 5 retries
              Duration::from_millis(200),  // Initial delay 200ms
              Duration::from_secs(30),  // Max delay 30s
          ))
          .build();

      // Use the custom client
      let product = client.product_api().get("RJ01014447").await.unwrap();
      println!("Product: {}", product.work_name);
  }
  ```
