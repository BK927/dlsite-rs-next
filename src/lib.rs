#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
//! # Feature flags
#![cfg_attr(
    feature = "document-features",
    cfg_attr(doc, doc = ::document_features::document_features!())
)]

pub mod cache;
pub mod client;
pub mod error;
pub mod interface;
pub mod retry;
mod utils;

#[cfg(feature = "cookie-store")]
pub mod adapters;

pub use cache::{GenericCache, ResponseCache};
pub use client::{DlsiteClient, DlsiteClientBuilder};
pub use error::DlsiteError;
pub use retry::RetryConfig;
