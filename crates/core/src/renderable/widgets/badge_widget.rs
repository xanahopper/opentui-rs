//! BadgeWidget — colored label/tag widget.
//!
//! Renders a small colored label with configurable background, foreground,
//! padding, and border radius (using bracket characters).

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::{Cell, Rgba, Style};

/// Badge shape style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeShape {
    Plain,
    Bracketed,
    Padded,
}

/// Styling for the badge widget.
#[derive(Debug, Clone)]
pub struct BadgeStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub shape: BadgeShape,
    pub left_char: char,
    pub right_char: char,
    pub pad_char: char,
}

impl Default for BadgeStyle {
    fn default() -> Self {
        Self {
            fg: Rgba::WHITE,
            bg: Rgba::from_rgb_u8(60, 60, 70),
            shape: BadgeShape::Padded,
            left_char: '[',
            right_char: ']',
            pad_char: ' ',
        }
    }
}

impl BadgeStyle {
    pub fn success() -> Self {
        Self {
            fg: Rgba::WHITE,
            bg: Rgba::from_rgb_u8(40, 120, 60),
            shape: BadgeShape::Padded,
            ..Default::default()
        }
    }

    pub fn warning() -> Self {
        Self {
            fg: Rgba::BLACK,
            bg: Rgba::from_rgb_u8(200, 160, 40),
            shape: BadgeShape::Padded,
            ..Default::default()
        }
    }

    pub fn error() -> Self {
        Self {
            fg: Rgba::WHITE,
            bg: Rgba::from_rgb_u8(160, 40, 40),
            shape: BadgeShape::Padded,
            ..Default::default()
        }
    }

    pub fn info() -> Self {
        Self {
            fg: Rgba::WHITE,
            bg: Rgba::from_rgb_u8(40, 100, 160),
            shape: BadgeShape::Padded,
            ..Default::default()
        }
    }
}

pub struct BadgeWidget {
    style: LayoutStyle,
    text: String,
    badge_style: BadgeStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl BadgeWidget {
    pub fn new(style: LayoutStyle, text: impl Into<String>) -> Self {
        Self {
            style,
            text: text.into(),
            badge_style: BadgeStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn badge_style(mut self, style: BadgeStyle) -> Self {
        self.badge_style = style;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    fn content_width(&self) -> u32 {
        let text_len = self.text.chars().count() as u32;
        match self.badge_style.shape {
            BadgeShape::Plain => text_len,
            BadgeShape::Bracketed => text_len + 2,
            BadgeShape::Padded => text_len + 4,
        }
    }

    fn render_content(&self, ctx: &mut RenderContext<'_>, x: u32, y: u32, max_w: u32) {
        let s = &self.badge_style;
        let cell_style = Style::builder().fg(s.fg).bg(s.bg).build();

        let mut cx = x;
        let mut drawn = 0u32;

        match s.shape {
            BadgeShape::Bracketed => {
                if drawn < max_w {
                    ctx.buffer.set(cx, y, Cell::new(s.left_char, cell_style));
                    cx += 1;
                    drawn += 1;
                }
            }
            BadgeShape::Padded => {
                for _ in 0..2 {
                    if drawn >= max_w {
                        break;
                    }
                    ctx.buffer.set(cx, y, Cell::new(s.pad_char, cell_style));
                    cx += 1;
                    drawn += 1;
                }
            }
            BadgeShape::Plain => {}
        }

        let avail = max_w.saturating_sub(drawn);
        let text_chars: Vec<char> = self.text.chars().collect();
        let truncated = text_chars.len().min(avail as usize);
        for ch in text_chars.iter().take(truncated) {
            ctx.buffer.set(cx, y, Cell::new(*ch, cell_style));
            cx += 1;
            drawn += 1;
        }

        match s.shape {
            BadgeShape::Bracketed => {
                if cx < x + max_w {
                    ctx.buffer.set(cx, y, Cell::new(s.right_char, cell_style));
                }
            }
            BadgeShape::Padded => {
                for _ in 0..2.min(max_w.saturating_sub(drawn)) {
                    ctx.buffer.set(cx, y, Cell::new(s.pad_char, cell_style));
                    cx += 1;
                }
            }
            BadgeShape::Plain => {}
        }
    }
}

impl Behavior for BadgeWidget {
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

        let content_w = self.content_width().min(w);
        self.render_content(ctx, x, y, content_w);

        let bg_style = Style::builder().bg(self.badge_style.bg).build();
        for col in content_w..w {
            for row in 0..h {
                ctx.buffer.set(x + col, y + row, Cell::new(' ', bg_style));
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
