//! Render command list for tree-based rendering.
//!
//! During the render phase, the tree traverses nodes and builds a flat list
//! of `RenderCommand`s. This list is then executed against the
//! `OptimizedBuffer` in order.

use crate::renderable::node::NodeId;

/// A render command collected during the update pass and executed during
/// the render pass.
#[derive(Debug, Clone, Copy)]
pub enum RenderCommand<Id = NodeId> {
    Render {
        id: Id,
    },
    PushScissor {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    PopScissor,
    PushOpacity {
        opacity: f32,
    },
    PopOpacity,
}
