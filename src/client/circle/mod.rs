//! Interfaces related to circle only. For more information, see [`CircleClient`].
//!
//! **Note:** This entire module requires the `search-html` feature flag.

mod query;

use scraper::{Html, Selector};

use super::search::{SearchProductItem, SearchResult};
use super::DlsiteClient;
use crate::error::Result;
use crate::utils::ToParseError as _;

pub use self::query::CircleQuery;

/// Determine the name bucket for circle name resolution.
///
/// DLsite organizes circles by the first character of their name:
/// - Hiragana (あ-わ) → bucket by first character
/// - Katakana (ア-ワ) → bucket by first character
/// - Alphabetic (A-Z) → bucket is "a"
/// - Numeric (0-9) → bucket is "0"
fn get_name_bucket(name: &str) -> &'static str {
    let first_char = name.chars().next().unwrap_or('a');

    // Hiragana ranges
    match first_char {
        'あ'..='お' => "あ",
        'か'..='ご' => "か",
        'さ'..='ぞ' => "さ",
        'た'..='ど' => "た",
        'な'..='の' => "な",
        'は'..='ぽ' => "は",
        'ま'..='も' => "ま",
        'や' | 'ゆ' | 'よ' => "や",
        'ら'..='ろ' => "ら",
        'わ'..='ん' => "わ",
        // Katakana ranges
        'ア'..='オ' => "ア",
        'カ'..='ゴ' => "カ",
        'サ'..='ゾ' => "サ",
        'タ'..='ド' => "タ",
        'ナ'..='ノ' => "ナ",
        'ハ'..='ポ' => "ハ",
        'マ'..='モ' => "マ",
        'ヤ' | 'ユ' | 'ヨ' => "ヤ",
        'ラ'..='ロ' => "ラ",
        'ワ'..='ン' => "ワ",
        // Alphabetic
        'a'..='z' | 'A'..='Z' => "a",
        // Numeric
        '0'..='9' => "0",
        // Default to "a" for other characters
        _ => "a",
    }
}

/// Basic profile metadata for a DLsite circle (maker).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircleProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub banner_url: Option<String>,
}

/// Client to get circle-related content from DLsite.
///
/// Enable it in your `Cargo.toml`:
/// ```toml
/// dlsite-rs = { version = "0.2", features = ["search-html"] }
/// ```
#[derive(Clone, Debug)]
pub struct CircleClient<'a> {
    pub(crate) c: &'a DlsiteClient,
}

impl<'a> CircleClient<'a> {
    /// Fetch basic profile metadata for a circle by scraping the circle's HTML page.
    ///
    /// This reuses the same HTTP request as [`Self::get_circle`] (the response is cached),
    /// so calling both methods for the same circle is inexpensive.
    ///
    /// # Arguments
    /// * `circle_id` - The circle/maker ID. Example: `RG24350`.
    ///
    /// # Returns
    /// [`CircleProfile`] with `id`, `name`, `description`, and `banner_url`.
    /// `description` and `banner_url` are `None` if not found on the page.
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

    /// List all games from a circle, filtering out non-game works.
    ///
    /// Game work types include: Action (ACN), Quiz (QIZ), Adventure (ADV),
    /// RPG, Table (TBL), Digital Novel (DNV), Simulation (SLN),
    /// Typing (TYP), Shooting (STG), Puzzle (PZL), and Other Games (ETC).
    ///
    /// # Arguments
    /// * `maker_id` - The circle/maker ID. Example: `RG24350`.
    ///
    /// # Returns
    /// `Vec<SearchProductItem>` containing only game-type products.
    ///
    /// # Example
    /// ```ignore
    /// use dlsite_rs::DlsiteClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     let games = client.circle().list_circle_games("RG24350").await.unwrap();
    ///     for game in games {
    ///         println!("{}: {}", game.id, game.title);
    ///     }
    /// }
    /// ```
    pub async fn list_circle_games(&self, maker_id: &str) -> Result<Vec<SearchProductItem>> {
        let result = self.get_circle(maker_id, &CircleQuery::default()).await?;
        Ok(result
            .products
            .into_iter()
            .filter(|p| p.work_type.is_game())
            .collect())
    }

    /// Resolve a circle name to its maker ID.
    ///
    /// This method scrapes the DLsite circle list page to find a circle
    /// by its exact name and returns the maker ID if found.
    ///
    /// # Arguments
    /// * `circle_name` - The exact circle name to search for.
    ///
    /// # Returns
    /// * `Some(String)` - The maker ID if the circle is found.
    /// * `None` - If no circle matches the exact name.
    ///
    /// # Example
    /// ```ignore
    /// use dlsite_rs::DlsiteClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     if let Some(maker_id) = client.circle().resolve_circle_name("Example Circle").await.unwrap() {
    ///         println!("Found maker ID: {}", maker_id);
    ///     }
    /// }
    /// ```
    pub async fn resolve_circle_name(&self, circle_name: &str) -> Result<Option<String>> {
        let bucket = get_name_bucket(circle_name);
        let path = format!("/home/circle/list/=/name_header/{}", bucket);
        let html = self.c.get(&path).await?;
        let html = Html::parse_document(&html);

        // Find the circle link with matching name
        let selector = Selector::parse(".circle_list a, .maker_list a").unwrap();
        for element in html.select(&selector) {
            if let Some(name) = element.text().next() {
                if name.trim() == circle_name {
                    if let Some(href) = element.value().attr("href") {
                        // Extract maker_id from URL like /circle/profile/=/maker_id/RG12345.html
                        if let Some(maker_id) = href
                            .split('/')
                            .next_back()
                            .and_then(|s| s.strip_suffix(".html"))
                        {
                            return Ok(Some(maker_id.to_string()));
                        }
                    }
                }
            }
        }

        Ok(None)
    }
}
