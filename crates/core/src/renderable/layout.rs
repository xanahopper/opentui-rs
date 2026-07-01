//! Flexbox and CSS Grid layout via Taffy.
//!
//! This module wraps the [Taffy](https://github.com/DioxusLabs/taffy) layout
//! engine to provide a familiar flexbox/grid API for terminal UI layout.
//!
//! Each widget in the tree holds a Taffy node. After all style properties
//! are set, `LayoutEngine::compute()` runs the layout pass and every node
//! receives its computed `x, y, width, height`.
//!
//! # Example (conceptual)
//!
//! ```ignore
//! use opentui_core::layout::{LayoutEngine, LayoutStyle};
//!
//! let mut engine = LayoutEngine::new();
//!
//! let root = engine.new_node(
//!     LayoutStyle::column().width(80.0).height(24.0),
//!     &[header, body, footer],
//! );
//!
//! engine.compute(root);
//! let rect = engine.layout(root);
//! ```

use taffy::TaffyTree;
use taffy::prelude::*;
use taffy::tree::Layout;

pub use taffy::style as taffy_style;

#[derive(Debug, Clone, Copy, Default)]
pub struct ComputedLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<Layout> for ComputedLayout {
    fn from(layout: Layout) -> Self {
        Self {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LayoutStyle {
    pub inner: taffy::style::Style,
    base_padding: (f32, f32, f32, f32),
}

impl LayoutStyle {
    pub fn column() -> Self {
        Self {
            inner: taffy::style::Style {
                display: taffy::style::Display::Flex,
                flex_direction: taffy::style::FlexDirection::Column,
                ..Default::default()
            },
            base_padding: (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn row() -> Self {
        Self {
            inner: taffy::style::Style {
                display: taffy::style::Display::Flex,
                flex_direction: taffy::style::FlexDirection::Row,
                ..Default::default()
            },
            base_padding: (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.inner.size.width = length(w);
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.inner.size.height = length(h);
        self
    }

    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.inner.flex_grow = grow;
        self
    }

    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.inner.flex_shrink = shrink;
        self
    }

    pub fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.base_padding = (top, right, bottom, left);
        self.inner.padding = taffy::geometry::Rect {
            top: length(top),
            right: length(right),
            bottom: length(bottom),
            left: length(left),
        };
        self
    }

    pub fn add_padding(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        if top == 0.0 && right == 0.0 && bottom == 0.0 && left == 0.0 {
            return self;
        }
        self.padding_extra(top, right, bottom, left)
    }

    fn padding_extra(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        let base = self.base_padding;
        self.padding(base.0 + top, base.1 + right, base.2 + bottom, base.3 + left)
    }

    pub fn padding_all(mut self, v: f32) -> Self {
        self.inner.padding = taffy::geometry::Rect {
            top: length(v),
            right: length(v),
            bottom: length(v),
            left: length(v),
        };
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.inner.gap = taffy::geometry::Size {
            width: length(gap),
            height: length(gap),
        };
        self
    }

    pub fn align_items(mut self, align: taffy::style::AlignItems) -> Self {
        self.inner.align_items = Some(align);
        self
    }

    pub fn justify_content(mut self, justify: taffy::style::JustifyContent) -> Self {
        self.inner.justify_content = Some(justify);
        self
    }

    pub fn overflow(mut self, overflow: taffy::style::Overflow) -> Self {
        self.inner.overflow = taffy::geometry::Point {
            x: overflow,
            y: overflow,
        };
        self
    }

    pub fn position_absolute(mut self) -> Self {
        self.inner.position = taffy::style::Position::Absolute;
        self
    }

    pub fn top(mut self, val: f32) -> Self {
        self.inner.inset.top = length(val);
        self
    }

    pub fn bottom(mut self, val: f32) -> Self {
        self.inner.inset.bottom = length(val);
        self
    }

    pub fn left(mut self, val: f32) -> Self {
        self.inner.inset.left = length(val);
        self
    }

    pub fn right(mut self, val: f32) -> Self {
        self.inner.inset.right = length(val);
        self
    }

    pub fn min_width(mut self, val: f32) -> Self {
        self.inner.min_size.width = length(val);
        self
    }

    pub fn max_width(mut self, val: f32) -> Self {
        self.inner.max_size.width = length(val);
        self
    }

    pub fn min_height(mut self, val: f32) -> Self {
        self.inner.min_size.height = length(val);
        self
    }

    pub fn max_height(mut self, val: f32) -> Self {
        self.inner.max_size.height = length(val);
        self
    }

    pub fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.inner.margin = taffy::geometry::Rect {
            top: length(top),
            right: length(right),
            bottom: length(bottom),
            left: length(left),
        };
        self
    }

    pub fn flex_basis(mut self, val: f32) -> Self {
        self.inner.flex_basis = length(val);
        self
    }

    pub fn align_self(mut self, align: taffy::style::AlignSelf) -> Self {
        self.inner.align_self = Some(align);
        self
    }

    pub fn flex_wrap(mut self, wrap: taffy::style::FlexWrap) -> Self {
        self.inner.flex_wrap = wrap;
        self
    }

    pub fn width_percent(mut self, pct: f32) -> Self {
        self.inner.size.width = percent(pct);
        self
    }

    pub fn height_percent(mut self, pct: f32) -> Self {
        self.inner.size.height = percent(pct);
        self
    }

    pub fn padding_x(mut self, val: f32) -> Self {
        self.inner.padding.left = length(val);
        self.inner.padding.right = length(val);
        self
    }

    pub fn padding_y(mut self, val: f32) -> Self {
        self.inner.padding.top = length(val);
        self.inner.padding.bottom = length(val);
        self
    }

    pub fn auto_width(mut self) -> Self {
        self.inner.size.width = auto();
        self
    }

    pub fn auto_height(mut self) -> Self {
        self.inner.size.height = auto();
        self
    }
}

pub struct LayoutEngine {
    tree: TaffyTree<()>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
        }
    }

    pub fn new_leaf(&mut self, style: LayoutStyle) -> taffy::tree::NodeId {
        self.tree.new_leaf(style.inner).unwrap()
    }

    pub fn new_node(
        &mut self,
        style: LayoutStyle,
        children: &[taffy::tree::NodeId],
    ) -> taffy::tree::NodeId {
        self.tree.new_with_children(style.inner, children).unwrap()
    }

    pub fn set_style(&mut self, node: taffy::tree::NodeId, style: LayoutStyle) {
        self.tree.set_style(node, style.inner).unwrap();
    }

    pub fn compute(&mut self, root: taffy::tree::NodeId) {
        self.tree.compute_layout(root, Size::MAX_CONTENT).unwrap();
    }

    pub fn compute_with_size(&mut self, root: taffy::tree::NodeId, width: f32, height: f32) {
        let available = Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        };
        self.tree.compute_layout(root, available).unwrap();
    }

    pub fn layout(&self, node: taffy::tree::NodeId) -> ComputedLayout {
        (*self.tree.layout(node).unwrap()).into()
    }

    pub fn children(&self, node: taffy::tree::NodeId) -> Vec<taffy::tree::NodeId> {
        self.tree.children(node).unwrap()
    }

    pub fn remove(&mut self, node: taffy::tree::NodeId) {
        self.tree.remove(node).unwrap();
    }

    pub fn add_child(&mut self, parent: taffy::tree::NodeId, child: taffy::tree::NodeId) {
        self.tree.add_child(parent, child).unwrap();
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}
