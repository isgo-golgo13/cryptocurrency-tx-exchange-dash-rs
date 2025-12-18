//! Mock data engine for development
//!
//! Generates realistic-looking market data for testing the dashboard.

use std::time::Duration;

use chrono::Utc;
use rand::Rng;
use tokio::sync::broadcast;
use tokio::time::interval;

use dash_core::{
    Candle, CandleInterval, MarketDepth, OrderBookLevel, OrderBookSnapshot,
    Price, Quantity, Symbol, Ticker, Trade, TradeSide, WsMessage,
};

/// Mock market state
struct MockMarket {
    symbol: Symbol,
    price: f64,
    volatility: f64,
    trend: f64,
    sequence: u64,
    candle_open_time: i64,
    current_candle: Option<Candle>,
}

impl MockMarket {
    fn new(symbol: Symbol, initial_price: f64) -> Self {
        Self {
            symbol,
            price: initial_price,
            volatility: 0.0005,
            trend: 0.0,
            sequence: 0,
            candle_open_time: 0,
            current_candle: None,
        }
    }

    /// Generate next price tick
    fn tick(&mut self) -> f64 {
        let mut rng = rand::thread_rng();

        // Random walk with mean reversion
        let drift = self.trend * 0.0001;
        let random = (rng.r#gen::<f64>() - 0.5) * 2.0 * self.volatility;

        // Occasionally shift trend
        if rng.r#gen::<f64>() < 0.01 {
            self.trend = (rng.r#gen::<f64>() - 0.5) * 2.0;
        }

        self.price *= 1.0 + drift + random;
        self.price = self.price.max(1000.0); // Floor price
        self.price
    }

    /// Generate a random trade
    fn generate_trade(&mut self) -> Trade {
        let mut rng = rand::thread_rng();

        let price = self.tick();
        let side = if rng.r#gen::<bool>() { TradeSide::Buy } else { TradeSide::Sell };

        // Log-normal quantity distribution (lots of small trades, few large)
        let base_qty = rng.r#gen::<f64>().exp() * 0.1;
        let quantity = base_qty.min(10.0);

        Trade::new(self.symbol.clone(), price, quantity, side)
    }

    /// Generate order book snapshot
    fn generate_orderbook(&mut self) -> OrderBookSnapshot {
        let mut rng = rand::thread_rng();
        self.sequence += 1;

        let mid = self.price;
        let spread = mid * 0.0002; // 0.02% spread

        let mut bids = Vec::with_capacity(20);
        let mut asks = Vec::with_capacity(20);

        // Generate bid levels (descending prices)
        let mut bid_price = mid - spread / 2.0;
        for _ in 0..20 {
            let qty = rng.r#gen::<f64>() * 2.0 + 0.1;
            let orders = rng.gen_range(1..10);
            bids.push(OrderBookLevel::new(bid_price, qty, orders));
            bid_price -= rng.r#gen::<f64>() * 5.0 + 1.0;
        }

        // Generate ask levels (ascending prices)
        let mut ask_price = mid + spread / 2.0;
        for _ in 0..20 {
            let qty = rng.r#gen::<f64>() * 2.0 + 0.1;
            let orders = rng.gen_range(1..10);
            asks.push(OrderBookLevel::new(ask_price, qty, orders));
            ask_price += rng.r#gen::<f64>() * 5.0 + 1.0;
        }

        OrderBookSnapshot {
            symbol: self.symbol.clone(),
            bids,
            asks,
            timestamp: Utc::now().timestamp_millis(),
            sequence: self.sequence,
        }
    }

    /// Generate ticker
    fn generate_ticker(&self) -> Ticker {
        let mut rng = rand::thread_rng();

        let open = self.price * (1.0 - rng.r#gen::<f64>() * 0.02);
        let high = self.price * (1.0 + rng.r#gen::<f64>() * 0.03);
        let low = self.price * (1.0 - rng.r#gen::<f64>() * 0.03);

        let change = self.price - open;
        let change_pct = change / open * 100.0;

        Ticker {
            symbol: self.symbol.clone(),
            last_price: Price::new(self.price),
            bid_price: Price::new(self.price * 0.9999),
            bid_qty: Quantity::new(rng.r#gen::<f64>() * 5.0),
            ask_price: Price::new(self.price * 1.0001),
            ask_qty: Quantity::new(rng.r#gen::<f64>() * 5.0),
            high_24h: Price::new(high),
            low_24h: Price::new(low),
            volume_24h: Quantity::new(rng.r#gen::<f64>() * 10000.0 + 1000.0),
            quote_volume_24h: rng.r#gen::<f64>() * 500_000_000.0,
            change_24h: change,
            change_percent_24h: change_pct,
            open_24h: Price::new(open),
            trade_count_24h: rng.gen_range(10000..100000),
            timestamp: Utc::now().timestamp_millis(),
        }
    }

    /// Update or create candle
    fn update_candle(&mut self, trade: &Trade) -> Option<Candle> {
        let now = Utc::now().timestamp_millis();
        let interval_ms = CandleInterval::M1.as_millis();
        let candle_time = (now / interval_ms) * interval_ms;

        let price = trade.price.as_f64();
        let qty = trade.quantity.as_f64();

        if self.candle_open_time != candle_time {
            // Close previous candle and start new one
            let prev = self.current_candle.take().map(|mut c| {
                c.close_candle();
                c
            });

            self.candle_open_time = candle_time;
            self.current_candle = Some(Candle::new(
                self.symbol.clone(),
                CandleInterval::M1,
                candle_time,
                price,
            ));

            prev
        } else {
            // Update current candle
            if let Some(ref mut candle) = self.current_candle {
                candle.update(price, qty);
            }
            None
        }
    }
}

/// Run the mock data engine
pub async fn run_mock_engine(tx: broadcast::Sender<WsMessage>) {