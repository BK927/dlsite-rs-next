//! Search query builder for DLsite product search.
//!
//! Use [`SearchProductQuery`] to construct search queries with various filters
//! and options. All fields are optional and default to DLsite's default values.
//!
//! # Example
//!
//! ```ignore
//! use dlsite_rs::{DlsiteClient, client::search::SearchProductQuery, interface::query::*};
//!
//! let query = SearchProductQuery {
//!     keyword: Some("ASMR".to_string()),
//!     sex_category: Some(vec![SexCategory::Male]),
//!     order: Some(Order::Trend),
//!     ..Default::default()
//! };
//!
//! let client = DlsiteClient::default();
//! let results = client.search().search_product(&query).await?;
//! ```

use crate::client::search::macros::*;
use crate::interface::product::*;
use crate::interface::query::*;

// Struct that can be converted dlsite url (below is example). All params are optional.
// https://www.dlsite.com/maniax/fsr/=
// /language/jp
// /sex_category%5B0%5D/male
// /keyword/a
// /regist_date_end/2022-08-25
// /regist_date_start/2022-01-01
// /price_low/801
// /price_high/1000
// /ana_flg/on
// /age_category%5B0%5D/r15
// /work_category%5B0%5D/doujin
// /order%5B0%5D/trend
// /work_type_category%5B0%5D/audio
// /work_type_category_name%5B0%5D/%E3%83%9C%E3%82%A4%E3%82%B9%E3%83%BBASMR
// /genre%5B0%5D/497
// /genre_name%5B0%5D/ASMR
// /options_and_or/and
// /options%5B0%5D/JPN/options%5B1%5D/NM
// /options_not%5B0%5D/AIG/options_not%5B1%5D/AIP
// /options_name%5B0%5D/%E6%97%A5%E6%9C%AC%E8%AA%9E%E4%BD%9C%E5%93%81/options_name%5B1%5D/%E8%A8%80%E8%AA%9E%E4%B8%8D%E5%95%8F%E4%BD%9C%E5%93%81
// /rate_average%5B0%5D/2
// /per_page/30
// /page/1
// /campaign/campaign
// /soon/1
// /dlsite_only/1
// /is_pointup/1
// /is_free/1
// /release_term/old
// /price_category/4
// /show_type/1
// /from/fs.detail

/// Search query parameters for DLsite product search.
///
/// All fields are optional. Use [`Default::default()`] to create a query with
/// default values (Japanese language, no filters).
///
/// # Example
///
/// ```ignore
/// use dlsite_rs::client::search::SearchProductQuery;
/// use dlsite_rs::interface::query::{SexCategory, Order};
///
/// let query = SearchProductQuery {
///     keyword: Some("ASMR".to_string()),
///     sex_category: Some(vec![SexCategory::Male]),
///     order: Some(Order::Trend),
///     per_page: Some(50),
///     ..Default::default()
/// };
/// ```
#[derive(Default)]
pub struct SearchProductQuery {
    /// Display language for the search interface.
    pub language: Language,
    /// Filter by creator/circle name.
    pub keyword_creator: Option<String>,
    /// Filter by target audience sex category (Male/Female).
    pub sex_category: Option<Vec<SexCategory>>,
    /// Free-text keyword search.
    pub keyword: Option<String>,
    /// End date for registration date filter (format: "YYYY-MM-DD").
    pub regist_date_end: Option<String>,
    /// Start date for registration date filter (format: "YYYY-MM-DD").
    pub regist_date_start: Option<String>,
    /// Minimum price filter (in JPY).
    pub price_low: Option<u32>,
    /// Maximum price filter (in JPY).
    pub price_high: Option<u32>,
    /// Sales status filter (on sale, reserved, etc.).
    pub ana_flg: Option<AnaFlg>,
    /// Age category filter (General, R-15, Adult).
    pub age_category: Option<Vec<AgeCategory>>,
    /// Work category filter (Doujin, Books, PC, App).
    pub work_category: Option<Vec<WorkCategory>>,
    /// Sort order for results.
    pub order: Option<Order>,
    /// Individual work type filter (ACN, RPG, etc.).
    pub work_type: Option<Vec<WorkType>>,
    /// Work type category filter (Game, Comic, Audio, etc.).
    pub work_type_category: Option<Vec<WorkTypeCategory>>,
    /// Work type category display names (for URL construction).
    pub work_type_category_name: Option<Vec<String>>,
    /// Genre IDs to filter by.
    pub genre: Option<Vec<u32>>,
    /// Genre display names (for URL construction).
    pub genre_name: Option<Vec<String>>,
    /// Logical operator for combining options (AND/OR).
    pub options_and_or: Option<OptionAndOr>,
    /// Product options to include (e.g., "JPN" for Japanese language).
    pub options: Option<Vec<String>>,
    /// Product options to exclude (e.g., "AIG" for AI-generated).
    pub options_not: Option<Vec<String>>,
    /// Option display names (for URL construction).
    pub options_name: Option<Vec<String>>,
    /// File type filter (EXE, PDF, MP3, etc.).
    pub file_type: Option<Vec<FileType>>,
    /// Minimum average rating filter (1-5).
    pub rate_average: Option<u32>,
    /// Number of results per page (30, 50, or 100).
    pub per_page: Option<u32>,
    /// Page number for pagination (1-indexed).
    pub page: Option<u32>,
    /// Filter to products in campaigns/sales.
    pub campaign: Option<bool>,
    /// Filter to products with sale ending within 24 hours.
    pub soon: Option<bool>,
    /// Filter to DLsite-exclusive products.
    pub dlsite_only: Option<bool>,
    /// Filter to products currently offering bonus points.
    pub is_pointup: Option<bool>,
    /// Filter to free products.
    pub is_free: Option<bool>,
    /// Release date range filter.
    pub release_term: Option<ReleaseTerm>,
    /// Price category filter.
    pub price_category: Option<u32>,
    /// Show type filter.
    pub show_type: Option<u32>,
    /// Referral source tracking parameter.
    pub from: Option<String>,
}

impl SearchProductQuery {
    /// Convert the struct to a path, which can be used to make a request to the dlsite.
    pub fn to_path(&self) -> String {
        let mut path = "/fsr/ajax/=".to_string();

        push!(path, self, language);
        push_option!(path, self, keyword_creator);
        push_option_array!(path, self, sex_category);
        push_option!(path, self, keyword);
        push_option!(path, self, regist_date_end);
        push_option!(path, self, regist_date_start);
        push_option!(path, self, price_low);
        push_option!(path, self, price_high);
        push_option!(path, self, ana_flg);
        push_option_array!(path, self, age_category);
        push_option_array!(path, self, work_category);
        push_option!(path, self, order);
        push_option_array!(path, self, work_type);
        push_option_array!(path, self, work_type_category);
        push_option_array!(path, self, work_type_category_name);
        push_option_array!(path, self, genre);
        push_option_array!(path, self, genre_name);
        push_option!(path, self, options_and_or);
        push_option_array!(path, self, options);
        push_option_array!(path, self, options_not);
        push_option_array!(path, self, options_name);
        push_option_array!(path, self, file_type);
        push_option!(path, self, rate_average);
        push_option!(path, self, per_page);
        push_option!(path, self, page);
        if let Some(true) = &self.campaign {
            path.push_str("/campaign/campaign");
        }
        push_option_bool!(path, self, soon);
        push_option_bool!(path, self, dlsite_only);
        push_option_bool!(path, self, is_pointup);
        push_option_bool!(path, self, is_free);
        push_option!(path, self, release_term);
        push_option!(path, self, price_category);
        push_option!(path, self, show_type);
        push_option!(path, self, from);

        path
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        client::search::SearchProductQuery,
        interface::{
            product::FileType,
            query::{Language, SexCategory},
        },
    };

    #[test]
    fn product_search_param_default() {
        assert_eq!(
            "/fsr/ajax/=/language/jp",
            SearchProductQuery::default().to_path()
        );
    }

    #[test]
    fn product_search_param_1() {
        assert_eq!(
            "/fsr/ajax/=/language/jp/sex_category[0]/male/price_low/801/file_type[0]/PNG/file_type[1]/EXE/soon/1",
            SearchProductQuery {
                sex_category: Some(vec![SexCategory::Male]),
                price_low: Some(801),
                file_type: Some(vec![FileType::PNG, FileType::EXE]),
                soon: Some(true),
                is_free: Some(false),
                ..Default::default()
            }
            .to_path()
        );
    }

    #[test]
    fn campaign_true_produces_path_segment() {
        let path = SearchProductQuery {
            campaign: Some(true),
            ..Default::default()
        }
        .to_path();
        assert!(
            path.contains("/campaign/campaign"),
            "expected /campaign/campaign in {path}"
        );
    }

    #[test]
    fn campaign_false_omits_path_segment() {
        let path = SearchProductQuery {
            campaign: Some(false),
            ..Default::default()
        }
        .to_path();
        assert!(
            !path.contains("campaign"),
            "expected no campaign segment in {path}"
        );
    }

    #[test]
    fn campaign_none_omits_path_segment() {
        let path = SearchProductQuery {
            campaign: None,
            ..Default::default()
        }
        .to_path();
        assert!(
            !path.contains("campaign"),
            "expected no campaign segment in {path}"
        );
    }

    #[test]
    fn regist_date_start_in_path() {
        let path = SearchProductQuery {
            regist_date_start: Some("2022-01-01".to_string()),
            ..Default::default()
        }
        .to_path();
        assert!(
            path.contains("/regist_date_start/2022-01-01"),
            "got: {path}"
        );
    }

    #[test]
    fn genre_name_array_in_path() {
        let path = SearchProductQuery {
            genre_name: Some(vec!["ASMR".to_string()]),
            ..Default::default()
        }
        .to_path();
        assert!(path.contains("/genre_name[0]/ASMR"), "got: {path}");
    }

    #[test]
    fn options_name_array_in_path() {
        let path = SearchProductQuery {
            options_name: Some(vec!["日本語作品".to_string()]),
            ..Default::default()
        }
        .to_path();
        assert!(path.contains("/options_name[0]/日本語作品"), "got: {path}");
    }

    #[test]
    fn work_type_category_name_array_in_path() {
        let path = SearchProductQuery {
            work_type_category_name: Some(vec!["ボイス・ASMR".to_string()]),
            ..Default::default()
        }
        .to_path();
        assert!(
            path.contains("/work_type_category_name[0]/ボイス・ASMR"),
            "got: {path}"
        );
    }

    #[test]
    fn show_type_in_path() {
        let path = SearchProductQuery {
            show_type: Some(1),
            ..Default::default()
        }
        .to_path();
        assert!(path.contains("/show_type/1"), "got: {path}");
    }

    #[test]
    fn from_in_path() {
        let path = SearchProductQuery {
            from: Some("fs.detail".to_string()),
            ..Default::default()
        }
        .to_path();
        assert!(path.contains("/from/fs.detail"), "got: {path}");
    }

    #[test]
    fn dlsite_only_true_in_path() {
        let path = SearchProductQuery {
            dlsite_only: Some(true),
            ..Default::default()
        }
        .to_path();
        assert!(path.contains("/dlsite_only/1"), "got: {path}");
    }

    #[test]
    fn dlsite_only_false_omitted() {
        let path = SearchProductQuery {
            dlsite_only: Some(false),
            ..Default::default()
        }
        .to_path();
        assert!(!path.contains("dlsite_only"), "got: {path}");
    }

    #[test]
    fn price_category_in_path() {
        let path = SearchProductQuery {
            price_category: Some(4),
            ..Default::default()
        }
        .to_path();
        assert!(path.contains("/price_category/4"), "got: {path}");
    }

    #[test]
    fn language_en_produces_en_segment() {
        let path = SearchProductQuery {
            language: Language::En,
            ..Default::default()
        }
        .to_path();
        assert_eq!("/fsr/ajax/=/language/en", path);
    }
}
