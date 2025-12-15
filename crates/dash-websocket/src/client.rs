//! WebSocket client implementation with auto-reconnection

use crate::{ReconnectPolicy, WsConfig};
use dash_core::{ConnectionState, WsMessage};
use dash_state::AppState;
use futures::StreamExt;
use gloo_net::websocket::{futures::WebSocket, Message};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
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
            // Check if client was stopped
            if handle.is_stopped() {
                tracing::info!("WebSocket client stopped by handle");
                self.state.set_disconnected();
                break;
            }

            // Update state to connecting
            self.state.set_connecting();
            tracing::info!("Connecting to WebSocket: {}", self.config.url);

            // Attempt connection
            match WebSocket::open(&self.config.url) {
                Ok(ws) => {
                    // Connection successful
                    self.state.set_connected();
                    policy.reset();
                    attempt = 0;

                    tracing::info!("WebSocket connected");

                    // Handle the connection
                    self.handle_connection(ws, &handle).await;

                    // Connection closed
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

            // Check if we should reconnect
            if !policy.should_reconnect(attempt) {
                tracing::error!("Max reconnection attempts ({}) reached", attempt);
                self.state.set_error("Max reconnection attempts reached");
                break;
            }

            // Calculate delay and wait
            let delay = policy.delay_ms(attempt);
            self.state.set_reconnecting();
            tracing::info!(
                "Reconnecting in {}ms (attempt {}/{})",
                delay,
                attempt + 1,
                if self.config.reconnect_policy.max_attempts == 0 {
                    "∞".to_string()
                } else {
                    self.config.reconnect_policy.max_attempts.to_string()
                }
            );

            // Wait before reconnecting
            TimeoutFuture::new(delay).await;
            attempt += 1;
        }
    }

    /// Handle an active WebSocket connection
    async fn handle_connection(&self, ws: WebSocket, handle: &WsHandle) {
        let (_write, mut read) = ws.split();

        // TODO: If heartbeat is enabled, spawn heartbeat task
        // let heartbeat_handle = if self.config.heartbeat_interval_ms > 0 {
        //     Some(spawn_heartbeat(write.clone(), self.config.heartbeat_interval_ms))
        // } else {
        //     None
        // };

        // Read messages
        while let Some(msg) = read.next().await {
            if handle.is_stopped() {
                break;
            }

            match msg {
                Ok(Message::Text(text)) => {
                    self.process_message(&text);
                }
                Ok(Message::Bytes(bytes)) => {
                    // Try to parse as UTF-8 JSON
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
                tracing::warn!("Failed to parse WebSocket message: {} - {}", e, &text[..text.len().min(100)]);
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
// WEBSOCKET HANDLE
// ============================================================================

/// Handle for controlling the WebSocket connection
#[derive(Clone)]
pub struct WsHandle {
    stopped: Rc<Cell<bool>>,
}

impl WsHandle {
    fn new() -> Self {
        Self {
            stopped: Rc::new(Cell::new(false)),
        }
    }

    /// Stop the WebSocket connection
    pub fn stop(&self) {
        self.stopped.set(true);
    }

    /// Check if stopped
    pub fn is_stopped(&self) -> bool {
        self.stopped.get()
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        !self.stopped.get()
    }
}

impl Drop for WsHandle {
    fn drop(&mut self) {
        // Auto-stop when handle is dropped (if this is the last reference)
        if Rc::strong_count(&self.stopped) == 1 {
            self.stop();
        }
    }
}

// ============================================================================
// LEPTOS INTEGRATION
// ============================================================================

/// Hook to create and manage WebSocket connection in Leptos components
pub fn use_websocket(state: AppState, url: Option<String>) -> WsHandle {
    let config = WsConfig::new(url.unwrap_or_else(|| crate::DEFAULT_WS_URL.to_string()));

    let handle = WsClient::with_config(state, config).connect();

    // Cleanup on component unmount
    let handle_cleanup = handle.clone();
    on_cleanup(move || {
        handle_cleanup.stop();
    });

    handle
}

/// Hook with custom configuration
pub fn use_websocket_with_config(state: AppState, config: WsConfig) -> WsHandle {
    let handle = WsClient::with_config(state, config).connect();

    let handle_cleanup = handle.clone();
    on_cleanup(move || {
        handle_cleanup.stop();
    });

    handle
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
