use crate::shared::state::AppState;
use anyhow::{anyhow, Result};
use dioxus::prelude::{ReadableExt, WritableExt};

const WALLET_UNLOCK_TTL_SECS: u64 = 5 * 60;

fn now_ts() -> u64 {
    // WASM-compatible: Use js_sys::Date instead of SystemTime
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

pub fn is_wallet_unlocked(app_state: &AppState, wallet_id: &str) -> bool {
    let map = app_state.wallet_unlock_time.read();
    if let Some(ts) = map.get(wallet_id) {
        let now = now_ts();
        now.saturating_sub(*ts) <= WALLET_UNLOCK_TTL_SECS
    } else {
        false
    }
}

fn mark_wallet_unlocked(app_state: &mut AppState, wallet_id: &str) {
    let mut map = app_state.wallet_unlock_time.write();
    map.insert(wallet_id.to_string(), now_ts());
}

/// 统一的双锁检查入口
///
/// - 已选择钱包但未在 TTL 内解锁时，返回业务错误
/// - 由调用方决定如何提示用户
pub fn ensure_wallet_unlocked(app_state: &AppState, wallet_id: &str) -> Result<()> {
    if !is_wallet_unlocked(app_state, wallet_id) {
        return Err(anyhow!("钱包已锁定，请先在钱包页解锁"));
    }
    Ok(())
}
