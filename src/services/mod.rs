pub mod address_detector;
pub mod auth;
pub mod balance;
pub mod bridge;
pub mod bridge_fee;
pub mod chain_config;
pub mod erc20;
pub mod fee;
pub mod gas;
pub mod payment_router;
pub mod payment_router_enterprise;
pub mod price;
pub mod storage;
pub mod swap;
pub mod token;
pub mod token_detection;
pub mod transaction;
pub mod tx_simple;
pub mod validation;
pub mod wallet;
pub mod wallet_manager;
pub mod wallet_transaction; // ✅ 非托管钱包管理器（企业级实现）
                            // 企业级服务：移除硬编码
pub mod bitcoin_fee;
pub mod gas_limit;

// 法币充值、提现和交易历史服务
pub mod fiat_offramp;
pub mod fiat_onramp;
pub mod transaction_history;

// 用户服务
pub mod user;

// 智能服务商选择服务
pub mod provider_selection;

// 限价单服务
pub mod limit_order;

// 前端优化服务
pub mod audit_log;
pub mod cache;
pub mod country_support;
pub mod error_logger;
pub mod error_reporter;
pub mod lazy_loader;
// pub mod payment_gateway; // 支付网关集成服务 - TODO: 需要实现
pub mod reconciliation;
pub mod webhook_handler;
pub mod withdrawal_review;
