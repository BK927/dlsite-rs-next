//! Viewer session API adapter
//!
//! Provides access to DLsite Play viewer for streaming content.
//!
//! **Requires `cookie-store` feature flag.**

use serde::Deserialize;

use crate::{error::Result, DlsiteClient, DlsiteError};

/// Adapter for DLsite Play v3 viewer API
pub struct ViewerAdapter<'a> {
    client: &'a DlsiteClient,
}

impl<'a> ViewerAdapter<'a> {
    pub(crate) fn new(client: &'a DlsiteClient) -> Self {
        Self { client }
    }

    /// Get Play manifest token
    ///
    /// GET https://play.dl.dlsite.com/api/v3/download/sign/cookie?workno=WORK_ID
    ///
    /// This endpoint returns a token required for streaming the work content.
    ///
    /// # Arguments
    /// * `workno` - The work ID (e.g., "RJ123456")
    ///
    /// # Returns
    /// Manifest token for the viewer
    pub async fn get_manifest_token(&self, workno: &str) -> Result<ManifestToken> {
        let url = format!(
            "https://play.dl.dlsite.com/api/v3/download/sign/cookie?workno={}",
            workno
        );

        let json_str = self.client.get_raw(&url).await?;
        let result: ManifestToken = serde_json::from_str(&json_str)?;

        Ok(result)
    }

    /// Create viewer session
    ///
    /// POST https://play.dlsite.com/api/v3/viewer/token/{workno}
    ///
    /// This endpoint creates a viewer session for streaming the work.
    ///
    /// # Arguments
    /// * `workno` - The work ID (e.g., "RJ123456")
    ///
    /// # Returns
    /// Viewer session token and ///
    /// **Note:** This requires POST request support which is not yet implemented
    pub async fn create_viewer_session(&self, workno: &str) -> Result<ViewerSession> {
        let url = format!(
            "https://play.dlsite.com/api/v3/viewer/token/{}",
            workno
        );

        // Note: This requires POST with empty or specific body
        // For now, we'll return an error indicating this needs implementation
        Err(DlsiteError::Parse(
            format!("POST /api/v3/viewer/token/{} not yet implemented - requires POST support", workno),
        ))
    }
}

/// Manifest token for viewer access
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "unknown-field-error", serde(deny_unknown_fields))]
pub struct ManifestToken {
    /// The work ID
    pub workno: Option<String>,
    /// Signed URL for the manifest
    pub manifest_url: Option<String>,
    /// Token for authentication
    pub token: Option<String>,
    /// Expiration timestamp
    pub expires_at: Option<String>,
    /// Error message if not available
    pub error: Option<String>,
}

/// Viewer session for streaming content
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "unknown-field-error", serde(deny_unknown_fields))]
pub struct ViewerSession {
    /// The work ID
    pub workno: String,
    /// Session token for viewer
    pub token: Option<String>,
    /// Viewer URL
    pub viewer_url: Option<String>,
    /// Expiration timestamp for the session
    pub expires_at: Option<String>,
    /// Whether the session is valid
    pub is_valid: Option<bool>,
    /// Error message if session creation failed
    pub error: Option<String>,
}
