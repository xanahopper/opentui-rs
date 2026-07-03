//! SelectWidget — self-contained selectable list widget.
//!
//! Renders a scrollable list of items with selection highlight, keyboard
//! navigation (Up/Down/j/k, Enter, Home/End, PageUp/PageDown), mouse click
//! selection, and optional descriptions. Ported from the official
//! `SelectRenderable` concept but fully self-contained as a `Behavior`.

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;
use crate::terminal::{MouseButton, MouseEventKind};
use crate::{Cell, KeyCode, KeyEvent, Rgba, Style};

/// A single option in a select list.
#[derive(Debug, Clone)]
pub struct SelectItem {
    pub name: String,
    pub description: Option<String>,
}

impl SelectItem {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    pub fn with_description(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
        }
    }
}

impl From<&str> for SelectItem {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for SelectItem {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Styling for the select widget.
#[derive(Debug, Clone)]
pub struct SelectStyle {
    pub text_fg: Rgba,
    pub text_bg: Rgba,
    pub selected_fg: Rgba,
    pub selected_bg: Rgba,
    pub focused_bg: Rgba,
    pub description_fg: Rgba,
    pub selected_description_fg: Rgba,
    pub indicator: char,
    pub indicator_fg: Rgba,
}

impl Default for SelectStyle {
    fn default() -> Self {
        Self {
            text_fg: Rgba::WHITE,
            text_bg: Rgba::TRANSPARENT,
            selected_fg: Rgba::WHITE,
            selected_bg: Rgba::from_rgb_u8(51, 68, 85),
            focused_bg: Rgba::from_rgb_u8(26, 26, 26),
            description_fg: Rgba::from_rgb_u8(136, 136, 136),
            selected_description_fg: Rgba::from_rgb_u8(180, 180, 180),
            indicator: '\u{25B8}',
            indicator_fg: Rgba::from_rgb_u8(100, 200, 255),
        }
    }
}

pub struct SelectWidget {
    style: LayoutStyle,
    items: Vec<SelectItem>,
    selected_index: usize,
    scroll_offset: u32,
    item_height: u32,
    show_description: bool,
    show_indicator: bool,
    wrap_selection: bool,
    select_style: SelectStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
    last_layout: Option<ComputedLayout>,
}

impl SelectWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            items: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            item_height: 1,
            show_description: false,
            show_indicator: true,
            wrap_selection: false,
            select_style: SelectStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
            last_layout: None,
        }
    }

    pub fn items(mut self, items: Vec<SelectItem>) -> Self {
        self.items = items;
        self
    }

    pub fn item_height(mut self, height: u32) -> Self {
        self.item_height = height.max(1);
        self
    }

    pub fn show_description(mut self, show: bool) -> Self {
        self.show_description = show;
        if show && self.item_height < 2 {
            self.item_height = 2;
        }
        self
    }

    pub fn show_indicator(mut self, show: bool) -> Self {
        self.show_indicator = show;
        self
    }

    pub fn wrap_selection(mut self, wrap: bool) -> Self {
        self.wrap_selection = wrap;
        self
    }

    pub fn select_style(mut self, style: SelectStyle) -> Self {
        self.select_style = style;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn set_items(&mut self, items: Vec<SelectItem>) {
        self.items = items;
        if self.selected_index >= self.items.len() && !self.items.is_empty() {
            self.selected_index = self.items.len() - 1;
        }
        self.scroll_offset = 0;
    }

    pub fn set_selected(&mut self, index: usize) {
        if !self.items.is_empty() {
            self.selected_index = index.min(self.items.len() - 1);
            self.ensure_visible();
        }
    }

    fn visible_count(&self, layout: &ComputedLayout) -> u32 {
        let h = layout.height.max(0.0) as u32;
        h / self.item_height.max(1)
    }

    fn ensure_visible(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let vis = self
            .last_layout
            .map_or(1, |l| self.visible_count(&l))
            .max(1);
        if self.selected_index < self.scroll_offset as usize {
            self.scroll_offset = self.selected_index as u32;
        } else if self.selected_index >= (self.scroll_offset + vis) as usize {
            self.scroll_offset = self.selected_index.saturating_sub(vis as usize - 1) as u32;
        }
    }

    fn move_up(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        let before = self.selected_index;
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if self.wrap_selection && !self.items.is_empty() {
            self.selected_index = self.items.len() - 1;
        } else {
            return false;
        }
        self.ensure_visible();
        self.selected_index != before
    }

    fn move_down(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        let before = self.selected_index;
        if self.selected_index + 1 < self.items.len() {
            self.selected_index += 1;
        } else if self.wrap_selection {
            self.selected_index = 0;
        } else {
            return false;
        }
        self.ensure_visible();
        self.selected_index != before
    }

    fn page_up(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        let vis = self
            .last_layout
            .map_or(1, |l| self.visible_count(&l))
            .max(1) as usize;
        let before = self.selected_index;
        self.selected_index = self.selected_index.saturating_sub(vis);
        self.ensure_visible();
        self.selected_index != before
    }

    fn page_down(&mut self) -> bool {
        if self.items.is_empty() {
            return false;
        }
        let vis = self
            .last_layout
            .map_or(1, |l| self.visible_count(&l))
            .max(1) as usize;
        let before = self.selected_index;
        self.selected_index = (self.selected_index + vis).min(self.items.len().saturating_sub(1));
        self.ensure_visible();
        self.selected_index != before
    }

    fn select_from_mouse(&mut self, mouse_y: u32, layout: &ComputedLayout) -> bool {
        if self.items.is_empty() {
            return false;
        }
        let y = layout.y as u32;
        let rel_y = mouse_y.saturating_sub(y);
        let item_idx = (rel_y / self.item_height.max(1)) as usize + self.scroll_offset as usize;
        if item_idx < self.items.len() {
            let before = self.selected_index;
            self.selected_index = item_idx;
            self.selected_index != before
        } else {
            false
        }
    }

    fn render_item(
        &self,
        ctx: &mut RenderContext<'_>,
        item: &SelectItem,
        index: usize,
        x: u32,
        y: u32,
        width: u32,
    ) {
        let is_selected = index == self.selected_index;
        let bg = if is_selected {
            self.select_style.selected_bg
        } else if self.focused {
            self.select_style.focused_bg
        } else {
            self.select_style.text_bg
        };
        let fg = if is_selected {
            self.select_style.selected_fg
        } else {
            self.select_style.text_fg
        };

        let base_style = Style::builder().fg(fg).bg(bg).build();

        for col in 0..width {
            ctx.buffer.set(x + col, y, Cell::new(' ', base_style));
        }

        let mut text_x = x;
        if self.show_indicator {
            let ind_style = if is_selected {
                Style::builder()
                    .fg(self.select_style.indicator_fg)
                    .bg(bg)
                    .build()
            } else {
                Style::builder().fg(fg).bg(bg).build()
            };
            let ind_char = if is_selected {
                self.select_style.indicator
            } else {
                ' '
            };
            ctx.buffer.set(text_x, y, Cell::new(ind_char, ind_style));
            text_x += 1;
            if width > 1 {
                ctx.buffer.set(text_x, y, Cell::new(' ', base_style));
                text_x += 1;
            }
        }

        let avail_width = width.saturating_sub(text_x - x);
        let name_chars: Vec<char> = item.name.chars().collect();
        let truncated_len = name_chars.len().min(avail_width as usize);
        for (i, ch) in name_chars.iter().take(truncated_len).enumerate() {
            ctx.buffer
                .set(text_x + i as u32, y, Cell::new(*ch, base_style));
        }

        if self.show_description {
            if let Some(desc) = &item.description {
                let desc_y = y + 1;
                if desc_y < y + self.item_height {
                    let desc_fg = if is_selected {
                        self.select_style.selected_description_fg
                    } else {
                        self.select_style.description_fg
                    };
                    let desc_style = Style::builder().fg(desc_fg).bg(bg).build();
                    let desc_chars: Vec<char> = desc.chars().collect();
                    let desc_width = width.saturating_sub(text_x - x);
                    let truncated = desc_chars.len().min(desc_width as usize);
                    for (i, ch) in desc_chars.iter().take(truncated).enumerate() {
                        ctx.buffer
                            .set(text_x + i as u32, desc_y, Cell::new(*ch, desc_style));
                    }
                }
            }
        }
    }
}

impl Behavior for SelectWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        self.last_layout = Some(*layout);
        if !self.visible || self.items.is_empty() {
            return;
        }

        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width.max(0.0) as u32;
        let h = layout.height.max(0.0) as u32;
        if w == 0 || h == 0 {
            return;
        }

        let vis_count = self.visible_count(layout);
        let item_h = self.item_height.max(1);

        for i in 0..vis_count {
            let item_index = self.scroll_offset as usize + i as usize;
            if item_index >= self.items.len() {
                break;
            }
            let item_y = y + (i * item_h);
            if item_y + item_h > y + h {
                break;
            }
            self.render_item(ctx, &self.items[item_index], item_index, x, item_y, w);
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
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::Home => {
                if self.items.is_empty() {
                    return false;
                }
                let before = self.selected_index;
                self.selected_index = 0;
                self.scroll_offset = 0;
                self.selected_index != before
            }
            KeyCode::End => {
                if self.items.is_empty() {
                    return false;
                }
                let before = self.selected_index;
                self.selected_index = self.items.len() - 1;
                self.ensure_visible();
                self.selected_index != before
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
        let Some(layout) = self.last_layout else {
            return false;
        };
        self.select_from_mouse(mouse.y, &layout)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
