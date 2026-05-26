//! Concrete widget implementations.
//!
//! This module provides ready-to-use widget types built on the [`Widget`](crate::Widget) trait.

mod box_widget;
mod editor_widget;
mod input_widget;
mod list_widget;
mod progress_widget;
mod scroll_view_widget;
mod status_line_widget;
mod tabs_widget;
mod text_widget;

pub use box_widget::{BorderChars, BorderSides, BorderStyle, BoxWidget};
pub use editor_widget::EditorWidget;
pub use input_widget::{InputMode, InputWidget};
pub use list_widget::ListWidget;
pub use progress_widget::{ProgressBarStyle, ProgressBarWidget, ProgressChars};
pub use scroll_view_widget::ScrollViewWidget;
pub use status_line_widget::StatusLineWidget;
pub use tabs_widget::{Tab, TabsWidget};
pub use text_widget::{TextAlign, TextWidget};
