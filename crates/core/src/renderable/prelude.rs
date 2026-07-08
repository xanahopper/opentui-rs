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
    ViewRuntime, badge, checkbox, empty, fill, fragment, gauge, input, overlay, panel, radio_group,
    rich_text, scrollbar, select, separator, slider, span, spinner, text, view, when,
};
pub use crate::widgets::{
    BadgeShape, BadgeStyle, BadgeWidget, BorderChars, BorderSides, BorderStyle, BoxWidget,
    CheckboxChars, CheckboxStyle, CheckboxWidget, EditorWidget, FillWidget, GaugeChars,
    GaugeOrientation, GaugeStyle, GaugeWidget, InputMode, InputWidget, ListWidget,
    ProgressBarStyle, ProgressBarWidget, ProgressChars, RadioGroupWidget, RadioOption,
    RadioOrientation, RadioStyle, ScrollBarWidget, ScrollViewWidget, SelectItem, SelectStyle,
    SelectWidget, SeparatorWidget, SliderOrientation, SliderStyle, SliderWidget, SpinnerFrames,
    SpinnerStyle, SpinnerWidget, StatusLineStyle, StatusLineWidget, StyledSegment, StyledTextAlign,
    StyledTextWidget, Tab, TabsStyle, TabsWidget, TextAlign, TextLineAlign, TextLineWidget,
    TextWidget, ViewWidget,
};
