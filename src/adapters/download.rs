//! Download handoff API adapter
//!
//! Provides access to download URLs for purchased works.
//!
//! **Requires `cookie-store` feature flag.**

use serde::Deserialize;

use crate::{error::Result, DlsiteClient};

/// Adapter for DLsite Play v3 download API
pub struct DownloadAdapter<'a> {
    client: &'a DlsiteClient,
}

impl<'a> DownloadAdapter<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(client: &'a DlsiteClient) -> Self {
        Self { client }
    }

    /// Get download target URL
    ///
    /// GET <https://play.dlsite.com/api/v3/download?workno=WORK_ID>
    ///
    /// This endpoint returns a 302 redirect to the actual download URL.
    /// The response contains the signed download URL and metadata.
    ///
    /// # Arguments
    /// * `workno` - The work ID (e.g., "RJ123456")
    ///
    /// # Returns
    /// Download target information including the signed URL
    pub async fn get_download_target(&self, workno: &str) -> Result<DownloadTarget> {
        let url = format!("https://play.dlsite.com/api/v3/download?workno={}", workno);

        let json_str = self.client.get_raw(&url).await?;
        let result: DownloadTarget = serde_json::from_str(&json_str)?;

        Ok(result)
    }
}

/// Download target information
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "unknown-field-error", serde(deny_unknown_fields))]
pub struct DownloadTarget {
    /// The work ID
    pub workno: String,
    /// Signed download URL (may expire after some time)
    pub url: Option<String>,
    /// File name for the download
    pub filename: Option<String>,
    /// File size in bytes
    pub filesize: Option<i64>,
    /// Expiration timestamp for the download URL
    pub expires_at: Option<String>,
    /// Whether the download is available
    pub is_available: Option<bool>,
    /// Error message if download is not available
    pub error: Option<String>,
}
