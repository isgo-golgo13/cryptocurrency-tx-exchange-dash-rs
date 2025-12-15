//! Candlestick chart component with volume overlay
//!
//! Renders OHLCV data as traditional candlestick chart with optional volume bars.

use crate::{
    chartkit::{BandScale, LinearScale, Scale, format_price},
    colors, ChartDimensions, ChartMargin,
};
use dash_core::{Candle, CandleHistory};
use leptos::prelude::*;

/// Candlestick chart configuration
#[derive(Debug, Clone)]
pub struct CandlestickConfig {
    pub width: f64,
    pub height: f64,
    pub show_volume: bool,
    pub volume_height_ratio: f64,
    pub show_grid: bool,
    pub show_crosshair: bool,
}

impl Default for CandlestickConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 400.0,
            show_volume: true,
            volume_height_ratio: 0.2,
            show_grid: true,
            show_crosshair: false,
        }
    }
}

impl CandlestickConfig {
    pub fn compact() -> Self {
        Self {
            width: 400.0,
            height: 200.0,
            show_volume: false,
            volume_height_ratio: 0.0,
            show_grid: false,
            show_crosshair: false,
        }
    }
}

/// Internal chart state computed from candle data
#[derive(Clone)]
struct ChartState {
    candles: Vec<Candle>,
    y_scale: LinearScale,
    vol_scale: LinearScale,
    x_scale: BandScale,
    bandwidth: f64,
}

/// Candlestick chart component
#[component]
pub fn CandlestickChart(
    #[prop(into)] candles: Signal<CandleHistory>,
    #[prop(optional)] config: Option<CandlestickConfig>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    
    let dims = ChartDimensions::new(config.width, config.height)
        .with_margin(ChartMargin::right_axis());

    let price_height = if config.show_volume {
        dims.inner_height() * (1.0 - config.volume_height_ratio)
    } else {
        dims.inner_height()
    };

    let volume_height = dims.inner_height() * config.volume_height_ratio;
    let volume_y_offset = price_height + 10.0;

    let show_volume = config.show_volume;
    let show_grid = config.show_grid;

    // Compute chart state from candle data
    let chart_state = move || {
        let history = candles.get();
        let candle_list = &history.candles;

        if candle_list.is_empty() {
            return None;
        }

        // Price range with padding
        let (price_min, price_max) = history.price_range().unwrap_or((0.0, 1.0));
        let price_padding = (price_max - price_min) * 0.05;
        
        let y_scale = LinearScale::new()
            .domain(price_min - price_padding, price_max + price_padding)
            .range(price_height, 0.0);

        // Volume scale
        let (_, vol_max) = history.volume_range().unwrap_or((0.0, 1.0));
        let vol_scale = LinearScale::new()
            .domain(0.0, vol_max * 1.1)
            .range(volume_height, 0.0);

        // X scale (band scale for candle positions)
        let x_scale = BandScale::new(candle_list.len())
            .range(0.0, dims.inner_width())
            .padding(0.2, 0.1);

        let bandwidth = x_scale.bandwidth();

        Some(ChartState {
            candles: candle_list.clone(),
            y_scale,
            vol_scale,
            x_scale,
            bandwidth,
        })
    };

    view! {
        <svg
            class="candlestick-chart"
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

            // Chart area
            <g transform=dims.inner_transform()>
                // Grid lines
                {move || {
                    if show_grid {
                        Some(view! {
                            <ChartGrid
                                width=dims.inner_width()
                                height=price_height
                                h_lines=5
                                v_lines=0
                            />
                        })
                    } else {
                        None
                    }
                }}

                // Candlesticks
                {move || {
                    chart_state().map(|state| {
                        state.candles.iter().enumerate().map(|(i, candle)| {
                            let x = state.x_scale.scale(i);
                            let x_center = x + state.bandwidth / 2.0;

                            // Wick coordinates
                            let wick_y1 = state.y_scale.scale(candle.high.as_f64());
                            let wick_y2 = state.y_scale.scale(candle.low.as_f64());

                            // Body coordinates
                            let body_top = candle.open.as_f64().max(candle.close.as_f64());
                            let body_bottom = candle.open.as_f64().min(candle.close.as_f64());
                            let body_y = state.y_scale.scale(body_top);
                            let body_h = (state.y_scale.scale(body_bottom) - body_y).max(1.0);

                            let fill = candle.fill_color();

                            view! {
                                <g class="candle" class=candle.css_class()>
                                    // Wick
                                    <line
                                        x1=x_center
                                        y1=wick_y1
                                        x2=x_center
                                        y2=wick_y2
                                        stroke=fill
                                        stroke-width="1"
                                    />
                                    // Body
                                    <rect
                                        x=x
                                        y=body_y
                                        width=state.bandwidth
                                        height=body_h
                                        fill=fill
                                        stroke=fill
                                        stroke-width="1"
                                        rx="1"
                                    />
                                </g>
                            }
                        }).collect_view()
                    })
                }}

                // Volume bars
                {move || {
                    if show_volume {
                        chart_state().map(|state| {
                            view! {
                                <g transform=format!("translate(0, {})", volume_y_offset)>
                                    {state.candles.iter().enumerate().map(|(i, candle)| {
                                        let x = state.x_scale.scale(i);
                                        let vol = candle.volume.as_f64();
                                        let bar_y = state.vol_scale.scale(vol);
                                        let bar_h = (volume_height - bar_y).max(0.0);
                                        
                                        let fill = if candle.is_bullish() {
                                            colors::bull_alpha(0.5)
                                        } else {
                                            colors::bear_alpha(0.5)
                                        };

                                        view! {
                                            <rect
                                                x=x
                                                y=bar_y
                                                width=state.bandwidth
                                                height=bar_h
                                                fill=fill
                                            />
                                        }
                                    }).collect_view()}
                                </g>
                            }
                        })
                    } else {
                        None
                    }
                }}

                // Y-Axis (right side)
                <g transform=format!("translate({}, 0)", dims.inner_width())>
                    <line
                        x1="0" y1="0"
                        x2="0" y2=price_height
                        stroke=colors::BORDER
                        stroke-width="1"
                    />
                    {move || {
                        chart_state().map(|state| {
                            let ticks = state.y_scale.nice_ticks(5);
                            ticks.into_iter().map(|tick| {
                                let y = state.y_scale.scale(tick);
                                let label = format_price(tick, 2);

                                view! {
                                    <g transform=format!("translate(0, {})", y)>
                                        <line x1="0" x2="5" stroke=colors::BORDER />
                                        <text
                                            x="8"
                                            dy="0.32em"
                                            fill=colors::TEXT_MUTED
                                            font-size="10"
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
        </svg>
    }
}

/// Grid lines component
#[component]
fn ChartGrid(
    width: f64,
    height: f64,
    h_lines: usize,
    v_lines: usize,
) -> impl IntoView {
    let h_positions: Vec<f64> = if h_lines > 0 {
        (0..=h_lines).map(|i| i as f64 * height / h_lines as f64).collect()
    } else {
        vec![]
    };

    let v_positions: Vec<f64> = if v_lines > 0 {
        (0..=v_lines).map(|i| i as f64 * width / v_lines as f64).collect()
    } else {
        vec![]
    };

    view! {
        <g class="chart-grid">
            // Horizontal lines
            {h_positions.into_iter().map(|y| {
                view! {
                    <line
                        x1="0" y1=y
                        x2=width y2=y
                        stroke=colors::GRID
                        stroke-width="1"
                        stroke-dasharray="2,2"
                    />
                }
            }).collect_view()}

            // Vertical lines
            {v_positions.into_iter().map(|x| {
                view! {
                    <line
                        x1=x y1="0"
                        x2=x y2=height
                        stroke=colors::GRID
                        stroke-width="1"
                        stroke-dasharray="2,2"
                    />
                }
            }).collect_view()}
        </g>
    }
}

/// Mini candlestick sparkline (compact version)
#[component]
pub fn CandlestickSparkline(
    #[prop(into)] candles: Signal<Vec<Candle>>,
    #[prop(default = 120.0)] width: f64,
    #[prop(default = 40.0)] height: f64,
) -> impl IntoView {
    let chart_data = move || {
        let candle_list = candles.get();
        if candle_list.is_empty() {
            return None;
        }

        // Find price range
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for c in &candle_list {
            min = min.min(c.low.as_f64());
            max = max.max(c.high.as_f64());
        }

        let padding = (max - min) * 0.1;
        let y_scale = LinearScale::new()
            .domain(min - padding, max + padding)
            .range(height - 2.0, 2.0);

        let x_scale = BandScale::new(candle_list.len())
            .range(2.0, width - 2.0)
            .padding(0.3, 0.1);

        Some((candle_list, y_scale, x_scale))
    };

    view! {
        <svg
            class="candlestick-sparkline"
            viewBox=format!("0 0 {} {}", width, height)
            style="width: 100%; height: 100%;"
        >
            {move || {
                chart_data().map(|(candles, y_scale, x_scale)| {
                    let bw = x_scale.bandwidth();
                    
                    candles.iter().enumerate().map(|(i, candle)| {
                        let x = x_scale.scale(i);
                        let x_center = x + bw / 2.0;

                        let wick_y1 = y_scale.scale(candle.high.as_f64());
                        let wick_y2 = y_scale.scale(candle.low.as_f64());

                        let body_top = candle.open.as_f64().max(candle.close.as_f64());
                        let body_bottom = candle.open.as_f64().min(candle.close.as_f64());
                        let body_y = y_scale.scale(body_top);
                        let body_h = (y_scale.scale(body_bottom) - body_y).max(1.0);

                        let fill = candle.fill_color();

                        view! {
                            <g>
                                <line
                                    x1=x_center y1=wick_y1
                                    x2=x_center y2=wick_y2
                                    stroke=fill
                                    stroke-width="0.5"
                                />
                                <rect
                                    x=x y=body_y
                                    width=bw height=body_h
                                    fill=fill
                                />
                            </g>
                        }
                    }).collect_view()
                })
            }}
        </svg>
    }
}
