//! `RenderNode` — the centralized per-node state stored in the arena.
//!
//! This struct holds ALL the state that the official OpenTUI's `Renderable`
//! class keeps as instance fields: geometry caches, dirty flags, focus state,
//! tree pointers, layout dirty bits, etc. The BEHAVIOR (how to render,
//! handle input) is provided by the `Behavior` trait object.

use slotmap::new_key_type;

use crate::renderable::behavior::Behavior;

new_key_type! {
    /// Opaque, generational handle to a `RenderNode` in the `RenderTree` arena.
    pub struct NodeId;
}

/// Overflow behavior for clipping children.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    /// Children render outside the node bounds.
    #[default]
    Visible,
    /// Children are clipped to the node bounds.
    Hidden,
}

/// Per-node render state. Stored in a `SlotMap` inside `RenderTree`.
pub struct RenderNode {
    /// Monotonic instance counter used for hit-grid resolution.
    pub num: u32,

    // ── Tree structure ──────────────────────────
    /// Parent node, or `None` for the root.
    pub parent: Option<NodeId>,
    /// Children in insertion / layout order.
    pub children: Vec<NodeId>,
    /// Children sorted by z-index (lazily rebuilt).
    pub children_z: Vec<NodeId>,
    /// Whether `children_z` needs re-sorting.
    pub needs_z_sort: bool,

    // ── Geometry (cached from layout pass) ──────
    /// Position relative to parent.
    pub x: f32,
    pub y: f32,
    /// Absolute screen position (cached for O(1) render reads).
    pub screen_x: f32,
    pub screen_y: f32,
    /// Computed width / height.
    pub width: f32,
    pub height: f32,

    // ── Layout ───────────────────────────────────
    /// Corresponding Taffy layout node.
    pub taffy_node: taffy::tree::NodeId,

    // ── Visual properties ───────────────────────
    pub visible: bool,
    pub opacity: f32,
    pub overflow: Overflow,
    pub z_index: i32,

    // ── State ────────────────────────────────────
    pub dirty: bool,
    pub destroyed: bool,

    // ── Focus ────────────────────────────────────
    pub focusable: bool,
    pub focused: bool,
    pub has_focused_descendant: bool,

    // ── Lifecycle ────────────────────────────────
    /// Registered for lifecycle pass.
    pub registered_lifecycle: bool,
    /// Frame ID of last `update_from_layout` call (dedup).
    pub last_layout_frame: u64,

    // ── Animation ────────────────────────────────
    pub live: bool,
    pub live_count: u32,

    // ── Offscreen buffer ─────────────────────────
    pub buffered: bool,

    // ── Behavior ─────────────────────────────────
    pub behavior: Box<dyn Behavior>,
}

impl std::fmt::Debug for RenderNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderNode")
            .field("num", &self.num)
            .field("parent", &self.parent)
            .field("children_len", &self.children.len())
            .field("screen_xy", &(self.screen_x, self.screen_y))
            .field("size", &(self.width, self.height))
            .field("visible", &self.visible)
            .field("dirty", &self.dirty)
            .field("focused", &self.focused)
            .finish_non_exhaustive()
    }
}
