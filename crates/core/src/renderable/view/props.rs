use crate::Rgba;
use crate::buffer::TitleAlign;

use crate::renderable::node::Overflow;
use crate::widgets::{BadgeShape, BorderStyle, StyledSegment, TextLineAlign};

#[derive(Debug, Clone)]
pub enum Props {
    View(ViewProps),
    Text(TextProps),
    StyledText(StyledTextProps),
    Input(InputProps),
    List(ListProps),
    Fill(FillProps),
    Separator(SeparatorProps),
    Checkbox(CheckboxProps),
    Spinner(SpinnerProps),
    Badge(BadgeProps),
    Slider(SliderProps),
    Select(SelectProps),
    RadioGroup(RadioGroupProps),
    Gauge(GaugeProps),
    ScrollBar(ScrollBarProps),
    Empty,
}

#[derive(Debug, Clone)]
pub struct ViewProps {
    pub bg: Option<Rgba>,
    pub border: Option<BorderStyle>,
    pub title: Option<String>,
    pub title_align: TitleAlign,
    pub title_color: Option<Rgba>,
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
            title_color: None,
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

#[derive(Debug, Clone, Default)]
pub struct StyledTextProps {
    pub segments: Vec<StyledSegment>,
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

#[derive(Debug, Clone, Default)]
pub struct CheckboxProps {
    pub checked: bool,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SpinnerProps {
    pub preset: SpinnerPreset,
    pub label: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerPreset {
    Braille,
    Dots,
    Arrow,
    Line,
    Bounce,
    Ascii,
}

impl Default for SpinnerProps {
    fn default() -> Self {
        Self {
            preset: SpinnerPreset::Braille,
            label: None,
            running: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BadgeProps {
    pub text: String,
    pub shape: BadgeShape,
    pub fg: Rgba,
    pub bg: Rgba,
}

impl Default for BadgeProps {
    fn default() -> Self {
        Self {
            text: String::new(),
            shape: BadgeShape::Padded,
            fg: Rgba::WHITE,
            bg: Rgba::from_rgb_u8(60, 60, 70),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SliderProps {
    pub horizontal: bool,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub viewport_size: f32,
}

impl Default for SliderProps {
    fn default() -> Self {
        Self {
            horizontal: true,
            min: 0.0,
            max: 100.0,
            value: 0.0,
            viewport_size: 10.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectProps {
    pub items: Vec<String>,
    pub selected: usize,
    pub wrap: bool,
    pub show_description: bool,
}

#[derive(Debug, Clone, Default)]
pub struct RadioGroupProps {
    pub horizontal: bool,
    pub options: Vec<String>,
    pub selected: usize,
}

#[derive(Debug, Clone)]
pub struct GaugeProps {
    pub horizontal: bool,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub segments: u32,
    pub show_label: bool,
}

impl Default for GaugeProps {
    fn default() -> Self {
        Self {
            horizontal: true,
            min: 0.0,
            max: 100.0,
            value: 0.0,
            segments: 10,
            show_label: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScrollBarProps {
    pub horizontal: bool,
    pub scroll_size: f32,
    pub viewport_size: f32,
    pub scroll_position: f32,
    pub show_arrows: bool,
}

impl Default for ScrollBarProps {
    fn default() -> Self {
        Self {
            horizontal: false,
            scroll_size: 0.0,
            viewport_size: 0.0,
            scroll_position: 0.0,
            show_arrows: false,
        }
    }
}
