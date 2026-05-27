//! TextLineWidget — renders a single line of styled text with optional background fill.
//!
//! This is the declarative equivalent of OpenCode's `<text>` element.
//! Renders text at position (0,0) within the allocated layout rectangle,
//! filling remaining width with background color.

use opentui_rust as ot;
use opentui_rust::{Rgba, Style};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextLineAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct TextLineWidget {
    id: WidgetId,
    style: LayoutStyle,
    text: String,
    fg: Rgba,
    bg: Option<Rgba>,
    bold: bool,
    italic: bool,
    underline: bool,
    align: TextLineAlign,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl TextLineWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            text: String::new(),
            fg: Rgba::new(1.0, 1.0, 1.0, 1.0),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            align: TextLineAlign::Left,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn with_text(id: WidgetId, style: LayoutStyle, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::new(id, style)
        }
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
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

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }

    pub fn align(mut self, align: TextLineAlign) -> Self {
        self.align = align;
        self
    }

    pub fn overflow_visible(mut self) -> Self {
        self.overflow = Overflow::Visible;
        self
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl Widget for TextLineWidget {
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

        if let Some(bg) = self.bg {
            if bg.a > 0.0 {
                ctx.buffer.fill_rect(x, y, w, h, bg);
            }
        }

        if self.text.is_empty() {
            return;
        }

        let text_width = ot::unicode::display_width(&self.text) as u32;

        let start_x = match self.align {
            TextLineAlign::Left => x,
            TextLineAlign::Center => {
                x + w.saturating_sub(text_width) / 2
            }
            TextLineAlign::Right => x + w.saturating_sub(text_width),
        };

        let mut builder = Style::builder().fg(self.fg);
        if let Some(bg) = self.bg {
            builder = builder.bg(bg);
        }
        if self.bold {
            builder = builder.bold();
        }
        if self.italic {
            builder = builder.italic();
        }
        if self.underline {
            builder = builder.underline();
        }
        let style = builder.build();

        let max_col = x + w;
        let mut col = start_x;

        for ch in self.text.chars() {
            if col >= max_col {
                break;
            }
            let dw = ot::unicode::display_width(ch.to_string().as_str()) as u32;
            if dw == 0 {
                continue;
            }
            ctx.buffer.set_blended(col, y, ot::Cell::new(ch, style));
            col += dw;
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
