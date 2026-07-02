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
