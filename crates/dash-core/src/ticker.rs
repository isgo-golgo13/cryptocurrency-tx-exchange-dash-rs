//! Real-time ticker data types

use crate::{colors, Price, Quantity, Symbol};
use serde::{Deserialize, Serialize};

/// Real-time market ticker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: Symbol,
    /// Last traded price
    pub last_price: Price,
    /// Best bid price
    pub bid_price: Price,
    /// Best bid quantity
    pub bid_qty: Quantity,
    /// Best ask price
    pub ask_price: Price,
    /// Best ask quantity
    pub ask_qty: Quantity,
    /// 24h high
    pub high_24h: Price,
    /// 24h low
    pub low_24h: Price,
    /// 24h volume in base currency
    pub volume_24h: Quantity,
    /// 24h volume in quote currency
    pub quote_volume_24h: f64,
    /// 24h price change
    pub change_24h: f64,
    /// 24h price change percentage
    pub change_percent_24h: f64,
    /// Opening price (24h ago)
    pub open_24h: Price,
    /// Number of trades in 24h
    pub trade_count_24h: u64,
    /// Timestamp in milliseconds
    pub timestamp: i64,
}

impl Ticker {
    /// Create new ticker with given price
    pub fn new(symbol: Symbol, price: f64) -> Self {
        let ts = chrono::Utc::now().timestamp_millis();
        Self {
            symbol,
            last_price: Price::new(price),
            bid_price: Price::new(price * 0.9999),
            bid_qty: Quantity::new(1.0),
            ask_price: Price::new(price * 1.0001),
            ask_qty: Quantity::new(1.0),
            high_24h: Price::new(price * 1.05),
            low_24h: Price::new(price * 0.95),
            volume_24h: Quantity::new(1000.0),
            quote_volume_24h: 1000.0 * price,
            change_24h: 0.0,
            change_percent_24h: 0.0,
            open_24h: Price::new(price),
            trade_count_24h: 0,
            timestamp: ts,
        }
    }

    /// Current spread (ask - bid)
    pub fn spread(&self) -> f64 {
        self.ask_price.as_f64() - self.bid_price.as_f64()
    }

    /// Spread as percentage of mid price
    pub fn spread_percent(&self) -> f64 {
        let mid = self.mid_price();
        if mid == 0.0 {
            0.0
        } else {
            self.spread() / mid * 100.0
        }
    }

    /// Mid price (average of bid and ask)
    pub fn mid_price(&self) -> f64 {
        (self.bid_price.as_f64() + self.ask_price.as_f64()) / 2.0
    }

    /// Is price up in last 24h?
    pub fn is_up(&self) -> bool {
        self.change_24h >= 0.0
    }

    /// Is price down in last 24h?
    pub fn is_down(&self) -> bool {
        self.change_24h < 0.0
    }

    /// Direction color
    pub fn color(&self) -> &'static str {
        if self.is_up() {
            colors::BULL
        } else {
            colors::BEAR
        }
    }

    /// CSS class
    pub fn css_class(&self) -> &'static str {
        if self.is_up() {
            "ticker-up"
        } else {
            "ticker-down"
        }
    }

    /// Direction arrow
    pub fn arrow(&self) -> &'static str {
        if self.is_up() {
            "▲"
        } else {
            "▼"
        }
    }

    /// Format price change with sign
    pub fn change_str(&self) -> String {
        let sign = if self.change_24h >= 0.0 { "+" } else { "" };
        format!("{}{:.2}", sign, self.change_24h)
    }

    /// Format percentage change with sign
    pub fn change_percent_str(&self) -> String {
        let sign = if self.change_percent_24h >= 0.0 { "+" } else { "" };
        format!("{}{:.2}%", sign, self.change_percent_24h)
    }

    /// 24h range percentage (position of current price within range)
    /// Returns 0.0 at low, 1.0 at high
    pub fn range_position(&self) -> f64 {
        let range = self.high_24h.as_f64() - self.low_24h.as_f64();
        if range == 0.0 {
            0.5
        } else {
            (self.last_price.as_f64() - self.low_24h.as_f64()) / range
        }
    }

    /// VWAP approximation from 24h data
    pub fn vwap_24h(&self) -> f64 {
        if self.volume_24h.as_f64() == 0.0 {
            self.last_price.as_f64()
        } else {
            self.quote_volume_24h / self.volume_24h.as_f64()
        }
    }

    /// Update from new trade
    pub fn update_from_trade(&mut self, price: f64, quantity: f64) {
        self.last_price = Price::new(price);
        self.volume_24h = Quantity::new(self.volume_24h.as_f64() + quantity);
        self.quote_volume_24h += price * quantity;
        self.trade_count_24h += 1;

        // Update high/low
        if price > self.high_24h.as_f64() {
            self.high_24h = Price::new(price);
        }
        if price < self.low_24h.as_f64() {
            self.low_24h = Price::new(price);
        }

        // Recalculate change
        self.change_24h = price - self.open_24h.as_f64();
        if self.open_24h.as_f64() > 0.0 {
            self.change_percent_24h = self.change_24h / self.open_24h.as_f64() * 100.0;
        }

        self.timestamp = chrono::Utc::now().timestamp_millis();
    }
}

/// Mini ticker for compact display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniTicker {
    pub symbol: Symbol,
    pub last_price: f64,
    pub change_percent_24h: f64,
}

impl MiniTicker {
    pub fn is_up(&self) -> bool {
        self.change_percent_24h >= 0.0
    }

    pub fn color(&self) -> &'static str {
        if self.is_up() {
            colors::BULL
        } else {
            colors::BEAR
        }
    }
}

impl From<&Ticker> for MiniTicker {
    fn from(t: &Ticker) -> Self {
        Self {
            symbol: t.symbol.clone(),
            last_price: t.last_price.as_f64(),
            change_percent_24h: t.change_percent_24h,
        }
    }
}

impl From<Ticker> for MiniTicker {
    fn from(t: Ticker) -> Self {
        Self::from(&t)
    }
}

/// Ticker statistics for dashboard header
#[derive(Debug, Clone, Default)]
pub struct TickerStats {
    pub last_price: f64,
    pub change_24h: f64,
    pub change_percent_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub volume_24h: f64,
    pub quote_volume_24h: f64,
}

impl From<&Ticker> for TickerStats {
    fn from(t: &Ticker) -> Self {
        Self {
            last_price: t.last_price.as_f64(),
            change_24h: t.change_24h,
            change_percent_24h: t.change_percent_24h,
            high_24h: t.high_24h.as_f64(),
            low_24h: t.low_24h.as_f64(),
            volume_24h: t.volume_24h.as_f64(),
            quote_volume_24h: t.quote_volume_24h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_spread() {
        let mut ticker = Ticker::new(Symbol::new("BTC-USD"), 50000.0);
        ticker.bid_price = Price::new(49990.0);
        ticker.ask_price = Price::new(50010.0);

        assert_eq!(ticker.spread(), 20.0);
        assert!((ticker.mid_price() - 50000.0).abs() < 0.01);
    }

    #[test]
    fn test_range_position() {
        let mut ticker = Ticker::new(Symbol::new("BTC-USD"), 50000.0);
        ticker.high_24h = Price::new(52000.0);
        ticker.low_24h = Price::new(48000.0);
        ticker.last_price = Price::new(50000.0);

        // 50000 is exactly in the middle of 48000-52000
        assert!((ticker.range_position() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_update_from_trade() {
        let mut ticker = Ticker::new(Symbol::new("BTC-USD"), 50000.0);
        ticker.open_24h = Price::new(50000.0);

        ticker.update_from_trade(51000.0, 1.0);

        assert_eq!(ticker.last_price.as_f64(), 51000.0);
        assert_eq!(ticker.change_24h, 1000.0);
        assert!((ticker.change_percent_24h - 2.0).abs() < 0.01);
    }
}