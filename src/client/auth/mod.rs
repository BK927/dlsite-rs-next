//! Authentication client for DLsite session management.
//!
//! # Status: Stub
//!
//! This module is a placeholder. Authentication requires:
//! 1. A confirmed login endpoint URL (not yet captured from network traffic).
//! 2. Cookie store support — enable the `cookie-store` feature.
//! 3. CSRF token handling (presence and format unverified).
//!
//! # Enabling
//!
//! This module is only available with the `cookie-store` feature:
//! ```toml
//! [dependencies]
//! dlsite-rs = { features = ["cookie-store"] }
//! ```

use super::DlsiteClient;

/// Client for DLsite authentication and session management.
///
/// # TODO
///
/// Implement `login(username, password)` once the login endpoint and CSRF
/// token flow are confirmed via network capture.
#[derive(Clone, Debug)]
pub struct AuthClient<'a> {
    #[allow(dead_code)]
    pub(crate) c: &'a DlsiteClient,
}

impl<'a> AuthClient<'a> {
    // TODO: Add login(), logout(), is_authenticated() once the auth flow is verified.
    // See docs/dlsite_gap_analysis.md for investigation notes.
}
