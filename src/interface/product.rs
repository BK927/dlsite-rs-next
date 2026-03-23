//! Product-related types for DLsite works.
//!
//! This module contains enums representing work types, age categories,
//! file types, and other product classification data.

use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_with::DeserializeFromStr;
use strum::{Display, EnumString};

/// Work type category (group) for broad classification.
///
/// Each category encompasses multiple individual [`WorkType`] values.
/// Used for filtering search results by broad product type.
#[derive(Debug, Display, Clone, PartialEq, EnumString, DeserializeFromStr)]
#[strum(serialize_all = "snake_case")]
pub enum WorkTypeCategory {
    /// Games (ゲーム).
    Game,
    /// Manga/comics (マンガ).
    Comic,
    /// CG and illustrations (CG・イラスト).
    Illust,
    /// Novels (ノベル).
    Novel,
    /// Video/animation works (動画作品/アニメ).
    Movie,
    /// Voice and ASMR content (ボイス・ASMR).
    Audio,
    /// Music (音楽).
    Music,
    /// Tools and accessories (ツール/アクセサリ).
    Tool,
    /// Other/miscellaneous (その他).
    Etc,

    /// Unknown or unrecognized work type category.
    #[strum(default)]
    Unknown(String),
}

/// Individual work type (作品形式) for fine-grained classification.
#[derive(Display, EnumString, Debug, PartialEq, Clone, DeserializeFromStr, serde::Serialize)]
pub enum WorkType {
    /// Game category
    ///
    /// JP: アクション
    /// EN: Action
    ACN,
    /// JP: クイズ
    /// EN: Quiz
    QIZ,
    /// JP: アドベンチャー
    /// EN: Adventure
    ADV,
    /// JP: ロールプレイング
    /// EN: Role-playing
    RPG,
    /// JP: テーブル
    /// EN: Table
    TBL,
    /// JP: デジタルノベル
    /// EN: Digital Novel
    DNV,
    /// JP: シミュレーション
    /// EN: Simulation
    SLN,
    /// JP: タイピング
    /// EN: Typing
    TYP,
    /// JP: シューティング
    /// EN: Shooting
    STG,
    /// JP: パズル
    /// EN: Puzzle
    PZL,
    /// JP: その他ゲーム
    /// EN: Miscellaneous Games
    ETC,

    /// Mange category
    ///
    /// JP: マンガ
    /// EN: Manga
    MNG,
    /// JP: 劇画
    /// EN: Gekiga
    SCM,
    /// JP: WEBTOON
    /// EN: Webtoon
    WBT,

    /// CG + Illustrations category
    ///
    /// JP: CG・イラスト
    /// EN: CG + Illustrations
    ICG,

    // Novel category
    //
    /// JP: ノベル
    /// EN: Novel
    NRE,
    /// JP: 官能小説
    /// EN: Erotic Novel
    KSV,

    /// Video category
    ///
    /// JP: 動画
    /// EN: Video
    MOV,

    /// Voice / ASMR category
    ///
    /// JP: ボイス・ASMR
    /// EN: Voice / ASMR
    SOU,

    /// Music category
    ///
    /// JP: 音楽
    /// EN: Music
    MUS,

    /// Tools / Accessories category
    ///
    /// JP: ツール/アクセサリ
    /// EN: Tools / Accessories
    TOL,
    /// JP: 画像素材
    /// EN: Illustration Materials
    IMT,
    /// JP: 音素材
    /// EN: Music Materials
    AMT,

    /// Miscellaneous category
    ///
    /// JP: その他
    /// EN: Miscellaneous
    ET3,
    /// JP: ボイスコミック
    /// EN: Voiced Comics
    VCM,

    #[strum(default)]
    Unknown(String),
}

impl WorkType {
    /// Check if this work type is a game.
    ///
    /// Game work types include: Action (ACN), Quiz (QIZ), Adventure (ADV),
    /// RPG, Table (TBL), Digital Novel (DNV), Simulation (SLN),
    /// Typing (TYP), Shooting (STG), Puzzle (PZL), and Other Games (ETC).
    ///
    /// # Example
    /// ```
    /// use dlsite_rs::interface::product::WorkType;
    ///
    /// assert!(WorkType::RPG.is_game());
    /// assert!(WorkType::ADV.is_game());
    /// assert!(!WorkType::SOU.is_game()); // Voice/ASMR is not a game
    /// ```
    pub fn is_game(&self) -> bool {
        matches!(
            self,
            WorkType::ACN
                | WorkType::QIZ
                | WorkType::ADV
                | WorkType::RPG
                | WorkType::TBL
                | WorkType::DNV
                | WorkType::SLN
                | WorkType::TYP
                | WorkType::STG
                | WorkType::PZL
                | WorkType::ETC
        )
    }
}

/// Age category/rating for DLsite products.
#[derive(Display, Debug, Clone, PartialEq, Deserialize_repr, Serialize_repr)]
#[repr(u16)]
#[strum(serialize_all = "snake_case")]
pub enum AgeCategory {
    /// All ages (全年齢).
    #[serde(with = "i8")]
    General = 1,
    /// R-15 rated content.
    #[serde(with = "i8")]
    R15 = 2,
    /// Adult only (18+).
    #[serde(with = "i8")]
    Adult = 3,
}

/// Work category (parent category) for market segmentation.
#[derive(Display, EnumString, PartialEq, DeserializeFromStr, Debug, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum WorkCategory {
    /// Doujin/fan works (同人).
    Doujin,
    /// Commercial books/manga (成年コミック).
    Books,
    /// PC software: Adult games (美少女ゲーム) or general software (PCソフト).
    Pc,
    /// Mobile/smartphone games (スマホゲーム).
    App,

    /// Unknown or unrecognized work category.
    #[strum(default)]
    Unknown(String),
}

/// File type/extension for downloadable content.
#[derive(Display, EnumString, PartialEq, Debug, Clone, DeserializeFromStr)]
pub enum FileType {
    EXE,
    HTI,
    HTE,
    HMO,
    IJP,
    IGF,
    IME,
    IBP,
    PNG,
    AVI,
    MVF,
    MPG,
    MWM,
    MP4,
    AAC,
    WAV,
    MP3,
    ADO,
    WMA,
    FLC,
    OGG,
    PDF,
    APK,
    ET1,

    #[strum(default)]
    Unknown(String),
}
