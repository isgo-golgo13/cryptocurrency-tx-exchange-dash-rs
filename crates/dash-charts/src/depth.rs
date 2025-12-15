//! Market depth chart (bid/ask visualization)
//!
//! Renders cumulative order book depth as filled area chart.

use crate::{
    chartkit::{area_path, format_large_number, format_price, LinearScale, Scale},
    colors, ChartDimensions, ChartMargin,
};
use dash_core::MarketDepth;
use leptos::prelude::*;

/// Depth chart configuration
#[derive(Debug, Clone)]
pub struct DepthChartConfig {
    pub width: f64,
    pub height: f64,
    pub spread_multiplier: f64, // How much of the spread to show (e.g., 20x)
    pub show_mid_line: bool,
    pub show_legend: bool,
}

impl Default for DepthChartConfig {
    fn default() -> Self {
        Self {
            width: 600.0,
            height: 300.0,
            spread_multiplier: 20.0,
            show_mid_line: true,
            show_legend: true,
        }
    }
}

/// Internal depth chart state
#[derive(Clone)]
struct DepthState {
    bid_path: String,
    ask_path: String,
    mid_x: Option<f64>,
    mid_price: Option<f64>,
    x_scale: LinearScale,
    y_scale: LinearScale,
}

/// Depth chart component
#[component]
pub fn DepthChart(
    #[prop(into)] depth: Signal<Option<MarketDepth>>,
    #[prop(optional)] config: Option<DepthChartConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    
    let dims = ChartDimensions::new(config.width, config.height)
        .with_margin(ChartMargin::new(20.0, 70.0, 30.0, 70.0));

    let show_mid = config.show_mid_line;
    let show_legend = config.show_legend;
    let spread_mult = config.spread_multiplier;

    // Compute chart state
    let chart_state = move || {
        depth.get().map(|d| {
            // Get best bid/ask for centering
            let bid_first = d.bid_depth.first().map(|p| p.price);
            let ask_first = d.ask_depth.first().map(|p| p.price);

            // Calculate price range centered on mid price
            let (min_price, max_price) = match (bid_first, ask_first) {
                (Some(bid), Some(ask)) => {
                    let mid = (bid + ask) / 2.0;
                    let spread = ask - bid;
                    let range = spread * spread_mult;
                    (mid - range / 2.0, mid + range / 2.0)
                }
                _ => d.price_range().unwrap_or((0.0, 100.0)),
            };

            let x_scale = LinearScale::new()
                .domain(min_price, max_price)
                .range(0.0, dims.inner_width());

            let max_depth = d.max_depth();
            let y_scale = LinearScale::new()
                .domain(0.0, max_depth * 1.1)
                .range(dims.inner_height(), 0.0);

            // Build bid area points
            let bid_points: Vec<(f64, f64)> = d.bid_depth
                .iter()
                .filter(|p| p.price >= min_price && p.price <= max_price)
                .map(|p| (x_scale.scale(p.price), y_scale.scale(p.cumulative_quantity)))
                .collect();

            // Build ask area points
            let ask_points: Vec<(f64, f64)> = d.ask_depth
                .iter()
                .filter(|p| p.price >= min_price && p.price <= max_price)
                .map(|p| (x_scale.scale(p.price), y_scale.scale(p.cumulative_quantity)))
                .collect();

            // Generate area paths
            let baseline = dims.inner_height();
            let bid_path = area_path(&bid_points, baseline);
            let ask_path = area_path(&ask_points, baseline);

            // Mid price
            let mid_price = bid_first.zip(ask_first).map(|(b, a)| (b + a) / 2.0);
            let mid_x = mid_price.map(|p| x_scale.scale(p));

            DepthState {
                bid_path,
                ask_path,
                mid_x,
                mid_price,
                x_scale,
                y_scale,
            }
        })
    };

    view! {
        <svg
            class="depth-chart"
            viewBox=dims.viewbox()
            preserveAspectRatio="xMidYMid meet"
            style="width: 100%; height: 100%;"
        >
            // Background
            <rect
                width=dims.width
                height=dims.height
                fill=colors::BG_PANEL
                rx="4"
            />

            <g transform=dims.inner_transform()>
                // Grid
                <DepthGrid
                    width=dims.inner_width()
                    height=dims.inner_height()
                />

                // Depth areas
                {move || {
                    chart_state().map(|state| {
                        view! {
                            <>
                                // Bid area (green)
                                <path
                                    d=state.bid_path.clone()
                                    fill=colors::bull_alpha(0.3)
                                    stroke=colors::BULL
                                    stroke-width="2"
                                />

                                // Ask area (red)
                                <path
                                    d=state.ask_path.clone()
                                    fill=colors::bear_alpha(0.3)
                                    stroke=colors::BEAR
                                    stroke-width="2"
                                />

                                // Mid price line
                                {if show_mid {
                                    state.mid_x.map(|x| {
                                        view! {
                                            <line
                                                x1=x y1="0"
                                                x2=x y2=dims.inner_height()
                                                stroke=colors::WARN
                                                stroke-width="1"
                                                stroke-dasharray="4,4"
                                            />
                                        }
                                    })
                                } else {
                                    None
                                }}

                                // Mid price label
                                {state.mid_price.zip(state.mid_x).map(|(price, x)| {
                                    view! {
                                        <text
                                            x=x
                                            y="-5"
                                            text-anchor="middle"
                                            fill=colors::WARN
                                            font-size="11"
                                            font-family="JetBrains Mono, monospace"
                                        >
                                            {format_price(price, 2)}
                                        </text>
                                    }
                                })}
                            </>
                        }
                    })
                }}

                // X-Axis (price)
                <g transform=format!("translate(0, {})", dims.inner_height())>
                    <line
                        x1="0" y1="0"
                        x2=dims.inner_width() y2="0"
                        stroke=colors::BORDER
                        stroke-width="1"
                    />
                    {move || {
                        chart_state().map(|state| {
                            let ticks = state.x_scale.nice_ticks(5);
                            ticks.into_iter().map(|tick| {
                                let x = state.x_scale.scale(tick);
                                let label = format_price(tick, 0);

                                view! {
                                    <g transform=format!("translate({}, 0)", x)>
                                        <line y1="0" y2="5" stroke=colors::BORDER />
                                        <text
                                            y="15"
                                            text-anchor="middle"
                                            fill=colors::TEXT_MUTED
                                            font-size="9"
                                            font-family="JetBrains Mono, monospace"
                                        >
                                            {label}
                                        </text>
                                    </g>
                                }
                            }).collect_view()
                        })
                    }}
                </g>

                // Y-Axis (quantity)
                <g>
                    <line
                        x1="0" y1="0"
                        x2="0" y2=dims.inner_height()
                        stroke=colors::BORDER
                        stroke-width="1"
                    />
                    {move || {
                        chart_state().map(|state| {
                            let ticks = state.y_scale.nice_ticks(5);
                            ticks.into_iter().map(|tick| {
                                let y = state.y_scale.scale(tick);
                                let label = format_large_number(tick);

                                view! {
                                    <g transform=format!("translate(0, {})", y)>
                                        <line x1="-5" x2="0" stroke=colors::BORDER />
                                        <text
                                            x="-8"
                                            dy="0.32em"
                                            text-anchor="end"
                                            fill=colors::TEXT_MUTED
                                            font-size="9"
                                            font-family="JetBrains Mono, monospace"
                                        >
                                            {label}
                                        </text>
                                    </g>
                                }
                            }).collect_view()
                        })
                    }}
                </g>
            </g>

            // Legend
            {if show_legend {
                Some(view! {
                    <g transform=format!("translate({}, 15)", dims.width - 100.0)>
                        <rect x="0" y="-4" width="12" height="12" fill=colors::bull_alpha(0.5) />
                        <text x="16" y="5" fill=colors::TEXT_MUTED font-size="10">"Bids"</text>

                        <rect x="50" y="-4" width="12" height="12" fill=colors::bear_alpha(0.5) />
                        <text x="66" y="5" fill=colors::TEXT_MUTED font-size="10">"Asks"</text>
                    </g>
                })
            } else {
                None
            }}
        </svg>
    }
}

/// Grid lines for depth chart
#[component]
fn DepthGrid(width: f64, height: f64) -> impl IntoView {
    let h_lines: Vec<f64> = (0..=4).map(|i| i as f64 * height / 4.0).collect();
    let v_lines: Vec<f64> = (0..=4).map(|i| i as f64 * width / 4.0).collect();

    view! {
        <g class="depth-grid">
            {h_lines.into_iter().map(|y| {
                view! {
                    <line
                        x1="0" y1=y
                        x2=width y2=y
                        stroke=colors::GRID
                        stroke-width="1"
                        opacity="0.5"
                    />
                }
            }).collect_view()}

            {v_lines.into_iter().map(|x| {
                view! {
                    <line
                        x1=x y1="0"
                        x2=x y2=height
                        stroke=colors::GRID
                        stroke-width="1"
                        opacity="0.5"
                    />
                }
            }).collect_view()}
        </g>
    }
}

/// Compact depth bar (horizontal bid/ask imbalance indicator)
#[component]
pub fn DepthBar(
    #[prop(into)] bid_depth: Signal<f64>,
    #[prop(into)] ask_depth: Signal<f64>,
    #[prop(default = 200.0)] width: f64,
    #[prop(default = 8.0)] height: f64,
) -> impl IntoView {
    let bar_data = move || {
        let bids = bid_depth.get();
        let asks = ask_depth.get();
        let total = bids + asks;

        if total == 0.0 {
            (0.5, 0.5)
        } else {
            (bids / total, asks / total)
        }
    };

    view! {
        <svg
            class="depth-bar"
            viewBox=format!("0 0 {} {}", width, height)
            style="width: 100%; height: 100%;"
        >
            // Background
            <rect
                width=width
                height=height
                fill=colors::BG_ELEVATED
                rx="4"
            />

            // Bid side (green, from center left)
            <rect
                x=move || {
                    let (bid_ratio, _) = bar_data();
                    width / 2.0 - bid_ratio * width / 2.0
                }
                y="0"
                width=move || {
                    let (bid_ratio, _) = bar_data();
                    bid_ratio * width / 2.0
                }
                height=height
                fill=colors::bull_alpha(0.6)
                rx="4"
            />

            // Ask side (red, from center right)
            <rect
                x=width / 2.0
                y="0"
                width=move || {
                    let (_, ask_ratio) = bar_data();
                    ask_ratio * width / 2.0
                }
                height=height
                fill=colors::bear_alpha(0.6)
                rx="4"
            />

            // Center line
            <line
                x1=width / 2.0
                y1="0"
                x2=width / 2.0
                y2=height
                stroke=colors::BORDER
                stroke-width="1"
            />
        </svg>
    }
}

/// Vertical depth bar (for order book visualization)
#[component]
pub fn DepthBarVertical(
    #[prop(into)] value: Signal<f64>,
    #[prop(into)] max_value: Signal<f64>,
    #[prop(default = false)] is_bid: bool,
    #[prop(default = 100.0)] width: f64,
    #[prop(default = 20.0)] height: f64,
) -> impl IntoView {
    let bar_width = move || {
        let val = value.get();
        let max = max_value.get();
        if max <= 0.0 {
            0.0
        } else {
            (val / max * width).min(width)
        }
    };

    let fill = if is_bid {
        colors::bull_alpha(0.3)
    } else {
        colors::bear_alpha(0.3)
    };

    view! {
        <svg
            class="depth-bar-vertical"
            viewBox=format!("0 0 {} {}", width, height)
            style="width: 100%; height: 100%;"
        >
            <rect
                x=move || if is_bid { width - bar_width() } else { 0.0 }
                y="0"
                width=bar_width
                height=height
                fill=fill
            />
        </svg>
    }
}
