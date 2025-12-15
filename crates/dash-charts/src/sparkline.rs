//! Sparkline charts for compact inline visualizations
//!
//! Lightweight, minimal charts for embedding in tables, headers, and tight spaces.

use crate::{
    chartkit::{line_path, LinearScale, Scale},
    colors,
};
use leptos::prelude::*;

// ============================================================================
// PRICE SPARKLINE
// ============================================================================

/// Price sparkline configuration
#[derive(Debug, Clone)]
pub struct SparklineConfig {
    pub width: f64,
    pub height: f64,
    pub stroke_width: f64,
    pub show_endpoint: bool,
    pub endpoint_radius: f64,
}

impl Default for SparklineConfig {
    fn default() -> Self {
        Self {
            width: 120.0,
            height: 32.0,
            stroke_width: 1.5,
            show_endpoint: true,
            endpoint_radius: 3.0,
        }
    }
}

/// Price sparkline component
#[component]
pub fn PriceSparkline(
    #[prop(into)] prices: Signal<Vec<f64>>,
    #[prop(optional)] config: Option<SparklineConfig>,
    #[prop(optional)] positive: Option<bool>,
) -> impl IntoView {
    let config = config.unwrap_or_default();
    let w = config.width;
    let h = config.height;
    let stroke_w = config.stroke_width;
    let show_end = config.show_endpoint;
    let end_r = config.endpoint_radius;

    let chart_data = move || {
        let data = prices.get();
        if data.len() < 2 {
            return None;
        }

        let min = data.iter().cloned().fold(f64::MAX, f64::min);
        let max = data.iter().cloned().fold(f64::MIN, f64::max);

        // Add padding
        let range = max - min;
        let padding = if range > 0.0 { range * 0.1 } else { 1.0 };
        
        let y_scale = LinearScale::new()
            .domain(min - padding, max + padding)
            .range(h - 2.0, 2.0);

        let x_step = (w - 4.0) / (data.len() - 1) as f64;

        let points: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, &price)| (2.0 + i as f64 * x_step, y_scale.scale(price)))
            .collect();

        let path = line_path(&points);

        // Determine color
        let is_positive = positive.unwrap_or_else(|| {
            data.last().unwrap_or(&0.0) >= data.first().unwrap_or(&0.0)
        });
        let color = if is_positive { colors::BULL } else { colors::BEAR };

        let last_point = points.last().cloned();

        Some((path, color, last_point))
    };

    view! {
        <svg
            class="price-sparkline"
            viewBox=format!("0 0 {} {}", w, h)
            style="width: 100%; height: 100%;"
        >
            {move || {
                chart_data().map(|(path, color, last_point)| {
                    view! {
                        <>
                            // Line
                            <path
                                d=path
                                fill="none"
                                stroke=color
                                stroke-width=stroke_w
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />

                            // Endpoint dot
                            {if show_end {
                                last_point.map(|(x, y)| {
                                    view! {
                                        <circle
                                            cx=x
                                            cy=y
                                            r=end_r
                                            fill=color
                                        />
                                    }
                                })
                            } else {
                                None
                            }}
                        </>
                    }
                })
            }}
        </svg>
    }
}

// ============================================================================
// VOLUME SPARKLINE
// ============================================================================

/// Volume sparkline with bars
#[component]
pub fn VolumeSparkline(
    #[prop(into)] volumes: Signal<Vec<f64>>,
    #[prop(default = 120.0)] width: f64,
    #[prop(default = 24.0)] height: f64,
    #[prop(optional)] color: Option<&'static str>,
) -> impl IntoView {
    let bar_color = color.unwrap_or(colors::BULL);

    let chart_data = move || {
        let data = volumes.get();
        if data.is_empty() {
            return None;
        }

        let max = data.iter().cloned().fold(0.0_f64, f64::max);
        let y_scale = LinearScale::new()
            .domain(0.0, max * 1.1)
            .range(height - 2.0, 2.0);

        let bar_width = ((width - 4.0) / data.len() as f64 - 1.0).max(1.0);
        let gap = 1.0;

        let bars: Vec<(f64, f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, &vol)| {
                let x = 2.0 + i as f64 * (bar_width + gap);
                let y = y_scale.scale(vol);
                let h = (height - 2.0 - y).max(0.0);
                (x, y, h)
            })
            .collect();

        Some((bars, bar_width))
    };

    view! {
        <svg
            class="volume-sparkline"
            viewBox=format!("0 0 {} {}", width, height)
            style="width: 100%; height: 100%;"
        >
            {move || {
                chart_data().map(|(bars, bar_width)| {
                    bars.into_iter().map(|(x, y, h)| {
                        view! {
                            <rect
                                x=x
                                y=y
                                width=bar_width
                                height=h
                                fill=colors::bull_alpha(0.4)
                                rx="1"
                            />
                        }
                    }).collect_view()
                })
            }}
        </svg>
    }
    .into_any()
}

// ============================================================================
// TRADE FLOW SPARKLINE
// ============================================================================

/// Trade flow sparkline (shows buy/sell pressure as mirrored bars)
#[component]
pub fn TradeFlowSparkline(
    #[prop(into)] buy_volumes: Signal<Vec<f64>>,
    #[prop(into)] sell_volumes: Signal<Vec<f64>>,
    #[prop(default = 120.0)] width: f64,
    #[prop(default = 32.0)] height: f64,
) -> impl IntoView {
    let mid_y = height / 2.0;

    let chart_data = move || {
        let buys = buy_volumes.get();
        let sells = sell_volumes.get();

        if buys.is_empty() && sells.is_empty() {
            return None;
        }

        let len = buys.len().max(sells.len());

        // Find max for scaling
        let max_buy = buys.iter().cloned().fold(0.0_f64, f64::max);
        let max_sell = sells.iter().cloned().fold(0.0_f64, f64::max);
        let max_vol = max_buy.max(max_sell);

        if max_vol == 0.0 {
            return None;
        }

        let scale = (mid_y - 2.0) / max_vol;
        let bar_width = ((width - 4.0) / len as f64 - 1.0).max(1.0);
        let gap = 1.0;

        let mut buy_bars = Vec::new();
        let mut sell_bars = Vec::new();

        for i in 0..len {
            let x = 2.0 + i as f64 * (bar_width + gap);

            // Buy bar (goes up from middle)
            if let Some(&vol) = buys.get(i) {
                let h = vol * scale;
                buy_bars.push((x, mid_y - h, h));
            }

            // Sell bar (goes down from middle)
            if let Some(&vol) = sells.get(i) {
                let h = vol * scale;
                sell_bars.push((x, mid_y, h));
            }
        }

        Some((buy_bars, sell_bars, bar_width))
    };

    view! {
        <svg
            class="trade-flow-sparkline"
            viewBox=format!("0 0 {} {}", width, height)
            style="width: 100%; height: 100%;"
        >
            // Center line
            <line
                x1="0"
                y1=mid_y
                x2=width
                y2=mid_y
                stroke=colors::BORDER
                stroke-width="0.5"
            />

            {move || {
                chart_data().map(|(buy_bars, sell_bars, bar_width)| {
                    view! {
                        <>
                            // Buy bars (green, above center)
                            {buy_bars.into_iter().map(|(x, y, h)| {
                                view! {
                                    <rect
                                        x=x y=y
                                        width=bar_width
                                        height=h.max(0.5)
                                        fill=colors::BULL
                                        rx="1"
                                    />
                                }
                            }).collect_view()}

                            // Sell bars (red, below center)
                            {sell_bars.into_iter().map(|(x, y, h)| {
                                view! {
                                    <rect
                                        x=x y=y
                                        width=bar_width
                                        height=h.max(0.5)
                                        fill=colors::BEAR
                                        rx="1"
                                    />
                                }
                            }).collect_view()}
                        </>
                    }
                })
            }}
        </svg>
    }
}

// ============================================================================
// PERCENT BAR
// ============================================================================

/// Horizontal percentage bar (for imbalance, completion, etc.)
/// Value range: -1.0 to +1.0 (0 is center)
#[component]
pub fn PercentBar(
    #[prop(into)] value: Signal<f64>,
    #[prop(default = 100.0)] width: f64,
    #[prop(default = 6.0)] height: f64,
    #[prop(default = colors::BULL)] positive_color: &'static str,
    #[prop(default = colors::BEAR)] negative_color: &'static str,
) -> impl IntoView {
    let center = width / 2.0;

    let bar_data = move || {
        let v = value.get().clamp(-1.0, 1.0);

        if v >= 0.0 {
            // Positive: draw from center to right
            (center, v * center, positive_color)
        } else {
            // Negative: draw from center to left
            let bar_width = v.abs() * center;
            (center - bar_width, bar_width, negative_color)
        }
    };

    view! {
        <svg
            class="percent-bar"
            viewBox=format!("0 0 {} {}", width, height)
            style="width: 100%; height: 100%;"
        >
            // Background
            <rect
                width=width
                height=height
                fill=colors::BG_ELEVATED
                rx="3"
            />

            // Value bar
            <rect
                x=move || bar_data().0
                y="0"
                width=move || bar_data().1
                height=height
                fill=move || bar_data().2
                rx="3"
            />

            // Center line
            <line
                x1=center
                y1="0"
                x2=center
                y2=height
                stroke=colors::BORDER
                stroke-width="1"
            />
        </svg>
    }
}

// ============================================================================
// MINI AREA CHART
// ============================================================================

/// Mini area chart (filled sparkline)
#[component]
pub fn AreaSparkline(
    #[prop(into)] values: Signal<Vec<f64>>,
    #[prop(default = 120.0)] width: f64,
    #[prop(default = 32.0)] height: f64,
    #[prop(optional)] color: Option<&'static str>,
) -> impl IntoView {
    let stroke_color = color.unwrap_or(colors::BULL);

    let chart_data = move || {
        let data = values.get();
        if data.len() < 2 {
            return None;
        }

        let min = data.iter().cloned().fold(f64::MAX, f64::min);
        let max = data.iter().cloned().fold(f64::MIN, f64::max);

        let range = max - min;
        let padding = if range > 0.0 { range * 0.1 } else { 1.0 };

        let y_scale = LinearScale::new()
            .domain(min - padding, max + padding)
            .range(height - 2.0, 2.0);

        let x_step = (width - 4.0) / (data.len() - 1) as f64;

        let points: Vec<(f64, f64)> = data
            .iter()
            .enumerate()
            .map(|(i, &val)| (2.0 + i as f64 * x_step, y_scale.scale(val)))
            .collect();

        // Build area path
        let baseline = height - 2.0;
        let area = crate::chartkit::area_path(&points, baseline);
        let line = line_path(&points);

        Some((area, line))
    };

    view! {
        <svg
            class="area-sparkline"
            viewBox=format!("0 0 {} {}", width, height)
            style="width: 100%; height: 100%;"
        >
            {move || {
                chart_data().map(|(area, line)| {
                    view! {
                        <>
                            // Filled area
                            <path
                                d=area
                                fill=colors::bull_alpha(0.2)
                            />
                            // Line
                            <path
                                d=line
                                fill="none"
                                stroke=stroke_color
                                stroke-width="1.5"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                            />
                        </>
                    }
                })
            }}
        </svg>
    }
}
