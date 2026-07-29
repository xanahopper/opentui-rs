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
    selectable: bool,
    selection: Option<(usize, usize)>,
    selection_style: Style,
    viewport: Option<(i32, i32, u32, u32)>,
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
            selectable: true,
            selection: None,
            selection_style: Style::builder()
                .bg(crate::Rgba::from_rgb_u8(60, 60, 120))
                .build(),
            viewport: None,
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
        self.selection = None;
    }

    pub fn wrap(mut self, mode: WrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    pub fn default_style(mut self, style: Style) -> Self {
        self.default_style = style;
        self.buffer.set_default_style(style);
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

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn selection_style(mut self, style: Style) -> Self {
        self.selection_style = style;
        self
    }

    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection?;
        let (start, end) = (start.min(end), start.max(end));
        if start == end {
            return None;
        }
        Some(self.buffer.rope().slice(start..end).to_string())
    }

    fn offset_at_screen_position(&self, position: (i32, i32)) -> Option<usize> {
        let (x, y, width, height) = self.viewport?;
        if width == 0 || height == 0 {
            return None;
        }
        let local_x = position.0 - x;
        let local_y = position.1 - y;
        if local_x < 0 || local_y < 0 {
            return Some(0);
        }
        let view = TextBufferView::new(&self.buffer)
            .viewport(0, 0, width, height)
            .wrap_mode(self.wrap_mode)
            .scroll(self.scroll_x, self.scroll_y);
        Some(view.offset_at_position(local_x as u32, local_y as u32))
    }

    fn selection_intersects(&self, anchor: (i32, i32), focus: (i32, i32)) -> bool {
        let Some((x, y, width, height)) = self.viewport else {
            return false;
        };
        if width == 0 || height == 0 {
            return false;
        }
        let (start, end) = if (anchor.1, anchor.0) <= (focus.1, focus.0) {
            (anchor, focus)
        } else {
            (focus, anchor)
        };
        let node_start = (y, x);
        let node_end = (y + height.saturating_sub(1) as i32, x + width as i32);
        (start.1, start.0) <= node_end && (end.1, end.0) >= node_start
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
            self.viewport = None;
            return;
        }

        self.viewport = Some((x, y, w, h));
        let mut view = TextBufferView::new(&self.buffer)
            .viewport(0, 0, w, h)
            .wrap_mode(self.wrap_mode)
            .scroll(self.scroll_x, self.scroll_y);

        if let Some((start, end)) = self.selection {
            view.set_selection(start, end, self.selection_style);
        }

        if let Some(ref mut pool) = ctx.grapheme_pool {
            view.render_to_with_pool(ctx.buffer, pool, x, y);
        } else {
            view.render_to(ctx.buffer, x, y);
        }
    }

    fn measure(
        &self,
        known_width: Option<f32>,
        _known_height: Option<f32>,
        available_width: Option<f32>,
        _available_height: Option<f32>,
    ) -> Option<(f32, f32)> {
        let text = self.buffer.to_string();
        if text.is_empty() {
            return Some((0.0, 0.0));
        }

        match self.wrap_mode {
            WrapMode::None => {
                let mut max_w: u32 = 0;
                let mut lines: u32 = 0;
                for line in text.lines() {
                    max_w = max_w.max(crate::unicode::display_width(line) as u32);
                    lines += 1;
                }
                Some((max_w as f32, lines.max(1) as f32))
            }
            WrapMode::Char | WrapMode::Word => {
                let wrap_width = known_width
                    .or(available_width)
                    .filter(|w| *w > 0.0)
                    .map(|w| w as u32);
                let Some(width) = wrap_width else {
                    let mut max_w: u32 = 0;
                    let mut lines: u32 = 0;
                    for line in text.lines() {
                        max_w = max_w.max(crate::unicode::display_width(line) as u32);
                        lines += 1;
                    }
                    return Some((max_w as f32, lines.max(1) as f32));
                };
                let mut total: u32 = 0;
                for line in text.lines() {
                    let line_w = crate::unicode::display_width(line) as u32;
                    total += line_w.div_ceil(width).max(1);
                }
                Some((width as f32, total.max(1) as f32))
            }
        }
    }

    fn handle_key(&mut self, _key: &crate::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _mouse: &crate::MouseEvent) -> bool {
        false
    }

    fn selectable(&self) -> bool {
        self.selectable
    }

    fn update_selection(&mut self, anchor: (i32, i32), focus: (i32, i32), is_start: bool) -> bool {
        if !self.selectable || !self.selection_intersects(anchor, focus) {
            self.selection = None;
            return false;
        }
        let Some(anchor_offset) = self.offset_at_screen_position(anchor) else {
            return false;
        };
        let Some(focus_offset) = self.offset_at_screen_position(focus) else {
            return false;
        };
        let mut end = anchor_offset.max(focus_offset);
        if !is_start && focus_offset < anchor_offset {
            end = end.saturating_add(1).min(self.buffer.len_chars());
        }
        self.selection = Some((anchor_offset.min(focus_offset), end));
        anchor_offset != focus_offset
    }

    fn clear_selection(&mut self) {
        self.selection = None;
    }

    fn selected_text(&self) -> Option<String> {
        TextWidget::selected_text(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
