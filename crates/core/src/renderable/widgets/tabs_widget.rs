//! TabsWidget — horizontal tab bar with content area.
//!
//! Provides a tabbed interface where only the active tab's content is rendered.

use crate::{Cell, Rgba, Style};

use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::Overflow;

#[derive(Debug, Clone)]
pub struct Tab {
    pub title: String,
}

impl Tab {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TabsStyle {
    pub bar_fg: Rgba,
    pub bar_bg: Rgba,
    pub active_fg: Rgba,
    pub active_bg: Rgba,
    pub separator: char,
    pub separator_fg: Rgba,
    pub underline_active: bool,
}

impl Default for TabsStyle {
    fn default() -> Self {
        Self {
            bar_fg: Rgba::new(0.6, 0.6, 0.65, 1.0),
            bar_bg: Rgba::new(0.12, 0.12, 0.15, 1.0),
            active_fg: Rgba::new(1.0, 1.0, 1.0, 1.0),
            active_bg: Rgba::new(0.18, 0.18, 0.22, 1.0),
            separator: '│',
            separator_fg: Rgba::new(0.35, 0.35, 0.4, 1.0),
            underline_active: true,
        }
    }
}

pub struct TabsWidget {
    style: LayoutStyle,
    tabs: Vec<Tab>,
    active: usize,
    tabs_style: TabsStyle,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl TabsWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            tabs: Vec::new(),
            active: 0,
            tabs_style: TabsStyle::default(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
        }
    }

    pub fn tabs(mut self, tabs: Vec<Tab>) -> Self {
        self.tabs = tabs;
        if self.active >= self.tabs.len() {
            self.active = 0;
        }
        self
    }

    pub fn active(mut self, index: usize) -> Self {
        if index < self.tabs.len() {
            self.active = index;
        }
        self
    }

    pub fn tabs_style(mut self, style: TabsStyle) -> Self {
        self.tabs_style = style;
        self
    }

    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() && index != self.active {
            self.active = index;
        }
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn select_next(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active = (self.active + 1) % self.tabs.len();
    }

    pub fn select_prev(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.active = if self.active == 0 {
            self.tabs.len() - 1
        } else {
            self.active - 1
        };
    }

    fn render_bar(&self, ctx: &mut RenderContext<'_>, x: u32, y: u32, width: u32) {
        if width == 0 || self.tabs.is_empty() {
            return;
        }

        let active_style = Style::builder()
            .fg(self.tabs_style.active_fg)
            .bg(self.tabs_style.active_bg)
            .bold()
            .build();
        let inactive_style = Style::builder()
            .fg(self.tabs_style.bar_fg)
            .bg(self.tabs_style.bar_bg)
            .build();
        let sep_style = Style::builder()
            .fg(self.tabs_style.separator_fg)
            .bg(self.tabs_style.bar_bg)
            .build();

        // Clear the bar row
        let clear_style = Style::builder().bg(self.tabs_style.bar_bg).build();
        for col in 0..width {
            ctx.buffer.set(x + col, y, Cell::new(' ', clear_style));
        }

        let mut col: u32 = 0;
        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == self.active;
            let s = if is_active {
                active_style
            } else {
                inactive_style
            };

            // Left padding
            if col < width {
                ctx.buffer.set(x + col, y, Cell::new(' ', s));
                col += 1;
            }

            // Title text
            for ch in tab.title.chars() {
                if col >= width {
                    break;
                }
                ctx.buffer.set(x + col, y, Cell::new(ch, s));
                col += 1;
            }

            // Right padding
            if col < width {
                ctx.buffer.set(x + col, y, Cell::new(' ', s));
                col += 1;
            }

            // Separator (skip for last tab)
            if i < self.tabs.len() - 1 && col < width {
                ctx.buffer
                    .set(x + col, y, Cell::new(self.tabs_style.separator, sep_style));
                col += 1;
            }
        }

        // Underline active tab
        if self.tabs_style.underline_active && !self.tabs.is_empty() {
            let underline = Style::builder()
                .fg(self.tabs_style.active_fg)
                .bg(self.tabs_style.active_bg)
                .underline()
                .build();
            // Draw underline on the row below the bar
            let ul_y = y + 1;
            // Find the active tab's column range — approximate by re-measuring
            let mut start_col: u32 = 0;
            for (i, tab) in self.tabs.iter().enumerate() {
                let tab_width = tab.title.chars().count() as u32 + 2; // padding
                if i == self.active {
                    for c in 0..tab_width.min(width.saturating_sub(start_col)) {
                        if start_col + c < width {
                            ctx.buffer
                                .set(x + start_col + c, ul_y, Cell::new(' ', underline));
                        }
                    }
                    break;
                }
                start_col += tab_width + 1; // +1 for separator
                if start_col > width {
                    break;
                }
            }
        }
    }
}

impl Behavior for TabsWidget {
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

        // Render tab bar (first 1-2 rows)
        let bar_height: u32 = if self.tabs_style.underline_active {
            2
        } else {
            1
        };
        self.render_bar(ctx, x, y, w);

        // Clear content area below bar
        if h > bar_height {
            let content_style = Style::builder()
                .fg(Rgba::new(0.85, 0.85, 0.88, 1.0))
                .bg(Rgba::new(0.12, 0.12, 0.15, 1.0))
                .build();
            for row in bar_height..h {
                for col in 0..w {
                    ctx.buffer
                        .set(x + col, y + row, Cell::new(' ', content_style));
                }
            }

            // Show active tab title in content area as placeholder
            if let Some(tab) = self.tabs.get(self.active) {
                let label = format!("Tab: {}", tab.title);
                let label_style = Style::builder().fg(Rgba::new(0.7, 0.7, 0.75, 1.0)).build();
                ctx.buffer
                    .draw_text(x + 2, y + bar_height + 1, &label, label_style);
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

    fn handle_key(&mut self, key: &crate::KeyEvent) -> bool {
        use crate::{KeyCode, KeyModifiers};
        match (key.modifiers, key.code) {
            (m, KeyCode::Tab)
                if m.contains(KeyModifiers::CTRL) && !m.contains(KeyModifiers::SHIFT) =>
            {
                self.select_next();
                true
            }
            (m, KeyCode::Tab)
                if m.contains(KeyModifiers::CTRL) && m.contains(KeyModifiers::SHIFT) =>
            {
                self.select_prev();
                true
            }
            (m, KeyCode::Left) if m.contains(KeyModifiers::ALT) => {
                self.select_prev();
                true
            }
            (m, KeyCode::Right) if m.contains(KeyModifiers::ALT) => {
                self.select_next();
                true
            }
            (m, KeyCode::Char(c)) if m.contains(KeyModifiers::ALT) && ('1'..='9').contains(&c) => {
                let idx = (c as usize) - ('1' as usize);
                if idx < self.tabs.len() {
                    self.active = idx;
                    true
                } else {
                    false
                }
            }
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
