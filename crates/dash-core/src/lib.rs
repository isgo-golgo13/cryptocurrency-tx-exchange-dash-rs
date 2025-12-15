//! # dash-core
//!
//! Core domain types for the BTC Exchange Dashboard.
//! Implements Strategy pattern for formatting and validation.

pub mod candle;
pub mod order;
pub mod ticker;
pub mod trade;

pub use candle::*;
pub use order::*;
pub use ticker::*;
pub use trade::*;

use serde::{Deserialize, Serialize};

// ============================================================================
// STRATEGY PATTERN: Formatters
// ============================================================================

/// Strategy trait for price formatting
pub trait PriceFormatter: Send + Sync {
    fn format(&self, price: f64) -> String;
}

/// Strategy trait for quantity formatting  
pub trait QuantityFormatter: Send + Sync {
    fn format(&self, qty: f64) -> String;
}

/// Strategy trait for large number formatting
pub trait LargeNumberFormatter: Send + Sync {
    fn format(&self, num: f64) -> String;
}

/// Default price formatter with configurable decimals
#[derive(Debug, Clone)]
pub struct DecimalPriceFormatter {
    pub decimals: usize,
}

impl Default for DecimalPriceFormatter {
    fn default() -> Self {
        Self { decimals: 2 }
    }
}

impl PriceFormatter for DecimalPriceFormatter {
    fn format(&self, price: f64) -> String {
        if price >= 10_000.0 {
            format!("{:.2}", price)
        } else if price >= 1.0 {
            format!("{:.prec$}", price, prec = self.decimals)
        } else if price >= 0.0001 {
            format!("{:.6}", price)
        } else {
            format!("{:.8}", price)
        }
    }
}

/// Compact formatter for large numbers (K, M, B suffixes)
#[derive(Debug, Clone, Default)]
pub struct CompactNumberFormatter;

impl LargeNumberFormatter for CompactNumberFormatter {
    fn format(&self, num: f64) -> String {
        let abs = num.abs();
        let sign = if num < 0.0 { "-" } else { "" };
        
        if abs >= 1_000_000_000.0 {
            format!("{}{:.2}B", sign, abs / 1_000_000_000.0)
        } else if abs >= 1_000_000.0 {
            format!("{}{:.2}M", sign, abs / 1_000_000.0)
        } else if abs >= 1_000.0 {
            format!("{}{:.2}K", sign, abs / 1_000.0)
        } else {
            format!("{}{:.2}", sign, abs)
        }
    }
}

/// Crypto quantity formatter (handles small decimals)
#[derive(Debug, Clone)]
pub struct CryptoQuantityFormatter {
    pub decimals: usize,
}

impl Default for CryptoQuantityFormatter {
    fn default() -> Self {
        Self { decimals: 8 }
    }
}

impl QuantityFormatter for CryptoQuantityFormatter {
    fn format(&self, qty: f64) -> String {
        if qty >= 1000.0 {
            format!("{:.2}", qty)
        } else if qty >= 1.0 {
            format!("{:.4}", qty)
        } else {
            format!("{:.prec$}", qty, prec = self.decimals)
        }
    }
}

// ============================================================================
// CORE VALUE TYPES
// ============================================================================

/// Trading pair identifier (e.g., "BTC-USD", "ETH-BTC")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract base currency (e.g., "BTC" from "BTC-USD")
    pub fn base(&self) -> &str {
        self.0.split('-').next().unwrap_or(&self.0)
    }

    /// Extract quote currency (e.g., "USD" from "BTC-USD")
    pub fn quote(&self) -> &str {
        self.0.split('-').nth(1).unwrap_or("USD")
    }
}

impl Default for Symbol {
    fn default() -> Self {
        Self("BTC-USD".to_string())
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Decimal price representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Price(pub f64);

impl Price {
    pub const ZERO: Price = Price(0.0);

    pub fn new(val: f64) -> Self {
        Self(val)
    }

    pub fn as_f64(&self) -> f64 {
        self.0
    }

    pub fn format(&self, decimals: usize) -> String {
        format!("{:.prec$}", self.0, prec = decimals)
    }

    pub fn format_with<F: PriceFormatter>(&self, formatter: &F) -> String {
        formatter.format(self.0)
    }
}

impl Default for Price {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Price {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::Sub for Price {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

/// Quantity representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quantity(pub f64);

impl Quantity {
    pub const ZERO: Quantity = Quantity(0.0);

    pub fn new(val: f64) -> Self {
        Self(val)
    }

    pub fn as_f64(&self) -> f64 {
        self.0
    }

    pub fn format(&self, decimals: usize) -> String {
        format!("{:.prec$}", self.0, prec = decimals)
    }

    pub fn format_with<F: QuantityFormatter>(&self, formatter: &F) -> String {
        formatter.format(self.0)
    }
}

impl Default for Quantity {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Quantity {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

// ============================================================================
// WEBSOCKET MESSAGE ENVELOPE
// ============================================================================

/// WebSocket message envelope with discriminated union
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    #[serde(rename = "trade")]
    Trade(Trade),
    #[serde(rename = "orderbook")]
    OrderBook(OrderBookSnapshot),
    #[serde(rename = "ticker")]
    Ticker(Ticker),
    #[serde(rename = "candle")]
    Candle(Candle),
    #[serde(rename = "depth")]
    Depth(MarketDepth),
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: i64 },
}

/// Connection state FSM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

impl ConnectionState {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Connecting => "Connecting...",
            Self::Connected => "Connected",
            Self::Reconnecting => "Reconnecting...",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Disconnected => "conn-disconnected",
            Self::Connecting => "conn-connecting",
            Self::Connected => "conn-connected",
            Self::Reconnecting => "conn-reconnecting",
        }
    }
}

// ============================================================================
// COLOR CONSTANTS
// ============================================================================

pub mod colors {
    pub const BULL: &str = "#22c55e";
    pub const BEAR: &str = "#ef4444";
    pub const NEUTRAL: &str = "#888888";
    pub const WARN: &str = "#fbbf24";
    pub const BG_VOID: &str = "#0a0a0a";
    pub const BG_PANEL: &str = "#141414";
    pub const BG_ELEVATED: &str = "#1a1a1a";
    pub const BORDER: &str = "#2a2a2a";
    pub const TEXT_PRIMARY: &str = "#fafafa";
    pub const TEXT_MUTED: &str = "#888888";
    pub const GRID: &str = "#1f1f1f";

    pub fn bull_alpha(alpha: f64) -> String {
        format!("rgba(34, 197, 94, {:.2})", alpha)
    }

    pub fn bear_alpha(alpha: f64) -> String {
        format!("rgba(239, 68, 68, {:.2})", alpha)
    }

    pub fn warn_alpha(alpha: f64) -> String {
        format!("rgba(251, 191, 36, {:.2})", alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_parsing() {
        let sym = Symbol::new("ETH-BTC");
        assert_eq!(sym.base(), "ETH");
        assert_eq!(sym.quote(), "BTC");
    }

    #[test]
    fn test_price_formatter_strategy() {
        let formatter = DecimalPriceFormatter { decimals: 4 };
        let price = Price::new(42.5678);
        assert_eq!(price.format_with(&formatter), "42.5678");
    }

    #[test]
    fn test_compact_formatter() {
        let formatter = CompactNumberFormatter;
        assert_eq!(formatter.format(1_500_000.0), "1.50M");
        assert_eq!(formatter.format(2_500.0), "2.50K");
        assert_eq!(formatter.format(500.0), "500.00");
    }
}