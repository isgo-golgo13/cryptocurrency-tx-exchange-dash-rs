//! Trade execution types with Strategy pattern for classification

use crate::{colors, Price, Quantity, Symbol};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// STRATEGY PATTERN: Trade Classification
// ============================================================================

/// Strategy trait for classifying trades (whale detection, etc.)
pub trait TradeClassifier: Send + Sync {
    fn classify(&self, trade: &Trade) -> TradeClassification;
}

/// Trade classification result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeClassification {
    Normal,
    Large,
    Whale,
    MicroTrade,
}

impl TradeClassification {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Normal => "trade-normal",
            Self::Large => "trade-large",
            Self::Whale => "trade-whale",
            Self::MicroTrade => "trade-micro",
        }
    }

    pub fn icon(&self) -> Option<&'static str> {
        match self {
            Self::Whale => Some("🐋"),
            Self::Large => Some("📈"),
            _ => None,
        }
    }
}

/// Default classifier based on USD value thresholds
#[derive(Debug, Clone)]
pub struct ValueThresholdClassifier {
    pub whale_threshold: f64,
    pub large_threshold: f64,
    pub micro_threshold: f64,
}

impl Default for ValueThresholdClassifier {
    fn default() -> Self {
        Self {
            whale_threshold: 1_000_000.0,  // $1M+
            large_threshold: 100_000.0,     // $100K+
            micro_threshold: 100.0,         // < $100
        }
    }
}

impl TradeClassifier for ValueThresholdClassifier {
    fn classify(&self, trade: &Trade) -> TradeClassification {
        let value = trade.value();
        if value >= self.whale_threshold {
            TradeClassification::Whale
        } else if value >= self.large_threshold {
            TradeClassification::Large
        } else if value < self.micro_threshold {
            TradeClassification::MicroTrade
        } else {
            TradeClassification::Normal
        }
    }
}

// ============================================================================
// CORE TYPES
// ============================================================================

/// Direction of a trade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

impl TradeSide {
    pub fn is_buy(&self) -> bool {
        matches!(self, Self::Buy)
    }

    pub fn is_sell(&self) -> bool {
        matches!(self, Self::Sell)
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Buy => "trade-buy",
            Self::Sell => "trade-sell",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Buy => colors::BULL,
            Self::Sell => colors::BEAR,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }

    pub fn arrow(&self) -> &'static str {
        match self {
            Self::Buy => "▲",
            Self::Sell => "▼",
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

impl Default for TradeSide {
    fn default() -> Self {
        Self::Buy
    }
}

/// Individual trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub symbol: Symbol,
    pub price: Price,
    pub quantity: Quantity,
    pub side: TradeSide,
    pub timestamp: DateTime<Utc>,
    /// Optional maker order ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maker_order_id: Option<String>,
    /// Optional taker order ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taker_order_id: Option<String>,
}

impl Trade {
    /// Create new trade with auto-generated ID
    pub fn new(symbol: Symbol, price: f64, quantity: f64, side: TradeSide) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            symbol,
            price: Price::new(price),
            quantity: Quantity::new(quantity),
            side,
            timestamp: Utc::now(),
            maker_order_id: None,
            taker_order_id: None,
        }
    }

    /// Builder: set maker order ID
    pub fn with_maker(mut self, order_id: impl Into<String>) -> Self {
        self.maker_order_id = Some(order_id.into());
        self
    }

    /// Builder: set taker order ID
    pub fn with_taker(mut self, order_id: impl Into<String>) -> Self {
        self.taker_order_id = Some(order_id.into());
        self
    }

    /// Calculate trade value (price × quantity)
    pub fn value(&self) -> f64 {
        self.price.as_f64() * self.quantity.as_f64()
    }

    /// Format timestamp for display (HH:MM:SS.mmm)
    pub fn time_str(&self) -> String {
        self.timestamp.format("%H:%M:%S%.3f").to_string()
    }

    /// Format timestamp short (HH:MM:SS)
    pub fn time_short(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }

    /// Classify trade using provided strategy
    pub fn classify_with<C: TradeClassifier>(&self, classifier: &C) -> TradeClassification {
        classifier.classify(self)
    }

    /// Check if whale trade using default classifier
    pub fn is_whale(&self) -> bool {
        let classifier = ValueThresholdClassifier::default();
        matches!(self.classify_with(&classifier), TradeClassification::Whale)
    }

    /// Age of trade in milliseconds
    pub fn age_ms(&self) -> i64 {
        (Utc::now() - self.timestamp).num_milliseconds()
    }
}

/// Aggregated trade statistics over a time window
#[derive(Debug, Clone, Default)]
pub struct TradeAggregation {
    pub symbol: Symbol,
    pub count: u64,
    pub buy_count: u64,
    pub sell_count: u64,
    pub total_volume: f64,
    pub buy_volume: f64,
    pub sell_volume: f64,
    pub total_value: f64,
    pub buy_value: f64,
    pub sell_value: f64,
    pub vwap: f64,
    pub high: f64,
    pub low: f64,
    pub first_price: f64,
    pub last_price: f64,
}

impl TradeAggregation {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            ..Default::default()
        }
    }

    /// Add trade to aggregation
    pub fn add(&mut self, trade: &Trade) {
        let price = trade.price.as_f64();
        let qty = trade.quantity.as_f64();
        let value = trade.value();

        self.count += 1;
        self.total_volume += qty;
        self.total_value += value;

        match trade.side {
            TradeSide::Buy => {
                self.buy_count += 1;
                self.buy_volume += qty;
                self.buy_value += value;
            }
            TradeSide::Sell => {
                self.sell_count += 1;
                self.sell_volume += qty;
                self.sell_value += value;
            }
        }

        // Update VWAP
        if self.total_volume > 0.0 {
            self.vwap = self.total_value / self.total_volume;
        }

        // Update high/low
        if self.count == 1 {
            self.high = price;
            self.low = price;
            self.first_price = price;
        } else {
            self.high = self.high.max(price);
            self.low = self.low.min(price);
        }
        self.last_price = price;
    }

    /// Buy/sell imbalance ratio (-1 to +1)
    pub fn imbalance(&self) -> f64 {
        let total = self.buy_volume + self.sell_volume;
        if total == 0.0 {
            0.0
        } else {
            (self.buy_volume - self.sell_volume) / total
        }
    }

    /// Price change from first to last
    pub fn price_change(&self) -> f64 {
        self.last_price - self.first_price
    }

    /// Price change as percentage
    pub fn price_change_pct(&self) -> f64 {
        if self.first_price == 0.0 {
            0.0
        } else {
            (self.last_price - self.first_price) / self.first_price * 100.0
        }
    }
}

/// Batch of trades for efficient transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeBatch {
    pub symbol: Symbol,
    pub trades: Vec<Trade>,
}

impl TradeBatch {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            trades: Vec::new(),
        }
    }

    pub fn with_capacity(symbol: Symbol, capacity: usize) -> Self {
        Self {
            symbol,
            trades: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    pub fn len(&self) -> usize {
        self.trades.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trades.is_empty()
    }

    /// Aggregate all trades in batch
    pub fn aggregate(&self) -> TradeAggregation {
        let mut agg = TradeAggregation::new(self.symbol.clone());
        for trade in &self.trades {
            agg.add(trade);
        }
        agg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trade_value() {
        let trade = Trade::new(Symbol::new("BTC-USD"), 50000.0, 0.5, TradeSide::Buy);
        assert_eq!(trade.value(), 25000.0);
    }

    #[test]
    fn test_trade_classification() {
        let classifier = ValueThresholdClassifier::default();

        let whale = Trade::new(Symbol::default(), 50000.0, 25.0, TradeSide::Buy);
        assert_eq!(whale.classify_with(&classifier), TradeClassification::Whale);

        let normal = Trade::new(Symbol::default(), 50000.0, 0.1, TradeSide::Sell);
        assert_eq!(normal.classify_with(&classifier), TradeClassification::Normal);
    }

    #[test]
    fn test_aggregation() {
        let mut agg = TradeAggregation::new(Symbol::default());

        agg.add(&Trade::new(Symbol::default(), 100.0, 1.0, TradeSide::Buy));
        agg.add(&Trade::new(Symbol::default(), 102.0, 2.0, TradeSide::Sell));
        agg.add(&Trade::new(Symbol::default(), 101.0, 1.0, TradeSide::Buy));

        assert_eq!(agg.count, 3);
        assert_eq!(agg.buy_count, 2);
        assert_eq!(agg.sell_count, 1);
        assert_eq!(agg.total_volume, 4.0);
    }
}