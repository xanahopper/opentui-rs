//! InputWidget — single-line text input widget.
//!
//! Wraps an `EditBuffer` for single-line text entry with cursor display,
//! optional placeholder text, and basic editing keybindings.

use crate::text::EditBuffer;
use crate::{Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

impl std::fmt::Debug for InputWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputWidget")
            .field("mode", &self.mode)
            .field("focused", &self.focused)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Password,
}

pub struct InputWidget {
    style: LayoutStyle,
    buffer: EditBuffer,
    placeholder: Option<String>,
    placeholder_style: Style,
    mode: InputMode,
    cursor_visible: bool,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    scroll_x: u32,
    cursor_pos: Option<(u32, u32)>,
}

impl InputWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            buffer: EditBuffer::new(),
            placeholder: None,
            placeholder_style: Style::builder().fg(Rgba::new(0.4, 0.4, 0.45, 1.0)).build(),
            mode: InputMode::Normal,
            cursor_visible: true,
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
            scroll_x: 0,
            cursor_pos: None,
        }
    }

    pub fn with_text(style: LayoutStyle, text: &str) -> Self {
        Self {
            buffer: EditBuffer::with_text(text),
            ..Self::new(style)
        }
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn placeholder_style(mut self, style: Style) -> Self {
        self.placeholder_style = style;
        self
    }

    pub fn password_mode(mut self) -> Self {
        self.mode = InputMode::Password;
        self
    }

    pub fn value(&self) -> String {
        self.buffer.text()
    }

    pub fn set_value(&mut self, text: &str) {
        self.buffer.set_text(text);
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.text().is_empty()
    }

    pub fn buffer(&self) -> &EditBuffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut EditBuffer {
        &mut self.buffer
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    pub fn cursor_pos(&self) -> Option<(u32, u32)> {
        self.cursor_pos
    }

    fn clamp_scroll(&mut self, width: u32) {
        let cursor = self.buffer.cursor();
        if cursor.col < self.scroll_x as usize {
            self.scroll_x = cursor.col as u32;
        } else if width > 0 && cursor.col >= (self.scroll_x + width) as usize {
            self.scroll_x = cursor.col as u32 - width + 1;
        }
    }

    pub fn compute_cursor_pos(&mut self, layout: &ComputedLayout) {
        if !self.focused || !self.cursor_visible {
            self.cursor_pos = None;
            return;
        }
        let w = layout.width as u32;
        let cursor_col = self.buffer.cursor().col as u32;
        let display_col = cursor_col.saturating_sub(self.scroll_x);
        if display_col < w {
            self.cursor_pos = Some((layout.x as u32 + display_col, layout.y as u32));
        } else {
            self.cursor_pos = None;
        }
    }
}

impl Behavior for InputWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
        }

        // We can't set self.cursor_pos from &self — the caller should call
        // compute_cursor_pos() after render.
        let text = self.buffer.text();

        if text.is_empty() {
            if let Some(ref ph) = self.placeholder {
                let display: String = ph.chars().take(w as usize).collect();
                ctx.buffer.draw_text(x, y, &display, self.placeholder_style);
            }
        } else {
            let display = match self.mode {
                InputMode::Normal => text,
                InputMode::Password => "*".repeat(text.len()),
            };
            let scrolled: String = display
                .chars()
                .skip(self.scroll_x as usize)
                .take(w as usize)
                .collect();
            let text_style = Style::default();
            ctx.buffer.draw_text(x, y, &scrolled, text_style);
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

    fn handle_key(&mut self, key: &crate::KeyEvent) -> bool {
        let width = 20;
        let ctrl = key.modifiers.contains(crate::KeyModifiers::CTRL);
        let alt = key.modifiers.contains(crate::KeyModifiers::ALT);

        match key.code {
            crate::KeyCode::Char(ch) if !ctrl && !alt => {
                self.buffer.insert(&ch.to_string());
                self.clamp_scroll(width);
                true
            }
            crate::KeyCode::Backspace => {
                self.buffer.delete_backward();
                self.clamp_scroll(width);
                true
            }
            crate::KeyCode::Delete => {
                self.buffer.delete_forward();
                true
            }
            crate::KeyCode::Left if alt => {
                self.buffer.move_word_left();
                self.clamp_scroll(width);
                true
            }
            crate::KeyCode::Left => {
                self.buffer.move_left();
                self.clamp_scroll(width);
                true
            }
            crate::KeyCode::Right if alt => {
                self.buffer.move_word_right();
                self.clamp_scroll(width);
                true
            }
            crate::KeyCode::Right => {
                self.buffer.move_right();
                self.clamp_scroll(width);
                true
            }
            crate::KeyCode::Home => {
                self.buffer.move_to_line_start();
                self.scroll_x = 0;
                true
            }
            crate::KeyCode::End => {
                self.buffer.move_to_line_end();
                self.clamp_scroll(width);
                true
            }
            crate::KeyCode::Enter => true,
            _ => false,
        }
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
