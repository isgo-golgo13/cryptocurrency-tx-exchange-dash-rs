//! # dash-charts
//!
//! D3.js-style SVG charting library built with Leptos.
//! Provides reactive, high-performance charts for financial data visualization.
//!
//! ## Architecture
//!
//! Uses Strategy pattern for:
//! - Scale computation (linear, log, time, band)
//! - Path generation (line, area, step)
//! - Axis rendering
//!
//! ## Modules
//!
//! - `chartkit` - Core primitives: scales, paths, axes
//! - `candlestick` - OHLCV candlestick charts
//! - `depth` - Market depth / order book visualization
//! - `sparkline` - Compact inline charts

pub mod candlestick;
pub mod chartkit;
pub mod depth;
pub mod sparkline;

pub use candlestick::*;
pub use chartkit::*;
pub use depth::*;
pub use sparkline::*;

// Re-export colors from dash-core for convenience
pub use dash_core::colors;

/// Chart margin configuration
#[derive(Debug, Clone, Copy)]
pub struct ChartMargin {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl ChartMargin {
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self { top, right, bottom, left }
    }

    pub const fn uniform(margin: f64) -> Self {
        Self::new(margin, margin, margin, margin)
    }

    pub const fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    /// Compact margins for sparklines
    pub const fn compact() -> Self {
        Self::new(2.0, 2.0, 2.0, 2.0)
    }

    /// Standard chart margins
    pub const fn standard() -> Self {
        Self::new(20.0, 60.0, 30.0, 60.0)
    }

    /// Right Y-axis layout (labels on right)
    pub const fn right_axis() -> Self {
        Self::new(10.0, 70.0, 25.0, 10.0)
    }
}

impl Default for ChartMargin {
    fn default() -> Self {
        Self::standard()
    }
}

/// Chart dimensions with margin handling
#[derive(Debug, Clone, Copy)]
pub struct ChartDimensions {
    pub width: f64,
    pub height: f64,
    pub margin: ChartMargin,
}

impl ChartDimensions {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            margin: ChartMargin::default(),
        }
    }

    pub fn with_margin(mut self, margin: ChartMargin) -> Self {
        self.margin = margin;
        self
    }

    /// Inner width (excluding margins)
    pub fn inner_width(&self) -> f64 {
        (self.width - self.margin.left - self.margin.right).max(0.0)
    }

    /// Inner height (excluding margins)
    pub fn inner_height(&self) -> f64 {
        (self.height - self.margin.top - self.margin.bottom).max(0.0)
    }

    /// SVG transform for inner chart area
    pub fn inner_transform(&self) -> String {
        format!("translate({}, {})", self.margin.left, self.margin.top)
    }

    /// ViewBox string for SVG
    pub fn viewbox(&self) -> String {
        format!("0 0 {} {}", self.width, self.height)
    }
}

impl Default for ChartDimensions {
    fn default() -> Self {
        Self::new(800.0, 400.0)
    }
}
