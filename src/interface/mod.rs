//! Common interface types for the DLsite API.
//!
//! This module contains data types used across the library for representing
//! DLsite concepts like products, search queries, and site configuration.
//!
//! # Modules
//!
//! - [`product`]: Product-related types (work types, age categories, file types)
//! - [`query`]: Query parameters for search and filtering (languages, sort orders)
//! - [`site`]: DLsite site/subdomain enumeration
//! - [`genre`]: Genre representation

pub mod product;
pub mod query;
pub mod site;
pub mod genre {
    //! Genre representation for DLsite products.
    //!
    //! Genres are categories assigned to products on DLsite (e.g., "ASMR", "RPG").
    //! Each genre has a display name and a numeric ID used in API queries.

    /// A genre associated with a DLsite product.
    #[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Genre {
        /// Display name of the genre (e.g., "ASMR").
        pub name: String,
        /// Numeric genre ID used in API queries.
        pub id: String,
    }
}
