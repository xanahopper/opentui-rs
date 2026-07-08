//! CheckboxWidget — toggle checkbox with optional label.
//!
//! Renders a checkbox (`[x]` / `[ ]`) followed by an optional text label.
//! Supports keyboard toggle (Space / Enter) and mouse click.

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::terminal::{MouseButton, MouseEventKind};
use crate::{Cell, KeyCode, KeyEvent, Rgba, Style};

/// Checkbox character set.
#[derive(Debug, Clone, Copy)]
pub struct CheckboxChars {
    pub left_bracket: char,
    pub right_bracket: char,
    pub checked: char,
    pub unchecked: char,
}

impl Default for CheckboxChars {
    fn default() -> Self {
        Self {
            left_bracket: '[',
            right_bracket: ']',
            checked: 'x',
            unchecked: ' ',
        }
    }
}

impl CheckboxChars {
    pub fn unicode() -> Self {
        Self {
            left_bracket: '\u{2713}',
            right_bracket: ' ',
            checked: '\u{2713}',
            unchecked: ' ',
        }
    }

    pub fn round() -> Self {
        Self {
            left_bracket: '(',
            right_bracket: ')',
            checked: 'x',
            unchecked: ' ',
        }
    }
}

/// Styling for the checkbox widget.
#[derive(Debug, Clone)]
pub struct CheckboxStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub checked_fg: Rgba,
    pub focused_fg: Rgba,
    pub label_fg: Rgba,
    pub chars: CheckboxChars,
}

impl Default for CheckboxStyle {
    fn default() -> Self {
        Self {
            fg: Rgba::from_rgb_u8(180, 180, 180),
            bg: Rgba::TRANSPARENT,
            checked_fg: Rgba::from_rgb_u8(100, 200, 100),
            focused_fg: Rgba::from_rgb_u8(100, 200, 255),
            label_fg: Rgba::WHITE,
            chars: CheckboxChars::default(),
        }
    }
}

pub struct CheckboxWidget {
    style: LayoutStyle,
    checked: bool,
    label: Option<String>,
    checkbox_style: CheckboxStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    last_layout: Option<ComputedLayout>,
}

impl CheckboxWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            checked: false,
            label: None,
            checkbox_style: CheckboxStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
            last_layout: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checkbox_style(mut self, style: CheckboxStyle) -> Self {
        self.checkbox_style = style;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    fn checkbox_width() -> u32 {
        4
    }

    fn label_width(&self) -> u32 {
        self.label.as_ref().map_or(0, |l| l.chars().count() as u32)
    }

    fn hits(x: u32, y: u32, layout: &ComputedLayout) -> bool {
        let lx = layout.x as u32;
        let ly = layout.y as u32;
        let lw = layout.width.max(0.0) as u32;
        let lh = layout.height.max(0.0) as u32;
        y >= ly && y < ly + lh && x >= lx && x < lx + lw
    }
}

impl Behavior for CheckboxWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        self.last_layout = Some(*layout);
        if !self.visible {
            return;
        }

        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width.max(0.0) as u32;
        if w == 0 {
            return;
        }

        let bg_color = self.checkbox_style.bg;

        let chars = &self.checkbox_style.chars;

        let mark_fg = if self.checked {
            self.checkbox_style.checked_fg
        } else if self.focused {
            self.checkbox_style.focused_fg
        } else {
            self.checkbox_style.fg
        };

        let bracket_style = Style::builder().fg(mark_fg).bg(bg_color).build();
        let check_char = if self.checked {
            chars.checked
        } else {
            chars.unchecked
        };

        ctx.buffer
            .set(x, y, Cell::new(chars.left_bracket, bracket_style));
        ctx.buffer
            .set(x + 1, y, Cell::new(check_char, bracket_style));
        ctx.buffer
            .set(x + 2, y, Cell::new(chars.right_bracket, bracket_style));
        ctx.buffer.set(x + 3, y, Cell::new(' ', bracket_style));

        if let Some(label) = &self.label {
            let label_fg = self.checkbox_style.label_fg;
            let label_style = Style::builder().fg(label_fg).bg(bg_color).build();
            let avail = w.saturating_sub(Self::checkbox_width());
            let label_chars: Vec<char> = label.chars().collect();
            let truncated = label_chars.len().min(avail as usize);
            for (i, ch) in label_chars.iter().take(truncated).enumerate() {
                ctx.buffer.set(
                    x + Self::checkbox_width() + i as u32,
                    y,
                    Cell::new(*ch, label_style),
                );
            }
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
        match key.code {
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.toggle();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, mouse: &crate::MouseEvent) -> bool {
        if !matches!(mouse.button, MouseButton::Left | MouseButton::None) {
            return false;
        }
        if !matches!(mouse.kind, MouseEventKind::Press) {
            return false;
        }
        let Some(layout) = self.last_layout else {
            return false;
        };
        if Self::hits(mouse.x, mouse.y, &layout) {
            self.toggle();
            true
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
