//! FillWidget — fills its layout rectangle with a solid background color.
//!
//! The declarative equivalent of an empty `<box>` with just a `backgroundColor`.
//! Used for spacers, padding rows, and background fills.

use crate::Rgba;

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

#[derive(Debug, Clone)]
pub struct FillWidget {
    style: LayoutStyle,
    bg: Rgba,
    overflow: Overflow,
}

impl FillWidget {
    pub fn new(style: LayoutStyle, bg: Rgba) -> Self {
        Self {
            style,
            bg,
            overflow: Overflow::Hidden,
        }
    }

    pub fn set_bg(&mut self, bg: Rgba) {
        self.bg = bg;
    }
}

impl Behavior for FillWidget {
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

        if self.bg.a > 0.0 {
            ctx.buffer.fill_rect(x, y, w, h, self.bg);
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
