//! # dash-state
//!
//! Reactive state management for the BTC Exchange Dashboard.
//! Uses Leptos signals for surgical DOM updates on market data changes.

pub mod market;

pub use market::*;

use dash_core::ConnectionState;
use leptos::prelude::*;

/// Configuration constants
pub const MAX_TRADES: usize = 100;
pub const MAX_CANDLES: usize = 200;

// ============================================================================
// UI STATE
// ============================================================================

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Dark => "theme-dark",
            Self::Light => "theme-light",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}

/// Panel visibility state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelVisibility {
    pub orderbook: bool,
    pub trades: bool,
    pub depth_chart: bool,
    pub candle_chart: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            orderbook: true,
            trades: true,
            depth_chart: true,
            candle_chart: true,
        }
    }
}

/// Global UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiState {
    pub theme: Theme,
    pub panels: PanelVisibility,
    pub compact_mode: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            panels: PanelVisibility::default(),
            compact_mode: false,
        }
    }
}

// ============================================================================
// APPLICATION STATE
// ============================================================================

/// Global application state with reactive signals
#[derive(Clone)]
pub struct AppState {
    /// Market data state
    pub market: MarketState,
    /// WebSocket connection state
    pub connection: RwSignal<ConnectionState>,
    /// UI state (theme, panels, etc.)
    pub ui: RwSignal<UiState>,
    /// Current error message
    pub error: RwSignal<Option<String>>,
    /// Loading state
    pub loading: RwSignal<bool>,
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self {
            market: MarketState::new(),
            connection: RwSignal::new(ConnectionState::Disconnected),
            ui: RwSignal::new(UiState::default()),
            error: RwSignal::new(None),
            loading: RwSignal::new(false),
        }
    }

    // ========================================================================
    // Connection State
    // ========================================================================

    /// Set connected state
    pub fn set_connected(&self) {
        self.connection.set(ConnectionState::Connected);
        self.error.set(None);
    }

    /// Set disconnected state
    pub fn set_disconnected(&self) {
        self.connection.set(ConnectionState::Disconnected);
    }

    /// Set connecting state
    pub fn set_connecting(&self) {
        self.connection.set(ConnectionState::Connecting);
    }

    /// Set reconnecting state
    pub fn set_reconnecting(&self) {
        self.connection.set(ConnectionState::Reconnecting);
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection.get().is_connected()
    }

    // ========================================================================
    // Error Handling
    // ========================================================================

    /// Set error message
    pub fn set_error(&self, msg: impl Into<String>) {
        self.error.set(Some(msg.into()));
    }

    /// Clear error
    pub fn clear_error(&self) {
        self.error.set(None);
    }

    /// Check if has error
    pub fn has_error(&self) -> bool {
        self.error.get().is_some()
    }

    // ========================================================================
    // UI State
    // ========================================================================

    /// Toggle theme
    pub fn toggle_theme(&self) {
        self.ui.update(|ui| {
            ui.theme = ui.theme.toggle();
        });
    }

    /// Set theme
    pub fn set_theme(&self, theme: Theme) {
        self.ui.update(|ui| {
            ui.theme = theme;
        });
    }

    /// Toggle panel visibility
    pub fn toggle_panel(&self, panel: Panel) {
        self.ui.update(|ui| {
            match panel {
                Panel::OrderBook => ui.panels.orderbook = !ui.panels.orderbook,
                Panel::Trades => ui.panels.trades = !ui.panels.trades,
                Panel::DepthChart => ui.panels.depth_chart = !ui.panels.depth_chart,
                Panel::CandleChart => ui.panels.candle_chart = !ui.panels.candle_chart,
            }
        });
    }

    /// Check if panel is visible
    pub fn is_panel_visible(&self, panel: Panel) -> bool {
        let ui = self.ui.get();
        match panel {
            Panel::OrderBook => ui.panels.orderbook,
            Panel::Trades => ui.panels.trades,
            Panel::DepthChart => ui.panels.depth_chart,
            Panel::CandleChart => ui.panels.candle_chart,
        }
    }

    /// Toggle compact mode
    pub fn toggle_compact_mode(&self) {
        self.ui.update(|ui| {
            ui.compact_mode = !ui.compact_mode;
        });
    }

    // ========================================================================
    // Loading State
    // ========================================================================

    /// Set loading state
    pub fn set_loading(&self, loading: bool) {
        self.loading.set(loading);
    }

    /// Check if loading
    pub fn is_loading(&self) -> bool {
        self.loading.get()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Dashboard panel identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    OrderBook,
    Trades,
    DepthChart,
    CandleChart,
}

impl Panel {
    pub fn label(&self) -> &'static str {
        match self {
            Self::OrderBook => "Order Book",
            Self::Trades => "Trades",
            Self::DepthChart => "Depth Chart",
            Self::CandleChart => "Chart",
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::OrderBook, Self::Trades, Self::DepthChart, Self::CandleChart]
    }
}

// ============================================================================
// CONTEXT HELPERS
// ============================================================================

/// Provide app state context to component tree
pub fn provide_app_state() -> AppState {
    let state = AppState::new();
    provide_context(state.clone());
    state
}

/// Use app state from context
pub fn use_app_state() -> AppState {
    expect_context::<AppState>()
}

/// Try to get app state from context (returns None if not provided)
pub fn try_use_app_state() -> Option<AppState> {
    use_context::<AppState>()
}