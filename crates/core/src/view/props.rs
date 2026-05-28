use opentui_rust::Rgba;
use opentui_rust::buffer::TitleAlign;

use crate::widget::Overflow;
use crate::widgets::{BorderStyle, TextLineAlign};

#[derive(Debug, Clone)]
pub enum Props {
    View(ViewProps),
    Text(TextProps),
    Empty,
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
    pub align: TextLineAlign,
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
            align: TextLineAlign::Left,
        }
    }
}
