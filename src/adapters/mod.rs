//! Adapters for DLsite Play v3 API
//!
//! These adapters provide access to authenticated DLsite Play features:
//! - Library access (owned works)
//! - Download handoff URLs
//! - Viewer session management
//!
//! All adapters require the `cookie-store` feature and an authenticated session.

#[cfg(feature = "cookie-store")]
pub mod play_library;

#[cfg(feature = "cookie-store")]
pub mod download;

#[cfg(feature = "cookie-store")]
pub mod viewer;
