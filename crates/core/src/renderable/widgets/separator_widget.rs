//! SeparatorWidget — renders a horizontal separator line.
//!
//! Fills its layout row with a repeated character (default `─`),
//! like OpenCode's `<box>` with a single line of `─` characters.

use crate::{Cell, Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

#[derive(Debug, Clone)]
pub struct SeparatorWidget {
    style: LayoutStyle,
    char: char,
    fg: Rgba,
    bg: Option<Rgba>,
    overflow: Overflow,
}

impl SeparatorWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            char: '\u{2500}',
            fg: Rgba::new(0.3, 0.3, 0.35, 1.0),
            bg: None,
            overflow: Overflow::Hidden,
        }
    }

    pub fn char_(mut self, ch: char) -> Self {
        self.char = ch;
        self
    }

    pub fn fg(mut self, color: Rgba) -> Self {
        self.fg = color;
        self
    }

    pub fn bg(mut self, color: Rgba) -> Self {
        self.bg = Some(color);
        self
    }
}

impl Behavior for SeparatorWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            overflow: self.overflow,
            ..Default::default()
        }
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
        }

        let mut builder = Style::builder().fg(self.fg);
        if let Some(bg) = self.bg {
            builder = builder.bg(bg);
        }
        let style = builder.build();

        for row in 0..h {
            for col in 0..w {
                ctx.buffer
                    .set(x + col, y + row, Cell::new(self.char, style));
            }
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
