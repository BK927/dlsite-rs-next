//! DLsite Play client for streaming purchased content.
//!
//! # Status: Stub
//!
//! This module is a placeholder. DLsite Play streaming requires:
//! 1. An authenticated session (see `auth` module).
//! 2. The streaming API endpoint URL (not yet captured from network traffic).
//! 3. Understanding of the streaming protocol (HLS, proprietary, or other).
//!
//! # Enabling
//!
//! This module is only available with the `cookie-store` feature:
//! ```toml
//! [dependencies]
//! dlsite-gamebox = { features = ["cookie-store"] }
//! ```

use super::DlsiteClient;

/// Client for DLsite Play streaming functionality.
///
/// # TODO
///
/// Implement streaming methods once the Play API endpoints and protocol
/// are confirmed via network capture from the DLsite app or website.
#[derive(Clone, Debug)]
pub struct PlayClient<'a> {
    pub(crate) c: &'a DlsiteClient,
}

impl<'a> PlayClient<'a> {
    // TODO: Add stream_product(), get_play_token() etc. once the Play API is verified.
    // See docs/dlsite_gap_analysis.md for investigation notes.
}
