//! Product API client for fetching DLsite product data via JSON API.
//!
//! This module provides [`ProductApiClient`] for retrieving detailed product
//! information from DLsite's JSON API endpoints.
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
//!     // Get product details
//!     let product = client.product_api().get("RJ123456").await.unwrap();
//!     println!("Product: {}", product.work_name);
//!
//!     // Get thumbnail URL
//!     let thumbnail = client.product_api().get_product_thumbnail("RJ123456").await.unwrap();
//!     println!("Thumbnail: {}", thumbnail);
//!
//!     // Get screenshot URLs
//!     let screenshots = client.product_api().list_product_screenshots("RJ123456").await.unwrap();
//!     for url in screenshots {
//!         println!("Screenshot: {}", url);
//!     }
//! }
//! ```

pub mod interface;

use crate::{error::Result, interface::query::Language, DlsiteClient, DlsiteError};

use self::interface::ProductApiContent;

/// Client to retrieve DLsite product data using 'scraping' method
///
/// For difference about "scraping" and "api" method, see [`super::product::ProductClient`].
#[derive(Clone, Debug)]
pub struct ProductApiClient<'a> {
    pub(crate) c: &'a DlsiteClient,
}

impl<'a> ProductApiClient<'a> {
    /// Get product detail using api.
    ///
    /// Uses the client's default locale. For an explicit locale, use [`Self::get_with_locale`].
    ///
    /// # Arguments
    /// * `id` - Product ID.
    ///
    /// # Returns
    /// * [`ProductApiContent`] - Product details.
    ///
    /// # Note
    /// This api does not return dl count.
    ///
    /// # Example
    /// ```no_run
    /// use dlsite_rs_next::DlsiteClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     let product = client.product_api().get("RJ01014447").await.unwrap();
    ///     assert_eq!(product.creators.unwrap().voice_by.unwrap()[0].name, "佐倉綾音");
    /// }
    /// ```
    pub async fn get(&self, id: &str) -> Result<ProductApiContent> {
        self.get_with_locale(id, self.c.default_locale().clone())
            .await
    }

    /// Get product detail using api with an explicit locale.
    ///
    /// # Arguments
    /// * `id` - Product ID.
    /// * `locale` - Locale to use for the response language.
    ///
    /// # Returns
    /// * [`ProductApiContent`] - Product details.
    pub async fn get_with_locale(&self, id: &str, locale: Language) -> Result<ProductApiContent> {
        let json = self
            .c
            .get(&format!(
                "/api/=/product.json?workno={}&locale={}",
                id,
                locale.to_review_locale()
            ))
            .await?;
        let jd = &mut serde_json::Deserializer::from_str(&json);
        #[cfg(feature = "unknown-field-log")]
        let result: std::result::Result<Vec<ProductApiContent>, _> = serde_ignored::deserialize(
            jd,
            |path| {
                tracing::error!("Ignored path: '{}' for '{id}'. Please report this to https://github.com/BK927/dlsite-rs-next", path.to_string());
            },
        );
        #[cfg(not(feature = "unknown-field-log"))]
        let result: std::result::Result<Vec<ProductApiContent>, _> =
            serde_path_to_error::deserialize(jd);
        match result {
            Ok(result) => {
                let Some(json) = result.into_iter().next() else {
                    return Err(DlsiteError::Parse("No product found".to_string()));
                };

                Ok(json)
            }
            Err(e) => Err(DlsiteError::Parse(format!("Failed to parse json: {}", e))),
        }
    }

    /// Get the thumbnail URL for a product.
    ///
    /// # Arguments
    /// * `id` - Product ID.
    ///
    /// # Returns
    /// * `String` - URL to the product's thumbnail image.
    ///
    /// # Example
    /// ```no_run
    /// use dlsite_rs_next::DlsiteClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     let thumbnail_url = client.product_api().get_product_thumbnail("RJ01014447").await.unwrap();
    ///     println!("Thumbnail: {}", thumbnail_url);
    /// }
    /// ```
    pub async fn get_product_thumbnail(&self, id: &str) -> Result<String> {
        let product = self.get(id).await?;
        Ok(product.image_thum.url)
    }

    /// List all screenshot URLs for a product.
    ///
    /// # Arguments
    /// * `id` - Product ID.
    ///
    /// # Returns
    /// * `Vec<String>` - List of screenshot URLs. Empty if no samples available.
    ///
    /// # Example
    /// ```no_run
    /// use dlsite_rs_next::DlsiteClient;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = DlsiteClient::default();
    ///     let screenshots = client.product_api().list_product_screenshots("RJ01014447").await.unwrap();
    ///     for url in screenshots {
    ///         println!("Screenshot: {}", url);
    ///     }
    /// }
    /// ```
    pub async fn list_product_screenshots(&self, id: &str) -> Result<Vec<String>> {
        let product = self.get(id).await?;
        Ok(product
            .image_samples
            .unwrap_or_default()
            .into_iter()
            .map(|file| file.url)
            .collect())
    }
}
