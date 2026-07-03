//! ScrollBarWidget — standalone scrollbar component.

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::terminal::{MouseButton, MouseEventKind};
use crate::{Cell, KeyCode, KeyEvent, Rgba, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollBarOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollUnit {
    Absolute,
    Viewport,
    Content,
    Step,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollBarWidgetStyle {
    pub track_bg: Rgba,
    pub thumb_fg: Rgba,
    pub thumb_bg: Rgba,
    pub focused_thumb_fg: Rgba,
    pub arrow_fg: Rgba,
    pub arrow_bg: Rgba,
    pub arrow_up: char,
    pub arrow_down: char,
    pub arrow_left: char,
    pub arrow_right: char,
}

impl Default for ScrollBarWidgetStyle {
    fn default() -> Self {
        Self {
            track_bg: Rgba::from_rgb_u8(37, 37, 39),
            thumb_fg: Rgba::from_rgb_u8(154, 158, 163),
            thumb_bg: Rgba::from_rgb_u8(37, 37, 39),
            focused_thumb_fg: Rgba::from_rgb_u8(220, 225, 232),
            arrow_fg: Rgba::WHITE,
            arrow_bg: Rgba::TRANSPARENT,
            arrow_up: '▲',
            arrow_down: '▼',
            arrow_left: '◀',
            arrow_right: '▶',
        }
    }
}

pub struct ScrollBarWidget {
    style: LayoutStyle,
    orientation: ScrollBarOrientation,
    scroll_size: f32,
    scroll_position: f32,
    viewport_size: f32,
    scroll_step: Option<f32>,
    show_arrows: bool,
    auto_hide: bool,
    bar_style: ScrollBarWidgetStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    last_layout: Option<ComputedLayout>,
}

impl ScrollBarWidget {
    pub fn new(style: LayoutStyle, orientation: ScrollBarOrientation) -> Self {
        Self {
            style,
            orientation,
            scroll_size: 0.0,
            scroll_position: 0.0,
            viewport_size: 0.0,
            scroll_step: None,
            show_arrows: false,
            auto_hide: true,
            bar_style: ScrollBarWidgetStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
            last_layout: None,
        }
    }

    pub fn horizontal(style: LayoutStyle) -> Self {
        Self::new(style, ScrollBarOrientation::Horizontal)
    }

    pub fn vertical(style: LayoutStyle) -> Self {
        Self::new(style, ScrollBarOrientation::Vertical)
    }

    pub fn scroll_size(mut self, size: f32) -> Self {
        self.set_scroll_size(size);
        self
    }

    pub fn viewport_size(mut self, size: f32) -> Self {
        self.set_viewport_size(size);
        self
    }

    pub fn scroll_position(mut self, position: f32) -> Self {
        self.set_scroll_position(position);
        self
    }

    pub fn scroll_step(mut self, step: f32) -> Self {
        self.scroll_step = Some(step.max(0.0));
        self
    }

    pub fn show_arrows(mut self, show: bool) -> Self {
        self.show_arrows = show;
        self
    }

    pub fn auto_hide(mut self, auto_hide: bool) -> Self {
        self.auto_hide = auto_hide;
        self
    }

    pub fn bar_style(mut self, style: ScrollBarWidgetStyle) -> Self {
        self.bar_style = style;
        self
    }

    pub fn scroll_size_value(&self) -> f32 {
        self.scroll_size
    }

    pub fn viewport_size_value(&self) -> f32 {
        self.viewport_size
    }

    pub fn scroll_position_value(&self) -> f32 {
        self.scroll_position
    }

    pub fn set_scroll_size(&mut self, size: f32) {
        self.scroll_size = size.max(0.0);
        self.set_scroll_position(self.scroll_position);
    }

    pub fn set_viewport_size(&mut self, size: f32) {
        self.viewport_size = size.max(0.0);
        self.set_scroll_position(self.scroll_position);
    }

    pub fn set_scroll_position(&mut self, position: f32) {
        self.scroll_position = position.round().clamp(0.0, self.scroll_range());
    }

    pub fn scroll_by(&mut self, delta: f32, unit: ScrollUnit) -> bool {
        let multiplier = match unit {
            ScrollUnit::Absolute => 1.0,
            ScrollUnit::Viewport => self.viewport_size,
            ScrollUnit::Content => self.scroll_size,
            ScrollUnit::Step => self.scroll_step.unwrap_or(1.0),
        };
        let before = self.scroll_position;
        self.set_scroll_position(self.scroll_position + delta * multiplier);
        value_changed(self.scroll_position, before)
    }

    fn scroll_range(&self) -> f32 {
        (self.scroll_size - self.viewport_size).max(0.0)
    }

    fn should_render(&self) -> bool {
        self.visible && (!self.auto_hide || self.scroll_size > self.viewport_size)
    }

    fn track_bounds(&self, layout: &ComputedLayout) -> (u32, u32, u32) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let length = match self.orientation {
            ScrollBarOrientation::Horizontal => layout.width.max(0.0) as u32,
            ScrollBarOrientation::Vertical => layout.height.max(0.0) as u32,
        };

        if !self.show_arrows || length <= 2 {
            return (x, y, length);
        }

        match self.orientation {
            ScrollBarOrientation::Horizontal => (x + 1, y, length - 2),
            ScrollBarOrientation::Vertical => (x, y + 1, length - 2),
        }
    }

    fn thumb_metrics(&self, track_len: u32) -> (u32, u32) {
        if track_len == 0 {
            return (0, 0);
        }
        if self.scroll_size <= 0.0 || self.scroll_size <= self.viewport_size {
            return (0, track_len);
        }

        let virtual_track_size = track_len.saturating_mul(2);
        let viewport_size = self.viewport_size.max(1.0);
        let content_size = self.scroll_range() + viewport_size;
        let virtual_thumb_size = (virtual_track_size as f32 * viewport_size / content_size)
            .floor()
            .clamp(1.0, virtual_track_size as f32) as u32;
        let max_thumb_start = virtual_track_size.saturating_sub(virtual_thumb_size);
        let virtual_thumb_start = if self.scroll_range() > 0.0 {
            (self.scroll_position / self.scroll_range() * max_thumb_start as f32).round() as u32
        } else {
            0
        };

        let start = virtual_thumb_start / 2;
        let end = (virtual_thumb_start + virtual_thumb_size).div_ceil(2);
        (start.min(track_len - 1), end.saturating_sub(start).max(1))
    }

    fn render_arrows(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        if !self.show_arrows {
            return;
        }
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;
        if w == 0 || h == 0 {
            return;
        }

        let style = Style::builder()
            .fg(self.bar_style.arrow_fg)
            .bg(self.bar_style.arrow_bg)
            .build();
        match self.orientation {
            ScrollBarOrientation::Horizontal if w >= 2 => {
                ctx.buffer
                    .set(x, y, Cell::new(self.bar_style.arrow_left, style));
                ctx.buffer
                    .set(x + w - 1, y, Cell::new(self.bar_style.arrow_right, style));
            }
            ScrollBarOrientation::Vertical if h >= 2 => {
                ctx.buffer
                    .set(x, y, Cell::new(self.bar_style.arrow_up, style));
                ctx.buffer
                    .set(x, y + h - 1, Cell::new(self.bar_style.arrow_down, style));
            }
            _ => {}
        }
    }

    fn render_track(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let (track_x, track_y, track_len) = self.track_bounds(layout);
        if track_len == 0 {
            return;
        }

        let thickness = match self.orientation {
            ScrollBarOrientation::Horizontal => layout.height.max(0.0) as u32,
            ScrollBarOrientation::Vertical => layout.width.max(0.0) as u32,
        };
        if thickness == 0 {
            return;
        }

        let track_style = Style::builder().bg(self.bar_style.track_bg).build();
        for i in 0..track_len {
            for cross in 0..thickness {
                let (x, y) = match self.orientation {
                    ScrollBarOrientation::Horizontal => (track_x + i, track_y + cross),
                    ScrollBarOrientation::Vertical => (track_x + cross, track_y + i),
                };
                ctx.buffer.set(x, y, Cell::new(' ', track_style));
            }
        }

        let (thumb_start, thumb_len) = self.thumb_metrics(track_len);
        let fg = if self.focused {
            self.bar_style.focused_thumb_fg
        } else {
            self.bar_style.thumb_fg
        };
        let thumb_style = Style::builder().fg(fg).bg(self.bar_style.thumb_bg).build();
        for i in thumb_start..(thumb_start + thumb_len).min(track_len) {
            for cross in 0..thickness {
                let (x, y) = match self.orientation {
                    ScrollBarOrientation::Horizontal => (track_x + i, track_y + cross),
                    ScrollBarOrientation::Vertical => (track_x + cross, track_y + i),
                };
                ctx.buffer.set(x, y, Cell::new('█', thumb_style));
            }
        }
    }

    fn update_from_mouse(&mut self, mouse_x: u32, mouse_y: u32, layout: &ComputedLayout) -> bool {
        let (track_x, track_y, track_len) = self.track_bounds(layout);
        if track_len == 0 || self.scroll_range() <= 0.0 {
            return false;
        }

        let pos = match self.orientation {
            ScrollBarOrientation::Horizontal => mouse_x.saturating_sub(track_x),
            ScrollBarOrientation::Vertical => mouse_y.saturating_sub(track_y),
        }
        .min(track_len);
        let before = self.scroll_position;
        self.set_scroll_position(pos as f32 / track_len as f32 * self.scroll_range());
        value_changed(self.scroll_position, before)
    }

    fn handle_arrow_mouse(&mut self, mouse_x: u32, mouse_y: u32, layout: &ComputedLayout) -> bool {
        if !self.show_arrows {
            return false;
        }
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;
        let is_start_arrow = mouse_x == x && mouse_y == y;
        if is_start_arrow {
            return self.scroll_by(-0.5, ScrollUnit::Viewport);
        }

        let is_end_arrow = match self.orientation {
            ScrollBarOrientation::Horizontal => mouse_y == y && w > 0 && mouse_x == x + w - 1,
            ScrollBarOrientation::Vertical => mouse_x == x && h > 0 && mouse_y == y + h - 1,
        };
        if is_end_arrow {
            return self.scroll_by(0.5, ScrollUnit::Viewport);
        }
        false
    }
}

impl Behavior for ScrollBarWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        self.last_layout = Some(*layout);
        if !self.should_render() {
            return;
        }
        self.render_arrows(ctx, layout);
        self.render_track(ctx, layout);
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: Overflow::Hidden,
            visible: self.visible,
            opacity: self.opacity,
        }
    }

    fn set_focus_state(&mut self, focused: bool, _has_focused_descendant: bool) {
        self.focused = focused;
    }

    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        if key.is_release() || key.ctrl() || key.alt() {
            return false;
        }
        match (self.orientation, key.code) {
            (ScrollBarOrientation::Horizontal, KeyCode::Left | KeyCode::Char('h'))
            | (ScrollBarOrientation::Vertical, KeyCode::Up | KeyCode::Char('k')) => {
                self.scroll_by(-0.2, ScrollUnit::Viewport)
            }
            (ScrollBarOrientation::Horizontal, KeyCode::Right | KeyCode::Char('l'))
            | (ScrollBarOrientation::Vertical, KeyCode::Down | KeyCode::Char('j')) => {
                self.scroll_by(0.2, ScrollUnit::Viewport)
            }
            (_, KeyCode::PageUp) => self.scroll_by(-0.5, ScrollUnit::Viewport),
            (_, KeyCode::PageDown) => self.scroll_by(0.5, ScrollUnit::Viewport),
            (_, KeyCode::Home) => self.scroll_by(-1.0, ScrollUnit::Content),
            (_, KeyCode::End) => self.scroll_by(1.0, ScrollUnit::Content),
            _ => false,
        }
    }

    fn handle_mouse(&mut self, mouse: &crate::MouseEvent) -> bool {
        if !matches!(mouse.button, MouseButton::Left | MouseButton::None) {
            return false;
        }
        if !matches!(
            mouse.kind,
            MouseEventKind::Press | MouseEventKind::Drag | MouseEventKind::DragEnd
        ) {
            return false;
        }
        let Some(layout) = self.last_layout else {
            return false;
        };
        self.handle_arrow_mouse(mouse.x, mouse.y, &layout)
            || self.update_from_mouse(mouse.x, mouse.y, &layout)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn value_changed(a: f32, b: f32) -> bool {
    (a - b).abs() > f32::EPSILON
}
