//! Performance Monitor Component - 性能监控组件
//! 显示页面性能指标和资源使用情况
#![allow(dead_code)]

use crate::shared::design_tokens::Colors;
use dioxus::prelude::*;
use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub page_load_time: Option<f64>,    // 页面加载时间（毫秒）
    pub render_time: Option<f64>,       // 渲染时间（毫秒）
    pub api_response_time: Option<f64>, // API响应时间（毫秒）
    pub memory_usage: Option<f64>,      // 内存使用（MB）
    pub cache_hit_rate: Option<f64>,    // 缓存命中率（%）
}

/// 性能监控组件属性
#[derive(Props, PartialEq, Clone)]
pub struct PerformanceMonitorProps {
    /// 是否显示详细信息
    #[props(default = false)]
    pub show_details: bool,
    /// 是否自动刷新
    #[props(default = true)]
    pub auto_refresh: bool,
}

/// 性能监控组件
#[component]
pub fn PerformanceMonitor(props: PerformanceMonitorProps) -> Element {
    #[allow(unused_mut)]
    let metrics = use_signal(|| PerformanceMetrics {
        page_load_time: None,
        render_time: None,
        api_response_time: None,
        memory_usage: None,
        cache_hit_rate: None,
    });

    // 自动刷新性能指标
    use_effect({
        let metrics_sig = metrics;
        let auto_refresh = props.auto_refresh;

        move || {
            if !auto_refresh {
                return;
            }

            let mut metrics_clone = metrics_sig;
            spawn(async move {
                loop {
                    // 获取性能指标
                    if let Some(window) = web_sys::window() {
                        // 页面加载时间 - 使用js_sys::Reflect访问performance API
                        let page_load = get_page_load_time(&window);

                        // 内存使用（如果支持）- Chrome特有的API
                        let memory = get_memory_usage(&window);

                        metrics_clone.set(PerformanceMetrics {
                            page_load_time: page_load,
                            render_time: None,
                            api_response_time: None,
                            memory_usage: memory,
                            cache_hit_rate: None,
                        });
                    }

                    // 每5秒刷新一次
                    gloo_timers::future::TimeoutFuture::new(5000).await;
                }
            });
        }
    });

    if !props.show_details {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "p-4 rounded-lg",
            style: format!("background: {}; border: 1px solid {};", Colors::BG_SECONDARY, Colors::BORDER_PRIMARY),
            h3 {
                class: "text-sm font-semibold mb-3",
                style: format!("color: {};", Colors::TEXT_PRIMARY),
                "⚡ 性能指标"
            }
            div {
                class: "space-y-2 text-xs",
                if let Some(load_time) = metrics.read().page_load_time {
                    div {
                        class: "flex justify-between",
                        span { style: format!("color: {};", Colors::TEXT_SECONDARY), "页面加载" }
                        span {
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "{load_time:.0}ms"
                        }
                    }
                }
                if let Some(memory) = metrics.read().memory_usage {
                    div {
                        class: "flex justify-between",
                        span { style: format!("color: {};", Colors::TEXT_SECONDARY), "内存使用" }
                        span {
                            style: format!("color: {};", Colors::TEXT_PRIMARY),
                            "{memory:.1}MB"
                        }
                    }
                }
            }
        }
    }
}

/// 获取页面加载时间（毫秒）
/// 使用js_sys::Reflect访问performance API
fn get_page_load_time(window: &web_sys::Window) -> Option<f64> {
    // 通过Reflect获取window.performance
    let performance_val = Reflect::get(window, &JsValue::from_str("performance")).ok()?;

    // 检查performance是否存在
    if performance_val.is_undefined() || performance_val.is_null() {
        return None;
    }

    // 获取performance.timing（旧API）或performance.getEntriesByType（新API）
    // 优先使用Navigation Timing API
    if let Ok(timing_val) = Reflect::get(&performance_val, &JsValue::from_str("timing")) {
        if !timing_val.is_undefined() && !timing_val.is_null() {
            // 使用旧API：loadEventEnd - navigationStart
            if let (Ok(nav_start), Ok(load_end)) = (
                Reflect::get(&timing_val, &JsValue::from_str("navigationStart")),
                Reflect::get(&timing_val, &JsValue::from_str("loadEventEnd")),
            ) {
                if let (Some(nav_start_num), Some(load_end_num)) =
                    (nav_start.as_f64(), load_end.as_f64())
                {
                    if load_end_num > 0.0 && nav_start_num > 0.0 {
                        return Some(load_end_num - nav_start_num);
                    }
                }
            }
        }
    }

    // 尝试使用Performance Navigation Timing API（新API）
    if let Ok(get_entries_fn) =
        Reflect::get(&performance_val, &JsValue::from_str("getEntriesByType"))
    {
        if get_entries_fn.is_function() {
            let entries_result = js_sys::Function::from(get_entries_fn)
                .call1(&performance_val, &JsValue::from_str("navigation"));
            if let Ok(entries_js) = entries_result {
                // 将 JsValue 转换为 Array
                if let Some(entries_array) = entries_js.dyn_ref::<js_sys::Array>() {
                    if entries_array.length() > 0 {
                        if let Some(first_entry) = entries_array.get(0).dyn_ref::<Object>() {
                            if let (Ok(duration), Ok(_start_time)) = (
                                Reflect::get(first_entry, &JsValue::from_str("duration")),
                                Reflect::get(first_entry, &JsValue::from_str("startTime")),
                            ) {
                                if let Some(duration_num) = duration.as_f64() {
                                    if duration_num > 0.0 {
                                        return Some(duration_num);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// 获取内存使用情况（MB）
/// 注意：这是Chrome特有的API，其他浏览器可能不支持
fn get_memory_usage(window: &web_sys::Window) -> Option<f64> {
    // 通过Reflect获取window.performance
    let performance_val = Reflect::get(window, &JsValue::from_str("performance")).ok()?;

    if performance_val.is_undefined() || performance_val.is_null() {
        return None;
    }

    // 尝试获取performance.memory（Chrome特有）
    if let Ok(memory_val) = Reflect::get(&performance_val, &JsValue::from_str("memory")) {
        if !memory_val.is_undefined() && !memory_val.is_null() {
            // 获取usedJSHeapSize（已使用的堆内存，字节）
            if let Ok(used_heap) = Reflect::get(&memory_val, &JsValue::from_str("usedJSHeapSize")) {
                if let Some(used_heap_num) = used_heap.as_f64() {
                    // 转换为MB
                    return Some(used_heap_num / (1024.0 * 1024.0));
                }
            }
        }
    }

    None
}
