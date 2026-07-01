//! UI theme definition and resolution.
//!
//! Provides a `UiTheme` struct that holds color tokens for a consistent UI
//! appearance. Themes can be loaded from JSON, defined in code, or generated
//! from terminal color palette.
//!
//! Note: This is distinct from `crate::highlight::Theme` which is used
//! for syntax highlighting. `UiTheme` covers the application UI chrome
//! (borders, backgrounds, text, scrollbars, etc.).

use crate::Rgba;

#[derive(Debug, Clone)]
pub struct UiTheme {
    pub name: String,

    pub primary: Rgba,
    pub secondary: Rgba,
    pub accent: Rgba,

    pub background: Rgba,
    pub background_panel: Rgba,
    pub background_element: Rgba,
    pub background_menu: Rgba,

    pub text: Rgba,
    pub text_muted: Rgba,
    pub text_selected: Rgba,
    pub text_inverse: Rgba,

    pub border: Rgba,
    pub border_subtle: Rgba,
    pub border_active: Rgba,

    pub success: Rgba,
    pub warning: Rgba,
    pub error: Rgba,
    pub info: Rgba,

    pub scrollbar_track: Rgba,
    pub scrollbar_thumb: Rgba,

    pub selection_bg: Rgba,
    pub selection_fg: Rgba,
}

impl UiTheme {
    pub fn dark_default() -> Self {
        Self {
            name: "dark-default".into(),
            primary: Rgba::new(0.5, 0.7, 1.0, 1.0),
            secondary: Rgba::new(0.6, 0.6, 0.7, 1.0),
            accent: Rgba::new(1.0, 0.6, 0.4, 1.0),
            background: Rgba::new(0.1, 0.1, 0.12, 1.0),
            background_panel: Rgba::new(0.14, 0.14, 0.17, 1.0),
            background_element: Rgba::new(0.18, 0.18, 0.22, 1.0),
            background_menu: Rgba::new(0.16, 0.16, 0.2, 1.0),
            text: Rgba::new(0.9, 0.9, 0.92, 1.0),
            text_muted: Rgba::new(0.5, 0.5, 0.55, 1.0),
            text_selected: Rgba::new(1.0, 1.0, 1.0, 1.0),
            text_inverse: Rgba::new(0.1, 0.1, 0.12, 1.0),
            border: Rgba::new(0.3, 0.3, 0.35, 1.0),
            border_subtle: Rgba::new(0.2, 0.2, 0.24, 1.0),
            border_active: Rgba::new(0.5, 0.7, 1.0, 1.0),
            success: Rgba::new(0.4, 0.85, 0.5, 1.0),
            warning: Rgba::new(1.0, 0.8, 0.3, 1.0),
            error: Rgba::new(1.0, 0.4, 0.4, 1.0),
            info: Rgba::new(0.4, 0.7, 1.0, 1.0),
            scrollbar_track: Rgba::new(0.15, 0.15, 0.18, 1.0),
            scrollbar_thumb: Rgba::new(0.4, 0.4, 0.45, 1.0),
            selection_bg: Rgba::new(0.2, 0.4, 0.7, 0.8),
            selection_fg: Rgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn light_default() -> Self {
        Self {
            name: "light-default".into(),
            primary: Rgba::new(0.2, 0.4, 0.8, 1.0),
            secondary: Rgba::new(0.4, 0.4, 0.45, 1.0),
            accent: Rgba::new(0.85, 0.4, 0.2, 1.0),
            background: Rgba::new(0.97, 0.97, 0.97, 1.0),
            background_panel: Rgba::new(0.95, 0.95, 0.95, 1.0),
            background_element: Rgba::new(0.92, 0.92, 0.92, 1.0),
            background_menu: Rgba::new(0.94, 0.94, 0.94, 1.0),
            text: Rgba::new(0.15, 0.15, 0.18, 1.0),
            text_muted: Rgba::new(0.5, 0.5, 0.55, 1.0),
            text_selected: Rgba::new(1.0, 1.0, 1.0, 1.0),
            text_inverse: Rgba::new(0.97, 0.97, 0.97, 1.0),
            border: Rgba::new(0.8, 0.8, 0.82, 1.0),
            border_subtle: Rgba::new(0.88, 0.88, 0.9, 1.0),
            border_active: Rgba::new(0.2, 0.4, 0.8, 1.0),
            success: Rgba::new(0.15, 0.7, 0.25, 1.0),
            warning: Rgba::new(0.9, 0.7, 0.1, 1.0),
            error: Rgba::new(0.9, 0.2, 0.2, 1.0),
            info: Rgba::new(0.2, 0.5, 0.9, 1.0),
            scrollbar_track: Rgba::new(0.9, 0.9, 0.9, 1.0),
            scrollbar_thumb: Rgba::new(0.6, 0.6, 0.62, 1.0),
            selection_bg: Rgba::new(0.3, 0.5, 0.85, 0.8),
            selection_fg: Rgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::dark_default()
    }
}

pub struct UiThemeRegistry {
    themes: Vec<UiTheme>,
    active: usize,
}

impl UiThemeRegistry {
    pub fn new() -> Self {
        Self {
            themes: vec![UiTheme::dark_default(), UiTheme::light_default()],
            active: 0,
        }
    }

    pub fn active(&self) -> &UiTheme {
        &self.themes[self.active]
    }

    pub fn set_active(&mut self, name: &str) -> bool {
        if let Some(idx) = self.themes.iter().position(|t| t.name == name) {
            self.active = idx;
            true
        } else {
            false
        }
    }

    pub fn register(&mut self, theme: UiTheme) {
        let name = theme.name.clone();
        if let Some(existing) = self.themes.iter_mut().find(|t| t.name == name) {
            *existing = theme;
        } else {
            self.themes.push(theme);
        }
    }

    pub fn names(&self) -> Vec<&str> {
        self.themes.iter().map(|t| t.name.as_str()).collect()
    }
}

impl Default for UiThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
