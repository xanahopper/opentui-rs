use opentui_rust as ot;
use opentui_rust::buffer::{BoxOptions, BoxSides, BoxStyle, TitleAlign};
use opentui_rust::{Rgba, Style};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::view::element::Element;
use crate::view::props::{Props, ViewProps};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};
use crate::widgets::{BorderChars, BorderSides, BorderStyle};

#[derive(Debug, Clone)]
pub struct ViewWidget {
    id: WidgetId,
    layout: LayoutStyle,
    bg: Option<Rgba>,
    border: Option<BorderStyle>,
    title: Option<String>,
    title_align: TitleAlign,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    border_padding: (f32, f32, f32, f32),
    interactive: bool,
    hover_bg: Option<Rgba>,
    hover_fg: Option<Rgba>,
}

impl ViewWidget {
    pub fn new(id: WidgetId, layout: LayoutStyle) -> Self {
        Self {
            id,
            layout,
            bg: None,
            border: None,
            title: None,
            title_align: TitleAlign::Left,
            overflow: Overflow::Visible,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
            border_padding: (0.0, 0.0, 0.0, 0.0),
            interactive: false,
            hover_bg: None,
            hover_fg: None,
        }
    }

    pub fn from_element<M>(id: WidgetId, elem: &Element<M>) -> Self {
        let mut widget = Self::new(id, elem.layout.clone());
        if let Props::View(ref props) = elem.props {
            widget.apply_view_props(props);
        }
        widget
    }

    pub fn apply_view_props(&mut self, props: &ViewProps) {
        self.bg = props.bg;
        self.border.clone_from(&props.border);
        self.title.clone_from(&props.title);
        self.title_align = props.title_align;
        self.overflow = props.overflow;
        self.opacity = props.opacity;
        self.focusable = props.focusable;
        self.visible = props.visible;
        self.interactive = props.interactive;
        self.hover_bg = props.hover_bg;
        self.hover_fg = props.hover_fg;
        self.compute_border_padding();
    }

    fn compute_border_padding(&mut self) {
        let mut top = 0.0_f32;
        let mut right = 0.0_f32;
        let mut bottom = 0.0_f32;
        let mut left = 0.0_f32;
        if let Some(ref b) = self.border {
            if b.sides.top && b.chars.horizontal != '\0' {
                top = 1.0;
            }
            if b.sides.bottom && b.chars.horizontal != '\0' {
                bottom = 1.0;
            }
            if b.sides.left && b.chars.vertical != '\0' {
                left = 1.0;
            }
            if b.sides.right && b.chars.vertical != '\0' {
                right = 1.0;
            }
        }
        self.border_padding = (top, right, bottom, left);
    }

    pub fn background(mut self, color: Rgba) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn border_rounded(mut self, color: Rgba) -> Self {
        self.border = Some(BorderStyle {
            chars: BorderChars::rounded(),
            color,
            focused_color: None,
            sides: BorderSides::all(),
        });
        self.compute_border_padding();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn overflow_hidden(mut self) -> Self {
        self.overflow = Overflow::Hidden;
        self
    }

    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    pub fn set_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn hide(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn interactive(&self) -> bool {
        self.interactive
    }

    fn is_hovered(&self, ctx: &RenderContext<'_>) -> bool {
        ctx.hovered_id == Some(self.id)
    }

    fn effective_bg(&self, ctx: &RenderContext<'_>) -> Option<Rgba> {
        if self.is_hovered(ctx) {
            self.hover_bg.or(self.bg)
        } else {
            self.bg
        }
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

impl Widget for ViewWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn style(&self) -> &LayoutStyle {
        &self.layout
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }

    fn render(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
        }

        let bg = self.effective_bg(ctx);

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

            let fill = bg.filter(|c| c.a > 0.0);

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
                title_align: self.title_align,
            };

            ctx.buffer.draw_box_with_options(x, y, w, h, options);
        } else if let Some(bg) = bg {
            if bg.a > 0.0 {
                ctx.buffer.fill_rect(x, y, w, h, bg);
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
