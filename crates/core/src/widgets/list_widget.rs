//! ListWidget — selectable virtual list widget.
//!
//! Renders a scrollable list of items, only drawing those within the visible
//! viewport. Supports keyboard navigation (Up/Down/Home/End/PageUp/PageDown),
//! mouse click selection, and optional scrollbar.

use opentui_rust as ot;
use opentui_rust::OptimizedBuffer;
use opentui_rust::renderer::HitGrid;

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::list::{ItemRenderer, VirtualList, VirtualListState};
use crate::scroll::ScrollBarStyle;
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

#[derive(Debug, Clone)]
pub struct ListWidget {
    id: WidgetId,
    style: LayoutStyle,
    state: VirtualListState,
    scrollbar: bool,
    scrollbar_style: ScrollBarStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl ListWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            state: VirtualListState::new(),
            scrollbar: true,
            scrollbar_style: ScrollBarStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
        }
    }

    pub fn state(&self) -> &VirtualListState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut VirtualListState {
        &mut self.state
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected
    }

    pub fn scrollbar(mut self, show: bool) -> Self {
        self.scrollbar = show;
        self
    }

    pub fn scrollbar_style(mut self, style: ScrollBarStyle) -> Self {
        self.scrollbar_style = style;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

impl Widget for ListWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render(&mut self, _ctx: &mut RenderContext<'_>, _layout: &ComputedLayout) {}

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

impl ListWidget {
    pub fn render_with_renderer(
        &mut self,
        buffer: &mut OptimizedBuffer,
        hit_grid: Option<&mut HitGrid>,
        layout: &ComputedLayout,
        renderer: &dyn ItemRenderer,
    ) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
        }

        VirtualList::render(
            buffer,
            hit_grid,
            x,
            y,
            w,
            h,
            &mut self.state,
            renderer,
            self.scrollbar,
            Some(&self.scrollbar_style),
        );
    }
}
