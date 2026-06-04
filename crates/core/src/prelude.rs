//! Prelude — common imports for opentui-core applications.
//!
//! ```
//! use opentui_core::prelude::*;
//! ```

pub use crate::event::{EventDispatcher, FocusId, FocusManager};
pub use crate::keybinding::KeyBindingRegistry;
pub use crate::layout::{ComputedLayout, LayoutEngine, LayoutStyle, NodeContext};
pub use crate::list::{FixedHeightItemRenderer, ItemRenderer, VirtualList, VirtualListState};
pub use crate::render_command::{RenderCommand, RenderCommandList};
pub use crate::scroll::{ScrollBarRenderer, ScrollState, ScrollView};
pub use crate::theme::{UiTheme, UiThemeRegistry};
pub use crate::view::{
    BgFill, ElementBuilder, ElementKind, Key, Node, Props, StyledSegment, TextProps,
    ViewMouseDispatchResult, ViewProps, ViewRuntime, empty, fill, fragment, input, overlay, panel,
    rich_text, separator, span, text, view, when,
};
pub use crate::widget::{
    KeyAction, KeyDispatchResult, MouseDispatchResult, Overflow, Overlay, OverlayZOrder,
    RenderContext, Widget, WidgetId, WidgetTree,
};
pub use crate::widgets::{
    BorderChars, BorderSides, BorderStyle, BoxWidget, EditorWidget, FillWidget, InputMode,
    InputWidget, ListWidget, ProgressBarStyle, ProgressBarWidget, ProgressChars, ScrollViewWidget,
    SeparatorWidget, StatusLineStyle, StatusLineWidget, Tab, TabsStyle, TabsWidget, TextAlign,
    TextWidget, ViewWidget,
};
