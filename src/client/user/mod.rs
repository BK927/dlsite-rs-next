//! User library and purchase history client.
//!
//! # Status: Stub
//!
//! This module is a placeholder. User data access requires:
//! 1. An authenticated session (see `auth` module).
//! 2. The user API endpoint URLs (not yet captured from network traffic).
//!
//! # Enabling
//!
//! This module is only available with the `cookie-store` feature:
//! ```toml
//! [dependencies]
//! dlsite-rs = { features = ["cookie-store"] }
//! ```

use super::DlsiteClient;

/// Client for user-specific data: library, purchase history, and wishlist.
///
/// # TODO
///
/// Implement `get_library()`, `get_purchase_history()` once the user API
/// endpoints are confirmed via network capture.
#[derive(Clone, Debug)]
pub struct UserClient<'a> {
    #[allow(dead_code)]
    pub(crate) c: &'a DlsiteClient,
}

impl<'a> UserClient<'a> {
    // TODO: Add get_library(), get_purchase_history(), get_wishlist() once verified.
    // See docs/dlsite_gap_analysis.md for investigation notes.
}
