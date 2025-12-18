//! WebSocket client implementation with auto-reconnection

use crate::{ReconnectPolicy, WsConfig};
use dash_core::WsMessage;
use dash_state::AppState;
use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wasm_bindgen_futures::spawn_local;

// ============================================================================
// WEBSOCKET CLIENT
// ============================================================================

/// WebSocket client for market data streaming
pub struct WsClient {
    config: WsConfig,
    state: AppState,
}

impl WsClient {
    /// Create new WebSocket client
    pub fn new(state: AppState) -> Self {
        Self {
            config: WsConfig::default(),
            state,
        }
    }

    /// Create with custom configuration
    pub fn with_config(state: AppState, config: WsConfig) -> Self {
        Self { config, state }
    }

    /// Set WebSocket URL
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.config.url = url.into();
        self
    }

    /// Start the WebSocket connection (spawns async task)
    pub fn connect(self) -> WsHandle {
        let handle = WsHandle::new();
        let handle_clone = handle.clone();

        spawn_local(async move {
            self.run_connection_loop(handle_clone).await;
        });

        handle
    }

    /// Main connection loop with reconnection logic
    async fn run_connection_loop(self, handle: WsHandle) {
        let mut attempt = 0u32;
        let mut policy = self.config.reconnect_policy.clone();

        loop {
            if handle.is_stopped() {
                tracing::info!("WebSocket client stopped by handle");
                self.state.set_disconnected();
                break;
            }

            self.state.set_connecting();
            tracing::info!("Connecting to WebSocket: {}", self.config.url);

            match WebSocket::open(&self.config.url) {
                Ok(ws) => {
                    self.state.set_connected();
                    policy.reset();
                    attempt = 0;

                    tracing::info!("WebSocket connected");

                    self.handle_connection(ws, &handle).await;

                    if handle.is_stopped() {
                        tracing::info!("WebSocket stopped during connection");
                        break;
                    }

                    self.state.set_disconnected();
                    tracing::warn!("WebSocket disconnected");
                }
                Err(e) => {
                    tracing::error!("WebSocket connection failed: {:?}", e);
                    self.state.set_error(format!("Connection failed: {:?}", e));
                }
            }

            if !policy.should_reconnect(attempt) {
                tracing::error!("Max reconnection attempts ({}) reached", attempt);
                self.state.set_error("Max reconnection attempts reached");
                break;
            }

            let delay = policy.delay_ms(attempt);
            self.state.set_reconnecting();
            tracing::info!("Reconnecting in {}ms (attempt {})", delay, attempt + 1);

            TimeoutFuture::new(delay).await;
            attempt += 1;
        }
    }

    /// Handle an active WebSocket connection
    async fn handle_connection(&self, ws: WebSocket, handle: &WsHandle) {
        let (_write, mut read) = ws.split();

        while let Some(msg) = read.next().await {
            if handle.is_stopped() {
                break;
            }

            match msg {
                Ok(Message::Text(text)) => {
                    self.process_message(&text);
                }
                Ok(Message::Bytes(bytes)) => {
                    if let Ok(text) = String::from_utf8(bytes) {
                        self.process_message(&text);
                    }
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {:?}", e);
                    break;
                }
            }
        }
    }

    /// Process a received WebSocket message
    fn process_message(&self, text: &str) {
        match serde_json::from_str::<WsMessage>(text) {
            Ok(msg) => {
                self.dispatch_message(msg);
            }
            Err(e) => {
                tracing::warn!("Failed to parse WebSocket message: {}", e);
            }
        }
    }

    /// Dispatch parsed message to appropriate state handler
    fn dispatch_message(&self, msg: WsMessage) {
        match msg {
            WsMessage::Trade(trade) => {
                self.state.market.add_trade(trade);
            }
            WsMessage::OrderBook(book) => {
                self.state.market.update_orderbook(book);
            }
            WsMessage::Ticker(ticker) => {
                self.state.market.update_ticker(ticker);
            }
            WsMessage::Candle(candle) => {
                self.state.market.update_candle(candle);
            }
            WsMessage::Depth(depth) => {
                self.state.market.depth.set(Some(depth));
            }
            WsMessage::Heartbeat { timestamp } => {
                tracing::trace!("Heartbeat received: {}", timestamp);
            }
        }
    }
}

// ============================================================================
// WEBSOCKET HANDLE (Send + Sync)
// ============================================================================

/// Handle for controlling the WebSocket connection
#[derive(Clone)]
pub struct WsHandle {
    stopped: Arc<AtomicBool>,
}

impl WsHandle {
    fn new() -> Self {
        Self {
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Stop the WebSocket connection
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        !self.is_stopped()
    }
}

// ============================================================================
// LEPTOS INTEGRATION
// ============================================================================

/// Hook to create and manage WebSocket connection in Leptos components
pub fn use_websocket(state: AppState, url: Option<String>) -> WsHandle {
    let config = WsConfig::new(url.unwrap_or_else(|| crate::DEFAULT_WS_URL.to_string()));
    WsClient::with_config(state, config).connect()
}

/// Hook with custom configuration
pub fn use_websocket_with_config(state: AppState, config: WsConfig) -> WsHandle {
    WsClient::with_config(state, config).connect()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_handle() {
        let handle = WsHandle::new();
        assert!(!handle.is_stopped());
        assert!(handle.is_running());

        handle.stop();
        assert!(handle.is_stopped());
        assert!(!handle.is_running());
    }

    #[test]
    fn test_ws_config() {
        let config = WsConfig::new("ws://localhost:8080")
            .heartbeat(15000)
            .timeout(5000);

        assert_eq!(config.url, "ws://localhost:8080");
        assert_eq!(config.heartbeat_interval_ms, 15000);
        assert_eq!(config.connect_timeout_ms, 5000);
    }
}