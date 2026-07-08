//! GaugeWidget — segmented level meter/gauge.
//!
//! Renders a horizontal or vertical gauge with discrete segments, showing
//! a value within a configurable range. Visually distinct from ProgressBar:
//! uses individual segment characters with color gradient support.

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::{Cell, Rgba, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GaugeOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub struct GaugeChars {
    pub full: char,
    pub empty: char,
    pub partial_chars: [char; 3],
}

impl Default for GaugeChars {
    fn default() -> Self {
        Self {
            full: '\u{2588}',
            empty: '\u{2591}',
            partial_chars: ['\u{258F}', '\u{258E}', '\u{258D}'],
        }
    }
}

impl GaugeChars {
    pub fn blocks() -> Self {
        Self {
            full: '\u{2588}',
            empty: '\u{2591}',
            partial_chars: ['\u{258F}', '\u{258E}', '\u{258C}'],
        }
    }

    pub fn shade() -> Self {
        Self {
            full: '\u{2593}',
            empty: '\u{2591}',
            partial_chars: ['\u{258F}', '\u{258E}', '\u{258D}'],
        }
    }

    pub fn dots() -> Self {
        Self {
            full: '\u{2022}',
            empty: '\u{00B7}',
            partial_chars: ['\u{00B7}', '\u{00B7}', '\u{2022}'],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GaugeStyle {
    pub low_fg: Rgba,
    pub mid_fg: Rgba,
    pub high_fg: Rgba,
    pub empty_fg: Rgba,
    pub bg: Rgba,
    pub chars: GaugeChars,
}

impl Default for GaugeStyle {
    fn default() -> Self {
        Self {
            low_fg: Rgba::from_rgb_u8(200, 80, 80),
            mid_fg: Rgba::from_rgb_u8(200, 180, 60),
            high_fg: Rgba::from_rgb_u8(80, 200, 100),
            empty_fg: Rgba::from_rgb_u8(60, 60, 65),
            bg: Rgba::TRANSPARENT,
            chars: GaugeChars::default(),
        }
    }
}

pub struct GaugeWidget {
    style: LayoutStyle,
    orientation: GaugeOrientation,
    value: f32,
    min: f32,
    max: f32,
    segments: u32,
    gauge_style: GaugeStyle,
    show_label: bool,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl GaugeWidget {
    pub fn new(style: LayoutStyle, orientation: GaugeOrientation) -> Self {
        Self {
            style,
            orientation,
            value: 0.0,
            min: 0.0,
            max: 100.0,
            segments: 10,
            gauge_style: GaugeStyle::default(),
            show_label: false,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn horizontal(style: LayoutStyle) -> Self {
        Self::new(style, GaugeOrientation::Horizontal)
    }

    pub fn vertical(style: LayoutStyle) -> Self {
        Self::new(style, GaugeOrientation::Vertical)
    }

    pub fn value(mut self, value: f32) -> Self {
        self.set_value(value);
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self.value = self.clamp_value(self.value);
        self
    }

    pub fn segments(mut self, segments: u32) -> Self {
        self.segments = segments.max(1);
        self
    }

    pub fn gauge_style(mut self, style: GaugeStyle) -> Self {
        self.gauge_style = style;
        self
    }

    pub fn show_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    pub fn value_value(&self) -> f32 {
        self.value
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = self.clamp_value(value);
    }

    fn clamp_value(&self, value: f32) -> f32 {
        let low = self.min.min(self.max);
        let high = self.min.max(self.max);
        value.clamp(low, high)
    }

    fn ratio(&self) -> f32 {
        let span = self.max - self.min;
        if span.abs() < f32::EPSILON {
            return 1.0;
        }
        ((self.value - self.min) / span).clamp(0.0, 1.0)
    }

    fn color_for_ratio(&self, ratio: f32) -> Rgba {
        if ratio < 0.34 {
            self.gauge_style.low_fg
        } else if ratio < 0.67 {
            self.gauge_style.mid_fg
        } else {
            self.gauge_style.high_fg
        }
    }

    fn filled_segments(&self) -> (u32, u32) {
        let total = self.segments;
        let filled = (self.ratio() * total as f32).round() as u32;
        let filled = filled.min(total);
        let sub = ((self.ratio() * total as f32).fract() * 3.0).round() as u32;
        (filled, sub.min(3))
    }

    fn label_text(&self) -> String {
        let pct = (self.ratio() * 100.0).round() as u32;
        format!("{pct}%")
    }
}

impl Behavior for GaugeWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        if !self.visible {
            return;
        }

        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width.max(0.0) as u32;
        let h = layout.height.max(0.0) as u32;
        if w == 0 || h == 0 {
            return;
        }

        let (track_len, track_x, track_y, cross) = match self.orientation {
            GaugeOrientation::Horizontal => {
                let len = if self.show_label {
                    w.saturating_sub(5)
                } else {
                    w
                };
                (len, x, y, h)
            }
            GaugeOrientation::Vertical => {
                let len = h;
                (len, x, y, w)
            }
        };

        let (filled, sub) = self.filled_segments();
        let chars = &self.gauge_style.chars;
        let empty_style = Style::builder()
            .fg(self.gauge_style.empty_fg)
            .bg(self.gauge_style.bg)
            .build();

        for i in 0..track_len {
            let (ch, fg) = if i < filled {
                (
                    chars.full,
                    self.color_for_ratio(i as f32 / track_len as f32),
                )
            } else if i == filled && sub > 0 && sub <= 3 {
                let idx = (sub as usize) - 1;
                (chars.partial_chars[idx], self.color_for_ratio(self.ratio()))
            } else {
                (chars.empty, self.gauge_style.empty_fg)
            };
            let cell_style = Style::builder().fg(fg).bg(self.gauge_style.bg).build();
            for c in 0..cross {
                let (cx, cy) = match self.orientation {
                    GaugeOrientation::Horizontal => (track_x + i, track_y + c),
                    GaugeOrientation::Vertical => (track_x + c, track_y + i),
                };
                ctx.buffer.set(cx, cy, Cell::new(ch, cell_style));
            }
        }

        if self.show_label && self.orientation == GaugeOrientation::Horizontal {
            let label = self.label_text();
            let label_x = track_x + track_len + 1;
            let label_style = Style::builder()
                .fg(self.color_for_ratio(self.ratio()))
                .bg(self.gauge_style.bg)
                .build();
            for (i, ch) in label.chars().enumerate() {
                if (label_x + i as u32) < x + w {
                    ctx.buffer
                        .set(label_x + i as u32, y, Cell::new(ch, label_style));
                }
            }
        }

        let _ = empty_style;
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: Overflow::Hidden,
            visible: self.visible,
            opacity: self.opacity,
        }
    }

    fn set_focus_state(&mut self, focused: bool, _has_focused_descendant: bool) {
        self.focused = focused;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
