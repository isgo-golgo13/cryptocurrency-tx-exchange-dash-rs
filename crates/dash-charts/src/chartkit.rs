//! # chartkit
//!
//! Core chart primitives: scales, path builders, axis generators.
//! Implements Strategy pattern for flexible scale and rendering behaviors.

use std::fmt::Write;

// ============================================================================
// STRATEGY PATTERN: Scale Trait
// ============================================================================

/// Strategy trait for scales (maps domain values to range values)
pub trait Scale: Send + Sync {
    /// Scale a value from domain to range
    fn scale(&self, value: f64) -> f64;
    
    /// Inverse scale (range to domain)
    fn invert(&self, value: f64) -> f64;
    
    /// Generate tick values
    fn ticks(&self, count: usize) -> Vec<f64>;
}

// ============================================================================
// LINEAR SCALE
// ============================================================================

/// Linear scale (D3-style continuous scale)
#[derive(Debug, Clone)]
pub struct LinearScale {
    domain: (f64, f64),
    range: (f64, f64),
    clamp: bool,
}

impl LinearScale {
    pub fn new() -> Self {
        Self {
            domain: (0.0, 1.0),
            range: (0.0, 1.0),
            clamp: false,
        }
    }

    pub fn domain(mut self, min: f64, max: f64) -> Self {
        self.domain = (min, max);
        self
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = (min, max);
        self
    }

    pub fn clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Get domain bounds
    pub fn domain_bounds(&self) -> (f64, f64) {
        self.domain
    }

    /// Get range bounds
    pub fn range_bounds(&self) -> (f64, f64) {
        self.range
    }

    /// Generate "nice" tick values (rounded to clean numbers)
    pub fn nice_ticks(&self, count: usize) -> Vec<f64> {
        let (min, max) = self.domain;
        let range = max - min;

        if range == 0.0 || count == 0 {
            return vec![min];
        }

        let rough_step = range / count as f64;
        let magnitude = 10.0_f64.powf(rough_step.log10().floor());
        let residual = rough_step / magnitude;

        let nice_step = if residual <= 1.0 {
            magnitude
        } else if residual <= 2.0 {
            2.0 * magnitude
        } else if residual <= 5.0 {
            5.0 * magnitude
        } else {
            10.0 * magnitude
        };

        let nice_min = (min / nice_step).floor() * nice_step;
        let nice_max = (max / nice_step).ceil() * nice_step;

        let mut ticks = Vec::new();
        let mut tick = nice_min;

        while tick <= nice_max + nice_step * 0.5 {
            if tick >= min && tick <= max {
                ticks.push(tick);
            }
            tick += nice_step;
        }

        ticks
    }
}

impl Default for LinearScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for LinearScale {
    fn scale(&self, value: f64) -> f64 {
        let (d_min, d_max) = self.domain;
        let (r_min, r_max) = self.range;

        if (d_max - d_min).abs() < f64::EPSILON {
            return (r_min + r_max) / 2.0;
        }

        let mut normalized = (value - d_min) / (d_max - d_min);

        if self.clamp {
            normalized = normalized.clamp(0.0, 1.0);
        }

        r_min + normalized * (r_max - r_min)
    }

    fn invert(&self, value: f64) -> f64 {
        let (d_min, d_max) = self.domain;
        let (r_min, r_max) = self.range;

        if (r_max - r_min).abs() < f64::EPSILON {
            return (d_min + d_max) / 2.0;
        }

        let normalized = (value - r_min) / (r_max - r_min);
        d_min + normalized * (d_max - d_min)
    }

    fn ticks(&self, count: usize) -> Vec<f64> {
        let (min, max) = self.domain;
        if count <= 1 {
            return vec![min];
        }

        let step = (max - min) / (count - 1) as f64;
        (0..count).map(|i| min + step * i as f64).collect()
    }
}

// ============================================================================
// TIME SCALE
// ============================================================================

/// Time scale (maps timestamps to pixel positions)
#[derive(Debug, Clone)]
pub struct TimeScale {
    domain: (i64, i64), // Unix timestamps in milliseconds
    range: (f64, f64),
}

impl TimeScale {
    pub fn new() -> Self {
        Self {
            domain: (0, 1),
            range: (0.0, 1.0),
        }
    }

    pub fn domain(mut self, min: i64, max: i64) -> Self {
        self.domain = (min, max);
        self
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = (min, max);
        self
    }

    /// Scale timestamp to pixel position
    pub fn scale(&self, timestamp: i64) -> f64 {
        let (d_min, d_max) = self.domain;
        let (r_min, r_max) = self.range;

        if d_max == d_min {
            return (r_min + r_max) / 2.0;
        }

        let normalized = (timestamp - d_min) as f64 / (d_max - d_min) as f64;
        r_min + normalized * (r_max - r_min)
    }

    /// Inverse scale (pixel to timestamp)
    pub fn invert(&self, value: f64) -> i64 {
        let (d_min, d_max) = self.domain;
        let (r_min, r_max) = self.range;

        if (r_max - r_min).abs() < f64::EPSILON {
            return (d_min + d_max) / 2;
        }

        let normalized = (value - r_min) / (r_max - r_min);
        (d_min as f64 + normalized * (d_max - d_min) as f64) as i64
    }
}

impl Default for TimeScale {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BAND SCALE (for categorical/ordinal data like candlesticks)
// ============================================================================

/// Band scale for categorical data (e.g., candlestick x positions)
#[derive(Debug, Clone)]
pub struct BandScale {
    domain_count: usize,
    range: (f64, f64),
    padding_inner: f64,
    padding_outer: f64,
}

impl BandScale {
    pub fn new(count: usize) -> Self {
        Self {
            domain_count: count,
            range: (0.0, 1.0),
            padding_inner: 0.1,
            padding_outer: 0.1,
        }
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.range = (min, max);
        self
    }

    pub fn padding(mut self, inner: f64, outer: f64) -> Self {
        self.padding_inner = inner.clamp(0.0, 1.0);
        self.padding_outer = outer.clamp(0.0, 1.0);
        self
    }

    pub fn padding_uniform(self, padding: f64) -> Self {
        self.padding(padding, padding)
    }

    /// Get band width (width of each bar/candle)
    pub fn bandwidth(&self) -> f64 {
        if self.domain_count == 0 {
            return 0.0;
        }

        let (r_min, r_max) = self.range;
        let total_range = r_max - r_min;
        let n = self.domain_count as f64;

        // Total padding
        let outer_total = self.padding_outer * 2.0;
        let inner_total = self.padding_inner * (n - 1.0).max(0.0);

        let available = total_range / (n + outer_total + inner_total);
        available * (1.0 - self.padding_inner)
    }

    /// Get step size (band + gap)
    pub fn step(&self) -> f64 {
        if self.domain_count == 0 {
            return 0.0;
        }

        let (r_min, r_max) = self.range;
        (r_max - r_min) / self.domain_count as f64
    }

    /// Get position for index
    pub fn scale(&self, index: usize) -> f64 {
        if self.domain_count == 0 {
            return self.range.0;
        }

        let (r_min, _) = self.range;
        let step = self.step();
        let offset = self.padding_outer * step;

        r_min + offset + index as f64 * step
    }

    /// Get center position for index
    pub fn scale_center(&self, index: usize) -> f64 {
        self.scale(index) + self.bandwidth() / 2.0
    }
}

impl Default for BandScale {
    fn default() -> Self {
        Self::new(10)
    }
}

// ============================================================================
// STRATEGY PATTERN: Path Generator Trait
// ============================================================================

/// Strategy trait for path generation
pub trait PathGenerator: Send + Sync {
    fn generate(&self, points: &[(f64, f64)]) -> String;
}

/// Line path generator
#[derive(Debug, Clone, Default)]
pub struct LinePath;

impl PathGenerator for LinePath {
    fn generate(&self, points: &[(f64, f64)]) -> String {
        if points.is_empty() {
            return String::new();
        }

        let mut path = String::with_capacity(points.len() * 20);
        let (x, y) = points[0];
        write!(path, "M{:.2},{:.2}", x, y).unwrap();

        for &(x, y) in &points[1..] {
            write!(path, "L{:.2},{:.2}", x, y).unwrap();
        }

        path
    }
}

/// Step path generator (for step charts)
#[derive(Debug, Clone)]
pub struct StepPath {
    pub step_position: StepPosition,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum StepPosition {
    #[default]
    Before,
    After,
    Middle,
}

impl Default for StepPath {
    fn default() -> Self {
        Self {
            step_position: StepPosition::Before,
        }
    }
}

impl PathGenerator for StepPath {
    fn generate(&self, points: &[(f64, f64)]) -> String {
        if points.is_empty() {
            return String::new();
        }

        let mut path = String::with_capacity(points.len() * 30);
        let (x, y) = points[0];
        write!(path, "M{:.2},{:.2}", x, y).unwrap();

        for i in 1..points.len() {
            let (x0, y0) = points[i - 1];
            let (x1, y1) = points[i];

            match self.step_position {
                StepPosition::Before => {
                    write!(path, "V{:.2}H{:.2}", y1, x1).unwrap();
                }
                StepPosition::After => {
                    write!(path, "H{:.2}V{:.2}", x1, y1).unwrap();
                }
                StepPosition::Middle => {
                    let mid_x = (x0 + x1) / 2.0;
                    write!(path, "H{:.2}V{:.2}H{:.2}", mid_x, y1, x1).unwrap();
                }
            }
        }

        path
    }
}

// ============================================================================
// PATH BUILDER (fluent API)
// ============================================================================

/// SVG path builder with fluent API
#[derive(Debug, Clone, Default)]
pub struct PathBuilder {
    commands: String,
}

impl PathBuilder {
    pub fn new() -> Self {
        Self {
            commands: String::with_capacity(256),
        }
    }

    pub fn move_to(mut self, x: f64, y: f64) -> Self {
        write!(self.commands, "M{:.2},{:.2}", x, y).unwrap();
        self
    }

    pub fn line_to(mut self, x: f64, y: f64) -> Self {
        write!(self.commands, "L{:.2},{:.2}", x, y).unwrap();
        self
    }

    pub fn horizontal_to(mut self, x: f64) -> Self {
        write!(self.commands, "H{:.2}", x).unwrap();
        self
    }

    pub fn vertical_to(mut self, y: f64) -> Self {
        write!(self.commands, "V{:.2}", y).unwrap();
        self
    }

    pub fn cubic_to(mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> Self {
        write!(
            self.commands,
            "C{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}",
            x1, y1, x2, y2, x, y
        )
        .unwrap();
        self
    }

    pub fn quadratic_to(mut self, x1: f64, y1: f64, x: f64, y: f64) -> Self {
        write!(self.commands, "Q{:.2},{:.2},{:.2},{:.2}", x1, y1, x, y).unwrap();
        self
    }

    pub fn arc_to(
        mut self,
        rx: f64,
        ry: f64,
        rotation: f64,
        large_arc: bool,
        sweep: bool,
        x: f64,
        y: f64,
    ) -> Self {
        write!(
            self.commands,
            "A{:.2},{:.2},{:.2},{},{},{:.2},{:.2}",
            rx,
            ry,
            rotation,
            large_arc as u8,
            sweep as u8,
            x,
            y
        )
        .unwrap();
        self
    }

    pub fn close(mut self) -> Self {
        self.commands.push('Z');
        self
    }

    pub fn build(self) -> String {
        self.commands
    }
}

// ============================================================================
// AREA PATH GENERATOR
// ============================================================================

/// Generate closed area path with baseline
pub fn area_path(points: &[(f64, f64)], baseline_y: f64) -> String {
    if points.is_empty() {
        return String::new();
    }

    let mut builder = PathBuilder::new()
        .move_to(points[0].0, baseline_y)
        .line_to(points[0].0, points[0].1);

    for &(x, y) in &points[1..] {
        builder = builder.line_to(x, y);
    }

    if let Some(&(last_x, _)) = points.last() {
        builder = builder.line_to(last_x, baseline_y);
    }

    builder.close().build()
}

/// Generate line path (non-closed)
pub fn line_path(points: &[(f64, f64)]) -> String {
    LinePath.generate(points)
}

// ============================================================================
// FORMATTERS
// ============================================================================

/// Format price for axis labels
pub fn format_price(price: f64, decimals: usize) -> String {
    if price >= 1_000_000.0 {
        format!("{:.1}M", price / 1_000_000.0)
    } else if price >= 10_000.0 {
        format!("{:.0}", price)
    } else if price >= 1_000.0 {
        format!("{:.1}", price)
    } else if price >= 1.0 {
        format!("{:.prec$}", price, prec = decimals)
    } else {
        format!("{:.6}", price)
    }
}

/// Format volume for axis labels
pub fn format_volume(volume: f64) -> String {
    if volume >= 1_000_000.0 {
        format!("{:.2}M", volume / 1_000_000.0)
    } else if volume >= 1_000.0 {
        format!("{:.2}K", volume / 1_000.0)
    } else {
        format!("{:.4}", volume)
    }
}

/// Format large numbers with K/M/B suffixes
pub fn format_large_number(num: f64) -> String {
    let abs = num.abs();
    let sign = if num < 0.0 { "-" } else { "" };

    if abs >= 1_000_000_000.0 {
        format!("{}{:.2}B", sign, abs / 1_000_000_000.0)
    } else if abs >= 1_000_000.0 {
        format!("{}{:.2}M", sign, abs / 1_000_000.0)
    } else if abs >= 1_000.0 {
        format!("{}{:.2}K", sign, abs / 1_000.0)
    } else {
        format!("{}{:.2}", sign, abs)
    }
}

/// Format timestamp for chart axes
pub fn format_time(timestamp_ms: i64, interval_secs: i64) -> String {
    use chrono::{TimeZone, Utc};

    let dt = Utc.timestamp_millis_opt(timestamp_ms).unwrap();

    if interval_secs >= 86400 {
        dt.format("%b %d").to_string()
    } else if interval_secs >= 3600 {
        dt.format("%H:%M").to_string()
    } else {
        dt.format("%H:%M").to_string()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_scale() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0);

        assert_eq!(scale.scale(0.0), 0.0);
        assert_eq!(scale.scale(50.0), 250.0);
        assert_eq!(scale.scale(100.0), 500.0);
    }

    #[test]
    fn test_linear_scale_invert() {
        let scale = LinearScale::new()
            .domain(0.0, 100.0)
            .range(0.0, 500.0);

        assert_eq!(scale.invert(250.0), 50.0);
    }

    #[test]
    fn test_band_scale() {
        let scale = BandScale::new(5).range(0.0, 100.0);
        let bw = scale.bandwidth();
        assert!(bw > 0.0);
        assert!(bw < 20.0); // Should be less than 100/5
    }

    #[test]
    fn test_path_builder() {
        let path = PathBuilder::new()
            .move_to(0.0, 0.0)
            .line_to(100.0, 100.0)
            .close()
            .build();

        assert!(path.contains("M0.00,0.00"));
        assert!(path.contains("L100.00,100.00"));
        assert!(path.contains("Z"));
    }

    #[test]
    fn test_line_path_generator() {
        let generator = LinePath;
        let path = generator.generate(&[(0.0, 0.0), (50.0, 50.0), (100.0, 0.0)]);

        assert!(path.starts_with("M0.00,0.00"));
        assert!(path.contains("L50.00,50.00"));
    }

    #[test]
    fn test_format_large_number() {
        assert_eq!(format_large_number(1_500_000.0), "1.50M");
        assert_eq!(format_large_number(2_500.0), "2.50K");
        assert_eq!(format_large_number(500.0), "500.00");
    }
}
