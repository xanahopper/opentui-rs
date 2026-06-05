//! TextWidget — unified text rendering widget.
//!
//! Handles all text display: single-line, multi-line, plain, and styled.
//! Wraps a [`TextBuffer`] + [`TextBufferView`] to render text with optional
//! word wrapping, bg fill modes, and per-range style highlights.

use opentui_rust as ot;
use opentui_rust::text::{TextBuffer, TextBufferView};
use ot::{Rgba, Style, WrapMode};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::view::element::Element;
use crate::view::props::{BgFill, Props};
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
    wrap: WrapMode,
    bg_fill: BgFill,
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
            wrap: WrapMode::None,
            bg_fill: BgFill::None,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn with_text(id: WidgetId, style: LayoutStyle, text: &str) -> Self {
        Self {
            buffer: TextBuffer::with_text(text),
            ..Self::new(id, style)
        }
    }

    pub fn from_element<M>(id: WidgetId, elem: &Element<M>) -> Self {
        let Props::Text(ref props) = elem.props else {
            return Self::new(id, elem.layout.clone());
        };

        let mut buffer = TextBuffer::with_text(&props.content);

        let mut builder = Style::builder().fg(props.fg);
        if let Some(bg) = props.bg {
            if props.bg_fill == BgFill::Text {
                builder = builder.bg(bg);
            }
        }
        if props.bold {
            builder = builder.bold();
        }
        if props.italic {
            builder = builder.italic();
        }
        if props.underline {
            builder = builder.underline();
        }
        buffer.set_default_style(builder.build());

        for &(start, end, ref style) in &props.highlights {
            buffer.add_highlight(start..end, *style, 0);
        }

        Self {
            id,
            style: elem.layout.clone(),
            buffer,
            wrap: props.wrap,
            bg_fill: props.bg_fill,
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
        self.wrap = mode;
        self
    }

    pub fn bg_fill(mut self, mode: BgFill) -> Self {
        self.bg_fill = mode;
        self
    }

    pub fn default_style(mut self, style: Style) -> Self {
        self.buffer.set_default_style(style);
        self
    }

    pub fn overflow_visible(mut self) -> Self {
        self.overflow = Overflow::Visible;
        self
    }

    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    fn bg_color(&self) -> Option<Rgba> {
        self.buffer.default_style().bg
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

        if self.bg_fill == BgFill::Block {
            if let Some(bg) = self.bg_color() {
                if bg.a > 0.0 {
                    ctx.buffer.fill_rect(x as u32, y as u32, w, h, bg);
                }
            }
        }

        if self.buffer.is_empty() {
            return;
        }

        let view = TextBufferView::new(&self.buffer)
            .viewport(0, 0, w, h)
            .wrap_mode(self.wrap);

        if let Some(pool) = ctx.grapheme_pool.take() {
            view.render_to_with_pool(ctx.buffer, pool, x, y);
            ctx.grapheme_pool = Some(pool);
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

    fn intrinsic_size(&self) -> Option<(f32, f32)> {
        if self.buffer.is_empty() {
            return None;
        }

        let mut max_width: usize = 0;
        let line_count = self.buffer.len_lines();
        for line in self.buffer.lines() {
            let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
            let w = ot::unicode::display_width(trimmed);
            max_width = max_width.max(w);
        }
        Some((max_width as f32, line_count.max(1) as f32))
    }

    fn text_content(&self) -> Option<String> {
        if self.buffer.is_empty() {
            return None;
        }
        Some(self.buffer.to_string())
    }
}
