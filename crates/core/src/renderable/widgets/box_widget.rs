//! BoxWidget — the fundamental container widget.
//!
//! Renders as a bordered or borderless rectangular area with optional
//! background fill, title text, and child layout. This is the primary
//! building block for composing TUI layouts.

use crate::buffer::{BoxOptions, BoxSides, BoxStyle, TitleAlign};
use crate::{Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

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

    /// All chars are null (invisible). Use as base for custom borders.
    pub fn empty() -> Self {
        Self {
            top_left: '\0',
            top_right: '\0',
            bottom_left: '\0',
            bottom_right: '\0',
            horizontal: '\0',
            vertical: '\0',
        }
    }

    /// OpenCode SplitBorder pattern: only a left vertical `┃` with `╹` bottom-left.
    pub fn split_left() -> Self {
        Self {
            vertical: '┃',
            bottom_left: '╹',
            ..Self::empty()
        }
    }

    /// OpenCode SplitBorder pattern: only a left vertical `┃`, no corners.
    pub fn split_left_no_bottom() -> Self {
        Self {
            vertical: '┃',
            ..Self::empty()
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
    user_style: LayoutStyle,
    base_padding: (f32, f32, f32, f32),
    effective_style: LayoutStyle,
    bg: Option<Rgba>,
    border: Option<BorderStyle>,
    title: Option<String>,
    title_align: TitleAlign,
    title_color: Option<Rgba>,
    bottom_title: Option<String>,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    has_focused_descendant: bool,
}

impl BoxWidget {
    pub fn new(style: LayoutStyle) -> Self {
        let eff = style.clone();
        Self {
            user_style: style,
            base_padding: (0.0, 0.0, 0.0, 0.0),
            effective_style: eff,
            bg: None,
            border: None,
            title: None,
            title_align: TitleAlign::Left,
            title_color: None,
            bottom_title: None,
            overflow: Overflow::Visible,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
            has_focused_descendant: false,
        }
    }

    fn recompute_effective_style(&mut self) {
        let mut eff = self.user_style.clone();
        let (pt, pr, pb, pl) = self.base_padding;
        let mut top = pt;
        let mut right = pr;
        let mut bottom = pb;
        let mut left = pl;

        if let Some(ref b) = self.border {
            if b.sides.top && b.chars.horizontal != '\0' {
                top += 1.0;
            }
            if b.sides.bottom && b.chars.horizontal != '\0' {
                bottom += 1.0;
            }
            if b.sides.left && b.chars.vertical != '\0' {
                left += 1.0;
            }
            if b.sides.right && b.chars.vertical != '\0' {
                right += 1.0;
            }
        }

        eff = eff.padding(top, right, bottom, left);
        self.effective_style = eff;
    }

    pub fn layout_style(&self) -> &LayoutStyle {
        &self.effective_style
    }

    pub fn base_padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.base_padding = (top, right, bottom, left);
        self.recompute_effective_style();
        self
    }

    pub fn background(mut self, color: Rgba) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        self.border = Some(border);
        self.recompute_effective_style();
        self
    }

    pub fn border_rounded(mut self, color: Rgba) -> Self {
        self.border = Some(BorderStyle {
            chars: BorderChars::rounded(),
            color,
            focused_color: None,
            sides: BorderSides::all(),
        });
        self.recompute_effective_style();
        self
    }

    pub fn border_custom(mut self, chars: BorderChars, color: Rgba, sides: BorderSides) -> Self {
        self.border = Some(BorderStyle {
            chars,
            color,
            focused_color: None,
            sides,
        });
        self.recompute_effective_style();
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

    pub fn title_color(mut self, color: Rgba) -> Self {
        self.title_color = Some(color);
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
        self.recompute_effective_style();
    }

    fn border_color(&self) -> Rgba {
        if let Some(ref b) = self.border {
            if self.focused || self.has_focused_descendant {
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

impl Behavior for BoxWidget {
    fn style(&self) -> &LayoutStyle {
        &self.effective_style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.user_style
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: self.overflow,
            visible: self.visible,
            opacity: self.opacity,
        }
    }

    fn set_focus_state(&mut self, focused: bool, has_focused_descendant: bool) {
        self.focused = focused;
        self.has_focused_descendant = has_focused_descendant;
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
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

            let fill = self.bg.filter(|bg| bg.a > 0.0);

            let options = BoxOptions {
                style: bs,
                sides: BoxSides {
                    top: border.sides.top,
                    right: border.sides.right,
                    bottom: border.sides.bottom,
                    left: border.sides.left,
                },
                fill,
                title: self.title.clone(),
                bottom_title: self.bottom_title.clone(),
                title_align: self.title_align,
                title_style: self.title_color.map(Style::fg),
            };

            ctx.buffer.draw_box_with_options(x, y, w, h, options);
        } else if let Some(bg) = self.bg {
            if bg.a > 0.0 {
                ctx.buffer.fill_rect(x, y, w, h, bg);
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
