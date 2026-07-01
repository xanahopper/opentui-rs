//! Virtual list with viewport culling for large datasets.
//!
//! `VirtualList` only renders items that fall within the visible viewport,
//! making it efficient for datasets with thousands of entries.
//!
//! The list delegates item rendering to a closure or an implementor of the
//! `ItemRenderer` trait. It integrates with `opentui_rust`'s `HitGrid` for
//! click detection on individual items.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::bool_to_int_with_if)]

use crate as ot;
use crate::OptimizedBuffer;
use crate::renderer::HitGrid;

use crate::scroll::{ScrollBarRenderer, ScrollBarStyle, ScrollState};

pub trait ItemRenderer {
    fn item_count(&self) -> usize;

    fn item_height(&self, index: usize) -> u32;

    fn render_item(
        &self,
        buffer: &mut OptimizedBuffer,
        index: usize,
        x: u32,
        y: u32,
        width: u32,
        selected: bool,
    );
}

pub struct FixedHeightItemRenderer<F>
where
    F: Fn(&mut OptimizedBuffer, usize, u32, u32, u32, bool),
{
    pub count: usize,
    pub height: u32,
    pub render_fn: F,
}

impl<F> ItemRenderer for FixedHeightItemRenderer<F>
where
    F: Fn(&mut OptimizedBuffer, usize, u32, u32, u32, bool),
{
    fn item_count(&self) -> usize {
        self.count
    }

    fn item_height(&self, _index: usize) -> u32 {
        self.height
    }

    fn render_item(
        &self,
        buffer: &mut OptimizedBuffer,
        index: usize,
        x: u32,
        y: u32,
        width: u32,
        selected: bool,
    ) {
        (self.render_fn)(buffer, index, x, y, width, selected);
    }
}

#[derive(Debug, Clone)]
pub struct VirtualListState {
    pub scroll: ScrollState,
    pub selected: Option<usize>,
    pub focused: bool,
}

impl VirtualListState {
    pub fn new() -> Self {
        Self {
            scroll: ScrollState::new(),
            selected: None,
            focused: false,
        }
    }

    pub fn select(&mut self, index: Option<usize>, _total: usize, item_height: u32) {
        self.selected = index;
        if let Some(i) = index {
            let item_top = i as f32 * item_height as f32;
            let item_bottom = item_top + item_height as f32;
            let vp_bottom = self.scroll.offset_y + self.scroll.viewport_height as f32;

            if item_top < self.scroll.offset_y {
                self.scroll.scroll_to(item_top);
            } else if item_bottom > vp_bottom {
                self.scroll
                    .scroll_to(item_bottom - self.scroll.viewport_height as f32);
            }
        }
    }

    pub fn select_next(&mut self, total: usize, item_height: u32) {
        let current = self.selected.unwrap_or(0);
        if current + 1 < total {
            self.select(Some(current + 1), total, item_height);
        }
    }

    pub fn select_prev(&mut self, total: usize, item_height: u32) {
        let current = self.selected.unwrap_or(0);
        if current > 0 {
            self.select(Some(current - 1), total, item_height);
        }
    }

    pub fn handle_key(&mut self, key: &ot::KeyEvent, total: usize, item_height: u32) {
        match key.code {
            ot::KeyCode::Up => self.select_prev(total, item_height),
            ot::KeyCode::Down => self.select_next(total, item_height),
            ot::KeyCode::Home => self.select(Some(0), total, item_height),
            ot::KeyCode::End if total > 0 => {
                self.select(Some(total - 1), total, item_height);
            }
            ot::KeyCode::PageUp => {
                let page = self.scroll.viewport_height / item_height.max(1);
                let current = self.selected.unwrap_or(0);
                let new = current.saturating_sub(page as usize);
                self.select(Some(new), total, item_height);
            }
            ot::KeyCode::PageDown => {
                let page = self.scroll.viewport_height / item_height.max(1);
                let current = self.selected.unwrap_or(0);
                let new = (current + page as usize).min(total.saturating_sub(1));
                self.select(Some(new), total, item_height);
            }
            _ => {}
        }
    }

    pub fn handle_mouse_click(
        &mut self,
        x: u32,
        y: u32,
        viewport_x: u32,
        viewport_y: u32,
        total: usize,
        item_height: u32,
    ) -> Option<usize> {
        if x < viewport_x || y < viewport_y {
            return None;
        }
        let rel_y = y - viewport_y;
        let offset_items = (self.scroll.offset_y / item_height.max(1) as f32).floor() as usize;
        let rel_item = rel_y / item_height.max(1);
        let index = offset_items + rel_item as usize;
        if index < total {
            self.selected = Some(index);
            Some(index)
        } else {
            None
        }
    }
}

impl Default for VirtualListState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VirtualList;

impl VirtualList {
    pub fn render(
        buffer: &mut OptimizedBuffer,
        mut hit_grid: Option<&mut HitGrid>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        state: &mut VirtualListState,
        renderer: &dyn ItemRenderer,
        show_scrollbar: bool,
        scrollbar_style: Option<&ScrollBarStyle>,
    ) {
        let total = renderer.item_count();
        if total == 0 || height == 0 || width == 0 {
            return;
        }

        let total_content_height: f32 = (0..total).map(|i| renderer.item_height(i) as f32).sum();

        state.scroll.set_content_height(total_content_height);
        state.scroll.set_viewport_height(height);

        let clip = ot::buffer::ClipRect::new(x as i32, y as i32, width, height);
        buffer.push_scissor(clip);

        let mut current_y: f32 = 0.0;
        let scroll_offset = state.scroll.offset_y;
        let viewport_bottom = scroll_offset + height as f32;

        for i in 0..total {
            let item_h = renderer.item_height(i) as f32;
            let item_top = current_y;
            let item_bottom = current_y + item_h;

            if item_bottom > scroll_offset && item_top < viewport_bottom {
                let draw_y = y + (item_top - scroll_offset).round() as u32;
                let selected = state.selected == Some(i);

                renderer.render_item(buffer, i, x, draw_y, width, selected);

                if let Some(ref mut grid) = hit_grid {
                    grid.register(
                        x,
                        draw_y,
                        width.saturating_sub(u32::from(show_scrollbar)),
                        item_h as u32,
                        i as u32,
                    );
                }
            }

            current_y = item_bottom;
        }

        buffer.pop_scissor();

        if show_scrollbar {
            let style = scrollbar_style.cloned().unwrap_or_default();
            let sb_x = x + width - style.width;
            ScrollBarRenderer::render(buffer, sb_x, y, height, &state.scroll, &style);
        }
    }
}
