//! Query parameter types for DLsite API requests.
//!
//! This module contains enums and types used to construct search queries
//! and configure API request parameters.

use strum::Display;

/// Supported languages for DLsite content and API responses.
///
/// Used to specify the locale for product information and reviews.
/// The default is [`Language::Jp`] (Japanese).
#[derive(Display, Default, Clone, Debug, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum Language {
    /// Japanese (ja_JP)
    #[default]
    Jp,
    /// English (en_US)
    En,
    /// Korean (ko_KR)
    Ko,
    /// Simplified Chinese (zh_CN)
    ZhCn,
    /// Traditional Chinese (zh_TW)
    ZhTw,
}

impl Language {
    /// Returns the locale string used by the DLsite review API (e.g. `ja_JP`).
    pub fn to_review_locale(&self) -> &'static str {
        match self {
            Language::Jp => "ja_JP",
            Language::En => "en_US",
            Language::Ko => "ko_KR",
            Language::ZhCn => "zh_CN",
            Language::ZhTw => "zh_TW",
        }
    }
}

/// Target audience sex category for search filtering.
#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum SexCategory {
    /// Male-targeted content.
    Male,
    /// Female-targeted content.
    Female,
}

/// Sales status filter for product searches.
#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum AnaFlg {
    /// Products not on sale.
    Off,
    /// Products currently on sale.
    On,
    /// Reserved/pre-order products.
    Reserve,
    /// All products regardless of sale status.
    All,
}

/// Sort order for search results.
#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum Order {
    /// Trending/popular products.
    Trend,
    /// Newest releases first (新しい).
    Release,
    /// Oldest releases first (古い).
    ReleaseD,
    /// Most downloads first (DL数が多い).
    DlD,
    /// Fewest downloads first (DL数が少ない).
    Dl,
    /// Lowest price first (安い).
    Price,
    /// Highest price first (高い).
    PriceD,
    /// Highest rated first (評価が高い).
    RateD,
    /// Most reviews first (レビューが多い).
    ReviewD,
}

/// Logical operator for combining multiple filter options.
#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum OptionAndOr {
    /// Match all specified options (AND).
    And,
    /// Match any specified option (OR).
    Or,
}

/// Release date range filter for searches.
#[derive(Display)]
pub enum ReleaseTerm {
    /// No date filter.
    None,
    /// Released within the last week.
    Week,
    /// Released within the last month.
    Month,
    /// Released within the last year.
    Year,
    /// Older than one year.
    Old,
}

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn language_display_jp() {
        assert_eq!("jp", Language::Jp.to_string());
    }

    #[test]
    fn language_display_en() {
        assert_eq!("en", Language::En.to_string());
    }

    #[test]
    fn language_display_ko() {
        assert_eq!("ko", Language::Ko.to_string());
    }

    #[test]
    fn language_display_zh_cn() {
        assert_eq!("zh_cn", Language::ZhCn.to_string());
    }

    #[test]
    fn language_display_zh_tw() {
        assert_eq!("zh_tw", Language::ZhTw.to_string());
    }

    #[test]
    fn review_locale_jp() {
        assert_eq!("ja_JP", Language::Jp.to_review_locale());
    }

    #[test]
    fn review_locale_en() {
        assert_eq!("en_US", Language::En.to_review_locale());
    }

    #[test]
    fn review_locale_ko() {
        assert_eq!("ko_KR", Language::Ko.to_review_locale());
    }

    #[test]
    fn review_locale_zh_cn() {
        assert_eq!("zh_CN", Language::ZhCn.to_review_locale());
    }

    #[test]
    fn review_locale_zh_tw() {
        assert_eq!("zh_TW", Language::ZhTw.to_review_locale());
    }

    #[test]
    fn default_language_is_jp() {
        assert_eq!(Language::Jp, Language::default());
    }
}
