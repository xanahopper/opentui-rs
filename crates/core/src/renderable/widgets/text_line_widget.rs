//! TextLineWidget — renders a single line of styled text with optional background fill.
//!
//! This is the declarative equivalent of OpenCode's `<text>` element.
//! Renders text at position (0,0) within the allocated layout rectangle,
//! filling remaining width with background color.

use crate::{Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::view::element::Element;
use crate::view::props::{Props, TextProps};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextLineAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct TextLineWidget {
    style: LayoutStyle,
    text: String,
    fg: Rgba,
    bg: Option<Rgba>,
    bold: bool,
    italic: bool,
    underline: bool,
    align: TextLineAlign,
    overflow: Overflow,
    selectable: bool,
    selection: Option<(usize, usize)>,
    selection_style: Style,
    viewport: Option<(u32, u32, u32, u32, u32)>,
}

impl TextLineWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            text: String::new(),
            fg: Rgba::new(1.0, 1.0, 1.0, 1.0),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            align: TextLineAlign::Left,
            overflow: Overflow::Hidden,
            selectable: true,
            selection: None,
            selection_style: Style::builder().bg(Rgba::from_rgb_u8(60, 60, 120)).build(),
            viewport: None,
        }
    }

    pub fn with_text(style: LayoutStyle, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::new(style)
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
        self.selection = None;
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn from_element(elem: &Element) -> Self {
        let mut widget = Self::new(elem.layout.clone());
        if let Props::Text(ref props) = elem.props {
            widget.apply_text_props(props);
        }
        widget
    }

    pub fn apply_text_props(&mut self, props: &TextProps) {
        self.text.clone_from(&props.content);
        self.fg = props.fg;
        self.bg = props.bg;
        self.bold = props.bold;
        self.italic = props.italic;
        self.underline = props.underline;
        self.align = props.align;
        self.selectable = props.selectable;
        let mut selection = Style::builder().bg(Rgba::from_rgb_u8(60, 60, 120));
        if let Some(fg) = props.selection_fg {
            selection = selection.fg(fg);
        }
        if let Some(bg) = props.selection_bg {
            selection = selection.bg(bg);
        }
        self.selection_style = selection.build();
    }

    fn offset_at_screen_position(&self, position: (i32, i32)) -> usize {
        let Some((_, y, _, height, text_x)) = self.viewport else {
            return 0;
        };
        if position.1 < y as i32 || position.0 <= text_x as i32 {
            return 0;
        }
        if position.1 >= y.saturating_add(height) as i32 {
            return self.text.chars().count();
        }
        let target = (position.0 - text_x as i32) as usize;
        let mut col = 0usize;
        let mut offset = 0usize;
        for (grapheme, width) in crate::unicode::split_graphemes_with_widths(&self.text) {
            if col >= target || target < col + width {
                break;
            }
            col += width;
            offset += grapheme.chars().count();
        }
        offset.min(self.text.chars().count())
    }

    fn selection_intersects(&self, anchor: (i32, i32), focus: (i32, i32)) -> bool {
        let Some((x, y, width, height, _)) = self.viewport else {
            return false;
        };
        let (start, end) = if (anchor.1, anchor.0) <= (focus.1, focus.0) {
            (anchor, focus)
        } else {
            (focus, anchor)
        };
        (start.1, start.0)
            <= (
                y.saturating_add(height.saturating_sub(1)) as i32,
                x.saturating_add(width) as i32,
            )
            && (end.1, end.0) >= (y as i32, x as i32)
    }
}

impl Behavior for TextLineWidget {
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
            self.viewport = None;
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

        let text_width = crate::unicode::display_width(&self.text) as u32;

        let start_x = match self.align {
            TextLineAlign::Left => x,
            TextLineAlign::Center => x + w.saturating_sub(text_width) / 2,
            TextLineAlign::Right => x + w.saturating_sub(text_width),
        };
        self.viewport = Some((x, y, w, h, start_x));

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

        if let Some(pool) = ctx.grapheme_pool.take() {
            ctx.buffer
                .draw_text_with_pool(pool, start_x, y, &self.text, style);
            ctx.grapheme_pool = Some(pool);
        } else {
            let max_col = x + w;
            let mut col = start_x;
            for (grapheme, dw) in crate::unicode::split_graphemes_with_widths(&self.text) {
                if col >= max_col {
                    break;
                }
                let dw = dw as u32;
                if dw == 0 {
                    continue;
                }
                let cell_bg = self.bg.unwrap_or(crate::Rgba::TRANSPARENT);
                if let Some(ch) = grapheme.chars().next() {
                    ctx.buffer.set_blended(col, y, crate::Cell::new(ch, style));
                }
                for i in 1..dw {
                    if col + i < max_col {
                        ctx.buffer
                            .set_blended(col + i, y, crate::Cell::continuation(cell_bg));
                    }
                }
                col += dw;
            }
        }

        if let Some((start, end)) = self.selection {
            let mut col = start_x;
            let mut offset = 0usize;
            for (grapheme, width) in crate::unicode::split_graphemes_with_widths(&self.text) {
                if offset >= start && offset < end {
                    for i in 0..width as u32 {
                        if col + i < x + w
                            && let Some(cell) = ctx.buffer.get_mut(col + i, y)
                        {
                            cell.apply_style(self.selection_style);
                        }
                    }
                }
                col += width as u32;
                offset += grapheme.chars().count();
            }
        }
    }

    fn handle_key(&mut self, _key: &crate::KeyEvent) -> bool {
        false
    }

    fn measure(
        &self,
        _known_width: Option<f32>,
        _known_height: Option<f32>,
        _available_width: Option<f32>,
        _available_height: Option<f32>,
    ) -> Option<(f32, f32)> {
        let w = crate::unicode::display_width(&self.text) as f32;
        Some((w, 1.0))
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
        let anchor_offset = self.offset_at_screen_position(anchor);
        let focus_offset = self.offset_at_screen_position(focus);
        let mut end = anchor_offset.max(focus_offset);
        if !is_start && focus_offset < anchor_offset {
            end = end.saturating_add(1).min(self.text.chars().count());
        }
        self.selection = Some((anchor_offset.min(focus_offset), end));
        anchor_offset != focus_offset
    }

    fn clear_selection(&mut self) {
        self.selection = None;
    }

    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection?;
        (start != end).then(|| self.text.chars().skip(start).take(end - start).collect())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
