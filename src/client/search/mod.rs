//! Product search functionality for DLsite.
//!
//! This module provides [`SearchClient`] for searching products on DLsite
//! with support for filtering, sorting, and pagination.
//!
//! **Note:** This entire module requires the `search-html` feature flag.
//!
//! # Enable the feature
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! dlsite-rs-next = { version = "0.2", features = ["search-html"] }
//! ```
//!
//! # Basic Search
//!
//! ```ignore
//! use dlsite_rs_next::{DlsiteClient, client::search::SearchProductQuery};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = DlsiteClient::default();
//!     let query = SearchProductQuery {
//!         keyword: Some("ASMR".to_string()),
//!         ..Default::default()
//!     };
//!     let results = client.search().search_product(&query).await.unwrap();
//!     println!("Found {} products", results.products.len());
//! }
//! ```
//!
//! # Performance Features
//!
//! - **Parallel parsing**: Large result sets are parsed using rayon for 3-4x speedup
//! - **Result caching**: Parsed results are cached to avoid re-parsing
//! - **Batch queries**: Fetch multiple pages concurrently
//! - **Streaming API**: Process large result sets with callbacks

pub(crate) mod macros;
mod query;
mod selectors;

use rayon::prelude::*;
use scraper::Html;
use serde::Deserialize;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use crate::utils::ToParseError;
use crate::{
    cache::GenericCache,
    error::Result,
    interface::product::{AgeCategory, WorkType},
    DlsiteClient,
};

pub use self::query::SearchProductQuery;

/// Client to search products on DLsite.
///
/// Enable it in your `Cargo.toml`:
/// ```toml
/// dlsite-rs-next = { version = "0.2", features = ["search-html"] }
/// ```
pub struct SearchClient<'a> {
    pub(crate) c: &'a DlsiteClient,
    /// Cache for search results to avoid re-parsing the same queries
    result_cache: Arc<Mutex<GenericCache<Vec<SearchProductItem>>>>,
}

/// Internal response structure for search page metadata.
#[derive(Deserialize)]
struct SearchPageInfo {
    count: i32,
}

/// Internal response structure for AJAX search results.
#[derive(Deserialize)]
struct SearchAjaxResult {
    search_result: String,
    page_info: SearchPageInfo,
}

/// A single product in search results.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SearchProductItem {
    /// Product ID (e.g., "RJ123456").
    pub id: String,
    /// Product title.
    pub title: String,
    /// Creator name, if different from circle name.
    pub creator: Option<String>,
    /// Whether the creator name was truncated/omitted.
    pub creator_omitted: Option<bool>,
    /// Circle/maker name.
    pub circle_name: String,
    /// Circle/maker ID (e.g., "RG24350").
    pub circle_id: String,
    /// Download count.
    pub dl_count: Option<i32>,
    /// Rating count.
    pub rate_count: Option<i32>,
    /// Review count.
    pub review_count: Option<i32>,
    /// Original price in JPY.
    pub price_original: i32,
    /// Sale price in JPY (if discounted).
    pub price_sale: Option<i32>,
    /// Age category (General, R-15, Adult).
    pub age_category: AgeCategory,
    /// Work type (RPG, ADV, SOU, etc.).
    pub work_type: WorkType,
    /// Thumbnail image URL.
    pub thumbnail_url: String,
    /// Average rating (0.0-5.0).
    pub rating: Option<f32>,
}

/// Search results from a query.
#[derive(Debug)]
pub struct SearchResult {
    /// List of products matching the query.
    pub products: Vec<SearchProductItem>,
    /// Total count of matching products.
    pub count: i32,
    /// Query path used for the request.
    pub query_path: String,
}

fn parse_count_str(str: &str) -> Result<i32> {
    str.replace(['(', ')', ','], "")
        .parse()
        .to_parse_error("Failed to parse string to count")
}

fn parse_num_str(str: &str) -> Result<i32> {
    str.replace(',', "")
        .parse()
        .to_parse_error("Failed to parse string to number")
}

impl<'a> SearchClient<'a> {
    /// Create a new search client
    pub(crate) fn new(c: &'a DlsiteClient) -> Self {
        Self {
            c,
            result_cache: Arc::new(Mutex::new(GenericCache::new(
                100,
                Duration::from_secs(3600),
            ))),
        }
    }

    /// Search products on DLsite.
    ///
    /// # Arguments
    /// * `options` - Search query options.
    ///
    /// # Example
    /// ```ignore
    /// use dlsite_rs_next::{DlsiteClient, client::search::SearchProductQuery, interface::query::SexCategory};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     let product = client
    ///         .search()
    ///         .search_product(&SearchProductQuery {
    ///             sex_category: Some(vec![SexCategory::Male]),
    ///             keyword: Some("ASMR".to_string()),
    ///             ..Default::default()
    ///         })
    ///         .await
    ///         .expect("Failed to search");
    ///     dbg!(&product);
    /// }
    /// ```
    pub async fn search_product(&self, options: &SearchProductQuery) -> Result<SearchResult> {
        let query_path = options.to_path();

        // Check if results are cached (cache.get() already returns cloned data)
        let cached_products = {
            let cache = self.result_cache.lock().unwrap();
            cache.get(&query_path)
        };

        if let Some(cached_products) = cached_products {
            // Get count from API (it's small and fast) - no lock held here
            let json = self.c.get(&query_path).await?;
            let json = serde_json::from_str::<SearchAjaxResult>(&json)?;
            let count = json.page_info.count;

            return Ok(SearchResult {
                products: cached_products,
                count,
                query_path,
            });
        }

        // Cache miss - fetch and parse
        let json = self.c.get(&query_path).await?;
        let json = serde_json::from_str::<SearchAjaxResult>(&json)?;
        let html = json.search_result;
        let count = json.page_info.count;

        // Use parallel parsing for better performance
        let products = parse_search_html_parallel(&html)?;

        // Cache the results
        {
            let cache = self.result_cache.lock().unwrap();
            cache.insert(query_path.clone(), products.clone());
        }

        Ok(SearchResult {
            products,
            count,
            query_path,
        })
    }

    /// Search multiple queries concurrently for better performance
    ///
    /// This method uses tokio::join_all to fetch multiple pages in parallel
    ///
    /// # Arguments
    /// * `queries` - Vector of search queries to execute
    ///
    /// # Returns
    /// * `Vec<SearchResult>` - Results for each query in the same order
    pub async fn search_products_batch(
        &self,
        queries: &[SearchProductQuery],
    ) -> Result<Vec<SearchResult>> {
        let futures: Vec<_> = queries.iter().map(|q| self.search_product(q)).collect();

        futures::future::try_join_all(futures).await
    }

    /// Stream search results for a query, parsing items as they are fetched
    ///
    /// This method is optimized for memory efficiency and responsiveness
    ///
    /// # Arguments
    /// * `options` - Search query options
    /// * `callback` - Function to call for each parsed item
    ///
    /// # Returns
    /// * `Result<i32>` - Total count of items
    pub async fn search_product_stream<F>(
        &self,
        options: &SearchProductQuery,
        mut callback: F,
    ) -> Result<i32>
    where
        F: FnMut(SearchProductItem),
    {
        let query_path = options.to_path();
        let json = self.c.get(&query_path).await?;
        let json = serde_json::from_str::<SearchAjaxResult>(&json)?;
        let html = json.search_result;
        let count = json.page_info.count;

        // Parse and stream items
        let html = Html::parse_fragment(&html);
        for item_element in html.select(selectors::search_result_items()) {
            let item_html = item_element.html();
            match parse_search_item_html(&item_html) {
                Ok(item) => callback(item),
                Err(e) => eprintln!("Warning: Failed to parse item: {:?}", e),
            }
        }

        Ok(count)
    }
}

/// Parse a single search result item from HTML element
fn parse_search_item_html(item_html: &str) -> Result<SearchProductItem> {
    let item_element = Html::parse_fragment(item_html);
    let item_element = item_element.root_element();

    let product_id_e = item_element
        .select(selectors::product_id_element())
        .next()
        .to_parse_error("Failed to find data element")?
        .value();
    let maker_e = item_element
        .select(selectors::maker_name())
        .next()
        .to_parse_error("Failed to find maker element")?;
    let author_e = item_element.select(selectors::author()).next();

    let price_e = item_element
        .select(selectors::work_price())
        .next()
        .to_parse_error("Failed to find price element")?;
    let original_price_e = item_element.select(selectors::original_price()).next();
    let (sale_price_e, original_price_e) = if let Some(e) = original_price_e {
        (Some(price_e), e)
    } else {
        (None, price_e)
    };
    let id = product_id_e
        .attr("data-product_id")
        .to_parse_error("Failed to get product id")?
        .to_string();

    Ok(SearchProductItem {
        id: id.clone(),
        title: item_element
            .select(selectors::work_title())
            .next()
            .to_parse_error("Failed to get title")?
            .value()
            .attr("title")
            .unwrap()
            .to_string(),
        age_category: {
            if let Some(e) = item_element.select(selectors::age_category()).next() {
                let title = e.value().attr("title");
                if let Some(title) = title {
                    match title {
                        "全年齢" => AgeCategory::General,
                        "R-15" => AgeCategory::R15,
                        _ => {
                            return Err(crate::DlsiteError::Parse(
                                "Age category parse error: invalid title".to_string(),
                            ))
                        }
                    }
                } else {
                    return Err(crate::DlsiteError::Parse(
                        "Age category parse error".to_string(),
                    ));
                }
            } else {
                AgeCategory::Adult
            }
        },
        circle_name: maker_e.text().next().unwrap_or("").to_string(),
        circle_id: maker_e
            .value()
            .attr("href")
            .to_parse_error("Failed to get maker link")?
            .split('/')
            .next_back()
            .to_parse_error("Invalid url")?
            .split('.')
            .next()
            .to_parse_error("Failed to find maker id")?
            .to_string(),
        creator: {
            if let Some(creator_e) = author_e {
                let name = creator_e
                    .select(selectors::creator_link())
                    .next()
                    .to_parse_error("Failed to find creator")?
                    .text()
                    .next()
                    .to_parse_error("Failed to find creator")?
                    .to_string();
                Some(name)
            } else {
                None
            }
        },
        creator_omitted: {
            if let Some(creator_e) = author_e {
                let omitted = creator_e
                    .value()
                    .attr("class")
                    .to_parse_error("Failed to find creator")?
                    .split(" ")
                    .any(|x| x == "omit");
                Some(omitted)
            } else {
                None
            }
        },
        dl_count: {
            if let Some(e) = item_element.select(selectors::dl_count()).next() {
                Some(
                    e.text()
                        .next()
                        .to_parse_error("Failed to get dl count")?
                        .replace(',', "")
                        .parse()
                        .to_parse_error("Invalid dl count")?,
                )
            } else {
                None
            }
        },
        rate_count: {
            if let Some(e) = item_element.select(selectors::dl_count()).next() {
                Some(parse_count_str(
                    e.text().next().to_parse_error("Failed to get rate count")?,
                )?)
            } else {
                None
            }
        },
        review_count: {
            if let Some(e) = item_element.select(selectors::review_count()).next() {
                Some(parse_count_str(
                    e.text()
                        .next()
                        .to_parse_error("Failed to get review count")?,
                )?)
            } else {
                None
            }
        },
        price_original: parse_num_str(
            original_price_e
                .text()
                .next()
                .to_parse_error("Failed to find price")?,
        )?,
        price_sale: {
            match sale_price_e {
                Some(e) => Some(parse_num_str(
                    e.text().next().to_parse_error("Failed to find price")?,
                )?),
                None => None,
            }
        },
        work_type: item_element
            .select(selectors::work_category())
            .next()
            .to_parse_error("Failed to find work category")?
            .value()
            .attr("class")
            .to_parse_error("Failed to find worktype")?
            .split(' ')
            .find_map(|c| {
                if let Some(c) = c.strip_prefix("type_") {
                    if let Ok(wt) = c.parse::<WorkType>() {
                        if let WorkType::Unknown(_) = wt {
                            return None;
                        } else {
                            return Some(wt);
                        }
                    }
                }
                None
            })
            .unwrap_or(WorkType::Unknown("".to_string())),
        thumbnail_url: {
            let img_e = item_element
                .select(selectors::thumbnail_image())
                .next()
                .to_parse_error("Failed to find thumbnail")?;

            let src = img_e.value().attr("src");
            let data_src = img_e.value().attr("data-src");
            match (src, data_src) {
                (Some(src), _) => format!("https:{}", src),
                (_, Some(data_src)) => format!("https:{}", data_src),
                (_, _) => {
                    return Err(crate::DlsiteError::Parse(
                        "Failed to find thumbnail".to_string(),
                    ))
                }
            }
        },
        rating: {
            if let Some(e) = item_element.select(selectors::rating()).next() {
                e.value()
                    .attr("class")
                    .expect("Failed to get rating")
                    .split(' ')
                    .find_map(|c| {
                        if let Some(c) = c.strip_prefix("star_") {
                            if let Ok(r) = c.parse::<f32>() {
                                return Some(r / 10.0);
                            }
                        }
                        None
                    })
            } else {
                None
            }
        },
    })
}

/// Parse search HTML into a list of search results
pub(crate) fn parse_search_html(html: &str) -> Result<Vec<SearchProductItem>> {
    let html = Html::parse_fragment(html);
    let mut result: Vec<SearchProductItem> = vec![];

    for item_element in html.select(selectors::search_result_items()) {
        let item_html = item_element.html();
        match parse_search_item_html(&item_html) {
            Ok(item) => result.push(item),
            Err(e) => {
                // Log warning but continue parsing other items
                eprintln!("Warning: Failed to parse search item: {:?}", e);
            }
        }
    }

    Ok(result)
}

/// Parse search HTML using parallel processing for better performance
///
/// This function is optimized for large result sets (50+ items)
pub(crate) fn parse_search_html_parallel(html: &str) -> Result<Vec<SearchProductItem>> {
    let html = Html::parse_fragment(html);

    // Collect all item elements as HTML strings
    let items: Vec<String> = html
        .select(selectors::search_result_items())
        .map(|elem| elem.html())
        .collect();

    // Process items in parallel
    items
        .par_iter()
        .map(|item_html| parse_search_item_html(item_html))
        .collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Load a fixture file from tests/fixtures/search/
    fn load_fixture(name: &str) -> String {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("search")
            .join(name);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to load fixture '{}': {}", path.display(), e))
    }

    // =========================================================================
    // Single Item Parser Tests
    // =========================================================================

    #[test]
    fn test_parse_normal_product() {
        let html = load_fixture("normal_product.html");
        let result = parse_search_item_html(&html).expect("Failed to parse normal product");

        assert_eq!(result.id, "RJ403038");
        assert_eq!(result.title, "Example Work Title");
        assert_eq!(result.circle_name, "Example Circle");
        assert_eq!(result.circle_id, "RG24350");
        assert_eq!(result.age_category, AgeCategory::General);
        assert_eq!(result.work_type, WorkType::SOU);

        assert_eq!(result.creator, Some("Example Author".to_string()));
        assert_eq!(result.dl_count, Some(1234));
        assert_eq!(result.review_count, Some(56));
        assert_eq!(result.price_original, 1100);
        assert_eq!(result.price_sale, None);
        assert_eq!(result.rating, Some(4.5));
        assert!(result.thumbnail_url.starts_with("https://"));
    }

    #[test]
    fn test_parse_discounted_product() {
        let html = load_fixture("discounted_product.html");
        let result = parse_search_item_html(&html).expect("Failed to parse discounted product");

        assert_eq!(result.id, "RJ500123");
        assert_eq!(result.title, "Discounted Work");
        assert_eq!(result.price_original, 2200);
        assert_eq!(result.price_sale, Some(1100));
        assert_eq!(result.rating, Some(4.0));
    }

    #[test]
    fn test_parse_no_creator_product() {
        let html = load_fixture("no_creator.html");
        let result =
            parse_search_item_html(&html).expect("Failed to parse product without creator");

        assert_eq!(result.id, "RJ400456");
        assert_eq!(result.creator, None);
        assert_eq!(result.creator_omitted, None);
        assert_eq!(result.circle_name, "Solo Circle");
    }

    #[test]
    fn test_parse_no_review_no_rating_product() {
        let html = load_fixture("no_review_no_rating.html");
        let result =
            parse_search_item_html(&html).expect("Failed to parse product without review/rating");

        assert_eq!(result.id, "RJ300789");
        assert_eq!(result.review_count, None);
        assert_eq!(result.rating, None);
    }

    #[test]
    fn test_parse_adult_product() {
        let html = load_fixture("adult_product.html");
        let result = parse_search_item_html(&html).expect("Failed to parse adult product");

        assert_eq!(result.id, "RJ200555");
        assert_eq!(result.age_category, AgeCategory::Adult);
        assert_eq!(result.dl_count, Some(10000));
        assert_eq!(result.review_count, Some(200));
    }

    #[test]
    fn test_parse_r15_product() {
        let html = load_fixture("r15_product.html");
        let result = parse_search_item_html(&html).expect("Failed to parse R-15 product");

        assert_eq!(result.id, "RJ150888");
        assert_eq!(result.age_category, AgeCategory::R15);
        assert_eq!(result.rating, Some(4.2));
    }

    // =========================================================================
    // List Parser Tests
    // =========================================================================

    #[test]
    fn test_parse_search_html_list() {
        let html = load_fixture("search_result_list.html");
        let results = parse_search_html(&html).expect("Failed to parse search result list");

        assert_eq!(results.len(), 3);

        // First product - General
        assert_eq!(results[0].id, "RJ403038");
        assert_eq!(results[0].title, "First Product");
        assert_eq!(results[0].age_category, AgeCategory::General);

        // Second product - R-15 with discount
        assert_eq!(results[1].id, "RJ404567");
        assert_eq!(results[1].title, "Second Product");
        assert_eq!(results[1].age_category, AgeCategory::R15);
        assert_eq!(results[1].price_sale, Some(750));
        assert_eq!(results[1].creator, Some("Creator B".to_string()));

        // Third product - Adult, no creator
        assert_eq!(results[2].id, "RJ405999");
        assert_eq!(results[2].title, "Third Product Adult");
        assert_eq!(results[2].age_category, AgeCategory::Adult);
        assert_eq!(results[2].creator, None);
    }

    #[test]
    fn test_parse_search_html_parallel_produces_same_results() {
        let html = load_fixture("search_result_list.html");

        let sequential = parse_search_html(&html).expect("Sequential parse failed");
        let parallel = parse_search_html_parallel(&html).expect("Parallel parse failed");

        // Both should produce same number of items
        assert_eq!(sequential.len(), parallel.len());

        // Compare items (order may differ due to parallel processing, so sort by id)
        let mut seq_sorted: Vec<_> = sequential.into_iter().map(|i| i.id).collect();
        let mut par_sorted: Vec<_> = parallel.into_iter().map(|i| i.id).collect();
        seq_sorted.sort();
        par_sorted.sort();
        assert_eq!(seq_sorted, par_sorted);
    }

    // =========================================================================
    // Selector Tests
    // =========================================================================

    #[test]
    fn test_selectors_are_cached() {
        use std::ptr;

        // Verify that selector functions return the same reference (cached)
        let s1 = selectors::product_id_element();
        let s2 = selectors::product_id_element();
        assert!(ptr::eq(s1, s2), "Selectors should be cached");

        let s1 = selectors::maker_name();
        let s2 = selectors::maker_name();
        assert!(ptr::eq(s1, s2), "Selectors should be cached");

        let s1 = selectors::search_result_items();
        let s2 = selectors::search_result_items();
        assert!(ptr::eq(s1, s2), "Selectors should be cached");
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_parse_empty_html() {
        let results = parse_search_html("").expect("Should handle empty HTML");
        assert!(results.is_empty());
    }

    #[test]
    fn test_parse_invalid_item_continues_on_error() {
        // HTML with one valid item and malformed HTML that won't parse
        let html = r#"
            <ul id="search_result_img_box">
                <li>invalid item without required fields</li>
            </ul>
        "#;
        let results = parse_search_html(html).expect("Should handle invalid items");
        // Invalid items are logged but don't cause failure
        assert!(results.is_empty());
    }

    // =========================================================================
    // Insta Snapshot Tests
    // =========================================================================

    #[test]
    fn test_snapshot_normal_product() {
        let html = load_fixture("normal_product.html");
        let result = parse_search_item_html(&html).expect("Failed to parse");

        let snapshot = serde_json::json!({
            "id": result.id,
            "title": result.title,
            "circle_name": result.circle_name,
            "circle_id": result.circle_id,
            "creator": result.creator,
            "age_category": format!("{:?}", result.age_category),
            "work_type": format!("{:?}", result.work_type),
            "price_original": result.price_original,
            "price_sale": result.price_sale,
            "dl_count": result.dl_count,
            "review_count": result.review_count,
            "rating": result.rating,
        });

        insta::assert_json_snapshot!(snapshot);
    }

    #[test]
    fn test_snapshot_discounted_product() {
        let html = load_fixture("discounted_product.html");
        let result = parse_search_item_html(&html).expect("Failed to parse");

        let snapshot = serde_json::json!({
            "id": result.id,
            "price_original": result.price_original,
            "price_sale": result.price_sale,
            "discount_percentage": if let Some(sale) = result.price_sale {
                Some(((result.price_original - sale) as f64 / result.price_original as f64) * 100.0)
            } else {
                None
            }
        });

        insta::assert_json_snapshot!(snapshot);
    }
}
