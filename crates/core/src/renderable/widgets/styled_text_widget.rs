//! StyledTextWidget — renders inline styled text segments.
//!
//! Each segment has its own foreground/background/bold/italic/underline style.
//! This is the equivalent of OpenCode's `<text>` component with nested `<span>` elements.

use crate::{Cell, Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

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
    pub fn new(style: LayoutStyle) -> Self {
        Self {
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

    pub fn from_segments(style: LayoutStyle, segments: Vec<StyledSegment>) -> Self {
        Self {
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

impl Behavior for StyledTextWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
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
            .map(|s| crate::unicode::display_width(&s.text))
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

                for (grapheme, dw) in crate::unicode::split_graphemes_with_widths(&segment.text) {
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

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: self.overflow,
            visible: self.visible,
            opacity: self.opacity,
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
