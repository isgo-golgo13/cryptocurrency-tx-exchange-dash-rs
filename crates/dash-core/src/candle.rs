//! Candlestick (OHLCV) types for charting

use crate::{colors, Price, Quantity, Symbol};
use serde::{Deserialize, Serialize};

// ============================================================================
// STRATEGY PATTERN: Candle Pattern Detection
// ============================================================================

/// Strategy trait for detecting candlestick patterns
pub trait CandlePatternDetector: Send + Sync {
    fn detect(&self, candles: &[Candle]) -> Vec<CandlePattern>;
}

/// Detected candlestick pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandlePattern {
    Doji,
    Hammer,
    InvertedHammer,
    BullishEngulfing,
    BearishEngulfing,
    MorningStar,
    EveningStar,
    ThreeWhiteSoldiers,
    ThreeBlackCrows,
}

impl CandlePattern {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Doji => "Doji",
            Self::Hammer => "Hammer",
            Self::InvertedHammer => "Inverted Hammer",
            Self::BullishEngulfing => "Bullish Engulfing",
            Self::BearishEngulfing => "Bearish Engulfing",
            Self::MorningStar => "Morning Star",
            Self::EveningStar => "Evening Star",
            Self::ThreeWhiteSoldiers => "Three White Soldiers",
            Self::ThreeBlackCrows => "Three Black Crows",
        }
    }

    pub fn is_bullish(&self) -> bool {
        matches!(
            self,
            Self::Hammer | Self::BullishEngulfing | Self::MorningStar | Self::ThreeWhiteSoldiers
        )
    }

    pub fn is_bearish(&self) -> bool {
        matches!(
            self,
            Self::InvertedHammer | Self::BearishEngulfing | Self::EveningStar | Self::ThreeBlackCrows
        )
    }
}

/// Basic pattern detector (single candle patterns)
#[derive(Debug, Clone, Default)]
pub struct BasicPatternDetector {
    /// Body to range ratio threshold for doji
    pub doji_threshold: f64,
}

impl BasicPatternDetector {
    pub fn new() -> Self {
        Self {
            doji_threshold: 0.1,
        }
    }
}

impl CandlePatternDetector for BasicPatternDetector {
    fn detect(&self, candles: &[Candle]) -> Vec<CandlePattern> {
        let mut patterns = Vec::new();

        if let Some(candle) = candles.last() {
            // Doji detection
            let range = candle.range();
            let body = candle.body_size();

            if range > 0.0 && body / range < self.doji_threshold {
                patterns.push(CandlePattern::Doji);
            }

            // Hammer detection (small body at top, long lower shadow)
            if candle.lower_shadow() > body * 2.0 && candle.upper_shadow() < body * 0.5 {
                patterns.push(CandlePattern::Hammer);
            }

            // Inverted hammer (small body at bottom, long upper shadow)
            if candle.upper_shadow() > body * 2.0 && candle.lower_shadow() < body * 0.5 {
                patterns.push(CandlePattern::InvertedHammer);
            }
        }

        patterns
    }
}

// ============================================================================
// CORE TYPES
// ============================================================================

/// Time interval for candlesticks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CandleInterval {
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "30m")]
    M30,
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "4h")]
    H4,
    #[serde(rename = "1d")]
    D1,
    #[serde(rename = "1w")]
    W1,
}

impl CandleInterval {
    /// Duration in seconds
    pub fn as_secs(&self) -> i64 {
        match self {
            Self::M1 => 60,
            Self::M5 => 300,
            Self::M15 => 900,
            Self::M30 => 1800,
            Self::H1 => 3600,
            Self::H4 => 14400,
            Self::D1 => 86400,
            Self::W1 => 604800,
        }
    }

    /// Duration in milliseconds
    pub fn as_millis(&self) -> i64 {
        self.as_secs() * 1000
    }

    /// Display label
    pub fn label(&self) -> &'static str {
        match self {
            Self::M1 => "1m",
            Self::M5 => "5m",
            Self::M15 => "15m",
            Self::M30 => "30m",
            Self::H1 => "1H",
            Self::H4 => "4H",
            Self::D1 => "1D",
            Self::W1 => "1W",
        }
    }

    /// All intervals
    pub fn all() -> &'static [Self] {
        &[
            Self::M1, Self::M5, Self::M15, Self::M30,
            Self::H1, Self::H4, Self::D1, Self::W1,
        ]
    }
}

impl Default for CandleInterval {
    fn default() -> Self {
        Self::M1
    }
}

impl std::fmt::Display for CandleInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Single OHLCV candlestick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: Symbol,
    pub interval: CandleInterval,
    /// Unix timestamp in milliseconds (candle open time)
    pub timestamp: i64,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
    /// Quote volume (volume × price)
    pub quote_volume: f64,
    /// Number of trades in this candle
    pub trade_count: u32,
    /// Is this candle still forming?
    pub is_closed: bool,
}

impl Candle {
    /// Create new candle at given timestamp with opening price
    pub fn new(symbol: Symbol, interval: CandleInterval, timestamp: i64, open: f64) -> Self {
        Self {
            symbol,
            interval,
            timestamp,
            open: Price::new(open),
            high: Price::new(open),
            low: Price::new(open),
            close: Price::new(open),
            volume: Quantity::ZERO,
            quote_volume: 0.0,
            trade_count: 0,
            is_closed: false,
        }
    }

    /// Update candle with new trade
    pub fn update(&mut self, price: f64, quantity: f64) {
        if price > self.high.as_f64() {
            self.high = Price::new(price);
        }
        if price < self.low.as_f64() {
            self.low = Price::new(price);
        }
        self.close = Price::new(price);
        self.volume = Quantity::new(self.volume.as_f64() + quantity);
        self.quote_volume += price * quantity;
        self.trade_count += 1;
    }

    /// Close the candle
    pub fn close_candle(&mut self) {
        self.is_closed = true;
    }

    /// Is this a bullish (green) candle?
    pub fn is_bullish(&self) -> bool {
        self.close.as_f64() >= self.open.as_f64()
    }

    /// Is this a bearish (red) candle?
    pub fn is_bearish(&self) -> bool {
        self.close.as_f64() < self.open.as_f64()
    }

    /// Candle body size (absolute)
    pub fn body_size(&self) -> f64 {
        (self.close.as_f64() - self.open.as_f64()).abs()
    }

    /// Candle range (high - low)
    pub fn range(&self) -> f64 {
        self.high.as_f64() - self.low.as_f64()
    }

    /// Upper shadow (wick) size
    pub fn upper_shadow(&self) -> f64 {
        let body_top = self.open.as_f64().max(self.close.as_f64());
        self.high.as_f64() - body_top
    }

    /// Lower shadow (wick) size
    pub fn lower_shadow(&self) -> f64 {
        let body_bottom = self.open.as_f64().min(self.close.as_f64());
        body_bottom - self.low.as_f64()
    }

    /// Price change (close - open)
    pub fn change(&self) -> f64 {
        self.close.as_f64() - self.open.as_f64()
    }

    /// Price change percentage
    pub fn change_percent(&self) -> f64 {
        if self.open.as_f64() == 0.0 {
            0.0
        } else {
            self.change() / self.open.as_f64() * 100.0
        }
    }

    /// Body top price
    pub fn body_top(&self) -> f64 {
        self.open.as_f64().max(self.close.as_f64())
    }

    /// Body bottom price
    pub fn body_bottom(&self) -> f64 {
        self.open.as_f64().min(self.close.as_f64())
    }

    /// Fill color for rendering
    pub fn fill_color(&self) -> &'static str {
        if self.is_bullish() {
            colors::BULL
        } else {
            colors::BEAR
        }
    }

    /// CSS class
    pub fn css_class(&self) -> &'static str {
        if self.is_bullish() {
            "candle-bullish"
        } else {
            "candle-bearish"
        }
    }

    /// Detect patterns using given strategy
    pub fn detect_patterns_with<D: CandlePatternDetector>(&self, detector: &D) -> Vec<CandlePattern> {
        detector.detect(&[self.clone()])
    }
}

/// Collection of candles for charting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleHistory {
    pub symbol: Symbol,
    pub interval: CandleInterval,
    pub candles: Vec<Candle>,
}

impl CandleHistory {
    pub fn new(symbol: Symbol, interval: CandleInterval) -> Self {
        Self {
            symbol,
            interval,
            candles: Vec::new(),
        }
    }

    pub fn with_capacity(symbol: Symbol, interval: CandleInterval, capacity: usize) -> Self {
        Self {
            symbol,
            interval,
            candles: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, candle: Candle) {
        self.candles.push(candle);
    }

    pub fn len(&self) -> usize {
        self.candles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// Get the most recent candle
    pub fn latest(&self) -> Option<&Candle> {
        self.candles.last()
    }

    /// Get mutable reference to latest candle
    pub fn latest_mut(&mut self) -> Option<&mut Candle> {
        self.candles.last_mut()
    }

    /// Get the most recent N candles
    pub fn tail(&self, n: usize) -> &[Candle] {
        let start = self.candles.len().saturating_sub(n);
        &self.candles[start..]
    }

    /// Price range across all candles (min low, max high)
    pub fn price_range(&self) -> Option<(f64, f64)> {
        if self.candles.is_empty() {
            return None;
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for candle in &self.candles {
            min = min.min(candle.low.as_f64());
            max = max.max(candle.high.as_f64());
        }

        Some((min, max))
    }

    /// Volume range across all candles
    pub fn volume_range(&self) -> Option<(f64, f64)> {
        if self.candles.is_empty() {
            return None;
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for candle in &self.candles {
            let vol = candle.volume.as_f64();
            min = min.min(vol);
            max = max.max(vol);
        }

        Some((min, max))
    }

    /// Time range (first timestamp, last timestamp)
    pub fn time_range(&self) -> Option<(i64, i64)> {
        match (self.candles.first(), self.candles.last()) {
            (Some(first), Some(last)) => Some((first.timestamp, last.timestamp)),
            _ => None,
        }
    }

    /// Detect patterns across history
    pub fn detect_patterns_with<D: CandlePatternDetector>(&self, detector: &D) -> Vec<CandlePattern> {
        detector.detect(&self.candles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_update() {
        let mut candle = Candle::new(Symbol::default(), CandleInterval::M1, 1700000000000, 50000.0);

        candle.update(50100.0, 0.5);
        candle.update(49900.0, 0.3);
        candle.update(50050.0, 0.2);

        assert_eq!(candle.high.as_f64(), 50100.0);
        assert_eq!(candle.low.as_f64(), 49900.0);
        assert_eq!(candle.close.as_f64(), 50050.0);
        assert_eq!(candle.volume.as_f64(), 1.0);
        assert_eq!(candle.trade_count, 3);
    }

    #[test]
    fn test_candle_bullish_bearish() {
        let mut bullish = Candle::new(Symbol::default(), CandleInterval::M1, 0, 100.0);
        bullish.close = Price::new(110.0);
        assert!(bullish.is_bullish());

        let mut bearish = Candle::new(Symbol::default(), CandleInterval::M1, 0, 100.0);
        bearish.close = Price::new(90.0);
        assert!(bearish.is_bearish());
    }

    #[test]
    fn test_doji_detection() {
        let detector = BasicPatternDetector::new();

        let mut doji = Candle::new(Symbol::default(), CandleInterval::M1, 0, 100.0);
        doji.high = Price::new(105.0);
        doji.low = Price::new(95.0);
        doji.close = Price::new(100.5); // Tiny body

        let patterns = detector.detect(&[doji]);
        assert!(patterns.contains(&CandlePattern::Doji));
    }
}