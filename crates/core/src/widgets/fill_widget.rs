//! FillWidget — fills its layout rectangle with a solid background color.
//!
//! The declarative equivalent of an empty `<box>` with just a `backgroundColor`.
//! Used for spacers, padding rows, and background fills.

use opentui_rust as ot;
use opentui_rust::Rgba;

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone)]
pub struct FillWidget {
    id: WidgetId,
    style: LayoutStyle,
    bg: Rgba,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl FillWidget {
    pub fn new(id: WidgetId, style: LayoutStyle, bg: Rgba) -> Self {
        Self {
            id,
            style,
            bg,
            overflow: Overflow::Hidden,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn set_bg(&mut self, bg: Rgba) {
        self.bg = bg;
    }
}

impl Widget for FillWidget {
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

        if self.bg.a > 0.0 {
            ctx.buffer.fill_rect(x, y, w, h, self.bg);
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
