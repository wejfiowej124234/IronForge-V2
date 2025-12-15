# 组件使用文档

> **最后更新**: 2025-01-27  
> **组件架构**: Atomic Design (原子设计)

---

## 📦 组件架构

```
components/
├── atoms/          # 原子组件（最小UI单元）
│   ├── button.rs   # 按钮组件
│   ├── input.rs    # 输入框组件
│   ├── card.rs     # 卡片组件
│   ├── icon.rs     # 图标组件
│   └── modal.rs    # 模态框组件
├── molecules/      # 分子组件（组合组件）
│   ├── gas_fee_card.rs    # Gas费显示卡片
│   ├── chain_selector.rs  # 链选择器
│   ├── qr_code_display.rs # 二维码显示组件
│   └── error_message.rs   # 错误消息显示组件
└── ...
```

---

## 🔵 Atoms - 原子组件

### Button 按钮组件

**位置**: `src/components/atoms/button.rs`

**用法**:
```rust
use crate::components::atoms::button::{Button, ButtonVariant, ButtonSize};

Button {
    variant: ButtonVariant::Primary,  // Primary | Secondary | Ghost
    size: ButtonSize::Medium,         // Small | Medium | Large
    onclick: move |_| {
        // 处理点击事件
    },
    "按钮文本"
}
```

**变体**:
- `Primary`: 主要按钮（蓝色背景）
- `Secondary`: 次要按钮（边框样式）
- `Ghost`: 幽灵按钮（无背景）

**尺寸**:
- `Small`: 小尺寸
- `Medium`: 中等尺寸（默认）
- `Large`: 大尺寸

---

### Input 输入框组件

**位置**: `src/components/atoms/input.rs`

**用法**:
```rust
use crate::components::atoms::input::{Input, InputType};

Input {
    input_type: InputType::Text,      // Text | Password | Number
    label: Some("标签".to_string()),
    placeholder: Some("占位符".to_string()),
    value: Some(value_signal.read().clone()),
    error: error_signal.read().clone(),
    onchange: Some(EventHandler::new(move |e: FormEvent| {
        // 处理输入变化
    })),
}
```

**类型**:
- `Text`: 文本输入
- `Password`: 密码输入（自动隐藏）
- `Number`: 数字输入

---

### Card 卡片组件

**位置**: `src/components/atoms/card.rs`

**用法**:
```rust
use crate::components::atoms::card::{Card, CardVariant};

Card {
    variant: CardVariant::Base,  // Base | Strong
    padding: Some("24px".to_string()),
    class: Some("mb-6".to_string()),
    children: rsx! {
        // 卡片内容
    }
}
```

**变体**:
- `Base`: 基础卡片（标准毛玻璃效果）
- `Strong`: 强化卡片（更强的毛玻璃效果，用于营销页面）

---

### Modal 模态框组件

**位置**: `src/components/atoms/modal.rs`

**用法**:
```rust
use crate::components::atoms::modal::Modal;

Modal {
    show: show_modal,
    title: "标题".to_string(),
    on_close: move || {
        show_modal.set(false);
    },
    children: rsx! {
        // 模态框内容
    }
}
```

---

## 🟢 Molecules - 分子组件

### ErrorMessage 错误消息组件

**位置**: `src/components/molecules/error_message.rs`

**功能**: 统一的错误消息显示样式

**用法**:
```rust
use crate::components::molecules::ErrorMessage;

let error_message = use_signal(|| Option::<String>::None);

rsx! {
    ErrorMessage {
        message: error_message.read().clone(),
        class: Some("mb-4".to_string())  // 可选的自定义类名
    }
}
```

**特性**:
- 统一的错误样式（红色背景、边框）
- 自动处理None值（无错误时不显示）
- 支持自定义类名

---

### QrCodeDisplay 二维码显示组件

**位置**: `src/components/molecules/qr_code_display.rs`

**功能**: 显示地址的二维码，支持复制功能

**用法**:
```rust
use crate::components::molecules::QrCodeDisplay;

rsx! {
    QrCodeDisplay {
        address: "0x1234...".to_string(),
        show_copy_button: Some(true)  // 可选，默认true
    }
}
```

**特性**:
- 自动生成二维码SVG
- 显示地址文本
- 复制到剪贴板功能
- 复制成功反馈

---

## 🟢 Molecules - 分子组件（续）

### ChainSelector 链选择器

**位置**: `src/components/molecules/chain_selector.rs`

**功能**: 选择区块链（Ethereum、Bitcoin、Solana、TON）

**用法**:
```rust
use crate::components::molecules::ChainSelector;

let selected_chain = use_signal(|| "ethereum".to_string());

rsx! {
    ChainSelector {
        selected_chain: selected_chain
    }
}
```

**支持的链**:
- `ethereum` / `eth` - Ethereum
- `bitcoin` / `btc` - Bitcoin
- `solana` / `sol` - Solana
- `ton` - TON

---

### GasFeeCard Gas费显示卡片

**位置**: `src/components/molecules/gas_fee_card.rs`

**功能**: 显示Gas费估算信息，支持加载状态

**用法**:
```rust
use crate::components::molecules::GasFeeCard;
use crate::services::gas::GasEstimate;

let gas_estimate = use_signal(|| Option::<GasEstimate>::None);
let gas_loading = use_signal(|| false);

rsx! {
    GasFeeCard {
        gas_estimate: gas_estimate.read().clone(),
        is_loading: gas_loading()
    }
}
```

**显示内容**:
- 预估Gas费（ETH）
- 预估时间（秒）
- 智能优化提示

**状态**:
- 加载中：显示"正在获取最优Gas费..."
- 有数据：显示Gas费详情
- 无数据：显示"Gas费将在发送时自动计算"

---

## 📝 使用示例

### Send页面示例

```rust
use crate::components::molecules::{ChainSelector, GasFeeCard};
use crate::components::atoms::input::{Input, InputType};

#[component]
pub fn Send() -> Element {
    let selected_chain = use_signal(|| "ethereum".to_string());
    let gas_estimate = use_signal(|| Option::<GasEstimate>::None);
    let gas_loading = use_signal(|| false);
    
    rsx! {
        div {
            // 链选择器
            ChainSelector {
                selected_chain: selected_chain
            }
            
            // 地址输入
            Input {
                input_type: InputType::Text,
                label: Some("接收地址".to_string()),
                placeholder: Some("请输入接收地址".to_string()),
                // ...
            }
            
            // Gas费显示
            GasFeeCard {
                gas_estimate: gas_estimate.read().clone(),
                is_loading: gas_loading()
            }
        }
    }
}
```

---

## 🎨 设计系统

所有组件都使用统一的设计令牌（`src/shared/design_tokens.rs`）:
- 颜色系统
- 间距系统
- 字体系统
- 阴影系统

---

## 🔄 组件复用原则

1. **原子组件**: 最小UI单元，不可再分割
2. **分子组件**: 由原子组件组合，可在多个页面复用
3. **页面组件**: 使用原子和分子组件构建完整页面

---

## 📚 相关文档

- [设计系统文档](../05-ui-ux/DESIGN_SYSTEM_V3.md)
- [开发指南](./DEVELOPMENT_PLAN.md)
- [路由状态](./ROUTER_STATUS.md)
- [页面状态](./PAGES_STATUS.md)

---

**最后更新**: 2025-01-27
