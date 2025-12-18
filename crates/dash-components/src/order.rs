//! Order book ladder display component

use dash_core::{colors, OrderBookLevel, OrderSide};
use dash_state::MarketState;
use leptos::prelude::*;

/// Order book configuration
#[derive(Debug, Clone)]
pub struct OrderBookConfig {
    pub depth: usize,
    pub show_spread: bool,
    pub show_totals: bool,
    pub compact: bool,
}

impl Default for OrderBookConfig {
    fn default() -> Self {
        Self {
            depth: 15,
            show_spread: true,
            show_totals: true,
            compact: false,
        }
    }
}

impl OrderBookConfig {
    pub fn compact() -> Self {
        Self {
            depth: 8,
            show_spread: true,
            show_totals: false,
            compact: true,
        }
    }
}

/// Main order book component
#[component]
pub fn OrderBook(
    #[prop(into)] market: MarketState,
    #[prop(optional)] config: Option<OrderBookConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let depth = config.depth;
    let show_spread = config.show_spread;
    let show_totals = config.show_totals;

    let orderbook = market.orderbook;

    let max_qty = move || {
        orderbook.get().map_or(1.0, |book| book.max_quantity().max(0.001))
    };

    let asks = move || {
        orderbook.get().map_or(vec![], |book| {
            let mut a: Vec<_> = book.asks.iter().take(depth).cloned().collect();
            a.reverse();
            a
        })
    };

    let bids = move || {
        orderbook.get().map_or(vec![], |book| {
            book.bids.iter().take(depth).cloned().collect()
        })
    };

    let spread_info = move || {
        orderbook.get().and_then(|book| {
            book.spread().zip(book.spread_percent()).map(|(s, pct)| {
                (format!("{:.2}", s), format!("{:.3}%", pct))
            })
        })
    };

    let totals = move || {
        orderbook.get().map(|book| {
            (book.total_bid_depth(), book.total_ask_depth())
        })
    };

    view! {
        <div class="orderbook">
            <div class="ob-header">
                <span class="ob-col price">"Price"</span>
                <span class="ob-col size">"Size"</span>
                <span class="ob-col total">"Total"</span>
            </div>

            <div class="ob-asks">
                <For
                    each=asks
                    key=|level| format!("{:.8}", level.price.as_f64())
                    children=move |level| {
                        let mq = max_qty();
                        view! { <OrderBookRow level=level side=OrderSide::Ask max_qty=mq /> }
                    }
                />
            </div>

            {move || {
                if show_spread {
                    spread_info().map(|(spread, pct)| {
                        view! {
                            <div class="ob-spread">
                                <span class="spread-label">"Spread"</span>
                                <span class="spread-value">{spread}</span>
                                <span class="spread-pct">{pct}</span>
                            </div>
                        }
                    })
                } else {
                    None
                }
            }}

            <div class="ob-bids">
                <For
                    each=bids
                    key=|level| format!("{:.8}", level.price.as_f64())
                    children=move |level| {
                        let mq = max_qty();
                        view! { <OrderBookRow level=level side=OrderSide::Bid max_qty=mq /> }
                    }
                />
            </div>

            {move || {
                if show_totals {
                    totals().map(|(bid_total, ask_total)| {
                        view! {
                            <div class="ob-totals">
                                <div class="total-bid">
                                    <span class="label">"Bid Total:"</span>
                                    <span class="value" style=format!("color: {}", colors::BULL)>
                                        {format!("{:.4}", bid_total)}
                                    </span>
                                </div>
                                <div class="total-ask">
                                    <span class="label">"Ask Total:"</span>
                                    <span class="value" style=format!("color: {}", colors::BEAR)>
                                        {format!("{:.4}", ask_total)}
                                    </span>
                                </div>
                            </div>
                        }
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

#[component]
fn OrderBookRow(
    level: OrderBookLevel,
    side: OrderSide,
    max_qty: f64,
) -> impl IntoView {
    let price = level.price.as_f64();
    let qty = level.quantity.as_f64();
    let bar_pct = (qty / max_qty * 100.0).min(100.0);

    let price_str = if price >= 1000.0 {
        format!("{:.2}", price)
    } else {
        format!("{:.4}", price)
    };

    let qty_str = format!("{:.4}", qty);
    let value = price * qty;
    let value_str = format!("{:.2}", value);

    let (bar_color, text_color) = match side {
        OrderSide::Bid => (colors::bull_alpha(0.2), colors::BULL),
        OrderSide::Ask => (colors::bear_alpha(0.2), colors::BEAR),
    };

    let bg_style = format!(
        "background: linear-gradient(to {}, {} {}%, transparent {}%)",
        if side == OrderSide::Bid { "left" } else { "right" },
        bar_color, bar_pct, bar_pct
    );

    view! {
        <div class="ob-row" style=bg_style>
            <span class="ob-col price" style=format!("color: {}", text_color)>{price_str}</span>
            <span class="ob-col size">{qty_str}</span>
            <span class="ob-col total">{value_str}</span>
        </div>
    }
}