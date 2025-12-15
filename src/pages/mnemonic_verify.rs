//! Mnemonic Verify Page - 助记词验证页面
//! 随机选择3个位置进行验证，提升用户体验

use crate::components::atoms::button::{Button, ButtonSize, ButtonVariant};
use crate::components::atoms::card::Card;
use crate::components::molecules::ErrorMessage;
use crate::router::Route;
use crate::shared::design_tokens::Colors;
use crate::shared::state::AppState;
use dioxus::prelude::*;
use js_sys::Math;

/// Word Button Component - 单词按钮组件
#[component]
fn WordButton(
    word: String,
    is_selected: bool,
    is_correct: bool,
    on_click: EventHandler<()>,
) -> Element {
    rsx! {
        Button {
            variant: if is_selected {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Secondary
            },
            size: ButtonSize::Medium,
            disabled: is_selected,
            onclick: move |_| {
                on_click.call(());
            },
            {word.clone()}
        }
    }
}

/// Mnemonic Verify Page - 助记词验证页面
///
/// 随机选择3个位置进行验证，提升用户体验
#[component]
pub fn MnemonicVerify(
    /// 正确的助记词短语
    phrase: String,
) -> Element {
    let navigator = use_navigator();
    let app_state = use_context::<AppState>();
    let wallet_controller = crate::features::wallet::hooks::use_wallet();

    // 将助记词分割成单词数组
    let correct_words: Vec<String> = phrase.split_whitespace().map(|s| s.to_string()).collect();

    let correct_words_len = correct_words.len();
    // 创建多个副本，因为不同的 use_memo 闭包需要不同的副本
    let correct_words_for_current = correct_words.clone();
    let correct_words_for_check = correct_words.clone();

    // 随机选择3个位置进行验证（提升用户体验）
    // 注意：固定为3个位置，而不是全部12个
    let verify_positions: Memo<Vec<usize>> = use_memo(move || {
        let mut positions: Vec<usize> = (0..correct_words_len).collect();
        // Fisher-Yates 洗牌算法
        for i in (1..positions.len()).rev() {
            let j = (Math::random() * (i + 1) as f64) as usize;
            positions.swap(i, j);
        }
        // 只取前3个位置进行验证（提升用户体验）
        let selected = positions.into_iter().take(3).collect::<Vec<usize>>();
        // 确保只返回3个位置
        selected
    });

    // 当前验证的位置索引（在verify_positions中的索引）
    let current_verify_index = use_signal(|| 0);

    // 当前需要验证的单词位置（在助记词中的实际位置）
    let current_word_position = use_memo(move || {
        let positions = verify_positions.read();
        let idx = *current_verify_index.read();
        if idx < positions.len() {
            positions[idx]
        } else {
            0
        }
    });

    // 当前需要验证的正确单词
    let current_correct_word = use_memo(move || {
        let pos = *current_word_position.read();
        let words = correct_words_for_current.clone();
        if pos < words.len() {
            words[pos].clone()
        } else {
            String::new()
        }
    });

    // 使用use_memo生成随机打乱的单词列表（依赖current_verify_index，每次验证时重新生成）
    let shuffled = use_memo(move || {
        // 依赖current_verify_index来触发重新生成
        let _ = *current_verify_index.read();
        let current_word = current_correct_word.read().clone();
        let mut all_words = vec![current_word];
        // 添加一些干扰项
        let distractors = vec![
            "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
            "absurd", "abuse", "access", "accident", "account", "accuse", "achieve", "acid",
            "across", "act", "action", "actor", "actual", "adapt", "add", "address", "adjust",
            "admit", "adult", "advance", "advice", "afford", "afraid", "again",
        ];
        for distractor in distractors.iter().take(8) {
            if !all_words.contains(&distractor.to_string()) {
                all_words.push(distractor.to_string());
            }
        }

        // 打乱顺序
        let mut shuffled = all_words;
        for i in (1..shuffled.len()).rev() {
            let j = (Math::random() * (i + 1) as f64) as usize;
            shuffled.swap(i, j);
        }
        shuffled
    });

    // 存储已选择的单词（按验证顺序）
    let selected_words = use_signal(|| Vec::<(usize, String)>::new()); // (位置, 单词)
    let error_message = use_signal(|| Option::<String>::None);

    let verify_count = verify_positions.read().len();
    let is_complete = selected_words.read().len() == verify_count;
    let is_correct = is_complete && {
        let selected = selected_words.read();
        let positions = verify_positions.read();
        let correct_words_check = correct_words_for_check.clone();
        selected
            .iter()
            .zip(positions.iter())
            .all(|((pos, word), expected_pos)| {
                *pos == *expected_pos && *word == correct_words_check[*expected_pos]
            })
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center p-4",
            style: format!("background: {};", Colors::BG_PRIMARY),

            Card {
                variant: crate::components::atoms::card::CardVariant::Base,
                padding: Some("32px".to_string()),
                children: rsx! {
                    // 标题
                    div {
                        class: "text-center mb-6",
                        h1 {
                            class: "text-2xl font-bold mb-2",
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "验证助记词"
                        }
                        p {
                            class: "text-sm",
                            style: format!("color: {};", Colors::TEXT_SECONDARY),
                            "请选择指定位置的单词，验证您已正确备份（只需验证3个随机位置）"
                        }
                    }

                    // 进度指示
                    div {
                        class: "mb-6",
                        div {
                            class: "flex justify-between items-center mb-2",
                            span {
                                class: "text-sm",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "进度: {selected_words.read().len()}/{verify_count}"
                            }
                            if !is_complete {
                                span {
                                    class: "text-sm font-semibold",
                                    style: format!("color: {};", Colors::TECH_PRIMARY),
                                    "请选择第 {*current_word_position.read() + 1} 个位置的单词"
                                }
                            }
                        }
                        div {
                            class: "w-full h-2 rounded-full overflow-hidden",
                            style: format!("background: {};", Colors::BG_SECONDARY),
                            div {
                                class: "h-full transition-all duration-300",
                                style: format!(
                                    "width: {}%; background: {};",
                                    (selected_words.read().len() as f64 / verify_count as f64 * 100.0),
                                    Colors::TECH_PRIMARY
                                ),
                            }
                        }
                    }

                    // 已选择的单词显示
                    if !selected_words.read().is_empty() {
                        div {
                            class: "mb-6 p-4 rounded-lg",
                            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
                            div {
                                class: "text-sm mb-2",
                                style: format!("color: {};", Colors::TEXT_SECONDARY),
                                "已选择的单词："
                            }
                            div {
                                class: "flex flex-wrap gap-2",
                                for (pos, word) in selected_words.read().iter() {
                                    div {
                                        class: "px-3 py-1 rounded-full",
                                        style: format!("background: {}; color: {};", Colors::TECH_PRIMARY, Colors::TEXT_PRIMARY),
                                        span {
                                            class: "text-xs mr-1",
                                            "位置 {pos + 1}:"
                                        }
                                        {word.clone()}
                                    }
                                }
                            }
                        }
                    }

                    // 错误提示
                    ErrorMessage {
                        message: error_message.read().clone()
                    }

                    // 成功提示
                    if is_complete && is_correct {
                        div {
                            class: "mb-4 p-4 rounded-lg",
                            style: format!("background: rgba(34, 197, 94, 0.1); border: 1px solid #22c55e; color: #22c55e;"),
                            "✅ 验证成功！您已正确备份助记词"
                        }
                    }

                    // 单词选择网格
                    if !is_complete {
                        div {
                            class: "mb-6",
                            div {
                                class: "grid grid-cols-3 gap-2",
                                for word in shuffled.read().iter().cloned() {
                                    WordButton {
                                        word: word.clone(),
                                        is_selected: {
                                            let selected = selected_words.read();
                                            let current_pos = *current_word_position.read();
                                            selected.iter().any(|(pos, w)| *pos == current_pos && *w == word)
                                        },
                                        is_correct: word == *current_correct_word.read(),
                                        on_click: {
                                            let word_clone = word.clone();
                                            let correct_word = current_correct_word.read().clone();
                                            let current_pos = *current_word_position.read();
                                            let mut selected_words = selected_words;
                                            let mut current_verify_index = current_verify_index;
                                            let mut error_message = error_message;
                                            let verify_count = verify_count;
                                            EventHandler::new(move |_| {
                                                let mut selected = selected_words.write();
                                                let current_idx = *current_verify_index.read();

                                                if current_idx < verify_count {
                                                    if word_clone == correct_word {
                                                        // 验证正确
                                                        selected.push((current_pos, word_clone.clone()));
                                                        current_verify_index.set(current_idx + 1);
                                                        error_message.set(None);
                                                    } else {
                                                        // 验证错误，重新开始
                                                        error_message.set(Some("单词错误，请重新开始验证".to_string()));
                                                        selected.clear();
                                                        current_verify_index.set(0);
                                                    }
                                                }
                                            })
                                        },
                                    }
                                }
                            }
                        }
                    }

                    // 操作按钮
                    div {
                        class: "flex gap-4",
                        if is_complete && is_correct {
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Large,
                                onclick: {
                                    let navigator = navigator;
                                    let app_state = app_state;
                                    let wallet_controller = wallet_controller;
                                    move |_| {
                                        // 验证通过后，完成钱包创建
                                        spawn(async move {
                                            match wallet_controller.finalize_wallet_creation().await {
                                                Ok(_) => {
                                                    // 创建成功，导航到成功页面
                                                    navigator.push(Route::WalletCreated {});
                                                }
                                                Err(e) => {
                                                    // 创建失败，显示错误
                                                    AppState::show_error(
                                                        app_state.toasts,
                                                        format!("钱包创建失败: {}", e)
                                                    );
                                                }
                                            }
                                        });
                                    }
                                },
                                "完成"
                            }
                        } else if is_complete && !is_correct {
                            Button {
                                variant: ButtonVariant::Primary,
                                size: ButtonSize::Large,
                                onclick: {
                                    let mut selected_words = selected_words;
                                    let mut current_verify_index = current_verify_index;
                                    let mut error_message = error_message;
                                    move |_| {
                                        // 重新开始
                                        selected_words.write().clear();
                                        current_verify_index.set(0);
                                        error_message.set(None);
                                    }
                                },
                                "重新验证"
                            }
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            size: ButtonSize::Large,
                            onclick: move |_| {
                                navigator.go_back();
                            },
                            "返回"
                        }
                    }
                }
            }
        }
    }
}
