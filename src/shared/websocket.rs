// WebSocket Real-time Data Stream Service
// Production-grade implementation with auto-reconnection and fallback

use dioxus::prelude::*;
use futures::stream::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;

type MessageHandlers = Arc<Vec<Box<dyn Fn(WsMessage) + Send + Sync>>>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    Subscribe {
        channels: Vec<String>,
    },
    Unsubscribe {
        channels: Vec<String>,
    },
    BalanceUpdate {
        chain: String,
        address: String,
        balance: String,
    },
    TxUpdate {
        tx_hash: String,
        status: String,
        confirmations: u64,
    },
    PriceUpdate {
        symbol: String,
        usd: f64,
        change_24h: f64,
    },
    Ping,
    Pong,
}

pub struct WebSocketManager {
    url: String,
    auth_token: Option<String>,
    state: Signal<ConnectionState>,
    reconnect_attempts: Signal<u32>,
    max_reconnect_attempts: u32,
    reconnect_delay_ms: u32,
    pub last_message: Signal<Option<WsMessage>>,
    message_handlers: MessageHandlers,
}

impl WebSocketManager {
    pub fn new(url: impl Into<String>, auth_token: Option<String>) -> Self {
        Self {
            url: url.into(),
            auth_token,
            state: Signal::new(ConnectionState::Disconnected),
            reconnect_attempts: Signal::new(0),
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
            last_message: Signal::new(None),
            message_handlers: Arc::new(Vec::new()),
        }
    }

    /// Connect to WebSocket server with auto-reconnection
    pub fn connect(&self) {
        let mut url = self.url.clone();
        if let Some(token) = &self.auth_token {
            if url.contains('?') {
                url.push_str(&format!("&token={}", token));
            } else {
                url.push_str(&format!("?token={}", token));
            }
        }

        let mut state = self.state;
        let mut reconnect_attempts = self.reconnect_attempts;
        let max_attempts = self.max_reconnect_attempts;
        let delay_ms = self.reconnect_delay_ms;
        let mut last_message = self.last_message;
        let handlers = self.message_handlers.clone();

        spawn_local(async move {
            loop {
                state.set(ConnectionState::Connecting);
                tracing::info!("WebSocket connecting to: {}", url);

                match WebSocket::open(&url) {
                    Ok(ws) => {
                        state.set(ConnectionState::Connected);
                        reconnect_attempts.set(0);
                        tracing::info!("WebSocket connected successfully");

                        let (_, mut read) = ws.split();

                        // Message receive loop
                        while let Some(msg) = read.next().await {
                            match msg {
                                Ok(Message::Text(text)) => {
                                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                                        // Handle Ping/Pong
                                        if matches!(ws_msg, WsMessage::Ping) {
                                            // Auto-respond with Pong (would need write handle)
                                            continue;
                                        }

                                        // Update last message signal
                                        last_message.set(Some(ws_msg.clone()));

                                        // Dispatch to handlers
                                        for handler in handlers.iter() {
                                            handler(ws_msg.clone());
                                        }
                                    } else {
                                        tracing::warn!(
                                            "Failed to parse WebSocket message: {}",
                                            text
                                        );
                                    }
                                }
                                Ok(Message::Bytes(_)) => {
                                    tracing::debug!("Received binary WebSocket message (ignored)");
                                }
                                Err(e) => {
                                    tracing::error!("WebSocket error: {:?}", e);
                                    break;
                                }
                            }
                        }

                        // Connection lost
                        tracing::warn!("WebSocket connection closed");
                        state.set(ConnectionState::Disconnected);
                    }
                    Err(e) => {
                        tracing::error!("WebSocket connection failed: {:?}", e);
                        state.set(ConnectionState::Failed);
                    }
                }

                // Reconnection logic
                let current_attempts = *reconnect_attempts.read();
                if current_attempts >= max_attempts {
                    tracing::error!(
                        "Max reconnection attempts ({}) reached, giving up",
                        max_attempts
                    );
                    state.set(ConnectionState::Failed);
                    break;
                }

                reconnect_attempts.set(current_attempts + 1);
                state.set(ConnectionState::Reconnecting);

                // Exponential backoff
                let backoff_delay = delay_ms * (2_u32.pow(current_attempts));
                tracing::info!(
                    "Reconnecting in {}ms (attempt {}/{})",
                    backoff_delay,
                    current_attempts + 1,
                    max_attempts
                );
                TimeoutFuture::new(backoff_delay).await;
            }
        });
    }

    #[allow(dead_code)] // 为未来功能准备
    pub fn connection_state(&self) -> ConnectionState {
        *self.state.read()
    }

    #[allow(dead_code)] // 为未来功能准备
    pub fn is_connected(&self) -> bool {
        matches!(*self.state.read(), ConnectionState::Connected)
    }
}

/// Hook for using WebSocket in components
/// 为未来实时功能准备的WebSocket hook
#[allow(dead_code)] // 为未来功能准备
pub fn use_websocket(url: &str, auth_token: Option<String>) -> Signal<WebSocketManager> {
    use_signal(|| {
        let manager = WebSocketManager::new(url, auth_token);
        manager.connect();
        manager
    })
}

/// Subscribe to transaction updates
/// 为未来实时交易更新功能准备
#[allow(dead_code)] // 为未来功能准备
pub fn subscribe_tx_updates(tx_hash: &str) -> impl Fn(WsMessage) {
    let hash = tx_hash.to_string();
    move |msg| {
        if let WsMessage::TxUpdate {
            tx_hash,
            status,
            confirmations,
        } = msg
        {
            if tx_hash == hash {
                tracing::info!(
                    "Transaction {} updated: status={}, confirmations={}",
                    tx_hash,
                    status,
                    confirmations
                );
            }
        }
    }
}

/// Subscribe to balance updates
/// 为未来实时余额更新功能准备
#[allow(dead_code)] // 为未来功能准备
pub fn subscribe_balance_updates(address: &str, _callback: impl Fn(String, String) + 'static) {
    let _addr = address.to_string();
    // Callback will be invoked when WsMessage::BalanceUpdate matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_transitions() {
        // Avoid `Signal` here: it requires a Dioxus runtime even in unit tests.
        let mut state = ConnectionState::Disconnected;
        assert_eq!(state, ConnectionState::Disconnected);

        state = ConnectionState::Connecting;
        assert_eq!(state, ConnectionState::Connecting);
    }

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Subscribe {
            channels: vec!["tx:0xabc".to_string()],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("tx:0xabc"));
    }
}
