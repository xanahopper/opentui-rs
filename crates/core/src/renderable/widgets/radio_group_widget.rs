//! RadioGroupWidget — single-selection radio button group.
//!
//! Renders a vertical or horizontal list of radio options where exactly one
//! can be selected at a time. Supports keyboard navigation and mouse click.

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::terminal::{MouseButton, MouseEventKind};
use crate::{Cell, KeyCode, KeyEvent, Rgba, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioOrientation {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
pub struct RadioChars {
    pub selected: char,
    pub unselected: char,
    pub left: char,
    pub right: char,
}

impl Default for RadioChars {
    fn default() -> Self {
        Self {
            selected: '\u{25CF}',
            unselected: '\u{25CB}',
            left: '(',
            right: ')',
        }
    }
}

#[derive(Debug, Clone)]
pub struct RadioStyle {
    pub fg: Rgba,
    pub bg: Rgba,
    pub selected_fg: Rgba,
    pub focused_fg: Rgba,
    pub label_fg: Rgba,
    pub chars: RadioChars,
}

impl Default for RadioStyle {
    fn default() -> Self {
        Self {
            fg: Rgba::from_rgb_u8(180, 180, 180),
            bg: Rgba::TRANSPARENT,
            selected_fg: Rgba::from_rgb_u8(100, 200, 255),
            focused_fg: Rgba::from_rgb_u8(140, 180, 220),
            label_fg: Rgba::WHITE,
            chars: RadioChars::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RadioOption {
    pub label: String,
    pub value: Option<String>,
}

impl RadioOption {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: None,
        }
    }

    pub fn with_value(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: Some(value.into()),
        }
    }
}

impl From<&str> for RadioOption {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

pub struct RadioGroupWidget {
    style: LayoutStyle,
    orientation: RadioOrientation,
    options: Vec<RadioOption>,
    selected: usize,
    radio_style: RadioStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    last_layout: Option<ComputedLayout>,
}

impl RadioGroupWidget {
    pub fn new(style: LayoutStyle, orientation: RadioOrientation) -> Self {
        Self {
            style,
            orientation,
            options: Vec::new(),
            selected: 0,
            radio_style: RadioStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
            last_layout: None,
        }
    }

    pub fn vertical(style: LayoutStyle) -> Self {
        Self::new(style, RadioOrientation::Vertical)
    }

    pub fn horizontal(style: LayoutStyle) -> Self {
        Self::new(style, RadioOrientation::Horizontal)
    }

    pub fn options(mut self, options: Vec<RadioOption>) -> Self {
        self.options = options;
        if self.selected >= self.options.len() && !self.options.is_empty() {
            self.selected = 0;
        }
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        if !self.options.is_empty() {
            self.selected = index.min(self.options.len() - 1);
        }
        self
    }

    pub fn radio_style(mut self, style: RadioStyle) -> Self {
        self.radio_style = style;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.options
            .get(self.selected)
            .and_then(|o| o.value.as_deref())
    }

    pub fn set_selected(&mut self, index: usize) {
        if !self.options.is_empty() {
            self.selected = index.min(self.options.len() - 1);
        }
    }

    pub fn set_options(&mut self, options: Vec<RadioOption>) {
        self.options = options;
        if self.selected >= self.options.len() && !self.options.is_empty() {
            self.selected = 0;
        }
    }

    fn item_width(&self, index: usize) -> u32 {
        4 + self.options[index].label.chars().count() as u32
    }

    fn item_position(&self, index: usize, layout: &ComputedLayout) -> (u32, u32) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        match self.orientation {
            RadioOrientation::Vertical => (x, y + index as u32),
            RadioOrientation::Horizontal => {
                let mut offset = 0u32;
                for i in 0..index {
                    offset += self.item_width(i) + 1;
                }
                (x + offset, y)
            }
        }
    }

    fn item_at_pos(&self, x: u32, y: u32, layout: &ComputedLayout) -> Option<usize> {
        for i in 0..self.options.len() {
            let (ix, iy) = self.item_position(i, layout);
            let iw = self.item_width(i);
            if y == iy && x >= ix && x < ix + iw {
                return Some(i);
            }
        }
        None
    }

    fn render_option(&self, ctx: &mut RenderContext<'_>, index: usize, x: u32, y: u32) {
        let is_selected = index == self.selected;
        let chars = &self.radio_style.chars;

        let fg = if is_selected {
            self.radio_style.selected_fg
        } else if self.focused {
            self.radio_style.focused_fg
        } else {
            self.radio_style.fg
        };
        let bg = self.radio_style.bg;

        let marker_style = Style::builder().fg(fg).bg(bg).build();
        let dot_char = if is_selected {
            chars.selected
        } else {
            chars.unselected
        };

        ctx.buffer.set(x, y, Cell::new(chars.left, marker_style));
        ctx.buffer.set(x + 1, y, Cell::new(dot_char, marker_style));
        ctx.buffer
            .set(x + 2, y, Cell::new(chars.right, marker_style));
        ctx.buffer.set(x + 3, y, Cell::new(' ', marker_style));

        let label_style = Style::builder()
            .fg(self.radio_style.label_fg)
            .bg(bg)
            .build();
        for (i, ch) in self.options[index].label.chars().enumerate() {
            ctx.buffer
                .set(x + 4 + i as u32, y, Cell::new(ch, label_style));
        }
    }
}

impl Behavior for RadioGroupWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        self.last_layout = Some(*layout);
        if !self.visible || self.options.is_empty() {
            return;
        }

        for i in 0..self.options.len() {
            let (x, y) = self.item_position(i, layout);
            self.render_option(ctx, i, x, y);
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
        if self.options.is_empty() {
            return false;
        }

        let (forward, backward) = match self.orientation {
            RadioOrientation::Vertical => (KeyCode::Down, KeyCode::Up),
            RadioOrientation::Horizontal => (KeyCode::Right, KeyCode::Left),
        };

        let before = self.selected;
        match key.code {
            k if k == forward || k == KeyCode::Char('j') || k == KeyCode::Char('l') => {
                if self.selected + 1 < self.options.len() {
                    self.selected += 1;
                } else {
                    self.selected = 0;
                }
            }
            k if k == backward || k == KeyCode::Char('k') || k == KeyCode::Char('h') => {
                if self.selected > 0 {
                    self.selected -= 1;
                } else {
                    self.selected = self.options.len() - 1;
                }
            }
            KeyCode::Home => self.selected = 0,
            KeyCode::End if !self.options.is_empty() => {
                self.selected = self.options.len() - 1;
            }
            _ => return false,
        }
        self.selected != before
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
        if let Some(index) = self.item_at_pos(mouse.x, mouse.y, &layout) {
            let before = self.selected;
            self.selected = index;
            self.selected != before
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
