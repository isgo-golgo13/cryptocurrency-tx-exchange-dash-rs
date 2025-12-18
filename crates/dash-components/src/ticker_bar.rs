//! Ticker bar component for dashboard header

use dash_core::{colors, ConnectionState, Ticker};
use dash_state::MarketState;
use leptos::prelude::*;

#[derive(Debug, Clone)]
pub struct TickerBarConfig {
    pub show_volume: bool,
    pub show_high_low: bool,
    pub show_spread: bool,
    pub compact: bool,
}

impl Default for TickerBarConfig {
    fn default() -> Self {
        Self {
            show_volume: true,
            show_high_low: true,
            show_spread: true,
            compact: false,
        }
    }
}

#[component]
pub fn TickerBar(
    #[prop(into)] market: MarketState,
    #[prop(into)] connection: Signal<ConnectionState>,
    #[prop(optional)] config: Option<TickerBarConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let show_volume = config.show_volume;
    let show_high_low = config.show_high_low;
    let show_spread = config.show_spread;

    let ticker = market.ticker;
    let symbol = market.symbol;

    view! {
        <div class="ticker-bar">
            <div class="tb-symbol">
                <span class="symbol-name">{move || symbol.get().to_string()}</span>
                <ConnectionIndicator state=connection />
            </div>

            <div class="tb-price">
                {move || {
                    ticker.get().map(|t| {
                        let color = t.color();
                        let arrow = t.arrow();
                        view! {
                            <span class="price-value" style=format!("color: {}", color)>
                                {format!("{:.2}", t.last_price.as_f64())}
                            </span>
                            <span class="price-change" style=format!("color: {}", color)>
                                {arrow} " " {t.change_percent_str()}
                            </span>
                        }
                    })
                }}
            </div>

            <div class="tb-stats">
                {move || {
                    let t = ticker.get()?;
                    let color = t.color();
                    Some(view! {
                        <div class="tb-stat">
                            <span class="stat-label">"24h Change"</span>
                            <span class="stat-value" style=format!("color: {}", color)>{t.change_str()}</span>
                        </div>
                    })
                }}

                {move || {
                    if show_high_low {
                        ticker.get().map(|t| view! {
                            <div class="tb-stat">
                                <span class="stat-label">"24h High"</span>
                                <span class="stat-value" style=format!("color: {}", colors::BULL)>
                                    {format!("{:.2}", t.high_24h.as_f64())}
                                </span>
                            </div>
                            <div class="tb-stat">
                                <span class="stat-label">"24h Low"</span>
                                <span class="stat-value" style=format!("color: {}", colors::BEAR)>
                                    {format!("{:.2}", t.low_24h.as_f64())}
                                </span>
                            </div>
                        })
                    } else {
                        None
                    }
                }}

                {move || {
                    if show_volume {
                        ticker.get().map(|t| {
                            let vol = t.volume_24h.as_f64();
                            let vol_str = if vol >= 1_000_000.0 {
                                format!("{:.2}M", vol / 1_000_000.0)
                            } else if vol >= 1_000.0 {
                                format!("{:.2}K", vol / 1_000.0)
                            } else {
                                format!("{:.4}", vol)
                            };
                            view! {
                                <div class="tb-stat">
                                    <span class="stat-label">"24h Volume"</span>
                                    <span class="stat-value">{vol_str}</span>
                                </div>
                            }
                        })
                    } else {
                        None
                    }
                }}

                {move || {
                    if show_spread {
                        ticker.get().map(|t| view! {
                            <div class="tb-stat">
                                <span class="stat-label">"Spread"</span>
                                <span class="stat-value" style=format!("color: {}", colors::WARN)>
                                    {format!("{:.2} ({:.3}%)", t.spread(), t.spread_percent())}
                                </span>
                            </div>
                        })
                    } else {
                        None
                    }
                }}
            </div>
        </div>
    }
}

#[component]
pub fn ConnectionIndicator(
    #[prop(into)] state: Signal<ConnectionState>,
) -> impl IntoView {
    let indicator_style = move || {
        let s = state.get();
        let color = match s {
            ConnectionState::Connected => colors::BULL,
            ConnectionState::Connecting | ConnectionState::Reconnecting => colors::WARN,
            ConnectionState::Disconnected => colors::BEAR,
        };
        format!("background-color: {}", color)
    };

    view! {
        <div class="connection-indicator" title=move || state.get().label()>
            <span class="indicator-dot" style=indicator_style />
            <span class="indicator-label">{move || state.get().label()}</span>
        </div>
    }
}