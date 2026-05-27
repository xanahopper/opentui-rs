//! Base widget trait and widget tree.
//!
//! A `Widget` is the fundamental UI unit. It owns a Taffy layout node and
//! knows how to draw itself into an `OptimizedBuffer`. Widgets form a tree;
//! the `WidgetTree` manages parent-child relationships and drives the
//! layout -> render pipeline.
//!
//! # Lifecycle
//!
//! 1. Build the widget tree (add children, set styles)
//! 2. `WidgetTree::layout()` -- runs Taffy, assigns `ComputedLayout` to each widget
//! 3. `WidgetTree::build_render_commands()` -- traverses tree, emits `RenderCommand`s
//! 4. `WidgetTree::execute_render_commands()` -- applies commands to buffer

use std::collections::HashMap;

use opentui_rust as ot;
use opentui_rust::OptimizedBuffer;

use crate::layout::{ComputedLayout, LayoutEngine, LayoutStyle};
use crate::render_command::{RenderCommand, RenderCommandList};

pub type WidgetId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    Visible,
    Hidden,
}

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub buffer: &'a mut OptimizedBuffer,
    pub grapheme_pool: Option<&'a mut ot::GraphemePool>,
    pub link_pool: Option<&'a mut ot::LinkPool>,
    pub hit_grid: Option<&'a mut ot::renderer::HitGrid>,
    pub theme: Option<&'a crate::theme::UiTheme>,
}

pub trait Widget {
    fn id(&self) -> WidgetId;

    fn style(&self) -> &LayoutStyle;

    fn style_mut(&mut self) -> &mut LayoutStyle;

    fn render(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout);

    fn children(&self) -> &[WidgetId] {
        &[]
    }

    fn visible(&self) -> bool {
        true
    }

    fn opacity(&self) -> f32 {
        1.0
    }

    fn overflow(&self) -> Overflow {
        Overflow::Visible
    }

    fn focusable(&self) -> bool {
        false
    }

    fn focused(&self) -> bool {
        false
    }

    fn set_focused(&mut self, _focused: bool) {}

    fn handle_key(&mut self, _key: &ot::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _mouse: &ot::MouseEvent) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any;

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

struct WidgetNode {
    widget: Box<dyn Widget>,
    parent: Option<WidgetId>,
    children: Vec<WidgetId>,
    computed_layout: Option<ComputedLayout>,
    has_focused_descendant: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayZOrder(u16);

impl OverlayZOrder {
    pub const BOTTOM: Self = Self(0);
    pub const MIDDLE: Self = Self(100);
    pub const TOP: Self = Self(200);
    pub const TOOLTIP: Self = Self(300);
    pub const MODAL: Self = Self(400);

    pub fn new(z: u16) -> Self {
        Self(z)
    }

    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for OverlayZOrder {
    fn default() -> Self {
        Self::MIDDLE
    }
}

impl Ord for OverlayZOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for OverlayZOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct Overlay {
    pub widget_id: WidgetId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub z_order: OverlayZOrder,
    pub backdrop: bool,
}

impl Overlay {
    pub fn new(widget_id: WidgetId, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            widget_id,
            x,
            y,
            width,
            height,
            z_order: OverlayZOrder::default(),
            backdrop: false,
        }
    }

    pub fn z_order(mut self, z: OverlayZOrder) -> Self {
        self.z_order = z;
        self
    }

    pub fn backdrop(mut self, enable: bool) -> Self {
        self.backdrop = enable;
        self
    }
}

pub struct WidgetTree {
    nodes: HashMap<WidgetId, WidgetNode>,
    layout_engine: LayoutEngine,
    taffy_nodes: HashMap<WidgetId, taffy::tree::NodeId>,
    next_id: WidgetId,
    root: Option<WidgetId>,
    render_commands: RenderCommandList,
    focus_chain: Vec<WidgetId>,
    focused_id: Option<WidgetId>,
    overlays: Vec<Overlay>,
}

impl WidgetTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            layout_engine: LayoutEngine::new(),
            taffy_nodes: HashMap::new(),
            next_id: 1,
            root: None,
            render_commands: RenderCommandList::new(),
            focus_chain: Vec::new(),
            focused_id: None,
            overlays: Vec::new(),
        }
    }

    pub fn add<W: Widget + 'static>(&mut self, widget: W) -> WidgetId {
        let id = widget.id();
        let taffy_node = self.layout_engine.new_leaf(widget.style().clone());

        self.nodes.insert(
            id,
            WidgetNode {
                widget: Box::new(widget),
                parent: None,
                children: Vec::new(),
                computed_layout: None,
                has_focused_descendant: false,
            },
        );
        self.taffy_nodes.insert(id, taffy_node);

        if self.root.is_none() {
            self.root = Some(id);
        }

        id
    }

    pub fn add_child<W: Widget + 'static>(&mut self, parent: WidgetId, widget: W) -> WidgetId {
        let child_id = widget.id();
        let child_taffy = self.layout_engine.new_leaf(widget.style().clone());

        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(child_id);
        }

        if let Some(&parent_taffy) = self.taffy_nodes.get(&parent) {
            self.layout_engine.add_child(parent_taffy, child_taffy);
        }

        self.nodes.insert(
            child_id,
            WidgetNode {
                widget: Box::new(widget),
                parent: Some(parent),
                children: Vec::new(),
                computed_layout: None,
                has_focused_descendant: false,
            },
        );
        self.taffy_nodes.insert(child_id, child_taffy);

        child_id
    }

    pub fn remove(&mut self, id: WidgetId) {
        if let Some(node) = self.nodes.remove(&id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.children.retain(|c| *c != id);
                }
            }
            if let Some(taffy_id) = self.taffy_nodes.remove(&id) {
                self.layout_engine.remove(taffy_id);
            }
            for child_id in node.children {
                self.remove_recursive(child_id);
            }
        }
    }

    fn remove_recursive(&mut self, id: WidgetId) {
        if let Some(node) = self.nodes.remove(&id) {
            if let Some(taffy_id) = self.taffy_nodes.remove(&id) {
                self.layout_engine.remove(taffy_id);
            }
            for child_id in node.children {
                self.remove_recursive(child_id);
            }
        }
    }

    pub fn layout(&mut self, width: f32, height: f32) {
        self.layout_with_offset(width, height, 0.0, 0.0);
    }

    pub fn layout_with_offset(&mut self, width: f32, height: f32, offset_x: f32, offset_y: f32) {
        let Some(root_id) = self.root else { return };
        let Some(root_taffy) = self.taffy_nodes.get(&root_id).copied() else {
            return;
        };

        self.layout_engine
            .compute_with_size(root_taffy, width, height);
        self.compute_layout_recursive(root_id, offset_x, offset_y);
    }

    fn compute_layout_recursive(&mut self, id: WidgetId, parent_x: f32, parent_y: f32) {
        let Some(taffy_id) = self.taffy_nodes.get(&id).copied() else {
            return;
        };
        let computed = self.layout_engine.layout(taffy_id);
        let abs_x = parent_x + computed.x;
        let abs_y = parent_y + computed.y;
        let abs_layout = ComputedLayout {
            x: abs_x,
            y: abs_y,
            width: computed.width,
            height: computed.height,
        };

        if let Some(node) = self.nodes.get_mut(&id) {
            node.computed_layout = Some(abs_layout);
            let child_ids: Vec<WidgetId> = node.children.clone();
            for child_id in child_ids {
                self.compute_layout_recursive(child_id, abs_x, abs_y);
            }
        }
    }

    pub fn build_render_commands(&mut self) {
        self.render_commands.clear();
        let Some(root_id) = self.root else { return };
        self.build_commands_recursive(root_id);
    }

    pub fn render_commands(&self) -> &[RenderCommand] {
        self.render_commands.commands()
    }

    fn build_commands_recursive(&mut self, id: WidgetId) {
        let Some(node) = self.nodes.get(&id) else {
            return;
        };
        if !node.widget.visible() {
            return;
        }
        let Some(ref layout) = node.computed_layout else {
            return;
        };

        let opacity = node.widget.opacity();
        let pushed_opacity = opacity < 1.0;
        let overflow = node.widget.overflow();

        if pushed_opacity {
            self.render_commands
                .push(RenderCommand::PushOpacity { opacity });
        }

        self.render_commands.push(RenderCommand::Render { id });

        let pushed_scissor =
            overflow == Overflow::Hidden && layout.width > 0.0 && layout.height > 0.0;
        if pushed_scissor {
            self.render_commands.push(RenderCommand::PushScissor {
                x: layout.x as i32,
                y: layout.y as i32,
                width: layout.width as u32,
                height: layout.height as u32,
            });
        }

        let child_ids: Vec<WidgetId> = node.children.clone();
        for child_id in child_ids {
            self.build_commands_recursive(child_id);
        }

        if pushed_scissor {
            self.render_commands.push(RenderCommand::PopScissor);
        }

        if pushed_opacity {
            self.render_commands.push(RenderCommand::PopOpacity);
        }
    }

    pub fn execute_render_commands(&mut self, ctx: &mut RenderContext<'_>) {
        let commands = std::mem::take(self.render_commands.commands_mut());
        for cmd in &commands {
            match cmd {
                RenderCommand::Render { id } => {
                    let Some(node) = self.nodes.get(id) else {
                        continue;
                    };
                    let Some(ref layout) = node.computed_layout else {
                        continue;
                    };
                    node.widget.render(ctx, layout);
                }
                RenderCommand::PushScissor {
                    x,
                    y,
                    width,
                    height,
                } => {
                    ctx.buffer
                        .push_scissor(ot::buffer::ClipRect::new(*x, *y, *width, *height));
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
        *self.render_commands.commands_mut() = commands;
    }

    pub fn render(&mut self, ctx: &mut RenderContext<'_>) {
        self.build_render_commands();
        self.execute_render_commands(ctx);
        self.render_overlays(ctx);
    }

    pub fn add_overlay(&mut self, overlay: Overlay) {
        self.overlays.push(overlay);
    }

    pub fn remove_overlay(&mut self, widget_id: WidgetId) {
        self.overlays.retain(|o| o.widget_id != widget_id);
    }

    pub fn overlays(&self) -> &[Overlay] {
        &self.overlays
    }

    pub fn has_overlay(&self, widget_id: WidgetId) -> bool {
        self.overlays.iter().any(|o| o.widget_id == widget_id)
    }

    pub fn top_overlay(&self) -> Option<&Overlay> {
        self.overlays.iter().max_by_key(|o| o.z_order)
    }

    fn render_overlays(&mut self, ctx: &mut RenderContext<'_>) {
        if self.overlays.is_empty() {
            return;
        }

        let mut sorted: Vec<(usize, OverlayZOrder)> = self
            .overlays
            .iter()
            .enumerate()
            .map(|(i, o)| (i, o.z_order))
            .collect();
        sorted.sort_by_key(|(_, z)| z.value());

        let backdrop_style = ot::Style::builder()
            .bg(ot::Rgba::new(0.0, 0.0, 0.0, 0.5))
            .build();

        for (idx, _) in sorted {
            let overlay = &self.overlays[idx];

            if overlay.backdrop {
                let buf_w = ctx.buffer.width();
                let buf_h = ctx.buffer.height();
                for row in 0..buf_h {
                    for col in 0..buf_w {
                        ctx.buffer.set(col, row, ot::Cell::new(' ', backdrop_style));
                    }
                }
            }

            let layout = ComputedLayout {
                x: overlay.x,
                y: overlay.y,
                width: overlay.width,
                height: overlay.height,
            };

            let widget_id = overlay.widget_id;
            if let Some(node) = self.nodes.get_mut(&widget_id) {
                node.computed_layout = Some(layout);
            }

            if let Some(node) = self.nodes.get(&widget_id) {
                if let Some(ref layout) = node.computed_layout {
                    ctx.buffer.push_scissor(ot::buffer::ClipRect::new(
                        layout.x as i32,
                        layout.y as i32,
                        layout.width as u32,
                        layout.height as u32,
                    ));
                    node.widget.render(ctx, layout);
                    ctx.buffer.pop_scissor();
                }
            }
        }
    }

    pub fn get(&self, id: WidgetId) -> Option<&dyn Widget> {
        self.nodes.get(&id).map(|n| n.widget.as_ref())
    }

    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut dyn Widget> {
        match self.nodes.get_mut(&id) {
            Some(node) => Some(node.widget.as_mut()),
            None => None,
        }
    }

    pub fn computed_layout(&self, id: WidgetId) -> Option<&ComputedLayout> {
        self.nodes.get(&id).and_then(|n| n.computed_layout.as_ref())
    }

    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }

    pub fn allocate_id(&mut self) -> WidgetId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn layout_engine(&self) -> &LayoutEngine {
        &self.layout_engine
    }

    pub fn layout_engine_mut(&mut self) -> &mut LayoutEngine {
        &mut self.layout_engine
    }

    pub fn set_widget_style(&mut self, id: WidgetId, style: LayoutStyle) {
        if let Some(node) = self.nodes.get_mut(&id) {
            *node.widget.style_mut() = style.clone();
            if let Some(&taffy_id) = self.taffy_nodes.get(&id) {
                self.layout_engine.set_style(taffy_id, style);
            }
        }
    }

    pub fn parent(&self, id: WidgetId) -> Option<WidgetId> {
        self.nodes.get(&id).and_then(|n| n.parent)
    }

    pub fn set_focused(&mut self, id: WidgetId, focused: bool) {
        if focused {
            self.set_focused_widget(Some(id));
        } else if self.focused_id == Some(id) {
            self.set_focused_widget(None);
        }
    }

    fn propagate_focus(&mut self, maybe_parent: Option<WidgetId>) {
        let mut current = maybe_parent;
        while let Some(pid) = current {
            let has_focused = if let Some(node) = self.nodes.get(&pid) {
                node.widget.focused()
                    || node.children.iter().any(|c| {
                        self.nodes
                            .get(c)
                            .is_some_and(|cn| cn.widget.focused() || cn.has_focused_descendant)
                    })
            } else {
                false
            };
            if let Some(node) = self.nodes.get_mut(&pid) {
                node.has_focused_descendant = has_focused;
                current = node.parent;
            } else {
                break;
            }
        }
    }

    pub fn has_focused_descendant(&self, id: WidgetId) -> bool {
        self.nodes
            .get(&id)
            .is_some_and(|n| n.has_focused_descendant || n.widget.focused())
    }

    pub fn build_focus_chain(&mut self) {
        self.focus_chain.clear();
        let Some(root_id) = self.root else { return };
        self.collect_focusable(root_id);
    }

    fn collect_focusable(&mut self, id: WidgetId) {
        let Some(node) = self.nodes.get(&id) else {
            return;
        };
        if node.widget.focusable() {
            self.focus_chain.push(id);
        }
        let child_ids: Vec<WidgetId> = node.children.clone();
        for child_id in child_ids {
            self.collect_focusable(child_id);
        }
    }

    pub fn focus_next(&mut self) -> Option<WidgetId> {
        if self.focus_chain.is_empty() {
            return None;
        }
        let next = match self.focused_id {
            Some(current) => {
                let idx = self
                    .focus_chain
                    .iter()
                    .position(|&id| id == current)
                    .map_or(0, |i| (i + 1) % self.focus_chain.len());
                self.focus_chain[idx]
            }
            None => self.focus_chain[0],
        };
        self.set_focused_widget(Some(next));
        Some(next)
    }

    pub fn focus_prev(&mut self) -> Option<WidgetId> {
        if self.focus_chain.is_empty() {
            return None;
        }
        let prev = if let Some(current) = self.focused_id {
            let idx = self
                .focus_chain
                .iter()
                .position(|&id| id == current)
                .map_or(0, |i| {
                    if i == 0 {
                        self.focus_chain.len() - 1
                    } else {
                        i - 1
                    }
                });
            self.focus_chain[idx]
        } else {
            let last = self.focus_chain.len() - 1;
            self.focus_chain[last]
        };
        self.set_focused_widget(Some(prev));
        Some(prev)
    }

    pub fn set_focused_widget(&mut self, id: Option<WidgetId>) {
        if let Some(old_id) = self.focused_id {
            if let Some(node) = self.nodes.get_mut(&old_id) {
                node.widget.set_focused(false);
            }
            let parent_id = self.nodes.get(&old_id).and_then(|n| n.parent);
            self.propagate_focus(parent_id);
        }

        self.focused_id = id;

        if let Some(new_id) = id {
            if let Some(node) = self.nodes.get_mut(&new_id) {
                node.widget.set_focused(true);
            }
            let parent_id = self.nodes.get(&new_id).and_then(|n| n.parent);
            self.propagate_focus(parent_id);
        }
    }

    pub fn focused_id(&self) -> Option<WidgetId> {
        self.focused_id
    }

    pub fn dispatch_key(&mut self, key: &ot::KeyEvent) -> KeyDispatchResult {
        let tab = key.code == ot::KeyCode::Tab;
        let shift_tab = tab && key.modifiers.contains(ot::KeyModifiers::SHIFT);

        if shift_tab {
            let target = self.focus_prev();
            return KeyDispatchResult {
                target,
                consumed: true,
                action: KeyAction::FocusChanged,
            };
        }
        if tab {
            let target = self.focus_next();
            return KeyDispatchResult {
                target,
                consumed: true,
                action: KeyAction::FocusChanged,
            };
        }

        if let Some(focused_id) = self.focused_id {
            if let Some(node) = self.nodes.get_mut(&focused_id) {
                let consumed = node.widget.handle_key(key);
                return KeyDispatchResult {
                    target: Some(focused_id),
                    consumed,
                    action: if consumed {
                        KeyAction::Consumed
                    } else {
                        KeyAction::Ignored
                    },
                };
            }
        }

        KeyDispatchResult {
            target: None,
            consumed: false,
            action: KeyAction::Ignored,
        }
    }

    pub fn dispatch_mouse(
        &mut self,
        mouse: &ot::MouseEvent,
        hit_grid: Option<&ot::renderer::HitGrid>,
    ) -> MouseDispatchResult {
        let target = hit_grid.and_then(|grid| grid.test(mouse.x, mouse.y));

        if let Some(hit_id) = target {
            let widget_id = self.widget_id_from_hit(hit_id);
            if let Some(wid) = widget_id {
                if self.nodes.get(&wid).is_some_and(|n| n.widget.focusable()) {
                    self.set_focused_widget(Some(wid));
                }
                if let Some(node) = self.nodes.get_mut(&wid) {
                    let consumed = node.widget.handle_mouse(mouse);
                    return MouseDispatchResult {
                        target: Some(wid),
                        consumed,
                    };
                }
            }
        }

        MouseDispatchResult {
            target: None,
            consumed: false,
        }
    }

    fn widget_id_from_hit(&self, hit_id: u32) -> Option<WidgetId> {
        let id = WidgetId::from(hit_id);
        if self.nodes.contains_key(&id) {
            Some(id)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct KeyDispatchResult {
    pub target: Option<WidgetId>,
    pub consumed: bool,
    pub action: KeyAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Consumed,
    FocusChanged,
    Ignored,
}

#[derive(Debug)]
pub struct MouseDispatchResult {
    pub target: Option<WidgetId>,
    pub consumed: bool,
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}
