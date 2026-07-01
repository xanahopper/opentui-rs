//! TextWidget — displays static or dynamic text content.
//!
//! Wraps a [`TextBuffer`] + [`TextBufferView`] to render text with optional
//! word wrapping, truncation, and scrolling within a layout region.

use crate as ot;
use crate::Style;
use crate::WrapMode;
use crate::text::{TextBuffer, TextBufferView};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub struct TextWidget {
    id: WidgetId,
    style: LayoutStyle,
    buffer: TextBuffer,
    wrap_mode: WrapMode,
    scroll_x: u32,
    scroll_y: u32,
    default_style: Style,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl TextWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            buffer: TextBuffer::new(),
            wrap_mode: WrapMode::None,
            scroll_x: 0,
            scroll_y: 0,
            default_style: Style::NONE,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn with_text(id: WidgetId, style: LayoutStyle, text: &str) -> Self {
        Self {
            id,
            style,
            buffer: TextBuffer::with_text(text),
            wrap_mode: WrapMode::None,
            scroll_x: 0,
            scroll_y: 0,
            default_style: Style::NONE,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
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

impl Widget for TextWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
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
