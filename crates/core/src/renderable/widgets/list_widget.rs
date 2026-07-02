//! ListWidget — selectable virtual list widget.
//!
//! Renders a scrollable list of items, only drawing those within the visible
//! viewport. Supports keyboard navigation (Up/Down/Home/End/PageUp/PageDown),
//! mouse click selection, and optional scrollbar.

use crate::OptimizedBuffer;
use crate::renderer::HitGrid;

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::list::{ItemRenderer, VirtualList, VirtualListState};
use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::node::Overflow;
use crate::scroll::ScrollBarStyle;

#[derive(Debug, Clone)]
pub struct ListWidget {
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
    pub fn new(style: LayoutStyle) -> Self {
        Self {
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

impl Behavior for ListWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: Overflow::Hidden,
            ..FrameworkDefaults::default()
        }
    }

    fn render_self(&mut self, _ctx: &mut RenderContext<'_>, _layout: &ComputedLayout) {}

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
