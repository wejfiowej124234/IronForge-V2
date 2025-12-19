//! 企业级标准 API 端点定义
//!
//! 统一管理所有 API 路径，确保前后端一致性
//!
//! ## 版本规范
//! - `/api/v1/*` - 企业级标准路径（推荐使用）
//! - `/api/*` - 兼容性路径（部分端点保留）
//!
//! ## 命名规范
//! - 使用复数形式：`/wallets`, `/transactions`, `/orders`
//! - RESTful 风格：GET /wallets, POST /wallets, GET /wallets/:id
//! - 子资源使用嵌套：`/wallets/:id/transactions`

/// 认证相关端点（✅ 企业级标准 V1）
pub mod auth {
    pub const REGISTER: &str = "/api/v1/auth/register";
    pub const LOGIN: &str = "/api/v1/auth/login";
    pub const LOGOUT: &str = "/api/v1/auth/logout";
    pub const REFRESH: &str = "/api/v1/auth/refresh";
    pub const ME: &str = "/api/v1/auth/me";
    pub const CHANGE_PASSWORD: &str = "/api/v1/auth/change-password";
    pub const SET_PASSWORD: &str = "/api/v1/auth/set-password";
    pub const RESET_PASSWORD: &str = "/api/v1/auth/reset-password";
    pub const LOGIN_HISTORY: &str = "/api/v1/auth/login-history";
    pub const CHALLENGE: &str = "/api/v1/auth/challenge";
    pub const VERIFY: &str = "/api/v1/auth/verify";
}

/// 钱包相关端点（企业级标准：v1）
pub mod wallets {
    pub const LIST: &str = "/api/v1/wallets";
    pub const CREATE_BATCH: &str = "/api/v1/wallets/batch";
    pub const ASSETS: &str = "/api/v1/wallets/assets";

    /// 钱包详情：/api/v1/wallets/:id
    pub fn detail(wallet_id: &str) -> String {
        format!("/api/v1/wallets/{}", wallet_id)
    }

    /// 钱包资产：/api/v1/wallets/:id/assets
    pub fn wallet_assets(wallet_id: &str) -> String {
        format!("/api/v1/wallets/{}/assets", wallet_id)
    }

    /// 钱包交易：/api/v1/wallets/:address/transactions
    pub fn transactions(address: &str) -> String {
        format!("/api/v1/wallets/{}/transactions", address)
    }

    /// 钱包余额：/api/v1/wallets/:address/balance
    pub fn balance(address: &str) -> String {
        format!("/api/v1/wallets/{}/balance", address)
    }
}

/// 交易相关端点（企业级标准：v1）
pub mod transactions {
    pub const LIST: &str = "/api/v1/transactions";
    pub const BROADCAST: &str = "/api/v1/transactions/broadcast";
    pub const NONCE: &str = "/api/v1/transactions/nonce";
    pub const HISTORY: &str = "/api/v1/transactions/history";

    /// 交易状态：/api/v1/transactions/:hash/status
    pub fn status(tx_hash: &str) -> String {
        format!("/api/v1/transactions/{}/status", tx_hash)
    }

    /// 交易详情：/api/v1/transactions/:id
    pub fn detail(tx_id: &str) -> String {
        format!("/api/v1/transactions/{}", tx_id)
    }
}

/// 兑换相关端点（企业级标准：v1）
pub mod swap {
    pub const QUOTE: &str = "/api/v1/swap/quote";
    pub const EXECUTE: &str = "/api/v1/swap/execute";
    pub const HISTORY: &str = "/api/v1/swap/history";

    /// 兑换状态：/api/v1/swap/:id/status
    pub fn status(swap_id: &str) -> String {
        format!("/api/v1/swap/{}/status", swap_id)
    }

    /// 兑换详情：/api/v1/swap/history/:id
    pub fn history_detail(swap_id: &str) -> String {
        format!("/api/v1/swap/history/{}", swap_id)
    }
}

/// 跨链桥接端点（企业级标准：v1）
pub mod bridge {
    pub const QUOTE: &str = "/api/v1/bridge/quote";
    pub const EXECUTE: &str = "/api/v1/bridge/execute";
    pub const HISTORY: &str = "/api/v1/bridge/history";

    /// 桥接状态：/api/v1/bridge/:id/status
    pub fn status(bridge_id: &str) -> String {
        format!("/api/v1/bridge/{}/status", bridge_id)
    }
}

/// 限价单端点（企业级标准：v1）
pub mod limit_orders {
    pub const LIST: &str = "/api/v1/limit-orders";
    pub const CREATE: &str = "/api/v1/limit-orders";

    /// 限价单详情：/api/v1/limit-orders/:id
    pub fn detail(order_id: &str) -> String {
        format!("/api/v1/limit-orders/{}", order_id)
    }

    /// 取消限价单：/api/v1/limit-orders/:id/cancel
    pub fn cancel(order_id: &str) -> String {
        format!("/api/v1/limit-orders/{}/cancel", order_id)
    }
}

/// 法币充值/提现端点（企业级标准：v1）
pub mod fiat {
    // 充值
    pub const ONRAMP_QUOTE: &str = "/api/v1/fiat/onramp/quote";
    pub const ONRAMP_ORDER: &str = "/api/v1/fiat/onramp/orders";
    pub const ONRAMP_ORDERS_LIST: &str = "/api/v1/fiat/onramp/orders";

    /// 充值订单详情：/api/v1/fiat/onramp/orders/:id
    pub fn onramp_order_detail(order_id: &str) -> String {
        format!("/api/v1/fiat/onramp/orders/{}", order_id)
    }

    /// 取消充值订单：/api/v1/fiat/onramp/orders/:id/cancel
    pub fn onramp_order_cancel(order_id: &str) -> String {
        format!("/api/v1/fiat/onramp/orders/{}/cancel", order_id)
    }

    /// 重试充值订单：/api/v1/fiat/onramp/orders/:id/retry
    pub fn onramp_order_retry(order_id: &str) -> String {
        format!("/api/v1/fiat/onramp/orders/{}/retry", order_id)
    }

    // 提现
    pub const OFFRAMP_QUOTE: &str = "/api/v1/fiat/offramp/quote";
    pub const OFFRAMP_ORDER: &str = "/api/v1/fiat/offramp/orders";
    pub const OFFRAMP_ORDERS_LIST: &str = "/api/v1/fiat/offramp/orders";

    /// 提现订单详情：/api/v1/fiat/offramp/orders/:id
    pub fn offramp_order_detail(order_id: &str) -> String {
        format!("/api/v1/fiat/offramp/orders/{}", order_id)
    }

    /// 取消提现订单：/api/v1/fiat/offramp/orders/:id/cancel
    pub fn offramp_order_cancel(order_id: &str) -> String {
        format!("/api/v1/fiat/offramp/orders/{}/cancel", order_id)
    }

    /// 重试提现订单：/api/v1/fiat/offramp/orders/:id/retry
    pub fn offramp_order_retry(order_id: &str) -> String {
        format!("/api/v1/fiat/offramp/orders/{}/retry", order_id)
    }
}

/// Gas 估算端点（✅ 企业级标准 V1）
pub mod gas {
    pub const ESTIMATE: &str = "/api/v1/gas/estimate";
    pub const ESTIMATE_ALL: &str = "/api/v1/gas/estimate-all";
}

/// Token 端点（✅ 企业级标准 V1）
pub mod tokens {
    pub const LIST: &str = "/api/v1/tokens/list";
    pub const DETECT: &str = "/api/v1/tokens/detect";
    pub const METADATA: &str = "/api/v1/tokens/metadata";
    pub const SEARCH: &str = "/api/v1/tokens/search";
    pub const POPULAR: &str = "/api/v1/tokens/popular";
    pub const BALANCES: &str = "/api/v1/tokens/balances";

    /// Token 详情：/api/v1/tokens/:address/info
    pub fn info(token_address: &str) -> String {
        format!("/api/v1/tokens/{}/info", token_address)
    }

    /// Token 余额：/api/v1/tokens/:address/balance
    pub fn balance(token_address: &str) -> String {
        format!("/api/v1/tokens/{}/balance", token_address)
    }
}

/// 服务商端点（✅ 企业级标准 V1）
pub mod providers {
    pub const LIST: &str = "/api/v1/providers";
    pub const COUNTRY_SUPPORT: &str = "/api/v1/providers/country-support";

    /// 服务商健康检查：/api/v1/providers/:provider/health
    pub fn health(provider_name: &str) -> String {
        format!("/api/v1/providers/{}/health", provider_name)
    }

    /// 服务商报价：/api/v1/providers/:provider/quote
    pub fn quote(provider_name: &str) -> String {
        format!("/api/v1/providers/{}/quote", provider_name)
    }

    /// 服务商国家列表：/api/v1/providers/:provider/countries
    pub fn countries(provider_name: &str) -> String {
        format!("/api/v1/providers/{}/countries", provider_name)
    }

    /// 服务商国家支持：/api/v1/providers/:provider/countries/:country
    pub fn country_support(provider_name: &str, country_code: &str) -> String {
        format!(
            "/api/v1/providers/{}/countries/{}",
            provider_name, country_code
        )
    }
}

/// 审计日志端点（✅ 企业级标准 V1）
pub mod audit {
    pub const LOGS: &str = "/api/v1/audit/logs";
    pub const COMPLIANCE_REPORT: &str = "/api/v1/audit/compliance/report";

    /// 合规报告详情：/api/v1/audit/compliance/report/:id
    pub fn compliance_report_detail(report_id: &str) -> String {
        format!("/api/v1/audit/compliance/report/{}", report_id)
    }
}

/// 费用端点（企业级标准：v1）
pub mod fees {
    pub const QUERY: &str = "/api/v1/fees";
    pub const CALCULATE: &str = "/api/v1/fees/calculate";
}

/// 系统状态端点（企业级标准：v1）
pub mod system {
    pub const NETWORK_STATUS: &str = "/api/v1/network/status";
    pub const BALANCE: &str = "/api/v1/balance";
}

/// 区块链特定端点（企业级标准：v1）
pub mod blockchain {
    pub const SOLANA_RECENT_BLOCKHASH: &str = "/api/v1/solana/recent-blockhash";
    pub const TON_SEQNO: &str = "/api/v1/ton/seqno";
    pub const TON_BROADCAST: &str = "/api/v1/ton/broadcast";
    pub const BITCOIN_FEE_ESTIMATES: &str = "/api/v1/bitcoin/fee-estimates";
}

/// 其他端点（✅ 企业级标准 V1）
pub mod misc {
    pub const NETWORK_CONFIG: &str = "/api/v1/network-config";
    pub const FEATURES: &str = "/api/v1/features";
    pub const PRICES: &str = "/api/v1/prices";
    pub const CHAINS: &str = "/api/v1/chains";
    pub const CHAINS_BY_CURVE: &str = "/api/v1/chains/by-curve";
}
