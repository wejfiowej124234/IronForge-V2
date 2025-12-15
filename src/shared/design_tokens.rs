//! Design Tokens - IronForge Design System V3
//! 设计系统 Token 定义
//!
//! 设计理念: 苹果风格 + 未来科技 + 智能支付 + 眼镜支付 + 层次感 + 质感

/// 颜色系统
#[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
pub struct Colors;

impl Colors {
    // 背景色系 - 深色科技感
    pub const BG_PRIMARY: &'static str = "#0A0A0F"; // 深空黑（主背景）
    pub const BG_SECONDARY: &'static str = "#12121A"; // 深灰蓝（卡片背景）
    #[allow(dead_code)]
    pub const BG_TERTIARY: &'static str = "#1A1A24"; // 中灰蓝（悬浮卡片）
    #[allow(dead_code)]
    pub const BG_ELEVATED: &'static str = "#24242F"; // 提升层（模态框）

    // 科技蓝紫渐变系统
    pub const TECH_PRIMARY: &'static str = "#6366F1"; // 靛蓝（主色）
    #[allow(dead_code)]
    pub const TECH_SECONDARY: &'static str = "#8B5CF6"; // 紫色（辅助色）
    #[allow(dead_code)]
    pub const TECH_ACCENT: &'static str = "#06B6D4"; // 青色（强调色）
    #[allow(dead_code)]
    pub const TECH_GLOW: &'static str = "#A78BFA"; // 光晕色

    // 智能支付色系
    #[allow(dead_code)]
    pub const PAYMENT_PRIMARY: &'static str = "#10B981"; // 支付绿
    pub const PAYMENT_SUCCESS: &'static str = "#34D399"; // 成功绿
    pub const PAYMENT_WARNING: &'static str = "#F59E0B"; // 警告橙
    pub const PAYMENT_ERROR: &'static str = "#EF4444"; // 错误红
    #[allow(dead_code)]
    pub const PAYMENT_SECONDARY: &'static str = "#10B981"; // 支付绿（备用）

    // 中性色
    pub const TEXT_PRIMARY: &'static str = "#FFFFFF"; // 主文本
    pub const TEXT_SECONDARY: &'static str = "#E5E7EB"; // 次要文本
    pub const TEXT_TERTIARY: &'static str = "#9CA3AF"; // 三级文本
    #[allow(dead_code)]
    pub const TEXT_DISABLED: &'static str = "#6B7280"; // 禁用文本

    // 边框与分割线
    pub const BORDER_PRIMARY: &'static str = "rgba(255, 255, 255, 0.1)";
    #[allow(dead_code)]
    pub const BORDER_SECONDARY: &'static str = "rgba(255, 255, 255, 0.05)";
    #[allow(dead_code)]
    pub const DIVIDER: &'static str = "rgba(255, 255, 255, 0.08)";
}

/// 渐变系统
pub struct Gradients;

impl Gradients {
    // 主渐变 - 科技蓝紫
    pub const PRIMARY: &'static str =
        "linear-gradient(135deg, #6366F1 0%, #8B5CF6 50%, #06B6D4 100%)";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const SECONDARY: &'static str = "linear-gradient(135deg, #8B5CF6 0%, #A78BFA 100%)";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const ACCENT: &'static str = "linear-gradient(135deg, #06B6D4 0%, #6366F1 100%)";

    // 智能支付渐变
    pub const PAYMENT: &'static str = "linear-gradient(135deg, #10B981 0%, #34D399 100%)";
    pub const SUCCESS: &'static str = "linear-gradient(135deg, #34D399 0%, #10B981 100%)";

    // 背景渐变（营销页面用）
    pub const BG_HERO: &'static str =
        "radial-gradient(ellipse at top, rgba(99, 102, 241, 0.15) 0%, transparent 50%)";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const BG_CARD: &'static str =
        "linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(139, 92, 246, 0.1) 100%)";
}

/// 间距系统（8px 基准）
#[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
pub struct Spacing;

impl Spacing {
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XS: &'static str = "4px";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const SM: &'static str = "8px";
    pub const MD: &'static str = "16px";
    pub const LG: &'static str = "24px";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XL: &'static str = "32px";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XXL: &'static str = "48px";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XXXL: &'static str = "64px";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XXXXL: &'static str = "96px"; // 营销页面用
}

/// 圆角系统
pub struct Radius;

impl Radius {
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XS: &'static str = "6px";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const SM: &'static str = "8px";
    pub const MD: &'static str = "12px";
    pub const LG: &'static str = "16px";
    pub const XL: &'static str = "20px"; // 在 modal.rs 中使用
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XXL: &'static str = "24px";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const FULL: &'static str = "9999px";
}

/// 阴影系统
pub struct Shadows;

impl Shadows {
    // 苹果风格多层次阴影
    pub const APPLE: &'static str = "0 2px 8px rgba(0, 0, 0, 0.2), 0 8px 24px rgba(0, 0, 0, 0.3), 0 16px 48px rgba(0, 0, 0, 0.2)";

    // 科技光晕阴影
    pub const TECH: &'static str = "0 4px 16px rgba(99, 102, 241, 0.3), 0 8px 32px rgba(99, 102, 241, 0.2), 0 0 40px rgba(99, 102, 241, 0.1)";

    // 智能支付阴影
    pub const PAYMENT: &'static str =
        "0 4px 20px rgba(16, 185, 129, 0.4), 0 8px 40px rgba(16, 185, 129, 0.2)";

    // 内发光
    pub const INNER_GLOW: &'static str = "inset 0 1px 0 rgba(255, 255, 255, 0.15)";

    // 外发光
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const OUTER_GLOW: &'static str =
        "0 0 20px rgba(99, 102, 241, 0.5), 0 0 40px rgba(99, 102, 241, 0.3)";
}

/// 毛玻璃效果类名
pub struct Glass;

impl Glass {
    // 基础毛玻璃 - 功能页面
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn base() -> String {
        format!(
            "background: rgba(18, 18, 26, 0.6); \
             backdrop-filter: blur(20px) saturate(180%); \
             -webkit-backdrop-filter: blur(20px) saturate(180%); \
             border: 1px solid {}; \
             box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4), {};",
            Colors::BORDER_PRIMARY,
            Shadows::INNER_GLOW
        )
    }

    // 强化毛玻璃 - 营销页面
    pub fn strong() -> String {
        format!(
            "background: rgba(26, 26, 36, 0.7); \
             backdrop-filter: blur(30px) saturate(200%); \
             -webkit-backdrop-filter: blur(30px) saturate(200%); \
             border: 1px solid rgba(255, 255, 255, 0.15); \
             box-shadow: 0 12px 48px rgba(0, 0, 0, 0.5), {}, 0 0 40px rgba(99, 102, 241, 0.1);",
            Shadows::INNER_GLOW
        )
    }

    // 提升层毛玻璃 - 模态框
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn elevated() -> String {
        format!(
            "background: rgba(36, 36, 47, 0.8); \
             backdrop-filter: blur(40px) saturate(220%); \
             -webkit-backdrop-filter: blur(40px) saturate(220%); \
             border: 1px solid rgba(255, 255, 255, 0.2); \
             box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6), {};",
            Shadows::INNER_GLOW
        )
    }
}

/// 字体系统
#[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
pub struct Typography;

#[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
impl Typography {
    // 字体族
    #[allow(dead_code)]
    pub const FONT_FAMILY: &'static str = r#"-apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "Helvetica Neue", "Segoe UI", "Roboto", sans-serif"#;
    #[allow(dead_code)]
    pub const FONT_MONO: &'static str = r#""SF Mono", "Monaco", "Menlo", "Consolas", monospace"#;

    // 字重
    #[allow(dead_code)]
    pub const WEIGHT_LIGHT: &'static str = "300";
    pub const WEIGHT_NORMAL: &'static str = "400";
    pub const WEIGHT_MEDIUM: &'static str = "500";
    pub const WEIGHT_SEMIBOLD: &'static str = "600";
    pub const WEIGHT_BOLD: &'static str = "700";

    // 字号 - 营销页面
    #[allow(dead_code)]
    pub const SIZE_HERO: &'static str = "64px";
    #[allow(dead_code)]
    pub const SIZE_DISPLAY: &'static str = "48px";
    #[allow(dead_code)]
    pub const SIZE_H1_MARKETING: &'static str = "36px";
    #[allow(dead_code)]
    pub const SIZE_H2_MARKETING: &'static str = "28px";
    #[allow(dead_code)]
    pub const SIZE_H3_MARKETING: &'static str = "24px";
    #[allow(dead_code)]
    pub const SIZE_BODY_LG: &'static str = "18px";

    // 字号 - 功能页面
    #[allow(dead_code)]
    pub const SIZE_H1: &'static str = "28px";
    pub const SIZE_H2: &'static str = "24px";
    pub const SIZE_H3: &'static str = "20px";
    pub const SIZE_BODY: &'static str = "16px";
    #[allow(dead_code)]
    pub const SIZE_BODY_SM: &'static str = "14px";
    #[allow(dead_code)]
    pub const SIZE_CAPTION: &'static str = "12px";
}

/// 动画系统
#[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
pub struct Animations;

impl Animations {
    // 过渡时间
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const DURATION_FAST: &'static str = "0.2s";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const DURATION_NORMAL: &'static str = "0.3s";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const DURATION_SLOW: &'static str = "0.5s";

    // 缓动函数
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const EASE_OUT: &'static str = "ease-out";
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const EASE_IN_OUT: &'static str = "cubic-bezier(0.4, 0, 0.2, 1)";

    // 标准过渡
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn transition() -> String {
        format!(
            "transition: all {} {};",
            Self::DURATION_NORMAL,
            Self::EASE_IN_OUT
        )
    }
}

/// 断点系统
#[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
pub struct Breakpoints;

impl Breakpoints {
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const SM: &'static str = "640px"; // 手机
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const MD: &'static str = "768px"; // 平板
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const LG: &'static str = "1024px"; // 小桌面
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XL: &'static str = "1280px"; // 大桌面
    #[allow(dead_code)] // 设计系统常量，用于未来 UI 开发
    pub const XXL: &'static str = "1536px"; // 超大桌面
}

/// 工具函数：生成 Tailwind 类名
pub mod tailwind {

    /// 生成背景色类名
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn bg_primary() -> &'static str {
        "bg-[#0A0A0F]"
    }
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn bg_secondary() -> &'static str {
        "bg-[#12121A]"
    }
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn bg_tertiary() -> &'static str {
        "bg-[#1A1A24]"
    }

    /// 生成文本色类名
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn text_primary() -> &'static str {
        "text-white"
    }
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn text_secondary() -> &'static str {
        "text-[#E5E7EB]"
    }
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn text_tertiary() -> &'static str {
        "text-[#9CA3AF]"
    }

    /// 生成边框类名
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn border_primary() -> &'static str {
        "border-white/10"
    }
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn border_secondary() -> &'static str {
        "border-white/5"
    }

    /// 生成渐变背景类名
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn gradient_primary() -> &'static str {
        "bg-gradient-to-br from-[#6366F1] via-[#8B5CF6] to-[#06B6D4]"
    }
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn gradient_payment() -> &'static str {
        "bg-gradient-to-r from-[#10B981] to-[#34D399]"
    }

    /// 生成毛玻璃类名（功能页面）
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn glass_base() -> &'static str {
        "bg-white/5 backdrop-blur-xl border border-white/10"
    }

    /// 生成毛玻璃类名（营销页面）
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn glass_strong() -> &'static str {
        "bg-white/[0.07] backdrop-blur-2xl border border-white/15"
    }

    /// 生成阴影类名
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn shadow_tech() -> &'static str {
        "shadow-[0_4px_16px_rgba(99,102,241,0.3),0_8px_32px_rgba(99,102,241,0.2),0_0_40px_rgba(99,102,241,0.1)]"
    }
    #[allow(dead_code)] // 设计系统函数，用于未来 UI 开发
    pub fn shadow_payment() -> &'static str {
        "shadow-[0_4px_20px_rgba(16,185,129,0.4),0_8px_40px_rgba(16,185,129,0.2)]"
    }
}
