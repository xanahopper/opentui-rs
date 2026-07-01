//! ScrollView with scrollbar and viewport culling.
//!
//! A `ScrollView` is a container that clips its content to a viewport and
//! allows vertical (and eventually horizontal) scrolling. It renders a
//! scrollbar track and thumb to indicate scroll position.
//!
//! Built on top of `opentui_rust`'s `ScissorStack` for clipping and
//! `MouseEvent` for scroll events.

#![allow(clippy::too_many_arguments)]

use crate as ot;
use crate::{OptimizedBuffer, Rgba, Style};

#[derive(Debug, Clone)]
pub struct ScrollState {
    pub offset_y: f32,
    pub content_height: f32,
    pub viewport_height: u32,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset_y: 0.0,
            content_height: 0.0,
            viewport_height: 0,
        }
    }

    pub fn scroll_to(&mut self, y: f32) {
        let max_offset = (self.content_height - self.viewport_height as f32).max(0.0);
        self.offset_y = y.clamp(0.0, max_offset);
    }

    pub fn scroll_by(&mut self, delta: f32) {
        self.scroll_to(self.offset_y + delta);
    }

    pub fn scroll_up(&mut self, amount: f32) {
        self.scroll_by(-amount);
    }

    pub fn scroll_down(&mut self, amount: f32) {
        self.scroll_by(amount);
    }

    pub fn scroll_to_top(&mut self) {
        self.offset_y = 0.0;
    }

    pub fn scroll_to_bottom(&mut self) {
        let max_offset = (self.content_height - self.viewport_height as f32).max(0.0);
        self.offset_y = max_offset;
    }

    pub fn is_at_top(&self) -> bool {
        self.offset_y <= 0.0
    }

    pub fn is_at_bottom(&self) -> bool {
        let max_offset = (self.content_height - self.viewport_height as f32).max(0.0);
        self.offset_y >= max_offset
    }

    pub fn set_content_height(&mut self, height: f32) {
        self.content_height = height;
        self.scroll_to(self.offset_y);
    }

    pub fn set_viewport_height(&mut self, height: u32) {
        self.viewport_height = height;
        self.scroll_to(self.offset_y);
    }

    pub fn handle_mouse(&mut self, event: &ot::MouseEvent) {
        match event.kind {
            ot::terminal::MouseEventKind::ScrollUp => {
                self.scroll_up(3.0);
            }
            ot::terminal::MouseEventKind::ScrollDown => {
                self.scroll_down(3.0);
            }
            _ => {}
        }
    }

    pub fn handle_key(&mut self, key: &ot::KeyEvent) {
        match key.code {
            ot::KeyCode::Up => self.scroll_up(1.0),
            ot::KeyCode::Down => self.scroll_down(1.0),
            ot::KeyCode::PageUp => {
                let amount = (self.viewport_height as f32).min(1.0);
                self.scroll_up(amount);
            }
            ot::KeyCode::PageDown => {
                let amount = (self.viewport_height as f32).min(1.0);
                self.scroll_down(amount);
            }
            ot::KeyCode::Home => self.scroll_to_top(),
            ot::KeyCode::End => self.scroll_to_bottom(),
            _ => {}
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ScrollBarStyle {
    pub track_bg: Rgba,
    pub track_fg: Rgba,
    pub thumb_bg: Rgba,
    pub thumb_fg: Rgba,
    pub width: u32,
}

impl Default for ScrollBarStyle {
    fn default() -> Self {
        Self {
            track_bg: Rgba::new(0.2, 0.2, 0.2, 1.0),
            track_fg: Rgba::new(0.2, 0.2, 0.2, 1.0),
            thumb_bg: Rgba::new(0.5, 0.5, 0.5, 1.0),
            thumb_fg: Rgba::new(0.5, 0.5, 0.5, 1.0),
            width: 1,
        }
    }
}

pub struct ScrollBarRenderer;

impl ScrollBarRenderer {
    pub fn render(
        buffer: &mut OptimizedBuffer,
        x: u32,
        y: u32,
        height: u32,
        state: &ScrollState,
        style: &ScrollBarStyle,
    ) {
        if height == 0 || state.content_height <= 0.0 {
            return;
        }

        for row in 0..height {
            let cell_style = Style::builder()
                .bg(style.track_bg)
                .fg(style.track_fg)
                .build();
            buffer.set(x, y + row, ot::Cell::new(' ', cell_style));
        }

        let thumb_ratio = height as f32 / state.content_height;
        let thumb_height = (height as f32 * thumb_ratio).max(1.0).round() as u32;
        let scrollable = (state.content_height - state.viewport_height as f32).max(0.0);
        let thumb_y = if scrollable > 0.0 {
            let ratio = state.offset_y / scrollable;
            let max_thumb_start = height.saturating_sub(thumb_height);
            (ratio * max_thumb_start as f32).round() as u32
        } else {
            0
        };

        for row in thumb_y..(thumb_y + thumb_height).min(height) {
            let thumb_style = Style::builder()
                .bg(style.thumb_bg)
                .fg(style.thumb_fg)
                .build();
            buffer.set(x, y + row, ot::Cell::new('█', thumb_style));
        }
    }
}

pub struct ScrollView;

impl ScrollView {
    pub fn render_content<F>(
        buffer: &mut OptimizedBuffer,
        viewport_x: u32,
        viewport_y: u32,
        viewport_w: u32,
        viewport_h: u32,
        state: &mut ScrollState,
        show_scrollbar: bool,
        scrollbar_style: Option<&ScrollBarStyle>,
        render_children: F,
    ) where
        F: FnOnce(&mut OptimizedBuffer, u32, u32, u32, u32),
    {
        state.set_viewport_height(viewport_h);

        let clip =
            ot::buffer::ClipRect::new(viewport_x as i32, viewport_y as i32, viewport_w, viewport_h);
        buffer.push_scissor(clip);

        let content_x = viewport_x;
        let content_y = viewport_y.saturating_sub(state.offset_y as u32);
        render_children(buffer, content_x, content_y, viewport_w, viewport_h);

        buffer.pop_scissor();

        if show_scrollbar {
            let style = scrollbar_style.cloned().unwrap_or_default();
            let sb_x = viewport_x + viewport_w - style.width;
            ScrollBarRenderer::render(buffer, sb_x, viewport_y, viewport_h, state, &style);
        }
    }
}
