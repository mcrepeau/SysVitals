//! Shared helpers for all chart panels.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Color;
use std::collections::VecDeque;

// ── History ──────────────────────────────────────────────────────────────────

/// Trim `history` to the last `width` samples and convert to `(x, y)` pairs
/// suitable for a ratatui `Dataset`.
pub fn trim_to_width(history: &VecDeque<f64>, width: usize) -> Vec<(f64, f64)> {
    history
        .iter()
        .rev()
        .take(width)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect()
}

// ── Color ─────────────────────────────────────────────────────────────────────

/// Return a colour for a percentage-based usage gauge:
/// green below 70 %, yellow 70–90 %, red at or above 90 %.
pub fn usage_color(percent: f64) -> Color {
    if percent >= 90.0 {
        Color::Red
    } else if percent >= 70.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

// ── Dynamic Y-axis ────────────────────────────────────────────────────────────

/// Scale the Y-axis to 120 % of the maximum value in `history`, with a
/// minimum floor of 1.0 so the axis is never degenerate.
pub fn dynamic_bound(history: &VecDeque<f64>) -> f64 {
    let max = history.iter().cloned().fold(0.0f64, f64::max);
    (max * 1.2).max(1.0)
}

/// Format a floating-point rate value with precision appropriate to its magnitude.
pub fn format_rate(val: f64) -> String {
    if val >= 100.0 {
        format!("{val:.0}")
    } else if val >= 10.0 {
        format!("{val:.1}")
    } else {
        format!("{val:.2}")
    }
}

/// Generate `[min, mid, max]` label strings for a dynamic Y-axis.
pub fn rate_axis_labels(bound: f64) -> [String; 3] {
    [format_rate(0.0), format_rate(bound / 2.0), format_rate(bound)]
}

// ── Layout ────────────────────────────────────────────────────────────────────

/// Standard vertical layout for a single-chart panel.
/// Returns `(title_rect, chart_rect)`; the two 1-row spacers exist only for
/// visual padding and do not need to be rendered into.
pub fn chart_areas(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // spacer
            Constraint::Min(0),    // chart
            Constraint::Length(1), // spacer
        ])
        .split(area);
    (chunks[0], chunks[2])
}

/// Split `area` into two equal horizontal halves.
pub fn split_horizontal(area: Rect) -> (Rect, Rect) {
    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    (halves[0], halves[1])
}
