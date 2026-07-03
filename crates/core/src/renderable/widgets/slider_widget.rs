//! SliderWidget — value selector with smooth thumb rendering.

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::terminal::{MouseButton, MouseEventKind};
use crate::{Cell, KeyCode, KeyEvent, Rgba, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliderOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub struct SliderStyle {
    pub track_bg: Rgba,
    pub thumb_fg: Rgba,
    pub thumb_bg: Rgba,
    pub focused_thumb_fg: Rgba,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_bg: Rgba::from_rgb_u8(37, 37, 39),
            thumb_fg: Rgba::from_rgb_u8(154, 158, 163),
            thumb_bg: Rgba::from_rgb_u8(37, 37, 39),
            focused_thumb_fg: Rgba::from_rgb_u8(220, 225, 232),
        }
    }
}

pub struct SliderWidget {
    style: LayoutStyle,
    orientation: SliderOrientation,
    value: f32,
    min: f32,
    max: f32,
    viewport_size: f32,
    slider_style: SliderStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    last_layout: Option<ComputedLayout>,
}

impl SliderWidget {
    pub fn new(style: LayoutStyle, orientation: SliderOrientation) -> Self {
        let min = 0.0;
        let max = 100.0;
        Self {
            style,
            orientation,
            value: min,
            min,
            max,
            viewport_size: (max - min) * 0.1,
            slider_style: SliderStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
            last_layout: None,
        }
    }

    pub fn horizontal(style: LayoutStyle) -> Self {
        Self::new(style, SliderOrientation::Horizontal)
    }

    pub fn vertical(style: LayoutStyle) -> Self {
        Self::new(style, SliderOrientation::Vertical)
    }

    pub fn value(mut self, value: f32) -> Self {
        self.set_value(value);
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.set_min(min);
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.set_max(max);
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self.value = self.clamp_value(self.value);
        self.viewport_size = self.clamp_viewport_size(self.viewport_size);
        self
    }

    pub fn viewport_size(mut self, size: f32) -> Self {
        self.set_viewport_size(size);
        self
    }

    pub fn slider_style(mut self, style: SliderStyle) -> Self {
        self.slider_style = style;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn value_value(&self) -> f32 {
        self.value
    }

    pub fn min_value(&self) -> f32 {
        self.min
    }

    pub fn max_value(&self) -> f32 {
        self.max
    }

    pub fn viewport_size_value(&self) -> f32 {
        self.viewport_size
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = self.clamp_value(value);
    }

    pub fn set_min(&mut self, min: f32) {
        self.min = min;
        self.value = self.clamp_value(self.value);
        self.viewport_size = self.clamp_viewport_size(self.viewport_size);
    }

    pub fn set_max(&mut self, max: f32) {
        self.max = max;
        self.value = self.clamp_value(self.value);
        self.viewport_size = self.clamp_viewport_size(self.viewport_size);
    }

    pub fn set_viewport_size(&mut self, size: f32) {
        self.viewport_size = self.clamp_viewport_size(size);
    }

    fn clamp_value(&self, value: f32) -> f32 {
        let low = self.min.min(self.max);
        let high = self.min.max(self.max);
        value.clamp(low, high)
    }

    fn clamp_viewport_size(&self, size: f32) -> f32 {
        let range = (self.max - self.min).abs();
        if range == 0.0 {
            return 0.01;
        }
        size.clamp(0.01, range)
    }

    fn value_span(&self) -> f32 {
        self.max - self.min
    }

    fn virtual_track_size(&self, layout: &ComputedLayout) -> i32 {
        let cells = match self.orientation {
            SliderOrientation::Horizontal => layout.width.max(0.0),
            SliderOrientation::Vertical => layout.height.max(0.0),
        };
        (cells as i32).saturating_mul(2)
    }

    fn virtual_thumb_size(&self, layout: &ComputedLayout) -> i32 {
        let virtual_track_size = self.virtual_track_size(layout);
        let range = self.value_span();
        if virtual_track_size <= 0 {
            return 0;
        }
        if range == 0.0 {
            return virtual_track_size;
        }

        let viewport_size = self.viewport_size.max(1.0);
        let content_size = range.abs() + viewport_size;
        if content_size <= viewport_size {
            return virtual_track_size;
        }

        let calculated_size = (virtual_track_size as f32 * viewport_size / content_size).floor();
        (calculated_size as i32).clamp(1, virtual_track_size)
    }

    fn virtual_thumb_start(&self, layout: &ComputedLayout) -> i32 {
        let virtual_track_size = self.virtual_track_size(layout);
        let range = self.value_span();
        if virtual_track_size <= 0 || range == 0.0 {
            return 0;
        }
        let value_ratio = (self.value - self.min) / range;
        let virtual_thumb_size = self.virtual_thumb_size(layout);
        (value_ratio * (virtual_track_size - virtual_thumb_size) as f32).round() as i32
    }

    fn update_value_from_position(&mut self, layout: &ComputedLayout, x: u32, y: u32) -> bool {
        let track_start = match self.orientation {
            SliderOrientation::Horizontal => layout.x,
            SliderOrientation::Vertical => layout.y,
        };
        let track_size = match self.orientation {
            SliderOrientation::Horizontal => layout.width,
            SliderOrientation::Vertical => layout.height,
        };
        if track_size <= 0.0 {
            return false;
        }

        let mouse_pos = match self.orientation {
            SliderOrientation::Horizontal => x as f32,
            SliderOrientation::Vertical => y as f32,
        };
        let relative = (mouse_pos - track_start).clamp(0.0, track_size);
        let new_value = self.min + relative / track_size * self.value_span();
        let before = self.value;
        self.set_value(new_value);
        value_changed(self.value, before)
    }

    fn step_size(&self) -> f32 {
        (self.value_span().abs() / 100.0).max(1.0)
    }

    fn page_size(&self) -> f32 {
        self.viewport_size.max(self.step_size())
    }

    fn adjust_by(&mut self, delta: f32) -> bool {
        let before = self.value;
        self.set_value(self.value + delta);
        value_changed(self.value, before)
    }

    fn render_horizontal(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;
        if w == 0 || h == 0 {
            return;
        }

        let track_style = Style::builder().bg(self.slider_style.track_bg).build();
        for row in 0..h {
            for col in 0..w {
                ctx.buffer
                    .set(x + col, y + row, Cell::new(' ', track_style));
            }
        }

        let virtual_thumb_start = self.virtual_thumb_start(layout);
        let virtual_thumb_end = virtual_thumb_start + self.virtual_thumb_size(layout);
        let start = (virtual_thumb_start / 2).max(0);
        let end = ((virtual_thumb_end + 1) / 2 - 1).min(w as i32 - 1);
        if end < start {
            return;
        }

        let fg = if self.focused {
            self.slider_style.focused_thumb_fg
        } else {
            self.slider_style.thumb_fg
        };
        let thumb_style = Style::builder()
            .fg(fg)
            .bg(self.slider_style.thumb_bg)
            .build();
        for real_x in start..=end {
            let virtual_cell_start = real_x * 2;
            let virtual_cell_end = virtual_cell_start + 2;
            let thumb_start_in_cell = virtual_thumb_start.max(virtual_cell_start);
            let thumb_end_in_cell = virtual_thumb_end.min(virtual_cell_end);
            let coverage = thumb_end_in_cell - thumb_start_in_cell;
            let ch = if coverage >= 2 {
                '█'
            } else if thumb_start_in_cell == virtual_cell_start {
                '▌'
            } else {
                '▐'
            };
            for row in 0..h {
                ctx.buffer
                    .set(x + real_x as u32, y + row, Cell::new(ch, thumb_style));
            }
        }
    }

    fn render_vertical(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;
        if w == 0 || h == 0 {
            return;
        }

        let track_style = Style::builder().bg(self.slider_style.track_bg).build();
        for row in 0..h {
            for col in 0..w {
                ctx.buffer
                    .set(x + col, y + row, Cell::new(' ', track_style));
            }
        }

        let virtual_thumb_start = self.virtual_thumb_start(layout);
        let virtual_thumb_end = virtual_thumb_start + self.virtual_thumb_size(layout);
        let start = (virtual_thumb_start / 2).max(0);
        let end = ((virtual_thumb_end + 1) / 2 - 1).min(h as i32 - 1);
        if end < start {
            return;
        }

        let fg = if self.focused {
            self.slider_style.focused_thumb_fg
        } else {
            self.slider_style.thumb_fg
        };
        let thumb_style = Style::builder()
            .fg(fg)
            .bg(self.slider_style.thumb_bg)
            .build();
        for real_y in start..=end {
            let virtual_cell_start = real_y * 2;
            let virtual_cell_end = virtual_cell_start + 2;
            let thumb_start_in_cell = virtual_thumb_start.max(virtual_cell_start);
            let thumb_end_in_cell = virtual_thumb_end.min(virtual_cell_end);
            let coverage = thumb_end_in_cell - thumb_start_in_cell;
            let ch = if coverage >= 2 {
                '█'
            } else if thumb_start_in_cell == virtual_cell_start {
                '▀'
            } else {
                '▄'
            };
            for col in 0..w {
                ctx.buffer
                    .set(x + col, y + real_y as u32, Cell::new(ch, thumb_style));
            }
        }
    }
}

impl Behavior for SliderWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        self.last_layout = Some(*layout);
        match self.orientation {
            SliderOrientation::Horizontal => self.render_horizontal(ctx, layout),
            SliderOrientation::Vertical => self.render_vertical(ctx, layout),
        }
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

        let step = self.step_size();
        let page = self.page_size();
        match (self.orientation, key.code) {
            (SliderOrientation::Horizontal, KeyCode::Left)
            | (SliderOrientation::Vertical, KeyCode::Up) => self.adjust_by(-step),
            (SliderOrientation::Horizontal, KeyCode::Right)
            | (SliderOrientation::Vertical, KeyCode::Down) => self.adjust_by(step),
            (_, KeyCode::PageUp) => self.adjust_by(-page),
            (_, KeyCode::PageDown) => self.adjust_by(page),
            (_, KeyCode::Home) => {
                let before = self.value;
                self.set_value(self.min);
                value_changed(self.value, before)
            }
            (_, KeyCode::End) => {
                let before = self.value;
                self.set_value(self.max);
                value_changed(self.value, before)
            }
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
        if let Some(layout) = self.last_layout {
            return self.update_value_from_position(&layout, mouse.x, mouse.y);
        }
        false
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
