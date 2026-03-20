use strum::Display;

#[derive(Display, Default, Clone, Debug, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum Language {
    #[default]
    Jp,
    En,
    Ko,
    ZhCn,
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

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum SexCategory {
    Male,
    Female,
}

/// Flag to represent sales status
#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum AnaFlg {
    Off,
    On,
    Reserve,
    All,
}

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum Order {
    Trend,
    /// 新しい
    Release,
    /// 古い
    ReleaseD,
    /// DL数が多い
    DlD,
    /// DL数が少ない
    Dl,
    /// 安い
    Price,
    /// 高い
    PriceD,
    /// 評価が高い
    RateD,
    /// レビューが多い
    ReviewD,
}

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum OptionAndOr {
    And,
    Or,
}

#[derive(Display)]
pub enum ReleaseTerm {
    None,
    Week,
    Month,
    Year,
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
