//! Prelude — common imports for opentui-core applications.
//!
//! ```
//! use opentui_core::prelude::*;
//! ```

pub use crate::keybinding::KeyBindingRegistry;
pub use crate::layout::{ComputedLayout, LayoutEngine, LayoutStyle};
pub use crate::list::{FixedHeightItemRenderer, ItemRenderer, VirtualList, VirtualListState};
pub use crate::render_command::RenderCommand;
pub use crate::renderable::behavior::{Behavior, FrameworkDefaults};
pub use crate::renderable::context::RenderContext;
pub use crate::renderable::event::{EventDispatcher, FocusId, FocusManager};
pub use crate::renderable::node::{NodeId, Overflow};
pub use crate::renderable::tree::{Overlay, RenderTree};
pub use crate::scroll::{ScrollBarRenderer, ScrollState, ScrollView};
pub use crate::theme::{UiTheme, UiThemeRegistry};
pub use crate::view::{
    ElementBuilder, ElementKind, Key, Node, Props, TextProps, ViewMouseDispatchResult, ViewProps,
    ViewRuntime, empty, fill, fragment, input, overlay, panel, rich_text, separator, span, text,
    view, when,
};
pub use crate::widgets::{
    BorderChars, BorderSides, BorderStyle, BoxWidget, EditorWidget, FillWidget, InputMode,
    InputWidget, ListWidget, ProgressBarStyle, ProgressBarWidget, ProgressChars, ScrollViewWidget,
    SeparatorWidget, StatusLineStyle, StatusLineWidget, StyledSegment, StyledTextAlign,
    StyledTextWidget, Tab, TabsStyle, TabsWidget, TextAlign, TextLineAlign, TextLineWidget,
    TextWidget, ViewWidget,
};
