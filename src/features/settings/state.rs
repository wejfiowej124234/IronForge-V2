use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Language {
    English,            // US - English
    Spanish,            // ES - Español
    French,             // FR - Français
    ChineseSimple,      // CN - 简体中文
    ChineseTraditional, // TW - 繁體中文
    Japanese,           // JP - 日本語
    Korean,             // KR - 한국어
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Currency {
    USD,
    CNY,
    EUR,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    pub theme: Theme,
    pub language: Language,
    pub currency: Currency,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            language: Language::ChineseSimple, // 默认简体中文
            currency: Currency::CNY,
        }
    }
}

impl Language {
    /// 获取语言代码
    /// 为未来扩展准备的方法
    #[allow(dead_code)] // 为未来扩展准备
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "US",
            Language::Spanish => "ES",
            Language::French => "FR",
            Language::ChineseSimple => "CN",
            Language::ChineseTraditional => "TW",
            Language::Japanese => "JP",
            Language::Korean => "KR",
        }
    }

    /// 获取显示名称
    /// 为未来扩展准备的方法
    #[allow(dead_code)] // 为未来扩展准备
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::ChineseSimple => "简体中文",
            Language::ChineseTraditional => "繁體中文",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
        }
    }

    /// 获取所有语言
    /// 为未来扩展准备的方法
    #[allow(dead_code)] // 为未来扩展准备
    pub fn all() -> Vec<Self> {
        vec![
            Language::English,
            Language::Spanish,
            Language::French,
            Language::ChineseSimple,
            Language::ChineseTraditional,
            Language::Japanese,
            Language::Korean,
        ]
    }
}

impl UserPreferences {
    pub fn load() -> Self {
        LocalStorage::get("user_preferences").unwrap_or_default()
    }

    /// 保存用户偏好设置
    /// 为未来扩展准备的方法
    #[allow(dead_code)] // 为未来扩展准备
    pub fn save(&self) {
        let _ = LocalStorage::set("user_preferences", self);
    }
}
