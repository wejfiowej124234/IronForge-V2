//! 国际化 (i18n) 模块
//! 提供多语言支持

pub mod translations;

use crate::shared::state::AppState;
use dioxus::prelude::*;

/// 获取翻译文本的 Hook
pub fn use_translation() -> impl Fn(&str) -> String {
    let app_state = use_context::<AppState>();
    
    move |key: &str| -> String {
        let lang = app_state.language.read();
        translations::get_text(key, &lang)
    }
}

/// 翻译宏 - 简化使用
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::translations::get_text($key, &app_state.language.read())
    };
}
