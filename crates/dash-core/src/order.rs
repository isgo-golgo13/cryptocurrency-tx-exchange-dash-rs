//! Order book types and market depth visualization

use crate::{colors, Price, Quantity, Symbol};
use serde::{Deserialize, Serialize};

// ============================================================================
// STRATEGY PATTERN: Depth Aggregation
// ============================================================================

/// Strategy trait for aggregating order book levels
pub trait DepthAggregator: Send + Sync {
    fn aggregate(&self, levels: &[OrderBookLevel]) -> Vec<AggregatedLevel>;
}

/// Aggregated level for visualization
#[derive(Debug, Clone)]
pub struct AggregatedLevel {
    pub price_min: f64,
    pub price_max: f64,
    pub total_quantity: f64,
    pub order_count: u32,
}

/// Fixed-bucket aggregator (groups levels into price buckets)
#[derive(Debug, Clone)]
pub struct FixedBucketAggregator {
    pub bucket_size: f64,
}

impl Default for FixedBucketAggregator {
    fn default() -> Self {
        Self { bucket_size: 10.0 }
    }
}

impl DepthAggregator for FixedBucketAggregator {
    fn aggregate(&self, levels: &[OrderBookLevel]) -> Vec<AggregatedLevel> {
        if levels.is_empty() {
            return Vec::new();
        }

        use std::collections::BTreeMap;
        let mut buckets: BTreeMap<i64, AggregatedLevel> = BTreeMap::new();

        for level in levels {
            let price = level.price.as_f64();
            let bucket_key = (price / self.bucket_size).floor() as i64;

            let entry = buckets.entry(bucket_key).or_insert_with(|| AggregatedLevel {
                price_min: bucket_key as f64 * self.bucket_size,
                price_max: (bucket_key + 1) as f64 * self.bucket_size,
                total_quantity: 0.0,
                order_count: 0,
            });

            entry.total_quantity += level.quantity.as_f64();
            entry.order_count += level.order_count;
        }

        buckets.into_values().collect()
    }
}

// ============================================================================
// CORE ORDER BOOK TYPES
// ============================================================================

/// Single level in the order book (price level aggregation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    pub price: Price,
    pub quantity: Quantity,
    /// Number of orders at this level
    pub order_count: u32,
}

impl OrderBookLevel {
    pub fn new(price: f64, quantity: f64, order_count: u32) -> Self {
        Self {
            price: Price::new(price),
            quantity: Quantity::new(quantity),
            order_count,
        }
    }

    /// Calculate total value at this level (price × quantity)
    pub fn value(&self) -> f64 {
        self.price.as_f64() * self.quantity.as_f64()
    }

    /// Percentage of max quantity (for bar sizing)
    pub fn quantity_percent(&self, max_qty: f64) -> f64 {
        if max_qty <= 0.0 {
            0.0
        } else {
            (self.quantity.as_f64() / max_qty * 100.0).min(100.0)
        }
    }
}

/// Order book side (bids or asks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Bid => colors::BULL,
            Self::Ask => colors::BEAR,
        }
    }

    pub fn bg_color(&self, alpha: f64) -> String {
        match self {
            Self::Bid => colors::bull_alpha(alpha),
            Self::Ask => colors::bear_alpha(alpha),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Bid => "BID",
            Self::Ask => "ASK",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Bid => "order-bid",
            Self::Ask => "order-ask",
        }
    }
}

/// Complete order book snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub symbol: Symbol,
    /// Sorted by price descending (highest bid first)
    pub bids: Vec<OrderBookLevel>,
    /// Sorted by price ascending (lowest ask first)
    pub asks: Vec<OrderBookLevel>,
    pub timestamp: i64,
    pub sequence: u64,
}

impl OrderBookSnapshot {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            bids: Vec::new(),
            asks: Vec::new(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            sequence: 0,
        }
    }

    /// Best bid price (highest buy order)
    pub fn best_bid(&self) -> Option<&OrderBookLevel> {
        self.bids.first()
    }

    /// Best ask price (lowest sell order)
    pub fn best_ask(&self) -> Option<&OrderBookLevel> {
        self.asks.first()
    }

    /// Current spread (ask - bid)
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price.as_f64() - bid.price.as_f64()),
            _ => None,
        }
    }

    /// Spread as percentage of mid price
    pub fn spread_percent(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => {
                let mid = (bid.price.as_f64() + ask.price.as_f64()) / 2.0;
                if mid == 0.0 {
                    None
                } else {
                    Some((ask.price.as_f64() - bid.price.as_f64()) / mid * 100.0)
                }
            }
            _ => None,
        }
    }

    /// Mid price (average of best bid and ask)
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price.as_f64() + ask.price.as_f64()) / 2.0),
            _ => None,
        }
    }

    /// Total bid depth (sum of all bid quantities)
    pub fn total_bid_depth(&self) -> f64 {
        self.bids.iter().map(|l| l.quantity.as_f64()).sum()
    }

    /// Total ask depth (sum of all ask quantities)
    pub fn total_ask_depth(&self) -> f64 {
        self.asks.iter().map(|l| l.quantity.as_f64()).sum()
    }

    /// Total bid value (sum of price × quantity)
    pub fn total_bid_value(&self) -> f64 {
        self.bids.iter().map(|l| l.value()).sum()
    }

    /// Total ask value
    pub fn total_ask_value(&self) -> f64 {
        self.asks.iter().map(|l| l.value()).sum()
    }

    /// Bid/Ask imbalance ratio (-1 to +1, positive = more bids)
    pub fn imbalance(&self) -> f64 {
        let bid_depth = self.total_bid_depth();
        let ask_depth = self.total_ask_depth();
        let total = bid_depth + ask_depth;
        if total == 0.0 {
            0.0
        } else {
            (bid_depth - ask_depth) / total
        }
    }

    /// Get max quantity across both sides (for bar scaling)
    pub fn max_quantity(&self) -> f64 {
        let bid_max = self.bids.iter().map(|l| l.quantity.as_f64()).fold(0.0_f64, f64::max);
        let ask_max = self.asks.iter().map(|l| l.quantity.as_f64()).fold(0.0_f64, f64::max);
        bid_max.max(ask_max)
    }

    /// Get price range (min bid, max ask)
    pub fn price_range(&self) -> Option<(f64, f64)> {
        let bid_min = self.bids.last().map(|l| l.price.as_f64());
        let ask_max = self.asks.last().map(|l| l.price.as_f64());

        match (bid_min, ask_max) {
            (Some(min), Some(max)) => Some((min, max)),
            (Some(min), None) => Some((min, min)),
            (None, Some(max)) => Some((max, max)),
            _ => None,
        }
    }

    /// Aggregate with custom strategy
    pub fn aggregate_with<A: DepthAggregator>(&self, aggregator: &A) -> (Vec<AggregatedLevel>, Vec<AggregatedLevel>) {
        (aggregator.aggregate(&self.bids), aggregator.aggregate(&self.asks))
    }
}

// ============================================================================
// MARKET DEPTH (for visualization)
// ============================================================================

/// Single point on depth chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthPoint {
    pub price: f64,
    /// Cumulative quantity up to this price
    pub cumulative_quantity: f64,
    /// Cumulative value up to this price
    pub cumulative_value: f64,
}

/// Aggregated market depth for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDepth {
    pub symbol: Symbol,
    /// Cumulative bid depth (sorted highest to lowest price)
    pub bid_depth: Vec<DepthPoint>,
    /// Cumulative ask depth (sorted lowest to highest price)
    pub ask_depth: Vec<DepthPoint>,
}

impl MarketDepth {
    /// Build from order book snapshot
    pub fn from_orderbook(book: &OrderBookSnapshot) -> Self {
        let mut bid_depth = Vec::with_capacity(book.bids.len());
        let mut ask_depth = Vec::with_capacity(book.asks.len());

        // Build cumulative bid depth (highest to lowest price)
        let mut cum_qty = 0.0;
        let mut cum_val = 0.0;
        for level in &book.bids {
            cum_qty += level.quantity.as_f64();
            cum_val += level.value();
            bid_depth.push(DepthPoint {
                price: level.price.as_f64(),
                cumulative_quantity: cum_qty,
                cumulative_value: cum_val,
            });
        }

        // Build cumulative ask depth (lowest to highest price)
        cum_qty = 0.0;
        cum_val = 0.0;
        for level in &book.asks {
            cum_qty += level.quantity.as_f64();
            cum_val += level.value();
            ask_depth.push(DepthPoint {
                price: level.price.as_f64(),
                cumulative_quantity: cum_qty,
                cumulative_value: cum_val,
            });
        }

        Self {
            symbol: book.symbol.clone(),
            bid_depth,
            ask_depth,
        }
    }

    /// Price range for depth chart
    pub fn price_range(&self) -> Option<(f64, f64)> {
        let bid_prices: Vec<f64> = self.bid_depth.iter().map(|p| p.price).collect();
        let ask_prices: Vec<f64> = self.ask_depth.iter().map(|p| p.price).collect();

        let all_prices: Vec<f64> = bid_prices.into_iter().chain(ask_prices).collect();

        if all_prices.is_empty() {
            None
        } else {
            let min = all_prices.iter().cloned().fold(f64::MAX, f64::min);
            let max = all_prices.iter().cloned().fold(f64::MIN, f64::max);
            Some((min, max))
        }
    }

    /// Maximum cumulative quantity (for Y-axis scaling)
    pub fn max_depth(&self) -> f64 {
        let bid_max = self.bid_depth.last().map(|p| p.cumulative_quantity).unwrap_or(0.0);
        let ask_max = self.ask_depth.last().map(|p| p.cumulative_quantity).unwrap_or(0.0);
        bid_max.max(ask_max)
    }

    /// Mid price from depth
    pub fn mid_price(&self) -> Option<f64> {
        let best_bid = self.bid_depth.first().map(|p| p.price);
        let best_ask = self.ask_depth.first().map(|p| p.price);

        match (best_bid, best_ask) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }
}

// ============================================================================
// ORDER BOOK DELTA (for incremental updates)
// ============================================================================

/// Delta update for order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDelta {
    pub symbol: Symbol,
    pub side: OrderSide,
    pub price: Price,
    pub quantity: Quantity,
    pub sequence: u64,
}

impl OrderBookDelta {
    /// Is this a removal (quantity = 0)?
    pub fn is_removal(&self) -> bool {
        self.quantity.as_f64() == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_orderbook() -> OrderBookSnapshot {
        let mut book = OrderBookSnapshot::new(Symbol::new("BTC-USD"));
        book.bids = vec![
            OrderBookLevel::new(50000.0, 1.0, 5),
            OrderBookLevel::new(49990.0, 2.0, 8),
            OrderBookLevel::new(49980.0, 1.5, 3),
        ];
        book.asks = vec![
            OrderBookLevel::new(50010.0, 0.8, 4),
            OrderBookLevel::new(50020.0, 1.2, 6),
            OrderBookLevel::new(50030.0, 2.0, 10),
        ];
        book
    }

    #[test]
    fn test_spread() {
        let book = sample_orderbook();
        assert_eq!(book.spread(), Some(10.0));
    }

    #[test]
    fn test_mid_price() {
        let book = sample_orderbook();
        assert_eq!(book.mid_price(), Some(50005.0));
    }

    #[test]
    fn test_imbalance() {
        let book = sample_orderbook();
        let imb = book.imbalance();
        assert!(imb > 0.0); // More bids than asks (4.5 vs 4.0)
    }

    #[test]
    fn test_market_depth() {
        let book = sample_orderbook();
        let depth = MarketDepth::from_orderbook(&book);

        assert_eq!(depth.bid_depth.len(), 3);
        assert_eq!(depth.ask_depth.len(), 3);

        // First bid cumulative should be 1.0
        assert_eq!(depth.bid_depth[0].cumulative_quantity, 1.0);
        // Last bid cumulative should be 4.5
        assert_eq!(depth.bid_depth[2].cumulative_quantity, 4.5);
    }

    #[test]
    fn test_aggregator_strategy() {
        let book = sample_orderbook();
        let aggregator = FixedBucketAggregator { bucket_size: 50.0 };

        let (agg_bids, _agg_asks) = book.aggregate_with(&aggregator);
        // All bids (50000, 49990, 49980) should fall into 49950-50000 bucket
        assert!(!agg_bids.is_empty());
    }
}