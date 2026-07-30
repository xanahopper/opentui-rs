//! Behavior trait — the rendering/interaction contract for a render node.
//!
//! Each component type (Box, Text, Input, etc.) implements `Behavior` to
//! define how it renders and handles input. The per-node STATE (geometry,
//! focus, dirty flag, children, etc.) lives on `RenderNode`, not here.
//! This separation is the Rust-idiomatic equivalent of the official
//! OpenTUI's `Renderable` class hierarchy where each instance owns both
//! state and behavior.

use crate::input::{KeyEvent, MouseEvent};
use crate::renderable::context::RenderContext;
use crate::renderable::event::RenderableMouseEvent;
use crate::renderable::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::node::{NodeId, Overflow};

/// Framework-level defaults that a behavior wants on its RenderNode.
#[derive(Debug, Clone)]
pub struct FrameworkDefaults {
    pub focusable: bool,
    pub overflow: Overflow,
    pub visible: bool,
    pub opacity: f32,
}

impl Default for FrameworkDefaults {
    fn default() -> Self {
        Self {
            focusable: false,
            overflow: Overflow::Visible,
            visible: true,
            opacity: 1.0,
        }
    }
}

/// The behavior contract for a render node.
///
/// Default implementations are provided for all optional methods so that
/// simple components (e.g. a plain `Box`) only need to implement
/// `render_self`, `style`, and `as_any`.
pub trait Behavior {
    /// Draw this node into the buffer at the given layout position.
    fn render_self(&mut self, ctx: &mut RenderContext, layout: &ComputedLayout);

    /// Called every frame when `live` is active or when the node is dirty.
    /// Use for animations and time-based updates.
    fn on_update(&mut self, _delta_time: f64) {}

    /// Called during the lifecycle pass (before layout) when registered
    /// via `RenderTree::register_lifecycle_pass`.
    fn on_lifecycle_pass(&mut self) {}

    /// Called when the node's computed size changes.
    fn on_resize(&mut self, _width: f32, _height: f32) {}

    /// Report the intrinsic content size for layout measurement.
    ///
    /// Called by the layout engine for leaf nodes whose style does not fix
    /// the relevant dimension. Return `Some((width, height))` to declare the
    /// natural content size under the given constraints, or `None` to fall
    /// back to a zero-sized leaf (the previous default behavior).
    fn measure(
        &self,
        _known_width: Option<f32>,
        _known_height: Option<f32>,
        _available_width: Option<f32>,
        _available_height: Option<f32>,
    ) -> Option<(f32, f32)> {
        None
    }

    /// Sync framework-owned focus state into behavior-owned rendering state.
    fn set_focus_state(&mut self, _focused: bool, _has_focused_descendant: bool) {}

    /// Handle a key event. Return `true` if consumed.
    fn handle_key(&mut self, _key: &KeyEvent) -> bool {
        false
    }

    /// Handle paste text.
    fn handle_paste(&mut self, _text: &str) {}

    /// Handle a mouse event. Return `true` if consumed.
    fn handle_mouse(&mut self, _event: &MouseEvent) -> bool {
        false
    }

    /// Handle a renderable-level mouse event with propagation controls.
    ///
    /// Existing widgets can continue implementing [`Self::handle_mouse`]. New
    /// handlers should override this method when they need target metadata,
    /// `stop_propagation`, or `prevent_default`.
    fn handle_mouse_event(&mut self, event: &mut RenderableMouseEvent) -> bool {
        self.handle_mouse(event)
    }

    /// Whether pointer dragging can create a text selection in this node.
    fn selectable(&self) -> bool {
        false
    }

    /// Update this node's selection from global screen coordinates.
    ///
    /// Return `true` when the node contains selected text.
    fn update_selection(
        &mut self,
        _anchor: (i32, i32),
        _focus: (i32, i32),
        _is_start: bool,
    ) -> bool {
        false
    }

    /// Clear text selection owned by this node.
    fn clear_selection(&mut self) {}

    /// Return text currently selected in this node.
    fn selected_text(&self) -> Option<String> {
        None
    }

    /// Whether this node should be registered in the pointer hit grid.
    fn hit_testable(&self) -> bool {
        self.selectable()
    }

    /// Return the subset of children that should be rendered.
    ///
    /// Override this to implement viewport culling (e.g. `ScrollBox`
    /// only returns children within the visible area).
    fn visible_children(&self, all_children: &[NodeId]) -> Vec<NodeId> {
        all_children.to_vec()
    }

    /// Whether this behavior overrides `visible_children`.
    /// When `false`, the frame driver skips the culling path entirely.
    fn has_visible_child_filter(&self) -> bool {
        false
    }

    /// Whether this node's render command list can be reused across
    /// frames when nothing has changed. Override and return `false`
    /// if the behavior overrides `on_update` (time-varying rendering).
    fn can_reuse_render_command_list(&self) -> bool {
        true
    }

    /// Vertical render offset applied to this node's children.
    fn child_offset_y(&self) -> f32 {
        0.0
    }

    /// Framework-level defaults for the RenderNode (focusable, overflow, etc.).
    /// Override to provide non-default values.
    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults::default()
    }

    /// The layout style for this node (flexbox properties).
    fn style(&self) -> &LayoutStyle;

    /// Mutable access to the layout style.
    fn style_mut(&mut self) -> &mut LayoutStyle;

    /// Downcast support for type-specific access.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Mutable downcast support.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
