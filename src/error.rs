use thiserror::Error;

/// Errors that can occur while using the Dlsite API
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum DlsiteError {
    /// HTTP request error
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// HTTP status code error
    #[error("HTTP error: {0}")]
    HttpStatus(u16),

    /// Rate limit error - too many requests
    #[error("Rate limited: {0}")]
    RateLimit(String),

    /// Request timeout error
    #[error("Request timeout")]
    Timeout,

    /// HTML/JSON parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Server-side error
    #[error("Server error: {0}")]
    Server(String),

    /// Authentication is required to access this resource (HTTP 401/403)
    #[error("Authentication required: {0}")]
    AuthRequired(String),

    /// Session has expired and the user must re-authenticate
    #[error("Session expired: {0}")]
    SessionExpired(String),

    /// The API response shape has drifted from the expected schema
    #[error("Schema drift: {0}")]
    SchemaDrift(String),

    /// Work not found in the DLsite catalog
    #[error("Work not found: {0}")]
    WorkNotFound(String),

    /// Download not available for this work
    #[error("Download not available for work: {0}")]
    DownloadNotAvailable(String),

    /// DLsite Play not available for this work
    #[error("DLsite Play not available for work: {0}")]
    PlayNotAvailable(String),

    /// Feature requires a specific feature flag to be enabled
    #[error("Feature '{feature}' requires '{required_flag}' feature flag")]
    FeatureGated {
        /// The feature that was requested
        feature: &'static str,
        /// The required feature flag
        required_flag: &'static str,
    },
}

pub(crate) type Result<T> = std::result::Result<T, DlsiteError>;
