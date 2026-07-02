//! ProgressBarWidget — horizontal progress indicator.
//!
//! Renders a configurable progress bar with optional percentage label.

use crate::{Cell, Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

#[derive(Debug, Clone)]
pub struct ProgressChars {
    pub filled: char,
    pub empty: char,
    pub left_cap: char,
    pub right_cap: char,
}

impl Default for ProgressChars {
    fn default() -> Self {
        Self {
            filled: '█',
            empty: '░',
            left_cap: '[',
            right_cap: ']',
        }
    }
}

impl ProgressChars {
    pub fn smooth() -> Self {
        Self {
            filled: '█',
            empty: '░',
            left_cap: ' ',
            right_cap: ' ',
        }
    }

    pub fn ascii() -> Self {
        Self {
            filled: '#',
            empty: '-',
            left_cap: '[',
            right_cap: ']',
        }
    }

    pub fn blocks() -> Self {
        Self {
            filled: '▓',
            empty: '░',
            left_cap: '▐',
            right_cap: '▌',
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgressBarStyle {
    pub filled_fg: Rgba,
    pub filled_bg: Rgba,
    pub empty_fg: Rgba,
    pub empty_bg: Rgba,
    pub label_fg: Rgba,
    pub chars: ProgressChars,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            filled_fg: Rgba::new(0.4, 0.85, 0.5, 1.0),
            filled_bg: Rgba::new(0.2, 0.5, 0.25, 1.0),
            empty_fg: Rgba::new(0.3, 0.3, 0.35, 1.0),
            empty_bg: Rgba::new(0.15, 0.15, 0.18, 1.0),
            label_fg: Rgba::new(0.9, 0.9, 0.92, 1.0),
            chars: ProgressChars::default(),
        }
    }
}

pub struct ProgressBarWidget {
    style: LayoutStyle,
    progress: f32,
    label: Option<String>,
    bar_style: ProgressBarStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl ProgressBarWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            progress: 0.0,
            label: None,
            bar_style: ProgressBarStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn bar_style(mut self, style: ProgressBarStyle) -> Self {
        self.bar_style = style;
        self
    }

    pub fn set_progress(&mut self, p: f32) {
        self.progress = p.clamp(0.0, 1.0);
    }

    pub fn progress_value(&self) -> f32 {
        self.progress
    }
}

impl Behavior for ProgressBarWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
        }

        let inner_w = w.saturating_sub(2);
        if inner_w == 0 {
            return;
        }

        let filled_count = (self.progress * inner_w as f32).round() as u32;
        let filled_count = filled_count.min(inner_w);

        // Draw left cap
        let cap_style = Style::builder()
            .fg(self.bar_style.empty_fg)
            .bg(self.bar_style.empty_bg)
            .build();
        ctx.buffer
            .set(x, y, Cell::new(self.bar_style.chars.left_cap, cap_style));

        // Draw filled portion
        let filled_style = Style::builder()
            .fg(self.bar_style.filled_fg)
            .bg(self.bar_style.filled_bg)
            .build();
        for col in 0..filled_count {
            ctx.buffer.set(
                x + 1 + col,
                y,
                Cell::new(self.bar_style.chars.filled, filled_style),
            );
        }

        // Draw empty portion
        let empty_style = Style::builder()
            .fg(self.bar_style.empty_fg)
            .bg(self.bar_style.empty_bg)
            .build();
        for col in filled_count..inner_w {
            ctx.buffer.set(
                x + 1 + col,
                y,
                Cell::new(self.bar_style.chars.empty, empty_style),
            );
        }

        // Draw right cap
        ctx.buffer.set(
            x + w - 1,
            y,
            Cell::new(self.bar_style.chars.right_cap, cap_style),
        );

        // Draw label centered on top
        let label_text = self.label.as_deref().unwrap_or_else(|| {
            static EMPTY: &str = "";
            EMPTY
        });
        let display = if label_text.is_empty() {
            format!("{:.0}%", self.progress * 100.0)
        } else {
            format!("{} {:.0}%", label_text, self.progress * 100.0)
        };
        let label_len = display.chars().count() as u32;
        if label_len < w {
            let label_x = x + (w.saturating_sub(label_len)) / 2;
            let label_style = Style::builder().fg(self.bar_style.label_fg).build();
            ctx.buffer.draw_text(label_x, y, &display, label_style);
        }

        // Multi-line bars: fill additional rows
        for row in 1..h {
            let row_cap_style = Style::builder()
                .fg(self.bar_style.empty_fg)
                .bg(self.bar_style.empty_bg)
                .build();
            ctx.buffer.set(x, y + row, Cell::new(' ', row_cap_style));
            ctx.buffer
                .set(x + w - 1, y + row, Cell::new(' ', row_cap_style));

            for col in 0..filled_count {
                ctx.buffer.set(
                    x + 1 + col,
                    y + row,
                    Cell::new(self.bar_style.chars.filled, filled_style),
                );
            }
            for col in filled_count..inner_w {
                ctx.buffer.set(
                    x + 1 + col,
                    y + row,
                    Cell::new(self.bar_style.chars.empty, empty_style),
                );
            }
        }
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: Overflow::Hidden,
            visible: self.visible,
            opacity: self.opacity,
        }
    }

    fn handle_key(&mut self, _key: &crate::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _mouse: &crate::MouseEvent) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
