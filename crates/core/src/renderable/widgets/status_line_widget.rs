//! StatusLineWidget — fixed-height bar with left/center/right segments.
//!
//! Renders a one-row status bar with three independently aligned segments,
//! commonly used for file name, mode indicator, and cursor position.

use crate::{Cell, Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

#[derive(Debug, Clone)]
pub struct StatusLineStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub separator: char,
    pub separator_fg: Rgba,
}

impl Default for StatusLineStyle {
    fn default() -> Self {
        Self {
            fg: Rgba::new(0.9, 0.9, 0.92, 1.0),
            bg: Rgba::new(0.2, 0.2, 0.25, 1.0),
            separator: '│',
            separator_fg: Rgba::new(0.4, 0.4, 0.45, 1.0),
        }
    }
}

pub struct StatusLineWidget {
    style: LayoutStyle,
    left: String,
    center: String,
    right: String,
    line_style: StatusLineStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl StatusLineWidget {
    pub fn new(style: LayoutStyle) -> Self {
        let style = style.height(1.0);
        Self {
            style,
            left: String::new(),
            center: String::new(),
            right: String::new(),
            line_style: StatusLineStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn left(mut self, text: impl Into<String>) -> Self {
        self.left = text.into();
        self
    }

    pub fn center(mut self, text: impl Into<String>) -> Self {
        self.center = text.into();
        self
    }

    pub fn right(mut self, text: impl Into<String>) -> Self {
        self.right = text.into();
        self
    }

    pub fn line_style(mut self, style: StatusLineStyle) -> Self {
        self.line_style = style;
        self
    }

    pub fn set_left(&mut self, text: impl Into<String>) {
        self.left = text.into();
    }

    pub fn set_center(&mut self, text: impl Into<String>) {
        self.center = text.into();
    }

    pub fn set_right(&mut self, text: impl Into<String>) {
        self.right = text.into();
    }

    fn char_width(s: &str) -> u32 {
        s.chars().count() as u32
    }
}

impl Behavior for StatusLineWidget {
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

        if w == 0 {
            return;
        }

        let base_style = Style::builder()
            .fg(self.line_style.fg)
            .bg(self.line_style.bg)
            .build();

        // Clear entire row
        for col in 0..w {
            ctx.buffer.set(x + col, y, Cell::new(' ', base_style));
        }

        let left_w = Self::char_width(&self.left);
        let center_w = Self::char_width(&self.center);
        let right_w = Self::char_width(&self.right);

        // Left segment — aligned to left
        let left_display = if left_w > w {
            let max_chars = w as usize;
            self.left.chars().take(max_chars).collect::<String>()
        } else {
            self.left.clone()
        };
        ctx.buffer.draw_text(x, y, &left_display, base_style);

        // Right segment — aligned to right
        if right_w > 0 {
            let right_start = w.saturating_sub(right_w);
            let right_display = if right_w > w {
                let max_chars = w as usize;
                self.right.chars().take(max_chars).collect::<String>()
            } else {
                self.right.clone()
            };
            ctx.buffer
                .draw_text(x + right_start, y, &right_display, base_style);
        }

        // Center segment — centered
        if center_w > 0 {
            let available = w.saturating_sub(left_w).saturating_sub(right_w);
            if center_w <= available {
                let center_start = left_w + (available.saturating_sub(center_w)) / 2;
                ctx.buffer
                    .draw_text(x + center_start, y, &self.center, base_style);
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
