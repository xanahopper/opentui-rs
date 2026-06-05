use std::collections::HashMap;

use opentui_rust::renderer::HitGrid;
use opentui_rust::terminal::{MouseButton, MouseEventKind};

use crate::view::event::{EventBinding, EventKind};
use crate::view::node::Node;
use crate::view::rebuild::build_tree_with_events;
use crate::widget::{KeyDispatchResult, MouseDispatchResult, RenderContext, WidgetId, WidgetTree};

pub struct ViewRuntime<M> {
    tree: WidgetTree,
    events: HashMap<WidgetId, Vec<EventBinding<M>>>,
    hit_grid: HitGrid,
    hovered_id: Option<WidgetId>,
    captured_id: Option<WidgetId>,
    pointer_pos: Option<(u32, u32)>,
    pointer_modifiers: (bool, bool, bool),
}

pub struct ViewMouseDispatchResult<M> {
    pub inner: MouseDispatchResult,
    pub action: Option<M>,
}

#[derive(Debug)]
pub struct DispatchResult<M> {
    pub messages: Vec<M>,
    pub consumed: bool,
}

impl<M> DispatchResult<M> {
    fn empty() -> Self {
        Self {
            messages: Vec::new(),
            consumed: false,
        }
    }
}

impl<M: Clone> ViewRuntime<M> {
    pub fn new() -> Self {
        Self {
            tree: WidgetTree::new(),
            events: HashMap::new(),
            hit_grid: HitGrid::default(),
            hovered_id: None,
            captured_id: None,
            pointer_pos: None,
            pointer_modifiers: (false, false, false),
        }
    }

    pub fn rebuild(&mut self, node: &Node<M>) {
        let (tree, events) = build_tree_with_events(node);
        self.tree = tree;
        self.events = events;
        self.tree.build_focus_chain();
    }

    pub fn layout(&mut self, width: f32, height: f32) {
        self.tree.layout(width, height);
    }

    pub fn render(&mut self, ctx: &mut RenderContext<'_>) {
        ctx.hovered_id = self.hovered_id;
        self.tree.render(ctx);
    }

    pub fn render_to_buffer(
        &mut self,
        ctx: &mut RenderContext<'_>,
        node: &Node<M>,
        width: f32,
        height: f32,
    ) {
        self.rebuild(node);
        self.layout(width, height);
        self.register_hit_areas(width as u32, height as u32);
        self.render(ctx);
    }

    pub fn register_hit_areas(&mut self, width: u32, height: u32) {
        self.hit_grid.resize(width, height);
        let events = &self.events;
        for (id, layout, focusable) in self.tree.hit_registration_order() {
            let has_event = events.contains_key(&id);
            let interactive = self.is_interactive(id);
            if has_event || focusable || interactive {
                let x = layout.x as u32;
                let y = layout.y as u32;
                let w = layout.width as u32;
                let h = layout.height as u32;
                let hit_id = id as u32;
                self.hit_grid.register(x, y, w, h, hit_id);
            }
        }
    }

    fn is_interactive(&self, id: WidgetId) -> bool {
        self.tree.get(id).is_some_and(|w| {
            w.as_any()
                .downcast_ref::<crate::widgets::ViewWidget>()
                .is_some_and(|vw| vw.interactive())
        })
    }

    // ── Key dispatch (unchanged) ──────────────────────────────────────

    pub fn dispatch_key(&mut self, key: &opentui_rust::KeyEvent) -> KeyDispatchResult {
        self.tree.dispatch_key(key)
    }

    // ── Legacy mouse dispatch (backwards compat) ──────────────────────

    pub fn dispatch_mouse(
        &mut self,
        mouse: &opentui_rust::MouseEvent,
    ) -> ViewMouseDispatchResult<M> {
        let inner = self.tree.dispatch_mouse(mouse, Some(&self.hit_grid));
        let action = inner.target.and_then(|id| {
            self.events
                .get(&id)
                .and_then(|bindings| bindings.first().map(|b| b.message.clone()))
        });
        ViewMouseDispatchResult { inner, action }
    }

    pub fn dispatch_mouse_with_grid(
        &mut self,
        mouse: &opentui_rust::MouseEvent,
        hit_grid: Option<&HitGrid>,
    ) -> ViewMouseDispatchResult<M> {
        let inner = self.tree.dispatch_mouse(mouse, hit_grid);
        let action = inner.target.and_then(|id| {
            self.events
                .get(&id)
                .and_then(|bindings| bindings.first().map(|b| b.message.clone()))
        });
        ViewMouseDispatchResult { inner, action }
    }

    // ── New unified event dispatch ────────────────────────────────────

    /// Process a raw mouse event through the full dispatch pipeline.
    ///
    /// Handles: hover detection, click (with auto-focus + bubbling),
    /// scroll (with focused fallback + bubbling), drag capture, and drop.
    /// Returns typed messages for the app to process.
    pub fn process_mouse_event(&mut self, mouse: &opentui_rust::MouseEvent) -> DispatchResult<M> {
        self.pointer_pos = Some((mouse.x, mouse.y));
        self.pointer_modifiers = (mouse.shift, mouse.ctrl, mouse.alt);

        if self.captured_id.is_some() {
            return self.process_captured(mouse);
        }

        let hit_id = self.hit_grid.test(mouse.x, mouse.y);
        let hit_wid = hit_id.map(|id| id as WidgetId);

        match mouse.kind {
            MouseEventKind::Press => self.process_down(mouse, hit_wid),
            MouseEventKind::Release => self.process_up(mouse, hit_wid),
            MouseEventKind::Move => self.process_move(hit_wid),
            MouseEventKind::Drag => self.process_drag(mouse, hit_wid),
            MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => self.process_scroll(mouse, hit_wid),
            _ => DispatchResult::empty(),
        }
    }

    /// Re-evaluate hover after a render (layout may have moved under cursor).
    pub fn recheck_hover(&mut self) -> DispatchResult<M> {
        let Some((x, y)) = self.pointer_pos else {
            return DispatchResult::empty();
        };
        if self.captured_id.is_some() {
            return DispatchResult::empty();
        }
        let hit = self.hit_grid.test(x, y).map(|id| id as WidgetId);
        self.update_hover(hit)
    }

    // ── Internal: Press/Down ──────────────────────────────────────────

    fn process_down(
        &mut self,
        mouse: &opentui_rust::MouseEvent,
        hit: Option<WidgetId>,
    ) -> DispatchResult<M> {
        let mut msgs = Vec::new();
        if let Some(wid) = hit {
            if mouse.button == MouseButton::Left && !mouse.default_prevented {
                self.auto_focus(wid);
            }
            let kind = match mouse.button {
                MouseButton::Left => EventKind::Click,
                MouseButton::Right => EventKind::RightClick,
                MouseButton::Middle => EventKind::MiddleClick,
                MouseButton::None => return DispatchResult::empty(),
            };
            self.collect_bubbling(wid, kind, &mut msgs);
        }
        let consumed = !msgs.is_empty();
        DispatchResult {
            messages: msgs,
            consumed,
        }
    }

    // ── Internal: Release/Up ──────────────────────────────────────────

    fn process_up(
        &mut self,
        _mouse: &opentui_rust::MouseEvent,
        _hit: Option<WidgetId>,
    ) -> DispatchResult<M> {
        DispatchResult::empty()
    }

    // ── Internal: Move (hover detection) ──────────────────────────────

    fn process_move(&mut self, hit: Option<WidgetId>) -> DispatchResult<M> {
        self.update_hover(hit)
    }

    // ── Internal: Drag (capture initiation) ───────────────────────────

    fn process_drag(
        &mut self,
        mouse: &opentui_rust::MouseEvent,
        hit: Option<WidgetId>,
    ) -> DispatchResult<M> {
        let mut msgs = Vec::new();
        if mouse.button == MouseButton::Left && self.captured_id.is_none() {
            self.captured_id = hit;
        }
        let hover_target = if self.captured_id.is_some() {
            hit.filter(|id| Some(*id) != self.captured_id)
        } else {
            hit
        };
        msgs.extend(self.update_hover(hover_target).messages);
        DispatchResult {
            messages: msgs,
            consumed: self.captured_id.is_some(),
        }
    }

    // ── Internal: Captured event routing ──────────────────────────────

    fn process_captured(&mut self, mouse: &opentui_rust::MouseEvent) -> DispatchResult<M> {
        let mut msgs = Vec::new();
        let captured = self.captured_id.unwrap();

        match mouse.kind {
            MouseEventKind::Release => {
                let hit = self
                    .hit_grid
                    .test(mouse.x, mouse.y)
                    .map(|id| id as WidgetId);

                self.collect_bubbling(captured, EventKind::Click, &mut msgs);

                if let Some(target) = hit {
                    if target != captured {
                        self.collect_bubbling(target, EventKind::Click, &mut msgs);
                    }
                }

                self.captured_id = None;
                msgs.extend(self.update_hover(hit).messages);
            }
            MouseEventKind::Move | MouseEventKind::Drag => {
                let hit = self
                    .hit_grid
                    .test(mouse.x, mouse.y)
                    .map(|id| id as WidgetId);
                let hover_target = hit.filter(|id| *id != captured);
                msgs.extend(self.update_hover(hover_target).messages);
            }
            _ => {}
        }

        DispatchResult {
            messages: msgs,
            consumed: true,
        }
    }

    // ── Internal: Scroll dispatch ─────────────────────────────────────

    fn process_scroll(
        &mut self,
        mouse: &opentui_rust::MouseEvent,
        hit: Option<WidgetId>,
    ) -> DispatchResult<M> {
        let mut msgs = Vec::new();

        let target = hit.or(self.tree.focused_id());

        if let Some(wid) = target {
            self.tree.dispatch_scroll_to_widget(wid, mouse);
            self.collect_bubbling(wid, EventKind::Scroll, &mut msgs);
        }

        let consumed = !msgs.is_empty();
        DispatchResult {
            messages: msgs,
            consumed,
        }
    }

    // ── Hover state management ────────────────────────────────────────

    fn update_hover(&mut self, new_hovered: Option<WidgetId>) -> DispatchResult<M> {
        let mut msgs = Vec::new();
        if new_hovered != self.hovered_id {
            if let Some(old) = self.hovered_id {
                self.collect_events(old, EventKind::Hover, &mut msgs);
            }
            if let Some(new) = new_hovered {
                self.collect_events(new, EventKind::Hover, &mut msgs);
            }
            self.hovered_id = new_hovered;
        }
        let consumed = !msgs.is_empty();
        DispatchResult {
            messages: msgs,
            consumed,
        }
    }

    // ── Event collection ──────────────────────────────────────────────

    fn collect_events(&self, wid: WidgetId, kind: EventKind, msgs: &mut Vec<M>) {
        if let Some(bindings) = self.events.get(&wid) {
            for b in bindings {
                if b.kind == kind {
                    msgs.push(b.message.clone());
                }
            }
        }
    }

    fn collect_bubbling(&self, wid: WidgetId, kind: EventKind, msgs: &mut Vec<M>) {
        let mut current = Some(wid);
        while let Some(id) = current {
            self.collect_events(id, kind, msgs);
            current = self.tree.parent(id);
        }
    }

    // ── Auto-focus ───────────────────────────────────────────────────

    fn auto_focus(&mut self, wid: WidgetId) {
        let mut current = Some(wid);
        while let Some(id) = current {
            if self.tree.get(id).is_some_and(|w| w.focusable()) {
                self.tree.set_focused_widget(Some(id));
                return;
            }
            current = self.tree.parent(id);
        }
    }

    // ── Accessors ────────────────────────────────────────────────────

    pub fn events_for_widget(&self, id: WidgetId) -> &[EventBinding<M>] {
        self.events.get(&id).map_or(&[], |v| v)
    }

    pub fn hit_grid(&self) -> &HitGrid {
        &self.hit_grid
    }

    pub fn tree(&self) -> &WidgetTree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut WidgetTree {
        &mut self.tree
    }

    pub fn hovered_id(&self) -> Option<WidgetId> {
        self.hovered_id
    }

    pub fn captured_id(&self) -> Option<WidgetId> {
        self.captured_id
    }
}

impl<M: Clone> Default for ViewRuntime<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewRuntime<String> {
    pub fn action_for_widget(&self, id: WidgetId) -> Option<&str> {
        self.events.get(&id).and_then(|bindings| {
            bindings
                .iter()
                .find(|b| b.kind == crate::view::event::EventKind::Click)
                .map(|b| b.message.as_str())
        })
    }
}
