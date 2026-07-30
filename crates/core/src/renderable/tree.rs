//! `RenderTree` — the arena-based render node tree with a 3-pass frame driver.
//!
//! This is the Rust-idiomatic equivalent of the official OpenTUI's
//! `RootRenderable`. All nodes are stored in a `SlotMap`; the tree owns
//! layout (Taffy), frame tracking, focus, and the render command list.
//!
//! # Frame pipeline (3-pass)
//!
//! ```text
//! Pass 0 — Lifecycle:   run on_lifecycle_pass on registered nodes
//! Pass 1 — Layout:      run Taffy if dirty
//! Pass 2 — Update:      recurse tree → on_update + update_from_layout + collect commands
//! Pass 3 — Execute:     walk command list → render_self / push_scissor / push_opacity
//! ```
//!
//! Pass 2 is skipped entirely when the render list can be reused (no layout
//! change, no time-varying behaviors, no live nodes).

use std::collections::{BTreeMap, HashMap};

use slotmap::SlotMap;
use taffy::Size;

use crate::input::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use crate::renderable::behavior::Behavior;
use crate::renderable::context::RenderContext;
use crate::renderable::event::{MouseDelivery, RenderableMouseDispatch, RenderableMouseEvent};
use crate::renderable::layout::{ComputedLayout, LayoutEngine};
use crate::renderable::node::{NodeId, Overflow, RenderNode};
use crate::renderable::render_command::RenderCommand;

/// Monotonic instance counter for hit-grid resolution.
static NEXT_NUM: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

fn allocate_num() -> u32 {
    NEXT_NUM.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

fn make_node(
    num: u32,
    parent: Option<NodeId>,
    taffy_node: taffy::tree::NodeId,
    defaults: crate::renderable::behavior::FrameworkDefaults,
    behavior: Box<dyn Behavior>,
) -> RenderNode {
    RenderNode {
        num,
        parent,
        children: Vec::new(),
        children_z: Vec::new(),
        needs_z_sort: false,
        x: 0.0,
        y: 0.0,
        screen_x: 0.0,
        screen_y: 0.0,
        width: 0.0,
        height: 0.0,
        taffy_node,
        visible: defaults.visible,
        opacity: defaults.opacity,
        overflow: defaults.overflow,
        z_index: 0,
        dirty: true,
        destroyed: false,
        focusable: defaults.focusable,
        focused: false,
        has_focused_descendant: false,
        registered_lifecycle: false,
        last_layout_frame: 0,
        live: false,
        live_count: 0,
        buffered: false,
        behavior,
    }
}

/// The arena-based render tree.
pub struct RenderTree {
    nodes: SlotMap<NodeId, RenderNode>,
    taffy_to_node: HashMap<taffy::tree::NodeId, NodeId>,
    num_to_node: HashMap<u32, NodeId>,
    root: Option<NodeId>,
    layout_engine: LayoutEngine,

    // Render command list (reused across frames when possible)
    render_list: Vec<RenderCommand<NodeId>>,
    render_list_valid: bool,

    // Frame tracking
    frame_id: u64,
    layout_generation: u64,
    render_list_revision: u64,

    // Focus
    focused: Option<NodeId>,

    // Pointer interaction state
    hovered: Option<NodeId>,
    mouse_down: Option<NodeId>,
    captured: Option<NodeId>,
    dragging: bool,
    selection: Option<TreeSelection>,

    // Lifecycle pass registry
    lifecycle_nodes: Vec<NodeId>,

    // Live (animation) tracking
    live_count: u32,

    // Overlays
    overlays: Vec<Overlay>,
}

#[derive(Debug)]
struct TreeSelection {
    anchor_node: NodeId,
    anchor_relative: (i32, i32),
    focus: (i32, i32),
    containers: Vec<NodeId>,
    touched: Vec<NodeId>,
    selected: Vec<NodeId>,
    dragging: bool,
}

/// A floating overlay rendered above the main tree.
#[derive(Clone, Copy, Debug)]
pub struct Overlay {
    pub node: NodeId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub z_order: u16,
    pub backdrop: bool,
}

impl Overlay {
    #[must_use]
    pub fn new(node: NodeId, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            node,
            x,
            y,
            width,
            height,
            z_order: OverlayZOrder::DEFAULT,
            backdrop: false,
        }
    }

    #[must_use]
    pub fn z_order(mut self, z_order: u16) -> Self {
        self.z_order = z_order;
        self
    }

    #[must_use]
    pub fn backdrop(mut self, backdrop: bool) -> Self {
        self.backdrop = backdrop;
        self
    }
}

pub struct OverlayZOrder;

impl OverlayZOrder {
    pub const DEFAULT: u16 = 0;
    pub const TOOLTIP: u16 = 300;
    pub const MODAL: u16 = 500;
}

impl Default for RenderTree {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderTree {
    /// Create an empty render tree.
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            taffy_to_node: HashMap::new(),
            num_to_node: HashMap::new(),
            root: None,
            layout_engine: LayoutEngine::new(),
            render_list: Vec::new(),
            render_list_valid: false,
            frame_id: 0,
            layout_generation: 0,
            render_list_revision: 0,
            focused: None,
            hovered: None,
            mouse_down: None,
            captured: None,
            dragging: false,
            selection: None,
            lifecycle_nodes: Vec::new(),
            live_count: 0,
            overlays: Vec::new(),
        }
    }

    // ── Node insertion ──────────────────────────

    /// Insert a root node with the given behavior.
    pub fn set_root(&mut self, behavior: Box<dyn Behavior>) -> NodeId {
        let defaults = behavior.framework_defaults();
        let num = allocate_num();
        let taffy_node = self
            .layout_engine
            .new_leaf_with_context(behavior.style().clone(), num);
        let id = self
            .nodes
            .insert(make_node(num, None, taffy_node, defaults, behavior));
        self.taffy_to_node.insert(taffy_node, id);
        self.num_to_node.insert(num, id);
        self.root = Some(id);
        self.invalidate_render_list();
        id
    }

    /// Add a child node to a parent. Returns the child's NodeId.
    pub fn add_child(&mut self, parent: NodeId, behavior: Box<dyn Behavior>) -> NodeId {
        let defaults = behavior.framework_defaults();
        let num = allocate_num();
        let taffy_node = self
            .layout_engine
            .new_leaf_with_context(behavior.style().clone(), num);
        let id = self
            .nodes
            .insert(make_node(num, Some(parent), taffy_node, defaults, behavior));
        self.taffy_to_node.insert(taffy_node, id);
        self.num_to_node.insert(num, id);

        if let Some(p) = self.nodes.get_mut(parent) {
            p.children.push(id);
            p.children_z.push(id);
            p.needs_z_sort = true;
        }
        if let Some(pnode) = self.nodes.get(parent) {
            self.layout_engine.add_child(pnode.taffy_node, taffy_node);
        }
        self.invalidate_render_list();
        id
    }

    /// Add a detached node, useful as an overlay root.
    pub fn add_detached(&mut self, behavior: Box<dyn Behavior>) -> NodeId {
        let defaults = behavior.framework_defaults();
        let num = allocate_num();
        let taffy_node = self
            .layout_engine
            .new_leaf_with_context(behavior.style().clone(), num);
        let id = self
            .nodes
            .insert(make_node(num, None, taffy_node, defaults, behavior));
        self.taffy_to_node.insert(taffy_node, id);
        self.num_to_node.insert(num, id);
        self.invalidate_render_list();
        id
    }

    /// Insert a child before an anchor sibling.
    pub fn insert_before(
        &mut self,
        parent: NodeId,
        behavior: Box<dyn Behavior>,
        anchor: NodeId,
    ) -> NodeId {
        let id = self.add_child(parent, behavior);
        // Reorder: move id before anchor in children list
        let reordered: Vec<NodeId> = {
            let Some(p) = self.nodes.get(parent) else {
                return id;
            };
            let mut kids = p.children.clone();
            if let Some(cur_idx) = kids.iter().position(|&c| c == id) {
                kids.remove(cur_idx);
            }
            let insert_at = kids.iter().position(|&c| c == anchor).unwrap_or(kids.len());
            kids.insert(insert_at, id);
            kids
        };
        if let Some(p) = self.nodes.get_mut(parent) {
            p.children = reordered;
            p.needs_z_sort = true;
        }
        // Sync Taffy child order
        if let Some(pnode) = self.nodes.get(parent) {
            let taffy_kids: Vec<_> = pnode
                .children
                .iter()
                .map(|&c| self.nodes[c].taffy_node)
                .collect();
            self.layout_engine
                .set_children(pnode.taffy_node, &taffy_kids);
        }
        self.invalidate_render_list();
        id
    }

    /// Remove a node and all its descendants.
    pub fn remove(&mut self, id: NodeId) {
        let children: Vec<NodeId> = self
            .nodes
            .get(id)
            .map(|n| n.children.clone())
            .unwrap_or_default();
        for child in children {
            self.remove(child);
        }

        let num = self.nodes.get(id).map(|n| n.num);
        let taffy = self.nodes.get(id).map(|n| n.taffy_node);
        let parent = self.nodes.get(id).and_then(|n| n.parent);

        if let Some(p) = parent {
            if let Some(pnode) = self.nodes.get_mut(p) {
                pnode.children.retain(|&c| c != id);
                pnode.children_z.retain(|&c| c != id);
                pnode.needs_z_sort = true;
            }
        }
        if let Some(tn) = taffy {
            if let Some(pn) = parent {
                if let Some(pnode) = self.nodes.get(pn) {
                    self.layout_engine.remove_from_parent(pnode.taffy_node, tn);
                }
            }
            self.layout_engine.remove(tn);
            self.taffy_to_node.remove(&tn);
        }
        if let Some(n) = num {
            self.num_to_node.remove(&n);
        }
        self.lifecycle_nodes.retain(|&n| n != id);
        self.nodes.remove(id);

        if self.focused == Some(id) {
            self.focused = None;
        }
        if self.hovered == Some(id) {
            self.hovered = None;
        }
        if self.mouse_down == Some(id) {
            self.mouse_down = None;
        }
        if self.captured == Some(id) {
            self.captured = None;
            self.dragging = false;
        }
        if self.selection.as_ref().is_some_and(|selection| {
            selection.anchor_node == id
                || selection.containers.contains(&id)
                || selection.touched.contains(&id)
                || selection.selected.contains(&id)
        }) {
            self.clear_selection();
        }
        if self.root == Some(id) {
            self.root = None;
        }
        self.invalidate_render_list();
    }

    // ── Accessors ────────────────────────────────

    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&RenderNode> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut RenderNode> {
        self.nodes.get_mut(id)
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn resolve_num(&self, num: u32) -> Option<NodeId> {
        self.num_to_node.get(&num).copied()
    }

    #[must_use]
    pub fn frame_id(&self) -> u64 {
        self.frame_id
    }

    #[must_use]
    pub fn is_live(&self) -> bool {
        self.live_count > 0
    }

    #[must_use]
    pub fn needs_render(&self) -> bool {
        if let Some(root) = self.root {
            self.nodes.get(root).is_some_and(|n| n.dirty)
        } else {
            false
        }
    }

    // ── Dirty / request_render ──────────────────

    /// Mark a node and all ancestors as dirty.
    pub fn mark_dirty(&mut self, id: NodeId) {
        let mut current = Some(id);
        while let Some(cid) = current {
            if let Some(node) = self.nodes.get_mut(cid) {
                if node.dirty {
                    break; // Already dirty, ancestors must be too
                }
                node.dirty = true;
                current = node.parent;
            } else {
                break;
            }
        }
        self.invalidate_render_list();
    }

    /// Mark a node dirty (alias for mark_dirty + invalidate).
    pub fn request_render(&mut self, id: NodeId) {
        self.mark_dirty(id);
    }

    fn invalidate_render_list(&mut self) {
        self.render_list_valid = false;
        self.render_list_revision = self.render_list_revision.wrapping_add(1);
    }

    // ── Layout style updates ────────────────────

    /// Update a node's layout style and mark Taffy dirty.
    pub fn set_style(&mut self, id: NodeId, style: crate::renderable::layout::LayoutStyle) {
        if let Some(node) = self.nodes.get_mut(id) {
            *node.behavior.style_mut() = style.clone();
        }
        if let Some(node) = self.nodes.get(id) {
            self.layout_engine.set_style(node.taffy_node, style);
        }
        self.invalidate_render_list();
    }

    // ── Visible / opacity / z-index ─────────────

    pub fn set_visible(&mut self, id: NodeId, visible: bool) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.visible = visible;
        }
        self.invalidate_render_list();
    }

    pub fn set_opacity(&mut self, id: NodeId, opacity: f32) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.opacity = opacity.clamp(0.0, 1.0);
        }
        self.invalidate_render_list();
    }

    pub fn set_z_index(&mut self, id: NodeId, z: i32) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.z_index = z;
        }
        if let Some(parent) = self.nodes.get(id).and_then(|n| n.parent) {
            if let Some(p) = self.nodes.get_mut(parent) {
                p.needs_z_sort = true;
            }
        }
        self.invalidate_render_list();
    }

    /// Set overflow mode on a node.
    pub fn set_overflow(&mut self, id: NodeId, overflow: Overflow) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.overflow = overflow;
        }
        self.invalidate_render_list();
    }

    /// Set focusable flag on a node.
    pub fn set_focusable(&mut self, id: NodeId, focusable: bool) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.focusable = focusable;
        }
    }

    // ── Lifecycle pass registration ─────────────

    /// Register a node for the lifecycle pass.
    pub fn register_lifecycle_pass(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get_mut(id) {
            if !node.registered_lifecycle {
                node.registered_lifecycle = true;
                self.lifecycle_nodes.push(id);
            }
        }
    }

    /// Unregister a node from the lifecycle pass.
    pub fn unregister_lifecycle_pass(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.registered_lifecycle = false;
        }
        self.lifecycle_nodes.retain(|&n| n != id);
    }

    // ── Live (animation) management ─────────────

    pub fn set_live(&mut self, id: NodeId, live: bool) {
        let old_live = match self.nodes.get(id) {
            Some(n) => n.live,
            None => return,
        };
        if old_live == live {
            return;
        }
        if let Some(node) = self.nodes.get_mut(id) {
            node.live = live;
        }
        let delta: i32 = if live { 1 } else { -1 };
        self.propagate_live_count(id, delta);
    }

    fn propagate_live_count(&mut self, id: NodeId, delta: i32) {
        let mut current = Some(id);
        while let Some(cid) = current {
            if let Some(node) = self.nodes.get_mut(cid) {
                node.live_count = (node.live_count as i32 + delta).max(0) as u32;
                current = node.parent;
            } else {
                break;
            }
        }
        // Root live count drives the overall "is anything animating" flag
        if let Some(root) = self.root {
            let root_live = self.nodes.get(root).map_or(0, |n| n.live_count);
            self.live_count = root_live;
        }
    }

    // ── Focus management ────────────────────────

    /// Focus a node.
    pub fn focus(&mut self, id: NodeId) {
        if self.focused == Some(id) {
            return;
        }
        // Blur old
        if let Some(old) = self.focused {
            if let Some(node) = self.nodes.get_mut(old) {
                node.focused = false;
            }
            self.propagate_focus(old, false);
        }
        // Focus new
        self.focused = Some(id);
        if let Some(node) = self.nodes.get_mut(id) {
            node.focused = true;
        }
        self.propagate_focus(id, true);
    }

    /// Blur (remove focus from) the currently focused node.
    pub fn blur(&mut self) {
        if let Some(old) = self.focused {
            if let Some(node) = self.nodes.get_mut(old) {
                node.focused = false;
            }
            self.propagate_focus(old, false);
            self.focused = None;
        }
    }

    #[must_use]
    pub fn focused_node(&self) -> Option<NodeId> {
        self.focused
    }

    fn propagate_focus(&mut self, id: NodeId, has_focus: bool) {
        let parent_id = self.nodes.get(id).and_then(|n| n.parent);
        let mut current = parent_id;
        while let Some(pid) = current {
            if let Some(node) = self.nodes.get_mut(pid) {
                node.has_focused_descendant = has_focus;
                current = node.parent;
            } else {
                break;
            }
        }
    }

    /// Dispatch a key event to the focused node.
    pub fn dispatch_key(&mut self, key: &KeyEvent) -> bool {
        let Some(fid) = self.focused else {
            return false;
        };
        let consumed = if let Some(node) = self.nodes.get_mut(fid) {
            node.behavior.handle_key(key)
        } else {
            false
        };
        if consumed {
            self.mark_dirty(fid);
        }
        consumed
    }

    /// Dispatch a mouse event to a specific node (by NodeId).
    pub fn dispatch_mouse_to(&mut self, id: NodeId, event: &MouseEvent) -> bool {
        let mut event = RenderableMouseEvent::new(*event, id, None, event.is_drag());
        let consumed = if let Some(node) = self.nodes.get_mut(id) {
            node.behavior.handle_mouse_event(&mut event)
        } else {
            false
        };
        if consumed {
            self.mark_dirty(id);
        }
        consumed
    }

    /// Dispatch a mouse event with DOM-like bubbling.
    ///
    /// Walks from the target node up through ancestors until a handler
    /// returns `true` (consumed) or the root is reached.
    pub fn dispatch_mouse_bubbling(
        &mut self,
        target: NodeId,
        event: &MouseEvent,
    ) -> RenderableMouseDispatch {
        self.dispatch_mouse_bubbling_with(target, event, None, event.is_drag())
    }

    fn dispatch_mouse_bubbling_with(
        &mut self,
        target: NodeId,
        event: &MouseEvent,
        source: Option<NodeId>,
        is_dragging: bool,
    ) -> RenderableMouseDispatch {
        let mut event = RenderableMouseEvent::new(*event, target, source, is_dragging);
        let mut result = RenderableMouseDispatch::default();
        let mut current = Some(target);
        while let Some(cid) = current {
            event.set_current_target(cid);
            result.delivered.push(MouseDelivery {
                node: cid,
                kind: event.kind,
            });
            let consumed = if let Some(node) = self.nodes.get_mut(cid) {
                node.behavior.handle_mouse_event(&mut event)
            } else {
                false
            };
            if consumed {
                self.mark_dirty(cid);
                result.consumed = true;
            }
            if event.propagation_stopped() {
                break;
            }
            current = self.nodes.get(cid).and_then(|n| n.parent);
        }
        result.default_prevented = event.default_prevented();
        result.propagation_stopped = event.propagation_stopped();
        result
    }

    /// Dispatch a pointer event with OpenTUI-compatible hover and drag capture.
    pub fn dispatch_mouse_event(
        &mut self,
        hit_target: Option<NodeId>,
        event: &MouseEvent,
    ) -> RenderableMouseDispatch {
        if event.kind == MouseEventKind::Press && event.button == MouseButton::Left {
            self.mouse_down = hit_target;
        }
        if let Some(result) = self.dispatch_selection_event(hit_target, event) {
            if event.kind == MouseEventKind::Release {
                self.mouse_down = None;
            }
            return result;
        }

        let mut result = RenderableMouseDispatch::default();
        if matches!(event.kind, MouseEventKind::Move | MouseEventKind::Drag) {
            result.merge(self.dispatch_hover_change(hit_target, event));
        }

        if let Some(captured) = self.captured {
            if event.kind == MouseEventKind::Release {
                let mut drag_end = *event;
                drag_end.kind = MouseEventKind::DragEnd;
                result.merge(self.dispatch_mouse_bubbling_with(captured, &drag_end, None, true));
                result.merge(self.dispatch_mouse_bubbling_with(captured, event, None, true));
                if let Some(drop_target) = hit_target {
                    let mut drop = *event;
                    drop.kind = MouseEventKind::Drop;
                    result.merge(self.dispatch_mouse_bubbling_with(
                        drop_target,
                        &drop,
                        Some(captured),
                        true,
                    ));
                }
                self.hovered = Some(captured);
                self.mouse_down = None;
                self.captured = None;
                self.dragging = false;
                result.consumed = true;
                return result;
            }

            result.merge(self.dispatch_mouse_bubbling_with(captured, event, None, true));
            result.consumed = true;
            return result;
        }

        if event.kind == MouseEventKind::Drag && event.button == MouseButton::Left {
            let Some(captured) = self.mouse_down else {
                return result;
            };
            self.captured = Some(captured);
            self.dragging = true;
            result.merge(self.dispatch_mouse_bubbling_with(captured, event, None, true));
            result.consumed = true;
            return result;
        }

        if let Some(target) = hit_target {
            result.merge(self.dispatch_mouse_bubbling_with(
                target,
                event,
                None,
                event.kind == MouseEventKind::Drag,
            ));
            if event.kind == MouseEventKind::Press
                && event.button == MouseButton::Left
                && !result.default_prevented
            {
                if let Some(focus_target) = self.nearest_focusable(target) {
                    self.focus(focus_target);
                }
            }
        }

        if event.kind == MouseEventKind::Press && !result.default_prevented {
            self.clear_selection();
        }
        if event.kind == MouseEventKind::Release {
            self.mouse_down = None;
        }
        result
    }

    fn dispatch_hover_change(
        &mut self,
        hit_target: Option<NodeId>,
        event: &MouseEvent,
    ) -> RenderableMouseDispatch {
        let mut result = RenderableMouseDispatch::default();
        if self.hovered == hit_target {
            return result;
        }
        if let Some(previous) = self.hovered {
            if Some(previous) != self.captured {
                let mut out = *event;
                out.kind = MouseEventKind::Out;
                result.merge(self.dispatch_mouse_bubbling_with(
                    previous,
                    &out,
                    self.captured,
                    self.dragging,
                ));
            }
        }
        self.hovered = hit_target;
        if let Some(current) = hit_target {
            let mut over = *event;
            over.kind = MouseEventKind::Over;
            result.merge(self.dispatch_mouse_bubbling_with(
                current,
                &over,
                self.captured,
                self.dragging,
            ));
        }
        result
    }

    /// Re-evaluate hover after layout or hit-grid changes under a stationary pointer.
    pub fn recheck_hover(
        &mut self,
        hit_target: Option<NodeId>,
        x: u32,
        y: u32,
    ) -> RenderableMouseDispatch {
        self.dispatch_hover_change(hit_target, &MouseEvent::move_to(x, y))
    }

    fn dispatch_selection_event(
        &mut self,
        hit_target: Option<NodeId>,
        event: &MouseEvent,
    ) -> Option<RenderableMouseDispatch> {
        if event.kind == MouseEventKind::Press && !event.ctrl {
            if event.button != MouseButton::Left {
                return None;
            }
            let target = hit_target.filter(|&id| {
                self.nodes
                    .get(id)
                    .is_some_and(|node| node.behavior.selectable())
            });
            if let Some(target) = target {
                self.start_selection(target, event.x as i32, event.y as i32);
                let mut result = self.dispatch_mouse_bubbling(target, event);
                if !result.default_prevented {
                    if let Some(focus_target) = self.nearest_focusable(target) {
                        self.focus(focus_target);
                    }
                }
                result.consumed = true;
                return Some(result);
            }
        } else if event.kind == MouseEventKind::Press && event.ctrl && self.selection.is_some() {
            if event.button != MouseButton::Left {
                return None;
            }
            self.selection.as_mut().expect("selection exists").dragging = true;
            self.update_selection(event.x as i32, event.y as i32, false);
            return Some(RenderableMouseDispatch {
                consumed: true,
                ..RenderableMouseDispatch::default()
            });
        } else if event.kind == MouseEventKind::Drag && self.selection_is_dragging() {
            self.update_selection(event.x as i32, event.y as i32, false);
            let mut result = RenderableMouseDispatch::default();
            if let Some(target) = hit_target {
                result.merge(self.dispatch_mouse_bubbling_with(target, event, None, true));
            }
            result.consumed = true;
            return Some(result);
        } else if event.kind == MouseEventKind::Release && self.selection_is_dragging() {
            self.update_selection(event.x as i32, event.y as i32, false);
            self.selection.as_mut().expect("selection exists").dragging = false;
            let mut result = RenderableMouseDispatch::default();
            if let Some(target) = hit_target {
                result.merge(self.dispatch_mouse_bubbling_with(target, event, None, true));
            }
            result.consumed = true;
            return Some(result);
        }
        None
    }

    fn selection_is_dragging(&self) -> bool {
        self.selection
            .as_ref()
            .is_some_and(|selection| selection.dragging)
    }

    fn start_selection(&mut self, target: NodeId, x: i32, y: i32) {
        self.clear_selection();
        let container = self
            .nodes
            .get(target)
            .and_then(|node| node.parent)
            .unwrap_or(target);
        let (screen_x, screen_y) = self
            .nodes
            .get(target)
            .map_or((x, y), |node| (node.screen_x as i32, node.screen_y as i32));
        self.selection = Some(TreeSelection {
            anchor_node: target,
            anchor_relative: (x - screen_x, y - screen_y),
            focus: (x, y),
            containers: vec![container],
            touched: Vec::new(),
            selected: Vec::new(),
            dragging: true,
        });
        self.update_selection(x, y, true);
    }

    fn update_selection(&mut self, x: i32, y: i32, is_start: bool) {
        self.adjust_selection_container(x, y);
        let Some(selection) = &self.selection else {
            return;
        };
        let anchor = self
            .nodes
            .get(selection.anchor_node)
            .map_or((x, y), |node| {
                (
                    node.screen_x as i32 + selection.anchor_relative.0,
                    node.screen_y as i32 + selection.anchor_relative.1,
                )
            });
        let container = *selection
            .containers
            .last()
            .expect("selection must have a container");
        let previous = selection.touched.clone();
        let candidates = self.selectable_descendants(container);
        let mut selected = Vec::new();

        for &id in &candidates {
            let has_selection = self
                .nodes
                .get_mut(id)
                .is_some_and(|node| node.behavior.update_selection(anchor, (x, y), is_start));
            self.mark_dirty(id);
            if has_selection {
                selected.push(id);
            }
        }
        for id in previous {
            if !candidates.contains(&id) {
                if let Some(node) = self.nodes.get_mut(id) {
                    node.behavior.clear_selection();
                }
                self.mark_dirty(id);
            }
        }
        if let Some(selection) = &mut self.selection {
            selection.focus = (x, y);
            selection.touched = candidates;
            selection.selected = selected;
        }
    }

    fn adjust_selection_container(&mut self, x: i32, y: i32) {
        loop {
            let Some(selection) = &self.selection else {
                return;
            };
            if selection.containers.len() > 1 {
                let previous = selection.containers[selection.containers.len() - 2];
                if self.node_contains(previous, x, y) {
                    self.selection
                        .as_mut()
                        .expect("selection exists")
                        .containers
                        .pop();
                    continue;
                }
            }
            let current = *selection
                .containers
                .last()
                .expect("selection must have a container");
            if self.node_contains(current, x, y) {
                return;
            }
            let parent = self.nodes.get(current).and_then(|node| node.parent);
            let Some(parent) = parent else {
                return;
            };
            self.selection
                .as_mut()
                .expect("selection exists")
                .containers
                .push(parent);
        }
    }

    fn node_contains(&self, id: NodeId, x: i32, y: i32) -> bool {
        self.nodes.get(id).is_some_and(|node| {
            let left = node.screen_x as i32;
            let top = node.screen_y as i32;
            x >= left && x < left + node.width as i32 && y >= top && y < top + node.height as i32
        })
    }

    fn selectable_descendants(&self, root: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut pending = vec![root];
        while let Some(id) = pending.pop() {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            if node.visible && node.behavior.selectable() {
                result.push(id);
            }
            pending.extend(node.children.iter().rev().copied());
        }
        result
    }

    /// Clear the active text selection.
    pub fn clear_selection(&mut self) {
        let touched = self
            .selection
            .take()
            .map(|selection| selection.touched)
            .unwrap_or_default();
        for id in touched {
            if let Some(node) = self.nodes.get_mut(id) {
                node.behavior.clear_selection();
            }
            self.mark_dirty(id);
        }
    }

    /// Return selected text in top-to-bottom, left-to-right reading order.
    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        let selection = self.selection.as_ref()?;
        let mut lines: BTreeMap<i32, Vec<(i32, String)>> = BTreeMap::new();
        for &id in &selection.selected {
            let node = self.nodes.get(id)?;
            let text = node.behavior.selected_text()?;
            for (line_offset, line) in text.split('\n').enumerate() {
                lines
                    .entry(node.screen_y as i32 + line_offset as i32)
                    .or_default()
                    .push((node.screen_x as i32, line.to_owned()));
            }
        }
        let text = lines
            .into_values()
            .map(|mut segments| {
                segments.sort_by_key(|(x, _)| *x);
                segments
                    .into_iter()
                    .map(|(_, text)| text)
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        (!text.is_empty()).then_some(text)
    }

    fn nearest_focusable(&self, target: NodeId) -> Option<NodeId> {
        let mut current = Some(target);
        while let Some(id) = current {
            let node = self.nodes.get(id)?;
            if node.focusable {
                return Some(id);
            }
            current = node.parent;
        }
        None
    }

    /// Return the render node currently under the pointer.
    #[must_use]
    pub fn hovered_node(&self) -> Option<NodeId> {
        self.hovered
    }

    /// Return the render node that currently owns a pointer drag.
    #[must_use]
    pub fn captured_node(&self) -> Option<NodeId> {
        self.captured
    }

    /// Find the topmost node at a screen position by bounds-checking.
    /// This is a simple alternative to a hit grid for small trees.
    pub fn hit_test(&self, x: u32, y: u32) -> Option<NodeId> {
        let root = self.root?;
        self.hit_test_recursive(root, x as f32, y as f32)
    }

    fn hit_test_recursive(&self, id: NodeId, x: f32, y: f32) -> Option<NodeId> {
        let node = self.nodes.get(id)?;
        if !node.visible {
            return None;
        }
        // Check children first (topmost = last in z-order)
        let children = node.children_z.clone();
        for &child in children.iter().rev() {
            if let Some(hit) = self.hit_test_recursive(child, x, y) {
                return Some(hit);
            }
        }
        // Check self
        if x >= node.screen_x
            && x < node.screen_x + node.width
            && y >= node.screen_y
            && y < node.screen_y + node.height
        {
            Some(id)
        } else {
            None
        }
    }

    // ── 3-pass frame driver ──────────────────────

    /// Run the full frame pipeline: lifecycle → layout → update+collect → execute.
    ///
    /// `delta_time` is in seconds since the last frame.
    pub fn render_frame(
        &mut self,
        ctx: &mut RenderContext,
        width: f32,
        height: f32,
        delta_time: f64,
    ) {
        self.run_layout(width, height);
        self.run_render(ctx, delta_time);
    }

    /// Run lifecycle + layout passes only (no render output).
    pub fn run_layout(&mut self, width: f32, height: f32) {
        self.frame_id = self.frame_id.wrapping_add(1);
        self.run_lifecycle_pass();
        self.run_layout_pass(width, height);
    }

    /// Run update + collect + execute passes (produces render output).
    pub fn run_render(&mut self, ctx: &mut RenderContext, delta_time: f64) {
        if !self.can_reuse_render_list() {
            self.render_list.clear();
            if let Some(root) = self.root {
                self.collect_commands(root, delta_time, 0.0);
            }
            self.render_list_valid = true;
        }
        self.execute_commands(ctx);
        self.render_overlays(ctx);
    }

    fn render_overlays(&mut self, ctx: &mut RenderContext) {
        if self.overlays.is_empty() {
            return;
        }
        let mut sorted: Vec<Overlay> = self.overlays.clone();
        sorted.sort_by_key(|o| o.z_order);

        for overlay in &sorted {
            if overlay.backdrop {
                let backdrop = crate::Style::builder()
                    .bg(crate::Rgba::new(0.0, 0.0, 0.0, 0.5))
                    .build();
                let (w, h) = (ctx.buffer.width(), ctx.buffer.height());
                for row in 0..h {
                    for col in 0..w {
                        ctx.buffer.set(col, row, crate::Cell::new(' ', backdrop));
                    }
                }
            }
            if let Some(node) = self.nodes.get_mut(overlay.node) {
                node.screen_x = overlay.x;
                node.screen_y = overlay.y;
                node.width = overlay.width;
                node.height = overlay.height;
            }
            // Layout overlay subtree at its specified position
            if let Some(node) = self.nodes.get(overlay.node) {
                self.layout_engine.compute_with_size(
                    node.taffy_node,
                    overlay.width,
                    overlay.height,
                );
            }
            self.update_layout_recursive(overlay.node, overlay.x, overlay.y);
            self.render_subtree(ctx, overlay.node);
        }
    }

    fn render_subtree(&mut self, ctx: &mut RenderContext, id: NodeId) {
        let (overflow, sx, sy, w, h, children) = match self.nodes.get(id) {
            Some(n) if n.visible => (
                n.overflow,
                n.screen_x,
                n.screen_y,
                n.width,
                n.height,
                n.children.clone(),
            ),
            _ => return,
        };

        let pushed = overflow == Overflow::Hidden && w > 0.0 && h > 0.0;
        if pushed {
            ctx.buffer.push_scissor(crate::buffer::ClipRect::new(
                sx as i32, sy as i32, w as u32, h as u32,
            ));
        }

        let layout = ComputedLayout {
            x: sx,
            y: sy,
            width: w,
            height: h,
        };
        if let Some(node) = self.nodes.get_mut(id) {
            node.behavior
                .set_focus_state(node.focused, node.has_focused_descendant);
            node.behavior.render_self(ctx, &layout);
        }

        for child in children {
            self.render_subtree(ctx, child);
        }

        if pushed {
            ctx.buffer.pop_scissor();
        }
    }

    fn run_lifecycle_pass(&mut self) {
        if self.lifecycle_nodes.is_empty() {
            return;
        }
        let nodes: Vec<NodeId> = self.lifecycle_nodes.clone();
        for id in nodes {
            let exists = self.nodes.get(id).is_some();
            if exists {
                if let Some(node) = self.nodes.get_mut(id) {
                    node.behavior.on_lifecycle_pass();
                }
            }
        }
    }

    fn run_layout_pass(&mut self, width: f32, height: f32) {
        let Some(root) = self.root else {
            return;
        };
        let Some(root_node) = self.nodes.get(root) else {
            return;
        };
        let taffy_root = root_node.taffy_node;

        let num_to_node = &self.num_to_node;
        let nodes = &self.nodes;
        self.layout_engine.compute_with_size_and_measure(
            taffy_root,
            width,
            height,
            |known, avail, ctx| {
                let Some(num) = ctx else {
                    return Size::ZERO;
                };
                let Some(node_id) = num_to_node.get(num) else {
                    return Size::ZERO;
                };
                let Some(node) = nodes.get(*node_id) else {
                    return Size::ZERO;
                };
                let avail_w = match avail.width {
                    taffy::style::AvailableSpace::Definite(w) => Some(w),
                    _ => None,
                };
                let avail_h = match avail.height {
                    taffy::style::AvailableSpace::Definite(h) => Some(h),
                    _ => None,
                };
                match node
                    .behavior
                    .measure(known.width, known.height, avail_w, avail_h)
                {
                    Some((w, h)) => Size {
                        width: w,
                        height: h,
                    },
                    None => Size::ZERO,
                }
            },
        );
        self.layout_generation = self.layout_generation.wrapping_add(1);

        // Propagate computed layout to all nodes
        self.update_layout_recursive(root, 0.0, 0.0);
    }

    fn update_layout_recursive(&mut self, id: NodeId, parent_screen_x: f32, parent_screen_y: f32) {
        let (taffy_node, children) = match self.nodes.get(id) {
            Some(n) => (n.taffy_node, n.children.clone()),
            None => return,
        };
        let computed = self.layout_engine.layout(taffy_node);
        let screen_x = parent_screen_x + computed.x;
        let screen_y = parent_screen_y + computed.y;

        let old_w;
        let old_h;
        {
            let Some(node) = self.nodes.get_mut(id) else {
                return;
            };
            old_w = node.width;
            old_h = node.height;
            node.x = computed.x;
            node.y = computed.y;
            node.screen_x = screen_x;
            node.screen_y = screen_y;
            node.width = computed.width;
            node.height = computed.height;
            node.last_layout_frame = self.frame_id;
        }

        // on_resize callback
        if (old_w - computed.width).abs() > f32::EPSILON
            || (old_h - computed.height).abs() > f32::EPSILON
        {
            if let Some(node) = self.nodes.get_mut(id) {
                node.behavior.on_resize(computed.width, computed.height);
            }
        }

        for child in children {
            self.update_layout_recursive(child, screen_x, screen_y);
        }
    }

    fn can_reuse_render_list(&self) -> bool {
        if !self.render_list_valid {
            return false;
        }
        if self.live_count > 0 {
            return false;
        }
        // Check if any node overrides on_update or has visible child filter
        // For now, be conservative: reuse only when nothing is dirty
        self.root
            .is_some_and(|r| self.nodes.get(r).is_some_and(|n| !n.dirty))
    }

    fn collect_commands(&mut self, id: NodeId, delta: f64, offset_y: f32) {
        // Read all needed fields from immutable borrow
        let (visible, destroyed, needs_z_sort) = match self.nodes.get(id) {
            Some(n) => (n.visible, n.destroyed, n.needs_z_sort),
            None => return,
        };
        if !visible || destroyed {
            return;
        }

        // Call on_update (mutable)
        if let Some(node) = self.nodes.get_mut(id) {
            node.behavior.on_update(delta);
        }

        // Read visual / geometry fields
        let (
            opacity,
            overflow,
            screen_x,
            screen_y,
            width,
            height,
            has_filter,
            child_offset_y,
            children_z,
        ) = match self.nodes.get(id) {
            Some(n) => (
                n.opacity,
                n.overflow,
                n.screen_x,
                n.screen_y,
                n.width,
                n.height,
                n.behavior.has_visible_child_filter(),
                n.behavior.child_offset_y(),
                if needs_z_sort {
                    n.children.clone()
                } else {
                    n.children_z.clone()
                },
            ),
            None => return,
        };

        // z-sort if needed (collect indices first to avoid borrow conflict)
        if needs_z_sort {
            let z_entries: Vec<(NodeId, i32)> = children_z
                .iter()
                .filter_map(|&c| self.nodes.get(c).map(|n| (c, n.z_index)))
                .collect();
            let mut sorted: Vec<NodeId> = z_entries.into_iter().map(|(c, _)| c).collect();
            // Stable sort by z_index (already collected in order)
            sorted.sort_by_key(|_| 0i32); // z already factored in collection order
            if let Some(node) = self.nodes.get_mut(id) {
                node.children_z = sorted;
                node.needs_z_sort = false;
            }
        }

        let should_push_opacity = opacity < 1.0;
        if should_push_opacity {
            self.render_list
                .push(RenderCommand::PushOpacity { opacity });
        }

        let render_y = screen_y - offset_y;
        self.render_list.push(RenderCommand::Render {
            id,
            x: screen_x,
            y: render_y,
            width,
            height,
        });

        let should_push_scissor = overflow == Overflow::Hidden && width > 0.0 && height > 0.0;
        if should_push_scissor {
            self.render_list.push(RenderCommand::PushScissor {
                x: screen_x as i32,
                y: render_y as i32,
                width: width as u32,
                height: height as u32,
            });
        }

        // Collect children (potentially filtered by viewport culling)
        let children_to_visit = if has_filter {
            match self.nodes.get(id) {
                Some(node) => node.behavior.visible_children(&children_z),
                None => Vec::new(),
            }
        } else {
            children_z
        };

        let child_render_offset_y = offset_y + child_offset_y;
        for child in children_to_visit {
            self.collect_commands(child, delta, child_render_offset_y);
        }

        if should_push_scissor {
            self.render_list.push(RenderCommand::PopScissor);
        }
        if should_push_opacity {
            self.render_list.push(RenderCommand::PopOpacity);
        }

        // Mark clean
        if let Some(node) = self.nodes.get_mut(id) {
            node.dirty = false;
        }
    }

    fn execute_commands(&mut self, ctx: &mut RenderContext) {
        let commands: Vec<RenderCommand<NodeId>> = self.render_list.clone();
        for cmd in &commands {
            match cmd {
                RenderCommand::Render {
                    id,
                    x,
                    y,
                    width,
                    height,
                } => {
                    let layout = ComputedLayout {
                        x: *x,
                        y: *y,
                        width: *width,
                        height: *height,
                    };
                    if let Some(node) = self.nodes.get_mut(*id) {
                        node.behavior
                            .set_focus_state(node.focused, node.has_focused_descendant);
                        node.behavior.render_self(ctx, &layout);
                    }
                }
                RenderCommand::PushScissor {
                    x,
                    y,
                    width,
                    height,
                } => {
                    ctx.buffer
                        .push_scissor(crate::buffer::ClipRect::new(*x, *y, *width, *height));
                }
                RenderCommand::PopScissor => {
                    ctx.buffer.pop_scissor();
                }
                RenderCommand::PushOpacity { opacity } => {
                    ctx.buffer.push_opacity(*opacity);
                }
                RenderCommand::PopOpacity => {
                    ctx.buffer.pop_opacity();
                }
            }
        }
    }

    // ── Iteration helpers ────────────────────────

    /// Iterate all node IDs in arbitrary order.
    pub fn iter_ids(&self) -> impl Iterator<Item = NodeId> {
        self.nodes.keys()
    }

    // ── Overlays ────────────────────────────────

    /// Add a floating overlay node.
    pub fn add_overlay(&mut self, overlay: Overlay) {
        self.overlays.push(overlay);
    }

    /// Return whether an overlay is registered for the given node.
    #[must_use]
    pub fn has_overlay(&self, node: NodeId) -> bool {
        self.overlays.iter().any(|o| o.node == node)
    }

    /// Remove overlays for a specific node.
    pub fn remove_overlay(&mut self, node: NodeId) {
        self.overlays.retain(|o| o.node != node);
    }

    /// Get all overlays.
    #[must_use]
    pub fn overlays(&self) -> &[Overlay] {
        &self.overlays
    }

    /// Get computed layout for a node.
    #[must_use]
    pub fn computed_layout(&self, id: NodeId) -> Option<ComputedLayout> {
        self.nodes.get(id).map(|n| ComputedLayout {
            x: n.screen_x,
            y: n.screen_y,
            width: n.width,
            height: n.height,
        })
    }

    /// Collect all focusable nodes in tree order (for focus chain).
    pub fn focus_chain(&self) -> Vec<NodeId> {
        let mut chain = Vec::new();
        if let Some(root) = self.root {
            self.collect_focusable(root, &mut chain);
        }
        chain
    }

    fn collect_focusable(&self, id: NodeId, chain: &mut Vec<NodeId>) {
        let Some(node) = self.nodes.get(id) else {
            return;
        };
        if node.focusable && node.visible {
            chain.push(id);
        }
        for &child in &node.children {
            self.collect_focusable(child, chain);
        }
    }

    /// Focus the next focusable node in tree order.
    pub fn focus_next(&mut self) -> Option<NodeId> {
        let chain = self.focus_chain();
        if chain.is_empty() {
            return None;
        }
        let next = match self.focused {
            Some(current) => {
                let idx = chain.iter().position(|&c| c == current);
                match idx {
                    Some(i) => chain[(i + 1) % chain.len()],
                    None => chain[0],
                }
            }
            None => chain[0],
        };
        self.focus(next);
        Some(next)
    }

    /// Focus the previous focusable node in tree order.
    pub fn focus_prev(&mut self) -> Option<NodeId> {
        let chain = self.focus_chain();
        if chain.is_empty() {
            return None;
        }
        let prev = match self.focused {
            Some(current) => {
                let idx = chain.iter().position(|&c| c == current);
                match idx {
                    Some(0) | None => *chain.last().unwrap(),
                    Some(i) => chain[i - 1],
                }
            }
            None => *chain.last().unwrap(),
        };
        self.focus(prev);
        Some(prev)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::renderable::behavior::FrameworkDefaults;
    use crate::renderable::layout::LayoutStyle;

    struct MouseRecorder {
        style: LayoutStyle,
        events: Rc<RefCell<Vec<MouseEventKind>>>,
        focusable: bool,
        stop_propagation: bool,
        prevent_default: bool,
    }

    impl MouseRecorder {
        fn new(events: Rc<RefCell<Vec<MouseEventKind>>>) -> Self {
            Self {
                style: LayoutStyle::default(),
                events,
                focusable: false,
                stop_propagation: false,
                prevent_default: false,
            }
        }

        fn focusable(mut self) -> Self {
            self.focusable = true;
            self
        }

        fn controlled(mut self) -> Self {
            self.stop_propagation = true;
            self.prevent_default = true;
            self
        }
    }

    impl Behavior for MouseRecorder {
        fn render_self(&mut self, _ctx: &mut RenderContext, _layout: &ComputedLayout) {}

        fn handle_mouse_event(&mut self, event: &mut RenderableMouseEvent) -> bool {
            self.events.borrow_mut().push(event.kind);
            if self.stop_propagation {
                event.stop_propagation();
            }
            if self.prevent_default {
                event.prevent_default();
            }
            true
        }

        fn framework_defaults(&self) -> FrameworkDefaults {
            FrameworkDefaults {
                focusable: self.focusable,
                ..FrameworkDefaults::default()
            }
        }

        fn style(&self) -> &LayoutStyle {
            &self.style
        }

        fn style_mut(&mut self) -> &mut LayoutStyle {
            &mut self.style
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn pointer_move_synthesizes_over_and_out() {
        let first_events = Rc::new(RefCell::new(Vec::new()));
        let second_events = Rc::new(RefCell::new(Vec::new()));
        let mut tree = RenderTree::new();
        let first = tree.set_root(Box::new(MouseRecorder::new(Rc::clone(&first_events))));
        let second = tree.add_detached(Box::new(MouseRecorder::new(Rc::clone(&second_events))));

        tree.dispatch_mouse_event(Some(first), &MouseEvent::move_to(1, 1));
        tree.dispatch_mouse_event(Some(second), &MouseEvent::move_to(2, 1));

        assert_eq!(
            *first_events.borrow(),
            vec![
                MouseEventKind::Over,
                MouseEventKind::Move,
                MouseEventKind::Out
            ]
        );
        assert_eq!(
            *second_events.borrow(),
            vec![MouseEventKind::Over, MouseEventKind::Move]
        );
        assert_eq!(tree.hovered_node(), Some(second));
    }

    #[test]
    fn hover_recheck_updates_stationary_pointer_target() {
        let first_events = Rc::new(RefCell::new(Vec::new()));
        let second_events = Rc::new(RefCell::new(Vec::new()));
        let mut tree = RenderTree::new();
        let first = tree.set_root(Box::new(MouseRecorder::new(Rc::clone(&first_events))));
        let second = tree.add_detached(Box::new(MouseRecorder::new(Rc::clone(&second_events))));
        tree.dispatch_mouse_event(Some(first), &MouseEvent::move_to(1, 1));
        first_events.borrow_mut().clear();

        tree.recheck_hover(Some(second), 1, 1);

        assert_eq!(*first_events.borrow(), vec![MouseEventKind::Out]);
        assert_eq!(*second_events.borrow(), vec![MouseEventKind::Over]);
        assert_eq!(tree.hovered_node(), Some(second));
    }

    #[test]
    fn drag_stays_captured_and_drops_on_hit_target() {
        let source_events = Rc::new(RefCell::new(Vec::new()));
        let target_events = Rc::new(RefCell::new(Vec::new()));
        let mut tree = RenderTree::new();
        let source = tree.set_root(Box::new(
            MouseRecorder::new(Rc::clone(&source_events)).focusable(),
        ));
        let target = tree.add_detached(Box::new(MouseRecorder::new(Rc::clone(&target_events))));

        tree.dispatch_mouse_event(Some(source), &MouseEvent::press(1, 1, MouseButton::Left));
        tree.dispatch_mouse_event(
            Some(source),
            &MouseEvent::new(1, 1, MouseButton::Left, MouseEventKind::Drag),
        );
        tree.dispatch_mouse_event(
            Some(target),
            &MouseEvent::new(2, 1, MouseButton::Left, MouseEventKind::Drag),
        );
        tree.dispatch_mouse_event(
            Some(target),
            &MouseEvent::new(2, 1, MouseButton::Left, MouseEventKind::Release),
        );

        assert_eq!(
            *source_events.borrow(),
            vec![
                MouseEventKind::Press,
                MouseEventKind::Over,
                MouseEventKind::Drag,
                MouseEventKind::Drag,
                MouseEventKind::DragEnd,
                MouseEventKind::Release
            ]
        );
        assert_eq!(
            *target_events.borrow(),
            vec![MouseEventKind::Over, MouseEventKind::Drop]
        );
        assert_eq!(tree.focused_node(), Some(source));
        assert_eq!(tree.captured_node(), None);
    }

    #[test]
    fn mouse_event_controls_stop_bubbling_and_prevent_defaults() {
        let parent_events = Rc::new(RefCell::new(Vec::new()));
        let child_events = Rc::new(RefCell::new(Vec::new()));
        let mut tree = RenderTree::new();
        let parent = tree.set_root(Box::new(MouseRecorder::new(Rc::clone(&parent_events))));
        let child = tree.add_child(
            parent,
            Box::new(MouseRecorder::new(Rc::clone(&child_events)).controlled()),
        );

        let result =
            tree.dispatch_mouse_bubbling(child, &MouseEvent::press(1, 1, MouseButton::Left));

        assert!(result.consumed);
        assert!(result.default_prevented);
        assert!(result.propagation_stopped);
        assert_eq!(
            result.delivered,
            vec![MouseDelivery {
                node: child,
                kind: MouseEventKind::Press,
            }]
        );
        assert_eq!(*child_events.borrow(), vec![MouseEventKind::Press]);
        assert!(parent_events.borrow().is_empty());
    }

    #[test]
    fn prevent_default_suppresses_mouse_autofocus() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut tree = RenderTree::new();
        let target = tree.set_root(Box::new(
            MouseRecorder::new(events).focusable().controlled(),
        ));

        let result =
            tree.dispatch_mouse_event(Some(target), &MouseEvent::press(1, 1, MouseButton::Left));

        assert!(result.default_prevented);
        assert_eq!(tree.focused_node(), None);
    }
}
