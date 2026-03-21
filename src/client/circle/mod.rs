//! Interfaces related to circle only. For more information, see [`CircleClient`].

mod query;

#[cfg(feature = "search-html")]
use scraper::{Html, Selector};

use super::{
    search::SearchResult,
    DlsiteClient,
};
use crate::error::Result;
#[cfg(feature = "search-html")]
use crate::utils::ToParseError as _;

pub use self::query::CircleQuery;

/// Basic profile metadata for a DLsite circle (maker).
///
/// **Note:** Profile data is only available with the `search-html` feature flag.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircleProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub banner_url: Option<String>,
}

/// Client to get circle-related content from DLsite.
///
/// **Note:** Circle functionality requires the `search-html` feature flag because
/// it relies on HTML parsing of circle pages.
///
/// Enable it in your `Cargo.toml`:
/// ```toml
/// dlsite-gamebox = { version = "0.2", features = ["search-html"] }
/// ```
#[derive(Clone, Debug)]
pub struct CircleClient<'a> {
    pub(crate) c: &'a DlsiteClient,
}

impl<'a> CircleClient<'a> {
    /// Fetch basic profile metadata for a circle by scraping the circle's HTML page.
    ///
    /// **Requires `search-html` feature flag.**
    ///
    /// This reuses the same HTTP request as [`get_circle`] (the response is cached),
    /// so calling both methods for the same circle is inexpensive.
    ///
    /// # Arguments
    /// * `circle_id` - The circle/maker ID. Example: `RG24350`.
    ///
    /// # Returns
    /// [`CircleProfile`] with `id`, `name`, `description`, and `banner_url`.
    /// `description` and `banner_url` are `None` if not found on the page.
    #[cfg(feature = "search-html")]
    pub async fn get_circle_profile(&self, circle_id: &str) -> Result<CircleProfile> {
        // Uses the default CircleQuery path so the response is shared with get_circle calls
        let query_path = CircleQuery::default().to_path(circle_id);
        let html_str = self.c.get(&query_path).await?;
        let html = Html::parse_fragment(&html_str);

        let name = html
            .select(&Selector::parse(".maker_name").unwrap())
            .next()
            .and_then(|e| e.text().next().map(|t| t.trim().to_string()))
            .unwrap_or_else(|| circle_id.to_string());

        let description = html
            .select(&Selector::parse(".maker_introduction").unwrap())
            .next()
            .map(|e| e.text().collect::<Vec<_>>().join("").trim().to_string())
            .filter(|s| !s.is_empty());

        let banner_url = html
            .select(&Selector::parse(".maker_header img").unwrap())
            .next()
            .and_then(|e| e.value().attr("src").map(|s| s.to_string()));

        Ok(CircleProfile {
            id: circle_id.to_string(),
            name,
            description,
            banner_url,
        })
    }

    /// Search circle-related products.
    ///
    /// **Requires `search-html` feature flag.**
    #[cfg(feature = "search-html")]
    pub async fn get_circle(&self, circle_id: &str, options: &CircleQuery) -> Result<SearchResult> {
        use super::search::parse_search_html;

        let query_path = options.to_path(circle_id);
        let html = self.c.get(&query_path).await?;
        let html = Html::parse_fragment(&html);
        let products_html = html
            .select(&Selector::parse("#search_result_list").unwrap())
            .next()
            .to_parse_error("Product list not found")?;

        let count: i32 = html
            .select(&Selector::parse(".page_total > strong").unwrap())
            .next()
            .to_parse_error("No total item count found")?
            .text()
            .next()
            .to_parse_error("No total item count found 2")?
            .parse()
            .to_parse_error("Failed to parse total item count")?;

        let products = parse_search_html(&products_html.html())?;

        Ok(SearchResult {
            products,
            count,
            query_path,
        })
    }
}
