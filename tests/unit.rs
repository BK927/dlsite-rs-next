//! Unit tests entry point
//!
//! This test file includes all unit tests for:
//! - ID normalization
//! - Endpoint builders
//! - Model mapping

mod common;

use dlsite_rs::client::product::ProductPeople;
use dlsite_rs::client::product_api::interface::{Creator, Creators};
use dlsite_rs::interface::product::WorkType;
use dlsite_rs::interface::query::Language;
use std::str::FromStr;

// =============================================================================
// ID Normalization Tests
// =============================================================================

/// Valid work ID prefixes
const VALID_PREFIXES: &[&str] = &["RJ", "VJ", "BJ", "RG"];

/// Check if a work ID has a valid format
fn is_valid_work_id_format(id: &str) -> bool {
    // Minimum: prefix (2 chars) + at least 5 digits
    if id.len() < 7 {
        return false;
    }
    let prefix = &id[..2].to_uppercase();
    if !VALID_PREFIXES.contains(&prefix.as_str()) {
        return false;
    }
    // Rest should be numeric
    id[2..].chars().all(|c| c.is_ascii_digit())
}

/// Normalize a work ID to uppercase
fn normalize_work_id(id: &str) -> String {
    id.to_uppercase()
}

#[test]
fn test_valid_rj_id_format() {
    assert!(is_valid_work_id_format("RJ403038"));
    assert!(is_valid_work_id_format("RJ01017217"));
    assert!(is_valid_work_id_format("RJ291224"));
}

#[test]
fn test_valid_vj_id_format() {
    assert!(is_valid_work_id_format("VJ01000513"));
    assert!(is_valid_work_id_format("VJ123456"));
}

#[test]
fn test_valid_bj_id_format() {
    assert!(is_valid_work_id_format("BJ123456"));
}

#[test]
fn test_valid_rg_id_format() {
    assert!(is_valid_work_id_format("RG62982"));
    assert!(is_valid_work_id_format("RG24350"));
}

#[test]
fn test_invalid_id_too_short() {
    assert!(!is_valid_work_id_format("RJ1234")); // 6 chars total - too short
    assert!(!is_valid_work_id_format("RJ")); // 2 chars - too short
    assert!(!is_valid_work_id_format("")); // 0 chars - too short
}

#[test]
fn test_invalid_id_wrong_prefix() {
    assert!(!is_valid_work_id_format("XX123456"));
    assert!(!is_valid_work_id_format("12345678"));
}

#[test]
fn test_invalid_id_non_numeric_suffix() {
    assert!(!is_valid_work_id_format("RJABCDE"));
    assert!(!is_valid_work_id_format("RJ123ABC"));
}

#[test]
fn test_normalize_lowercase_to_uppercase() {
    assert_eq!(normalize_work_id("rj403038"), "RJ403038");
    assert_eq!(normalize_work_id("vj01000513"), "VJ01000513");
}

#[test]
fn test_normalize_preserves_uppercase() {
    assert_eq!(normalize_work_id("RJ403038"), "RJ403038");
}

#[test]
fn test_work_type_from_str() {
    // Test known work types
    assert_eq!(WorkType::from_str("SOU").unwrap(), WorkType::SOU);
    assert_eq!(WorkType::from_str("ACN").unwrap(), WorkType::ACN);
    assert_eq!(WorkType::from_str("ADV").unwrap(), WorkType::ADV);
    assert_eq!(WorkType::from_str("RPG").unwrap(), WorkType::RPG);
    assert_eq!(WorkType::from_str("MNG").unwrap(), WorkType::MNG);
    assert_eq!(WorkType::from_str("ICG").unwrap(), WorkType::ICG);
    assert_eq!(WorkType::from_str("MOV").unwrap(), WorkType::MOV);
    assert_eq!(WorkType::from_str("MUS").unwrap(), WorkType::MUS);
    assert_eq!(WorkType::from_str("TOL").unwrap(), WorkType::TOL);
}

#[test]
fn test_work_type_unknown() {
    // Unknown work types should be captured
    let unknown = WorkType::from_str("XYZ").unwrap();
    match unknown {
        WorkType::Unknown(s) => assert_eq!(s, "XYZ"),
        _ => panic!("Expected Unknown variant"),
    }
}

#[test]
fn test_work_type_case_sensitive() {
    // WorkType parsing is case-sensitive based on the enum
    assert!(WorkType::from_str("sou").is_ok()); // lowercase should become Unknown
    match WorkType::from_str("sou") {
        Ok(WorkType::Unknown(s)) => assert_eq!(s, "sou"),
        Ok(_) => panic!("Expected Unknown for lowercase"),
        Err(_) => panic!("Should parse to Unknown"),
    }
}

#[test]
fn test_work_type_is_game() {
    // Game types should return true
    assert!(WorkType::ACN.is_game()); // Action
    assert!(WorkType::QIZ.is_game()); // Quiz
    assert!(WorkType::ADV.is_game()); // Adventure
    assert!(WorkType::RPG.is_game()); // RPG
    assert!(WorkType::TBL.is_game()); // Table
    assert!(WorkType::DNV.is_game()); // Digital Novel
    assert!(WorkType::SLN.is_game()); // Simulation
    assert!(WorkType::TYP.is_game()); // Typing
    assert!(WorkType::STG.is_game()); // Shooting
    assert!(WorkType::PZL.is_game()); // Puzzle
    assert!(WorkType::ETC.is_game()); // Other Games

    // Non-game types should return false
    assert!(!WorkType::MNG.is_game()); // Manga
    assert!(!WorkType::ICG.is_game()); // CG/Illustrations
    assert!(!WorkType::SOU.is_game()); // Voice/ASMR
    assert!(!WorkType::MUS.is_game()); // Music
    assert!(!WorkType::MOV.is_game()); // Video
    assert!(!WorkType::TOL.is_game()); // Tools
    assert!(!WorkType::ET3.is_game()); // Miscellaneous
    assert!(!WorkType::Unknown("XYZ".to_string()).is_game());
}

// =============================================================================
// Endpoint Builder Tests
// =============================================================================

/// Build a product API URL with locale
fn build_product_api_url(workno: &str, locale: &Language) -> String {
    format!(
        "/api/=/product.json?workno={}&locale={}",
        workno,
        locale.to_review_locale()
    )
}

/// Build an AJAX API URL
fn build_ajax_url(product_ids: &[&str]) -> String {
    format!("/product/info/ajax?product_id={}", product_ids.join(","))
}

/// Build a review API URL
fn build_review_url(product_id: &str, limit: u32, page: u32, locale: &Language) -> String {
    format!(
        "/api/review?product_id={}&limit={}&mix_pickup=true&page={}&order=regist_d&locale={}",
        product_id,
        limit,
        page,
        locale.to_review_locale()
    )
}

#[test]
fn test_product_api_url_japanese() {
    let url = build_product_api_url("RJ403038", &Language::Jp);
    assert_eq!(url, "/api/=/product.json?workno=RJ403038&locale=ja_JP");
}

#[test]
fn test_product_api_url_english() {
    let url = build_product_api_url("RJ403038", &Language::En);
    assert_eq!(url, "/api/=/product.json?workno=RJ403038&locale=en_US");
}

#[test]
fn test_product_api_url_korean() {
    let url = build_product_api_url("RJ403038", &Language::Ko);
    assert_eq!(url, "/api/=/product.json?workno=RJ403038&locale=ko_KR");
}

#[test]
fn test_product_api_url_chinese_simplified() {
    let url = build_product_api_url("RJ403038", &Language::ZhCn);
    assert_eq!(url, "/api/=/product.json?workno=RJ403038&locale=zh_CN");
}

#[test]
fn test_product_api_url_chinese_traditional() {
    let url = build_product_api_url("RJ403038", &Language::ZhTw);
    assert_eq!(url, "/api/=/product.json?workno=RJ403038&locale=zh_TW");
}

#[test]
fn test_ajax_url_single_product() {
    let url = build_ajax_url(&["RJ403038"]);
    assert_eq!(url, "/product/info/ajax?product_id=RJ403038");
}

#[test]
fn test_ajax_url_multiple_products() {
    let url = build_ajax_url(&["RJ403038", "RJ01017217", "RJ291224"]);
    assert_eq!(
        url,
        "/product/info/ajax?product_id=RJ403038,RJ01017217,RJ291224"
    );
}

#[test]
fn test_review_url_basic() {
    let url = build_review_url("RJ403038", 6, 1, &Language::Jp);
    assert!(url.contains("product_id=RJ403038"));
    assert!(url.contains("limit=6"));
    assert!(url.contains("page=1"));
    assert!(url.contains("locale=ja_JP"));
}

#[test]
fn test_review_url_pagination() {
    let url = build_review_url("RJ403038", 10, 5, &Language::En);
    assert!(url.contains("limit=10"));
    assert!(url.contains("page=5"));
    assert!(url.contains("locale=en_US"));
}

#[test]
fn test_language_default_is_japanese() {
    assert_eq!(Language::default(), Language::Jp);
}

#[test]
fn test_language_display() {
    assert_eq!(Language::Jp.to_string(), "jp");
    assert_eq!(Language::En.to_string(), "en");
    assert_eq!(Language::Ko.to_string(), "ko");
    assert_eq!(Language::ZhCn.to_string(), "zh_cn");
    assert_eq!(Language::ZhTw.to_string(), "zh_tw");
}

#[test]
fn test_review_locale_format() {
    // Verify locale format matches DLsite API expectations
    assert_eq!(Language::Jp.to_review_locale(), "ja_JP");
    assert_eq!(Language::En.to_review_locale(), "en_US");
    assert_eq!(Language::Ko.to_review_locale(), "ko_KR");
    assert_eq!(Language::ZhCn.to_review_locale(), "zh_CN");
    assert_eq!(Language::ZhTw.to_review_locale(), "zh_TW");
}

// =============================================================================
// Model Mapping Tests
// =============================================================================

#[test]
fn test_creators_to_product_people_basic() {
    let creators = Creators {
        created_by: Some(vec![Creator {
            id: "123".to_string(),
            name: "桃鳥".to_string(),
            classification: "author".to_string(),
            sub_classification: None,
        }]),
        voice_by: Some(vec![
            Creator {
                id: "456".to_string(),
                name: "丹羽うさぎ".to_string(),
                classification: "voice".to_string(),
                sub_classification: None,
            },
            Creator {
                id: "457".to_string(),
                name: "藤堂れんげ".to_string(),
                classification: "voice".to_string(),
                sub_classification: None,
            },
        ]),
        illust_by: None,
        scenario_by: None,
    };

    let people = ProductPeople::from(creators);

    // Verify author mapping
    let author = people.author.expect("author should be set");
    assert_eq!(author.len(), 1);
    assert_eq!(author[0], "桃鳥");

    // Verify voice actor mapping
    let voice_actor = people.voice_actor.expect("voice_actor should be set");
    assert_eq!(voice_actor.len(), 2);
    assert!(voice_actor.contains(&"丹羽うさぎ".to_string()));
    assert!(voice_actor.contains(&"藤堂れんげ".to_string()));
}

#[test]
fn test_creators_to_product_people_empty() {
    let creators = Creators {
        created_by: None,
        voice_by: None,
        illust_by: None,
        scenario_by: None,
    };

    let people = ProductPeople::from(creators);

    assert!(people.author.is_none());
    assert!(people.voice_actor.is_none());
    assert!(people.scenario.is_none());
    assert!(people.illustrator.is_none());
}

#[test]
fn test_creators_to_product_people_all_fields() {
    let creators = Creators {
        created_by: Some(vec![Creator {
            id: "1".to_string(),
            name: "Author Name".to_string(),
            classification: "author".to_string(),
            sub_classification: None,
        }]),
        voice_by: Some(vec![Creator {
            id: "2".to_string(),
            name: "Voice Actor".to_string(),
            classification: "voice".to_string(),
            sub_classification: None,
        }]),
        illust_by: Some(vec![Creator {
            id: "3".to_string(),
            name: "Illustrator".to_string(),
            classification: "illustration".to_string(),
            sub_classification: None,
        }]),
        scenario_by: Some(vec![Creator {
            id: "4".to_string(),
            name: "Scenario Writer".to_string(),
            classification: "scenario".to_string(),
            sub_classification: None,
        }]),
    };

    let people = ProductPeople::from(creators);

    assert_eq!(people.author.as_ref().unwrap()[0], "Author Name");
    assert_eq!(people.voice_actor.as_ref().unwrap()[0], "Voice Actor");
    assert_eq!(people.illustrator.as_ref().unwrap()[0], "Illustrator");
    assert_eq!(people.scenario.as_ref().unwrap()[0], "Scenario Writer");
}

#[test]
fn test_product_people_default() {
    let people = ProductPeople::default();

    assert!(people.author.is_none());
    assert!(people.scenario.is_none());
    assert!(people.illustrator.is_none());
    assert!(people.voice_actor.is_none());
}

#[test]
fn test_creator_multiple_same_role() {
    // Test that multiple creators with the same role are all mapped
    let creators = Creators {
        created_by: Some(vec![
            Creator {
                id: "1".to_string(),
                name: "First Author".to_string(),
                classification: "author".to_string(),
                sub_classification: None,
            },
            Creator {
                id: "2".to_string(),
                name: "Second Author".to_string(),
                classification: "author".to_string(),
                sub_classification: None,
            },
            Creator {
                id: "3".to_string(),
                name: "Third Author".to_string(),
                classification: "author".to_string(),
                sub_classification: None,
            },
        ]),
        voice_by: None,
        illust_by: None,
        scenario_by: None,
    };

    let people = ProductPeople::from(creators);

    let authors = people.author.unwrap();
    assert_eq!(authors.len(), 3);
    assert!(authors.contains(&"First Author".to_string()));
    assert!(authors.contains(&"Second Author".to_string()));
    assert!(authors.contains(&"Third Author".to_string()));
}

// =============================================================================
// Circle Name Bucket Tests (for circle_name -> maker_id resolution)
// =============================================================================

/// Determine the name bucket for circle name resolution.
/// This mirrors the implementation in src/client/circle/mod.rs.
fn get_name_bucket(name: &str) -> &'static str {
    let first_char = name.chars().next().unwrap_or('a');

    match first_char {
        'あ'..='お' => "あ",
        'か'..='ご' => "か",
        'さ'..='ぞ' => "さ",
        'た'..='ど' => "た",
        'な'..='の' => "な",
        'は'..='ぽ' => "は",
        'ま'..='も' => "ま",
        'や' | 'ゆ' | 'よ' => "や",
        'ら'..='ろ' => "ら",
        'わ'..='ん' => "わ",
        'ア'..='オ' => "ア",
        'カ'..='ゴ' => "カ",
        'サ'..='ゾ' => "サ",
        'タ'..='ド' => "タ",
        'ナ'..='ノ' => "ナ",
        'ハ'..='ポ' => "ハ",
        'マ'..='モ' => "マ",
        'ヤ' | 'ユ' | 'ヨ' => "ヤ",
        'ラ'..='ロ' => "ラ",
        'ワ'..='ン' => "ワ",
        'a'..='z' | 'A'..='Z' => "a",
        '0'..='9' => "0",
        _ => "a",
    }
}

#[test]
fn test_name_bucket_hiragana() {
    // Hiragana buckets
    assert_eq!(get_name_bucket("あいうえお"), "あ");
    assert_eq!(get_name_bucket("かきくけこ"), "か");
    assert_eq!(get_name_bucket("がぎぐげご"), "か"); // voiced
    assert_eq!(get_name_bucket("さしすせそ"), "さ");
    assert_eq!(get_name_bucket("ざじずぜぞ"), "さ"); // voiced
    assert_eq!(get_name_bucket("たちつてと"), "た");
    assert_eq!(get_name_bucket("だぢづでど"), "た"); // voiced
    assert_eq!(get_name_bucket("なにぬねの"), "な");
    assert_eq!(get_name_bucket("はひふへほ"), "は");
    assert_eq!(get_name_bucket("ばびぶべぼ"), "は"); // voiced
    assert_eq!(get_name_bucket("ぱぴぷぺぽ"), "は"); // semi-voiced
    assert_eq!(get_name_bucket("まみむめも"), "ま");
    assert_eq!(get_name_bucket("やゆよ"), "や");
    assert_eq!(get_name_bucket("らりるれろ"), "ら");
    assert_eq!(get_name_bucket("わをん"), "わ");
}

#[test]
fn test_name_bucket_katakana() {
    // Katakana buckets
    assert_eq!(get_name_bucket("アイウエオ"), "ア");
    assert_eq!(get_name_bucket("カキクケコ"), "カ");
    assert_eq!(get_name_bucket("ガギグゲゴ"), "カ"); // voiced
    assert_eq!(get_name_bucket("サシスセソ"), "サ");
    assert_eq!(get_name_bucket("タチツテト"), "タ");
    assert_eq!(get_name_bucket("ナニヌネノ"), "ナ");
    assert_eq!(get_name_bucket("ハヒフヘホ"), "ハ");
    assert_eq!(get_name_bucket("マミムメモ"), "マ");
    assert_eq!(get_name_bucket("ヤユヨ"), "ヤ");
    assert_eq!(get_name_bucket("ラリルレロ"), "ラ");
    assert_eq!(get_name_bucket("ワヲン"), "ワ");
}

#[test]
fn test_name_bucket_alphabetic() {
    // All alphabetic names go to "a" bucket
    assert_eq!(get_name_bucket("Apple"), "a");
    assert_eq!(get_name_bucket("Banana"), "a");
    assert_eq!(get_name_bucket("Zebra"), "a");
    assert_eq!(get_name_bucket("circle"), "a");
}

#[test]
fn test_name_bucket_numeric() {
    // All numeric names go to "0" bucket
    assert_eq!(get_name_bucket("123Circle"), "0");
    assert_eq!(get_name_bucket("42Studio"), "0");
}

#[test]
fn test_name_bucket_empty_or_special() {
    // Empty or special characters default to "a"
    assert_eq!(get_name_bucket(""), "a");
    assert_eq!(get_name_bucket("☆Star"), "a");
    assert_eq!(get_name_bucket("★Planet"), "a");
}

// =============================================================================
// ProductApiContent Custom Genres Tests
// =============================================================================

use dlsite_rs::client::product_api::interface::{CustomGenre, GenreApi};

#[test]
fn test_custom_genre_structure() {
    // Test that CustomGenre can be deserialized with expected fields
    let json = r#"{
        "genre_key": "custom_001",
        "lang": "ja",
        "name": "カスタムジャンル",
        "layout": {},
        "status": "active",
        "is_active": 1,
        "start_date": "2024-01-01",
        "end_date": "2024-12-31"
    }"#;

    let genre: CustomGenre = serde_json::from_str(json).unwrap();
    assert_eq!(genre.genre_key, "custom_001");
    assert_eq!(genre.lang, "ja");
    assert_eq!(genre.name, "カスタムジャンル");
    assert_eq!(genre.status, "active");
    assert_eq!(genre.is_active, 1);
}

#[test]
fn test_genre_api_structure() {
    // Test that GenreApi can be deserialized with expected fields
    let json = r#"{
        "name": "ロールプレイング",
        "id": 401,
        "search_val": "RPG",
        "name_base": "Role Playing"
    }"#;

    let genre: GenreApi = serde_json::from_str(json).unwrap();
    assert_eq!(genre.name, "ロールプレイング");
    assert_eq!(genre.id, 401);
    assert_eq!(genre.search_val, "RPG");
    assert_eq!(genre.name_base, "Role Playing");
}

// =============================================================================
// Review Sort Order Tests
// =============================================================================

use dlsite_rs::client::product::review::ReviewSortOrder;

#[test]
fn test_review_sort_order_exists() {
    // Verify ReviewSortOrder enum exists with expected variants
    let _new = ReviewSortOrder::New;
    let _top = ReviewSortOrder::Top;
}

// =============================================================================
// Circle Profile Structure Tests
// =============================================================================

#[cfg(feature = "search-html")]
use dlsite_rs::client::circle::CircleProfile;

#[cfg(feature = "search-html")]
#[test]
fn test_circle_profile_structure() {
    // Test that CircleProfile can be created with expected fields
    let profile = CircleProfile {
        id: "RG24350".to_string(),
        name: "桃色CODE".to_string(),
        description: Some("A circle description".to_string()),
        banner_url: Some("https://example.com/banner.jpg".to_string()),
    };

    assert_eq!(profile.id, "RG24350");
    assert_eq!(profile.name, "桃色CODE");
    assert!(profile.description.is_some());
    assert!(profile.banner_url.is_some());
}

#[cfg(feature = "search-html")]
#[test]
fn test_circle_profile_minimal() {
    // Test CircleProfile with minimal fields
    let profile = CircleProfile {
        id: "RG00000".to_string(),
        name: "Unknown Circle".to_string(),
        description: None,
        banner_url: None,
    };

    assert_eq!(profile.id, "RG00000");
    assert_eq!(profile.name, "Unknown Circle");
    assert!(profile.description.is_none());
    assert!(profile.banner_url.is_none());
}
