//! StyledTextWidget — renders inline styled text segments.
//!
//! Each segment has its own foreground/background/bold/italic/underline style.
//! This is the equivalent of OpenCode's `<text>` component with nested `<span>` elements.

use crate as ot;
use crate::{Cell, Rgba, Style};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone)]
pub struct StyledSegment {
    pub text: String,
    pub fg: Rgba,
    pub bg: Option<Rgba>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl StyledSegment {
    pub fn new(text: impl Into<String>, fg: Rgba) -> Self {
        Self {
            text: text.into(),
            fg,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyledTextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug)]
pub struct StyledTextWidget {
    id: WidgetId,
    style: LayoutStyle,
    segments: Vec<StyledSegment>,
    align: StyledTextAlign,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl StyledTextWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            segments: Vec::new(),
            align: StyledTextAlign::Left,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn from_segments(id: WidgetId, style: LayoutStyle, segments: Vec<StyledSegment>) -> Self {
        Self {
            id,
            style,
            segments,
            align: StyledTextAlign::Left,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn set_segments(&mut self, segments: Vec<StyledSegment>) {
        self.segments = segments;
    }

    pub fn segments(&self) -> &[StyledSegment] {
        &self.segments
    }

    pub fn align(mut self, align: StyledTextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn overflow_visible(mut self) -> Self {
        self.overflow = Overflow::Visible;
        self
    }
}

impl Widget for StyledTextWidget {
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
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 || self.segments.is_empty() {
            return;
        }

        let total_width: usize = self
            .segments
            .iter()
            .map(|s| ot::unicode::display_width(&s.text))
            .sum();

        let start_col = match self.align {
            StyledTextAlign::Left => x,
            StyledTextAlign::Center => x + ((w as usize).saturating_sub(total_width) / 2) as u32,
            StyledTextAlign::Right => x + (w as usize).saturating_sub(total_width) as u32,
        };

        let max_col = x + w;
        let mut col = start_col;

        for row_offset in 0..h {
            if row_offset > 0 {
                col = x;
            }
            let row = y + row_offset;

            for segment in &self.segments {
                if col >= max_col {
                    break;
                }

                let mut builder = Style::builder()
                    .fg(segment.fg)
                    .bg(segment.bg.unwrap_or(Rgba::TRANSPARENT));
                if segment.bold {
                    builder = builder.bold();
                }
                if segment.italic {
                    builder = builder.italic();
                }
                if segment.underline {
                    builder = builder.underline();
                }
                let seg_style = builder.build();

                for (grapheme, dw) in ot::unicode::split_graphemes_with_widths(&segment.text) {
                    if col >= max_col {
                        break;
                    }
                    let dw = dw as u32;
                    if dw == 0 {
                        continue;
                    }
                    if let Some(ch) = grapheme.chars().next() {
                        ctx.buffer.set_blended(col, row, Cell::new(ch, seg_style));
                    }
                    col += dw;
                }
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
