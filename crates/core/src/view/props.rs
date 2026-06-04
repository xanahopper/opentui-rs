use opentui_rust::Rgba;
use opentui_rust::Style;
use opentui_rust::WrapMode;
use opentui_rust::buffer::TitleAlign;

use crate::widget::Overflow;
use crate::widgets::BorderStyle;

#[derive(Debug, Clone)]
pub enum Props {
    View(ViewProps),
    Text(TextProps),
    Input(InputProps),
    List(ListProps),
    Fill(FillProps),
    Separator(SeparatorProps),
    Empty,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BgFill {
    #[default]
    None,
    Text,
    Block,
}

#[derive(Debug, Clone)]
pub struct ViewProps {
    pub bg: Option<Rgba>,
    pub border: Option<BorderStyle>,
    pub title: Option<String>,
    pub title_align: TitleAlign,
    pub overflow: Overflow,
    pub opacity: f32,
    pub focusable: bool,
    pub visible: bool,
}

impl Default for ViewProps {
    fn default() -> Self {
        Self {
            bg: None,
            border: None,
            title: None,
            title_align: TitleAlign::Left,
            overflow: Overflow::Visible,
            opacity: 1.0,
            focusable: false,
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextProps {
    pub content: String,
    pub fg: Rgba,
    pub bg: Option<Rgba>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub wrap: WrapMode,
    pub bg_fill: BgFill,
    pub highlights: Vec<(usize, usize, Style)>,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            fg: Rgba::new(1.0, 1.0, 1.0, 1.0),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            wrap: WrapMode::None,
            bg_fill: BgFill::None,
            highlights: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InputProps {
    pub placeholder: Option<String>,
    pub password: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ListProps {
    pub item_count: usize,
    pub scrollbar: bool,
}

#[derive(Debug, Clone)]
pub struct FillProps {
    pub color: Rgba,
}

#[derive(Debug, Clone)]
pub struct SeparatorProps {
    pub char: char,
    pub fg: Rgba,
}

impl Default for FillProps {
    fn default() -> Self {
        Self {
            color: Rgba::TRANSPARENT,
        }
    }
}

impl Default for SeparatorProps {
    fn default() -> Self {
        Self {
            char: '\u{2500}',
            fg: Rgba::new(0.3, 0.3, 0.35, 1.0),
        }
    }
}
