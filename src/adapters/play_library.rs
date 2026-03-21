//! Play v3 Library API adapter
//!
//! Provides access to the user's library of purchased works.
//!
//! **Requires `cookie-store` feature flag.**

use serde::Deserialize;

use crate::{error::Result, DlsiteClient, DlsiteError};

/// Adapter for DLsite Play v3 library API
pub struct PlayLibraryAdapter<'a> {
    client: &'a DlsiteClient,
}

impl<'a> PlayLibraryAdapter<'a> {
    pub(crate) fn new(client: &'a DlsiteClient) -> Self {
        Self { client }
    }

    /// Get library count
    ///
    /// GET https://play.dlsite.com/api/v3/content/count?last=N
    ///
    /// # Arguments
    /// * `last` - Optional pagination token/ID
    ///
    /// # Returns
    /// The total count of works in the user's library
    pub async fn get_count(&self, last: Option<i32>) -> Result<LibraryCount> {
        let url = match last {
            Some(l) => format!("https://play.dlsite.com/api/v3/content/count?last={}", l),
            None => "https://play.dlsite.com/api/v3/content/count".to_string(),
        };

        let json_str = self.client.get_raw(&url).await?;
        let result: LibraryCount = serde_json::from_str(&json_str)?;

        Ok(result)
    }

    /// Get library sales (purchased works)
    ///
    /// GET https://play.dlsite.com/api/v3/content/sales?last=N
    ///
    /// # Arguments
    /// * `last` - Optional pagination token/ID for getting works after this ID
    ///
    /// # Returns
    /// List of library entries (purchased works)
    pub async fn get_sales(&self, last: Option<i32>) -> Result<Vec<LibraryEntry>> {
        let url = match last {
            Some(l) => format!("https://play.dlsite.com/api/v3/content/sales?last={}", l),
            None => "https://play.dlsite.com/api/v3/content/sales".to_string(),
        };

        let json_str = self.client.get_raw(&url).await?;
        let result: Vec<LibraryEntry> = serde_json::from_str(&json_str)?;

        Ok(result)
    }

    /// Get works by IDs (batch lookup)
    ///
    /// POST https://play.dlsite.com/api/v3/content/works
    ///
    /// # Arguments
    /// * `worknos` - List of work IDs to fetch (e.g., ["RJ123456", "RJ789012"])
    ///
    /// # Returns
    /// List of library entries for the requested works
    pub async fn get_works(&self, worknos: &[&str]) -> Result<Vec<LibraryEntry>> {
        let url = "https://play.dlsite.com/api/v3/content/works";

        // Note: This requires POST with JSON body
        // For now, we'll return an error indicating this needs implementation
        // The actual implementation would need reqwest::Client to support POST
        Err(DlsiteError::Parse(
            "POST /api/v3/content/works not yet implemented - requires POST support".to_string(),
        ))
    }
}

/// Library count response
#[derive(Debug, Clone, Deserialize)]
pub struct LibraryCount {
    /// Total number of works in library
    pub count: i32,
    /// Whether there are more works to fetch
    pub has_more: Option<bool>,
}

/// Library entry representing a purchased work
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "unknown-field-error", serde(deny_unknown_fields))]
pub struct LibraryEntry {
    /// Work ID (e.g., "RJ123456")
    pub workno: String,
    /// Work name/title
    pub work_name: String,
    /// Maker/circle ID
    pub maker_id: Option<String>,
    /// Maker/circle name
    pub maker_name: Option<String>,
    /// Purchase date
    pub purchase_date: Option<String>,
    /// Whether the work is downloadable
    pub is_downloadable: Option<bool>,
    /// Whether the work supports DLsite Play streaming
    pub is_play_available: Option<bool>,
    /// File size (human-readable string)
    pub file_size: Option<String>,
    /// Main image URL
    pub image_main: Option<String>,
    /// Thumbnail image URL
    pub image_thumb: Option<String>,
}
