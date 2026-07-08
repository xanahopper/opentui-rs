//! Concrete widget implementations.
//!
//! This module provides ready-to-use widget types built on the [`Widget`](crate::Widget) trait.

mod badge_widget;
mod box_widget;
mod checkbox_widget;
mod editor_widget;
mod fill_widget;
mod input_widget;
mod list_widget;
mod progress_widget;
mod scroll_view_widget;
mod scrollbar_widget;
mod select_widget;
mod separator_widget;
mod slider_widget;
mod spinner_widget;
mod status_line_widget;
mod styled_text_widget;
mod tabs_widget;
mod text_line_widget;
mod text_widget;
mod view_widget;

pub use badge_widget::{BadgeShape, BadgeStyle, BadgeWidget};
pub use box_widget::{BorderChars, BorderSides, BorderStyle, BoxWidget};
pub use checkbox_widget::{CheckboxChars, CheckboxStyle, CheckboxWidget};
pub use editor_widget::EditorWidget;
pub use fill_widget::FillWidget;
pub use input_widget::{InputMode, InputWidget};
pub use list_widget::ListWidget;
pub use progress_widget::{ProgressBarStyle, ProgressBarWidget, ProgressChars};
pub use scroll_view_widget::ScrollViewWidget;
pub use scrollbar_widget::{
    ScrollBarOrientation, ScrollBarWidget, ScrollBarWidgetStyle, ScrollUnit,
};
pub use select_widget::{SelectItem, SelectStyle, SelectWidget};
pub use separator_widget::SeparatorWidget;
pub use slider_widget::{SliderOrientation, SliderStyle, SliderWidget};
pub use spinner_widget::{SpinnerFrames, SpinnerStyle, SpinnerWidget};
pub use status_line_widget::{StatusLineStyle, StatusLineWidget};
pub use styled_text_widget::{StyledSegment, StyledTextAlign, StyledTextWidget};
pub use tabs_widget::{Tab, TabsStyle, TabsWidget};
pub use text_line_widget::{TextLineAlign, TextLineWidget};
pub use text_widget::{TextAlign, TextWidget};
pub use view_widget::ViewWidget;
