//! ScrollViewWidget — scrollable container widget.
//!
//! Wraps a viewport that clips its children and supports vertical scrolling
//! with an optional scrollbar. This widget manages its own `ScrollState` and
//! handles mouse wheel / keyboard scroll events.

#![allow(clippy::float_cmp)]

use crate as ot;

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::scroll::{ScrollBarRenderer, ScrollBarStyle, ScrollState};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone)]
pub struct ScrollViewWidget {
    id: WidgetId,
    style: LayoutStyle,
    state: ScrollState,
    scrollbar: bool,
    scrollbar_style: ScrollBarStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl ScrollViewWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            state: ScrollState::new(),
            scrollbar: true,
            scrollbar_style: ScrollBarStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn state(&self) -> &ScrollState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ScrollState {
        &mut self.state
    }

    pub fn scrollbar(mut self, show: bool) -> Self {
        self.scrollbar = show;
        self
    }

    pub fn scrollbar_style(mut self, style: ScrollBarStyle) -> Self {
        self.scrollbar_style = style;
        self
    }

    pub fn content_height(mut self, h: f32) -> Self {
        self.state.set_content_height(h);
        self
    }

    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    pub fn scroll_to(&mut self, y: f32) {
        self.state.scroll_to(y);
    }

    pub fn scroll_by(&mut self, delta: f32) {
        self.state.scroll_by(delta);
    }
}

impl Widget for ScrollViewWidget {
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

        if w == 0 || h == 0 {
            return;
        }

        // Push scissor clip for viewport
        let clip = ot::buffer::ClipRect::new(x as i32, y as i32, w, h);
        ctx.buffer.push_scissor(clip);

        // Children are rendered by WidgetTree with the scissor active.
        // The actual child rendering happens via WidgetTree's render commands,
        // so this widget just sets up the scissor + scrollbar.

        ctx.buffer.pop_scissor();

        // Draw scrollbar on top
        if self.scrollbar && self.state.content_height > 0.0 {
            let sb_x = x + w - self.scrollbar_style.width;
            ScrollBarRenderer::render(ctx.buffer, sb_x, y, h, &self.state, &self.scrollbar_style);
        }
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn opacity(&self) -> f32 {
        self.opacity
    }

    fn overflow(&self) -> Overflow {
        Overflow::Hidden
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

    fn handle_key(&mut self, key: &ot::KeyEvent) -> bool {
        let before = self.state.offset_y;
        self.state.handle_key(key);
        self.state.offset_y != before
    }

    fn handle_mouse(&mut self, mouse: &ot::MouseEvent) -> bool {
        let before = self.state.offset_y;
        self.state.handle_mouse(mouse);
        self.state.offset_y != before
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
