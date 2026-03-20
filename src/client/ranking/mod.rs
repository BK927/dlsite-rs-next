//! Ranking client for DLsite trending and bestseller lists.
//!
//! # Status
//!
//! This module is a **placeholder**. The ranking endpoint URL and response shape
//! have not been verified against live DLsite traffic. Do not implement actual
//! HTTP calls here until the endpoint is confirmed via network capture.
//!
//! Known unknowns:
//! - Exact API path for trending/bestseller lists
//! - Whether the endpoint is JSON (ajax) or HTML (scraped)
//! - Which query parameters are supported (site, category, period, locale)
//! - Authentication requirements

use super::DlsiteClient;

/// Client for DLsite ranking data.
///
/// # TODO
///
/// All methods here are unimplemented. Before implementing, capture the network
/// traffic from the DLsite website or app to determine:
///
/// 1. The ranking API endpoint URL.
/// 2. The response format (JSON vs HTML).
/// 3. Supported query parameters (category, period, locale, page).
/// 4. Whether authentication is required.
#[derive(Clone, Debug)]
pub struct RankingClient<'a> {
    pub(crate) c: &'a DlsiteClient,
}

impl<'a> RankingClient<'a> {
    // TODO: Add get_trending(), get_bestseller(), get_curated() once endpoint URLs
    // are confirmed via network capture. See docs/dlsite_gap_analysis.md for details.
}
