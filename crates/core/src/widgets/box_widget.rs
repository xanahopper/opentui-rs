//! BoxWidget — the fundamental container widget.
//!
//! Renders as a bordered or borderless rectangular area with optional
//! background fill, title text, and child layout. This is the primary
//! building block for composing TUI layouts.

use opentui_rust as ot;
use opentui_rust::buffer::{BoxOptions, BoxSides, BoxStyle, TitleAlign};
use opentui_rust::{Rgba, Style};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub chars: BorderChars,
    pub color: Rgba,
    pub focused_color: Option<Rgba>,
    pub sides: BorderSides,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            chars: BorderChars::rounded(),
            color: Rgba::new(0.3, 0.3, 0.35, 1.0),
            focused_color: None,
            sides: BorderSides::all(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorderChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl BorderChars {
    pub fn rounded() -> Self {
        Self {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
        }
    }

    pub fn single() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
        }
    }

    pub fn double() -> Self {
        Self {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
        }
    }

    pub fn thick() -> Self {
        Self {
            top_left: '┏',
            top_right: '┓',
            bottom_left: '┗',
            bottom_right: '┛',
            horizontal: '━',
            vertical: '┃',
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorderSides {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl BorderSides {
    pub fn all() -> Self {
        Self {
            top: true,
            right: true,
            bottom: true,
            left: true,
        }
    }

    pub fn none() -> Self {
        Self {
            top: false,
            right: false,
            bottom: false,
            left: false,
        }
    }

    pub fn left_only() -> Self {
        Self {
            top: false,
            right: false,
            bottom: false,
            left: true,
        }
    }

    pub fn left_right() -> Self {
        Self {
            top: false,
            right: true,
            bottom: false,
            left: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxWidget {
    id: WidgetId,
    style: LayoutStyle,
    bg: Option<Rgba>,
    border: Option<BorderStyle>,
    title: Option<String>,
    title_align: TitleAlign,
    bottom_title: Option<String>,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl BoxWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            bg: None,
            border: None,
            title: None,
            title_align: TitleAlign::Left,
            bottom_title: None,
            overflow: Overflow::Visible,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn background(mut self, color: Rgba) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        self.border = Some(border);
        self
    }

    pub fn border_rounded(mut self, color: Rgba) -> Self {
        self.border = Some(BorderStyle {
            chars: BorderChars::rounded(),
            color,
            focused_color: None,
            sides: BorderSides::all(),
        });
        self
    }

    pub fn border_focused_color(mut self, color: Rgba) -> Self {
        if let Some(ref mut b) = self.border {
            b.focused_color = Some(color);
        }
        self
    }

    pub fn set_border_focused_color(&mut self, color: Rgba) {
        if let Some(ref mut b) = self.border {
            b.focused_color = Some(color);
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn title_align(mut self, align: TitleAlign) -> Self {
        self.title_align = align;
        self
    }

    pub fn bottom_title(mut self, title: impl Into<String>) -> Self {
        self.bottom_title = Some(title.into());
        self
    }

    pub fn overflow_hidden(mut self) -> Self {
        self.overflow = Overflow::Hidden;
        self
    }

    pub fn hide(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn set_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    pub fn set_bg(&mut self, color: Option<Rgba>) {
        self.bg = color;
    }

    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    pub fn set_border(&mut self, border: Option<BorderStyle>) {
        self.border = border;
    }

    fn border_color(&self) -> Rgba {
        if let Some(ref b) = self.border {
            if self.focused {
                if let Some(fc) = b.focused_color {
                    return fc;
                }
            }
            b.color
        } else {
            Rgba::new(0.3, 0.3, 0.35, 1.0)
        }
    }
}

impl Widget for BoxWidget {
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

        if let Some(ref border) = self.border {
            let border_color = self.border_color();
            let style = Style::builder().fg(border_color).build();

            let bs = BoxStyle {
                top_left: border.chars.top_left,
                top_right: border.chars.top_right,
                bottom_left: border.chars.bottom_left,
                bottom_right: border.chars.bottom_right,
                horizontal: border.chars.horizontal,
                vertical: border.chars.vertical,
                style,
            };

            let options = BoxOptions {
                style: bs,
                sides: BoxSides {
                    top: border.sides.top,
                    right: border.sides.right,
                    bottom: border.sides.bottom,
                    left: border.sides.left,
                },
                fill: None,
                title: self.title.clone(),
                title_align: self.title_align,
            };

            ctx.buffer.draw_box_with_options(x, y, w, h, options);
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
