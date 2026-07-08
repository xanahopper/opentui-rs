//! SpinnerWidget — animated loading indicator.
//!
//! Renders a spinner that cycles through frames on each `on_update` call.
//! Supports configurable frame sets, colors, and an optional label.

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::{Cell, Rgba, Style};

/// A set of spinner frames.
#[derive(Debug, Clone)]
pub struct SpinnerFrames {
    pub frames: Vec<char>,
    pub interval_ms: f64,
}

impl SpinnerFrames {
    pub fn braille() -> Self {
        Self {
            frames: "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".chars().collect(),
            interval_ms: 80.0,
        }
    }

    pub fn dots() -> Self {
        Self {
            frames: "⣾⣽⣻⢿⡿⣟⣯⣷".chars().collect(),
            interval_ms: 80.0,
        }
    }

    pub fn arrow() -> Self {
        Self {
            frames: "←↖↑↗→↘↓↙".chars().collect(),
            interval_ms: 120.0,
        }
    }

    pub fn line() -> Self {
        Self {
            frames: vec!['-', '\\', '|', '/'],
            interval_ms: 120.0,
        }
    }

    pub fn bounce() -> Self {
        Self {
            frames: vec!['⠁', '⠂', '⠄', '⠂'],
            interval_ms: 100.0,
        }
    }

    pub fn ascii() -> Self {
        Self {
            frames: vec!['|', '/', '-', '\\'],
            interval_ms: 250.0,
        }
    }
}

impl Default for SpinnerFrames {
    fn default() -> Self {
        Self::braille()
    }
}

/// Styling for the spinner widget.
#[derive(Debug, Clone)]
pub struct SpinnerStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub label_fg: Rgba,
}

impl Default for SpinnerStyle {
    fn default() -> Self {
        Self {
            fg: Rgba::from_rgb_u8(100, 200, 255),
            bg: Rgba::TRANSPARENT,
            label_fg: Rgba::from_rgb_u8(180, 180, 180),
        }
    }
}

pub struct SpinnerWidget {
    style: LayoutStyle,
    frames: SpinnerFrames,
    spinner_style: SpinnerStyle,
    label: Option<String>,
    current_frame: usize,
    elapsed_ms: f64,
    running: bool,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl SpinnerWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            frames: SpinnerFrames::default(),
            spinner_style: SpinnerStyle::default(),
            label: None,
            current_frame: 0,
            elapsed_ms: 0.0,
            running: true,
            visible: true,
            opacity: 1.0,
            focusable: false,
            focused: false,
        }
    }

    pub fn frames(mut self, frames: SpinnerFrames) -> Self {
        self.frames = frames;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn spinner_style(mut self, style: SpinnerStyle) -> Self {
        self.spinner_style = style;
        self
    }

    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    pub fn current_char(&self) -> char {
        self.frames
            .frames
            .get(self.current_frame)
            .copied()
            .unwrap_or(' ')
    }

    fn advance(&mut self) {
        if self.frames.frames.is_empty() {
            return;
        }
        self.current_frame = (self.current_frame + 1) % self.frames.frames.len();
    }
}

impl Behavior for SpinnerWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        if !self.visible {
            return;
        }

        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width.max(0.0) as u32;
        if w == 0 {
            return;
        }

        let spinner_style = Style::builder()
            .fg(self.spinner_style.fg)
            .bg(self.spinner_style.bg)
            .build();
        ctx.buffer
            .set(x, y, Cell::new(self.current_char(), spinner_style));

        if let Some(label) = &self.label {
            let label_style = Style::builder()
                .fg(self.spinner_style.label_fg)
                .bg(self.spinner_style.bg)
                .build();
            let avail = w.saturating_sub(2);
            let label_chars: Vec<char> = label.chars().collect();
            let truncated = label_chars.len().min(avail as usize);
            for (i, ch) in label_chars.iter().take(truncated).enumerate() {
                ctx.buffer
                    .set(x + 2 + i as u32, y, Cell::new(*ch, label_style));
            }
        }
    }

    fn on_update(&mut self, delta_time: f64) {
        if !self.running || self.frames.frames.is_empty() {
            return;
        }
        self.elapsed_ms += delta_time * 1000.0;
        if self.elapsed_ms >= self.frames.interval_ms {
            let steps = (self.elapsed_ms / self.frames.interval_ms) as usize;
            self.elapsed_ms %= self.frames.interval_ms;
            for _ in 0..steps {
                self.advance();
            }
        }
    }

    fn can_reuse_render_command_list(&self) -> bool {
        false
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
