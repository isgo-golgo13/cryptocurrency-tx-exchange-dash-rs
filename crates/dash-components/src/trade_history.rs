//! Trade history (tape) component

use dash_core::{colors, Trade, TradeSide, TradeClassification, ValueThresholdClassifier, TradeClassifier};
use dash_state::MarketState;
use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct TradeHistoryConfig {
    pub max_visible: usize,
    pub show_value: bool,
    pub highlight_whales: bool,
    pub compact: bool,
}

impl Default for TradeHistoryConfig {
    fn default() -> Self {
        Self {
            max_visible: 50,
            show_value: true,
            highlight_whales: true,
            compact: false,
        }
    }
}

impl TradeHistoryConfig {
    pub fn compact() -> Self {
        Self {
            max_visible: 20,
            show_value: false,
            highlight_whales: true,
            compact: true,
        }
    }
}

#[component]
pub fn TradeHistory(
    #[prop(into)] market: MarketState,
    #[prop(optional)] config: Option<TradeHistoryConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let max_visible = config.max_visible;
    let show_value = config.show_value;
    let highlight_whales = config.highlight_whales;
    let compact = config.compact;

    let trades = market.trades;
    let classifier = ValueThresholdClassifier::default();

    let visible_trades = move || {
        trades.get().into_iter().take(max_visible).collect::<Vec<_>>()
    };

    view! {
        <div class="trade-history">
            <div class="th-header">
                <span class="th-col time">"Time"</span>
                <span class="th-col side">"Side"</span>
                <span class="th-col price">"Price"</span>
                <span class="th-col size">"Size"</span>
                {if show_value {
                    Some(view! { <span class="th-col value">"Value"</span> })
                } else {
                    None
                }}
            </div>

            <div class="th-list">
                <For
                    each=visible_trades
                    key=|trade| trade.id.clone()
                    children=move |trade| {
                        let classification = if highlight_whales {
                            Some(classifier.classify(&trade))
                        } else {
                            None
                        };
                        view! {
                            <TradeRow
                                trade=trade
                                show_value=show_value
                                classification=classification
                                compact=compact
                            />
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn TradeRow(
    trade: Trade,
    show_value: bool,
    classification: Option<TradeClassification>,
    compact: bool,
) -> impl IntoView {
    let time_str = if compact { trade.time_short() } else { trade.time_str() };
    let price = trade.price.as_f64();
    let qty = trade.quantity.as_f64();
    let value = trade.value();

    let price_str = if price >= 1000.0 { format!("{:.2}", price) } else { format!("{:.4}", price) };
    let qty_str = format!("{:.4}", qty);
    let value_str = if value >= 1_000_000.0 {
        format!("{:.2}M", value / 1_000_000.0)
    } else if value >= 1_000.0 {
        format!("{:.2}K", value / 1_000.0)
    } else {
        format!("{:.2}", value)
    };

    let side_color = trade.side.color();
    let side_arrow = trade.side.arrow();

    let row_class = match classification {
        Some(TradeClassification::Whale) => "th-row whale",
        Some(TradeClassification::Large) => "th-row large",
        _ => "th-row",
    };

    view! {
        <div class=row_class>
            <span class="th-col time">{time_str}</span>
            <span class="th-col side" style=format!("color: {}", side_color)>{side_arrow}</span>
            <span class="th-col price" style=format!("color: {}", side_color)>{price_str}</span>
            <span class="th-col size">{qty_str}</span>
            {if show_value {
                Some(view! { <span class="th-col value">{value_str}</span> })
            } else {
                None
            }}
        </div>
    }
}