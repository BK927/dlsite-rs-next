//! Product client for fetching DLsite product data via multiple APIs.
//!
//! This module provides [`ProductClient`] for retrieving comprehensive product
//! information by combining data from multiple DLsite API endpoints:
//!
//! - **AJAX API**: Basic info (title, price, ratings, sales count)
//! - **Product API**: Detailed info (maker, genres, creators, images)
//! - **Review API**: Reviewer demographics and reviews
//!
//! # Example
//!
//! ```no_run
//! use dlsite_rs_next::DlsiteClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = DlsiteClient::default();
//!
//!     // Get comprehensive product info (combines all APIs)
//!     let product = client.product().get_all("RJ123456").await.unwrap();
//!     println!("{} by {}", product.title, product.circle_name.unwrap_or_default());
//!
//!     // Get just AJAX data (faster, less detail)
//!     let ajax = client.product().get_ajax("RJ123456").await.unwrap();
//!     println!("Price: {} JPY", ajax.price);
//!
//!     // Get reviews
//!     use dlsite_rs_next::client::product::review::ReviewSortOrder;
//!     let reviews = client.product()
//!         .get_review("RJ123456", 10, 1, true, ReviewSortOrder::New)
//!         .await
//!         .unwrap();
//! }
//! ```

use std::collections::HashMap;

use super::product_api::interface::{Creator, Creators, GenreApi};
use crate::{
    error::Result,
    interface::genre::Genre,
    interface::product::{AgeCategory, WorkType},
    interface::query::Language,
    utils::ToParseError as _,
    DlsiteClient, DlsiteError,
};
use ajax::ProductAjax;
use chrono::NaiveDate;

pub mod ajax;
pub mod review;

/// Client to retrieve DLsite product data using JSON APIs.
///
/// # API Methods
///
/// There are multiple ways to get product data from DLsite:
/// 1. **AJAX API** (`get_ajax`): Returns product info from the AJAX endpoint
/// 2. **Product API** (`product_api`): Returns detailed product info from the JSON API
/// 3. **Review API** (`get_review`): Returns product reviews
/// 4. **Combined** (`get_all`): Combines all APIs for complete product info
///
/// The `get_all` method is recommended for most use cases as it provides
/// the most comprehensive product information.
#[derive(Clone, Debug)]
pub struct ProductClient<'a> {
    pub(crate) c: &'a DlsiteClient,
}

/// A product on DLsite.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Product {
    pub id: String,
    pub title: String,
    pub work_type: WorkType,
    pub released_at: Option<NaiveDate>,
    pub age_rating: Option<AgeCategory>,
    pub genre: Vec<Genre>,
    pub circle_id: Option<String>,
    pub circle_name: Option<String>,
    pub price: i32,
    pub series: Option<String>,
    pub sale_count: Option<i32>,
    pub review_count: Option<i32>,
    pub rating: Option<f32>,
    pub rate_count: Option<i32>,
    pub images: Vec<String>,
    pub people: Option<ProductPeople>,
    pub reviewer_genre: Vec<(Genre, i32)>,
    pub file_size: Option<String>,
}

/// People who contributed to a product on DLsite.
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct ProductPeople {
    pub author: Option<Vec<String>>,
    pub scenario: Option<Vec<String>>,
    pub illustrator: Option<Vec<String>>,
    pub voice_actor: Option<Vec<String>>,
}

impl From<Creators> for ProductPeople {
    fn from(creators: Creators) -> Self {
        Self {
            author: creators
                .created_by
                .map(|v: Vec<Creator>| v.into_iter().map(|c: Creator| c.name).collect()),
            scenario: creators
                .scenario_by
                .map(|v: Vec<Creator>| v.into_iter().map(|c: Creator| c.name).collect()),
            illustrator: creators
                .illust_by
                .map(|v: Vec<Creator>| v.into_iter().map(|c: Creator| c.name).collect()),
            voice_actor: creators
                .voice_by
                .map(|v: Vec<Creator>| v.into_iter().map(|c: Creator| c.name).collect()),
        }
    }
}

fn parse_date(date_str: &str) -> Option<NaiveDate> {
    // Try parsing various date formats
    // Format: "YYYY-MM-DD HH:MM:SS" or "YYYY-MM-DD"
    let date_str = date_str.split(' ').next()?;
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

impl<'a> ProductClient<'a> {
    /// Get full information about a product by combining multiple JSON APIs.
    ///
    /// This method fetches data from:
    /// - AJAX API: Basic product info (title, price, ratings, sales)
    /// - Product API: Detailed info (maker, genres, creators, images)
    /// - Review API: Reviewer genre breakdown
    ///
    /// # Arguments
    /// * `product_id` - The product ID to get information about. Example: `RJ123456`. NOTE: This must be capitalized.
    ///
    /// # Example
    /// ```no_run
    /// use dlsite_rs_next::DlsiteClient;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     let product = client.product().get_all("RJ123456").await.unwrap();
    ///     println!("{:#?}", product);
    /// }
    /// ```
    pub async fn get_all(&self, product_id: &str) -> Result<Product> {
        // Fetch all data concurrently
        let ajax_future = self.get_ajax(product_id);
        let product_api_client = self.c.product_api();
        let api_future = product_api_client.get(product_id);
        let review_future = self.get_review(product_id, 6, 1, true, review::ReviewSortOrder::New);

        let (ajax_data, api_data, review_data) =
            tokio::try_join!(ajax_future, api_future, review_future)?;

        // Convert genres from API format
        let genre: Vec<Genre> = api_data
            .genres
            .into_iter()
            .map(|g: GenreApi| Genre {
                name: g.name,
                id: g.id.to_string(),
            })
            .collect();

        // Convert creators to ProductPeople
        let people = api_data.creators.map(ProductPeople::from);

        // Extract images from API data
        let images: Vec<String> = api_data
            .image_samples
            .map(|samples| samples.into_iter().map(|f| f.url).collect())
            .unwrap_or_default();

        Ok(Product {
            id: product_id.to_string(),
            title: ajax_data.work_name,
            work_type: ajax_data.work_type,
            released_at: api_data.regist_date.as_ref().and_then(|d| parse_date(d)),
            age_rating: Some(api_data.age_category),
            genre,
            series: api_data.series_name,
            circle_id: Some(api_data.maker_id),
            circle_name: Some(api_data.maker_name),
            price: ajax_data.price,
            rating: ajax_data.rate_average_2dp,
            rate_count: ajax_data.rate_count,
            sale_count: ajax_data.dl_count,
            review_count: ajax_data.review_count,
            images,
            people,
            reviewer_genre: review_data.reviewer_genre_list.unwrap_or_default(),
            file_size: api_data.file_size,
        })
    }

    /// Fetch detailed product information using 'ajax api'.
    pub async fn get_ajax(&self, product_id: &str) -> Result<ProductAjax> {
        let path = format!("/product/info/ajax?product_id={}", product_id);
        let ajax_json_str = self.c.get(&path).await?;

        let mut json: HashMap<String, ProductAjax> = serde_json::from_str(&ajax_json_str)?;
        let product = json
            .remove(product_id)
            .ok_or_else(|| DlsiteError::Parse("Failed to parse ajax json".to_string()))?;

        Ok(product)
    }

    /// Fetch detailed multiple products information using 'ajax api'.
    ///
    /// It is more efficient to use this method than calling `get_ajax` multiple times.
    #[tracing::instrument(err)]
    pub async fn get_ajax_multiple(
        &self,
        product_ids: Vec<&str>,
    ) -> Result<HashMap<String, ProductAjax>> {
        let path = format!("/product/info/ajax?product_id={}", product_ids.join(","));
        let ajax_json_str = self.c.get(&path).await?;

        let json: HashMap<String, ProductAjax> = serde_json::from_str(&ajax_json_str)?;

        Ok(json)
    }

    /// Get product reviews and related informations using 'review api'.
    ///
    /// Uses Japanese locale (`ja_JP`). For other locales, use [`Self::get_review_with_locale`].
    ///
    /// # Arguments
    /// * `product_id` - Product ID.
    /// * `limit` - Number of reviews to get.
    /// * `page` - Page number.
    /// * `mix_pickup` - Mixes picked up review. To get user genre, this must be true.
    /// * `order` - Sort order of reviews.
    ///
    /// # Returns
    /// Product reviews and related informations.
    #[tracing::instrument(err, skip_all)]
    pub async fn get_review(
        &self,
        product_id: &str,
        limit: u32,
        page: u32,
        mix_pickup: bool,
        order: review::ReviewSortOrder,
    ) -> Result<review::ProductReview> {
        self.get_review_with_locale(product_id, limit, page, mix_pickup, order, Language::Jp)
            .await
    }

    /// Get product reviews with an explicit locale.
    ///
    /// # Arguments
    /// * `product_id` - Product ID.
    /// * `limit` - Number of reviews to get.
    /// * `page` - Page number.
    /// * `mix_pickup` - Mixes picked up review. To get user genre, this must be true.
    /// * `order` - Sort order of reviews.
    /// * `locale` - Locale for the review response language.
    ///
    /// # Returns
    /// Product reviews and related informations.
    #[tracing::instrument(err, skip_all)]
    pub async fn get_review_with_locale(
        &self,
        product_id: &str,
        limit: u32,
        page: u32,
        mix_pickup: bool,
        order: review::ReviewSortOrder,
        locale: Language,
    ) -> Result<review::ProductReview> {
        let order_str = match order {
            review::ReviewSortOrder::New => "regist_d",
            review::ReviewSortOrder::Top => "top",
        };

        let path = format!(
            "/api/review?product_id={}&limit={}&mix_pickup={}&page={}&order={}&locale={}",
            product_id,
            limit,
            mix_pickup,
            page,
            order_str,
            locale.to_review_locale()
        );
        let json_str = self.c.get(&path).await?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;

        if !json["is_success"]
            .as_bool()
            .to_parse_error("Failed to parse revire json")?
        {
            let message = json["error_msg"]
                .as_str()
                .unwrap_or("Failed to get error message");
            return Err(crate::DlsiteError::Server(format!(
                "Failed to get review: {}",
                message
            )));
        }

        let json: review::ProductReview = serde_json::from_value(json)?;
        Ok(json)
    }
}
