//! SeparatorWidget — renders a horizontal separator line.
//!
//! Fills its layout row with a repeated character (default `─`),
//! like OpenCode's `<box>` with a single line of `─` characters.

use opentui_rust as ot;
use opentui_rust::{Cell, Rgba, Style};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone)]
pub struct SeparatorWidget {
    id: WidgetId,
    style: LayoutStyle,
    char: char,
    fg: Rgba,
    bg: Option<Rgba>,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl SeparatorWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            char: '\u{2500}',
            fg: Rgba::new(0.3, 0.3, 0.35, 1.0),
            bg: None,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
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

impl Widget for SeparatorWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
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
                ctx.buffer.set(x + col, y + row, Cell::new(self.char, style));
            }
        }
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn opacity(&self) -> f32 {
        self.opacity
    }

    fn overflow(&self) -> Overflow {
        self.overflow
    }

    fn focusable(&self) -> bool {
        self.focusable
    }

    fn focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn handle_key(&mut self, _key: &ot::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _mouse: &ot::MouseEvent) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
