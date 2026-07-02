//! TextWidget — displays static or dynamic text content.
//!
//! Wraps a [`TextBuffer`] + [`TextBufferView`] to render text with optional
//! word wrapping, truncation, and scrolling within a layout region.

use crate::Style;
use crate::WrapMode;
use crate::text::{TextBuffer, TextBufferView};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub struct TextWidget {
    style: LayoutStyle,
    buffer: TextBuffer,
    wrap_mode: WrapMode,
    scroll_x: u32,
    scroll_y: u32,
    default_style: Style,
    overflow: Overflow,
    focusable: bool,
}

impl TextWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            buffer: TextBuffer::new(),
            wrap_mode: WrapMode::None,
            scroll_x: 0,
            scroll_y: 0,
            default_style: Style::NONE,
            overflow: Overflow::Hidden,
            focusable: false,
        }
    }

    pub fn with_text(style: LayoutStyle, text: &str) -> Self {
        Self {
            buffer: TextBuffer::with_text(text),
            ..Self::new(style)
        }
    }

    pub fn buffer(&self) -> &TextBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    pub fn set_text(&mut self, text: &str) {
        self.buffer = TextBuffer::with_text(text);
    }

    pub fn wrap(mut self, mode: WrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    pub fn default_style(mut self, style: Style) -> Self {
        self.default_style = style;
        self
    }

    pub fn overflow_visible(mut self) -> Self {
        self.overflow = Overflow::Visible;
        self
    }

    pub fn set_scroll(&mut self, x: u32, y: u32) {
        self.scroll_x = x;
        self.scroll_y = y;
    }

    pub fn scroll_y(&self) -> u32 {
        self.scroll_y
    }

    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }
}

impl Behavior for TextWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: self.overflow,
            ..Default::default()
        }
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as i32;
        let y = layout.y as i32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
        }

        let view = TextBufferView::new(&self.buffer)
            .viewport(0, 0, w, h)
            .wrap_mode(self.wrap_mode)
            .scroll(self.scroll_x, self.scroll_y);

        if let Some(ref mut pool) = ctx.grapheme_pool {
            view.render_to_with_pool(ctx.buffer, pool, x, y);
        } else {
            view.render_to(ctx.buffer, x, y);
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
