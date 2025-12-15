//! Reactive market data state with fine-grained signal updates

use crate::{MAX_CANDLES, MAX_TRADES};
use dash_core::{
    Candle, CandleHistory, CandleInterval, MarketDepth, OrderBookSnapshot,
    Symbol, Ticker, Trade, TradeSide,
};
use leptos::prelude::*;

/// Reactive market state for a single symbol
#[derive(Clone)]
pub struct MarketState {
    /// Current trading symbol
    pub symbol: RwSignal<Symbol>,
    /// Current ticker data
    pub ticker: RwSignal<Option<Ticker>>,
    /// Order book snapshot
    pub orderbook: RwSignal<Option<OrderBookSnapshot>>,
    /// Market depth (derived from orderbook)
    pub depth: RwSignal<Option<MarketDepth>>,
    /// Recent trades (most recent first)
    pub trades: RwSignal<Vec<Trade>>,
    /// Candlestick history
    pub candles: RwSignal<CandleHistory>,
    /// Current candle interval
    pub interval: RwSignal<CandleInterval>,
    /// Last update timestamps
    pub last_update: LastUpdateSignals,
}

/// Signals tracking last update times for each data type
#[derive(Clone)]
pub struct LastUpdateSignals {
    pub ticker: RwSignal<i64>,
    pub orderbook: RwSignal<i64>,
    pub trade: RwSignal<i64>,
    pub candle: RwSignal<i64>,
}

impl LastUpdateSignals {
    fn new() -> Self {
        Self {
            ticker: RwSignal::new(0),
            orderbook: RwSignal::new(0),
            trade: RwSignal::new(0),
            candle: RwSignal::new(0),
        }
    }
}

impl MarketState {
    /// Create new market state
    pub fn new() -> Self {
        let symbol = Symbol::default();
        Self {
            symbol: RwSignal::new(symbol.clone()),
            ticker: RwSignal::new(None),
            orderbook: RwSignal::new(None),
            depth: RwSignal::new(None),
            trades: RwSignal::new(Vec::with_capacity(MAX_TRADES)),
            candles: RwSignal::new(CandleHistory::new(symbol, CandleInterval::M1)),
            interval: RwSignal::new(CandleInterval::M1),
            last_update: LastUpdateSignals::new(),
        }
    }

    // ========================================================================
    // Ticker Updates
    // ========================================================================

    /// Update ticker data
    pub fn update_ticker(&self, ticker: Ticker) {
        self.last_update.ticker.set(ticker.timestamp);
        self.ticker.set(Some(ticker));
    }

    /// Get current price (from ticker)
    pub fn current_price(&self) -> Option<f64> {
        self.ticker.get().map(|t| t.last_price.as_f64())
    }

    // ========================================================================
    // Order Book Updates
    // ========================================================================

    /// Update order book snapshot
    pub fn update_orderbook(&self, book: OrderBookSnapshot) {
        // Derive market depth from order book
        let depth = MarketDepth::from_orderbook(&book);
        self.last_update.orderbook.set(book.timestamp);
        self.depth.set(Some(depth));
        self.orderbook.set(Some(book));
    }

    /// Get current mid price (from orderbook)
    pub fn mid_price(&self) -> Option<f64> {
        self.orderbook.get().as_ref().and_then(|b| b.mid_price())
    }

    /// Get current spread (from orderbook)
    pub fn spread(&self) -> Option<f64> {
        self.orderbook.get().as_ref().and_then(|b| b.spread())
    }

    /// Get order book imbalance
    pub fn imbalance(&self) -> f64 {
        self.orderbook.get().map_or(0.0, |b| b.imbalance())
    }

    // ========================================================================
    // Trade Updates
    // ========================================================================

    /// Add single trade to history
    pub fn add_trade(&self, trade: Trade) {
        self.last_update.trade.set(trade.timestamp.timestamp_millis());
        self.trades.update(|trades| {
            trades.insert(0, trade);
            if trades.len() > MAX_TRADES {
                trades.pop();
            }
        });
    }

    /// Add batch of trades
    pub fn add_trades(&self, new_trades: Vec<Trade>) {
        if new_trades.is_empty() {
            return;
        }

        if let Some(first) = new_trades.first() {
            self.last_update.trade.set(first.timestamp.timestamp_millis());
        }

        self.trades.update(|trades| {
            for trade in new_trades {
                trades.insert(0, trade);
            }
            trades.truncate(MAX_TRADES);
        });
    }

    /// Get latest trade
    pub fn latest_trade(&self) -> Option<Trade> {
        self.trades.get().first().cloned()
    }

    /// Get recent N trades
    pub fn recent_trades(&self, n: usize) -> Vec<Trade> {
        self.trades.get().iter().take(n).cloned().collect()
    }

    // ========================================================================
    // Candle Updates
    // ========================================================================

    /// Update or add candle
    pub fn update_candle(&self, candle: Candle) {
        self.last_update.candle.set(candle.timestamp);
        self.candles.update(|history| {
            // Check if we should update existing candle or add new one
            if let Some(last) = history.candles.last_mut() {
                if last.timestamp == candle.timestamp && !last.is_closed {
                    // Update existing candle
                    *last = candle;
                    return;
                }
            }
            // Add new candle
            history.candles.push(candle);
            // Maintain max size
            if history.candles.len() > MAX_CANDLES {
                history.candles.remove(0);
            }
        });
    }

    /// Set full candle history (bulk load)
    pub fn set_candles(&self, candles: Vec<Candle>) {
        if let Some(last) = candles.last() {
            self.last_update.candle.set(last.timestamp);
        }

        let symbol = self.symbol.get();
        let interval = self.interval.get();

        self.candles.update(|history| {
            history.symbol = symbol;
            history.interval = interval;
            history.candles = candles;
        });
    }

    // ========================================================================
    // Symbol & Interval Changes
    // ========================================================================

    /// Change trading symbol (clears all data)
    pub fn set_symbol(&self, symbol: Symbol) {
        self.symbol.set(symbol.clone());
        // Clear all market data
        self.ticker.set(None);
        self.orderbook.set(None);
        self.depth.set(None);
        self.trades.set(Vec::new());
        self.candles.set(CandleHistory::new(symbol, self.interval.get()));
    }

    /// Change candle interval (clears candle history)
    pub fn set_interval(&self, interval: CandleInterval) {
        self.interval.set(interval);
        self.candles.set(CandleHistory::new(self.symbol.get(), interval));
    }

    // ========================================================================
    // Clear Methods
    // ========================================================================

    /// Clear all market data
    pub fn clear(&self) {
        let symbol = self.symbol.get();
        let interval = self.interval.get();

        self.ticker.set(None);
        self.orderbook.set(None);
        self.depth.set(None);
        self.trades.set(Vec::new());
        self.candles.set(CandleHistory::new(symbol, interval));
    }
}

impl Default for MarketState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// COMPUTED SIGNALS
// ============================================================================

/// Price direction indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PriceDirection {
    Up,
    Down,
    #[default]
    Unchanged,
}

impl PriceDirection {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Up => "price-up",
            Self::Down => "price-down",
            Self::Unchanged => "price-unchanged",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Up => dash_core::colors::BULL,
            Self::Down => dash_core::colors::BEAR,
            Self::Unchanged => dash_core::colors::NEUTRAL,
        }
    }

    pub fn arrow(&self) -> &'static str {
        match self {
            Self::Up => "▲",
            Self::Down => "▼",
            Self::Unchanged => "●",
        }
    }
}

/// Computed market signals (derived from raw data)
#[derive(Clone)]
pub struct MarketComputed {
    /// Current price direction (from ticker)
    pub price_direction: Memo<PriceDirection>,
    /// Order book imbalance (-1 to +1)
    pub imbalance: Memo<f64>,
    /// VWAP from recent trades
    pub vwap: Memo<f64>,
    /// Buy volume ratio (0 to 1)
    pub buy_ratio: Memo<f64>,
}

impl MarketComputed {
    /// Create computed signals from market state
    pub fn new(state: &MarketState) -> Self {
        let ticker_signal = state.ticker;
        let orderbook_signal = state.orderbook;
        let trades_signal = state.trades;

        Self {
            price_direction: Memo::new(move |_| {
                ticker_signal.get().map_or(PriceDirection::Unchanged, |t| {
                    if t.change_24h > 0.0 {
                        PriceDirection::Up
                    } else if t.change_24h < 0.0 {
                        PriceDirection::Down
                    } else {
                        PriceDirection::Unchanged
                    }
                })
            }),

            imbalance: Memo::new(move |_| {
                orderbook_signal.get().map_or(0.0, |b| b.imbalance())
            }),

            vwap: Memo::new(move |_| {
                let trades = trades_signal.get();
                if trades.is_empty() {
                    return 0.0;
                }

                let mut total_value = 0.0;
                let mut total_volume = 0.0;

                for trade in trades.iter().take(50) {
                    total_value += trade.value();
                    total_volume += trade.quantity.as_f64();
                }

                if total_volume == 0.0 {
                    0.0
                } else {
                    total_value / total_volume
                }
            }),

            buy_ratio: Memo::new(move |_| {
                let trades = trades_signal.get();
                if trades.is_empty() {
                    return 0.5;
                }

                let recent: Vec<_> = trades.iter().take(50).collect();
                let buy_count = recent.iter().filter(|t| t.side == TradeSide::Buy).count();

                buy_count as f64 / recent.len() as f64
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_direction() {
        assert_eq!(PriceDirection::Up.arrow(), "▲");
        assert_eq!(PriceDirection::Down.arrow(), "▼");
    }
}