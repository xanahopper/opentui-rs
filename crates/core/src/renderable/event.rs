//! Higher-level event dispatch, focus management, and hit testing.
//!
//! This module provides an abstraction over raw `crate::Event` that
//! handles focus tracking, keyboard dispatch to the focused widget, and
//! mouse hit testing via the `HitGrid`.

use crate as ot;
use crate::renderable::node::NodeId;
use crate::renderer::HitGrid;
use crate::terminal::MouseEventKind;

/// A mouse event while it is being dispatched through the render tree.
///
/// The raw terminal event is available through `Deref`. Dispatch metadata and
/// controls mirror OpenTUI's renderable-level `MouseEvent`.
#[derive(Debug, Clone)]
pub struct RenderableMouseEvent {
    raw: ot::MouseEvent,
    target: NodeId,
    current_target: NodeId,
    source: Option<NodeId>,
    is_dragging: bool,
    propagation_stopped: bool,
    default_prevented: bool,
}

impl RenderableMouseEvent {
    pub(crate) fn new(
        raw: ot::MouseEvent,
        target: NodeId,
        source: Option<NodeId>,
        is_dragging: bool,
    ) -> Self {
        Self {
            raw,
            target,
            current_target: target,
            source,
            is_dragging,
            propagation_stopped: false,
            default_prevented: false,
        }
    }

    #[must_use]
    pub fn target(&self) -> NodeId {
        self.target
    }

    #[must_use]
    pub fn current_target(&self) -> NodeId {
        self.current_target
    }

    #[must_use]
    pub fn source(&self) -> Option<NodeId> {
        self.source
    }

    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    pub fn stop_propagation(&mut self) {
        self.propagation_stopped = true;
    }

    #[must_use]
    pub fn propagation_stopped(&self) -> bool {
        self.propagation_stopped
    }

    pub fn prevent_default(&mut self) {
        self.default_prevented = true;
    }

    #[must_use]
    pub fn default_prevented(&self) -> bool {
        self.default_prevented
    }

    pub(crate) fn set_current_target(&mut self, target: NodeId) {
        self.current_target = target;
    }
}

impl std::ops::Deref for RenderableMouseEvent {
    type Target = ot::MouseEvent;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

/// Result of bubbling one renderable-level mouse event.
#[derive(Debug, Default)]
pub struct RenderableMouseDispatch {
    pub consumed: bool,
    pub default_prevented: bool,
    pub propagation_stopped: bool,
    pub delivered: Vec<MouseDelivery>,
}

impl RenderableMouseDispatch {
    pub(crate) fn merge(&mut self, other: Self) {
        self.consumed |= other.consumed;
        self.default_prevented |= other.default_prevented;
        self.propagation_stopped |= other.propagation_stopped;
        self.delivered.extend(other.delivered);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseDelivery {
    pub node: NodeId,
    pub kind: MouseEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FocusId(u64);

impl FocusId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub enum FocusEvent {
    Gained(FocusId),
    Lost(FocusId),
}

#[derive(Debug)]
pub enum DispatchResult {
    Consumed,
    Ignored,
}

pub struct FocusManager {
    focused: Option<FocusId>,
    focusable: Vec<FocusId>,
    focus_index: Option<usize>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focused: None,
            focusable: Vec::new(),
            focus_index: None,
        }
    }

    pub fn register(&mut self, id: FocusId) {
        if !self.focusable.contains(&id) {
            self.focusable.push(id);
        }
    }

    pub fn unregister(&mut self, id: FocusId) {
        self.focusable.retain(|f| *f != id);
        if self.focused == Some(id) {
            self.focused = None;
            self.focus_index = None;
        }
    }

    pub fn focus(&mut self, id: FocusId) -> Option<FocusEvent> {
        let prev = self.focused.replace(id);
        self.focus_index = self.focusable.iter().position(|f| *f == id);

        match prev {
            Some(old) if old != id => Some(FocusEvent::Lost(old)),
            _ => None,
        }
    }

    pub fn blur(&mut self) -> Option<FocusEvent> {
        self.focused.take().map(FocusEvent::Lost)
    }

    pub fn focused(&self) -> Option<FocusId> {
        self.focused
    }

    pub fn focus_next(&mut self) -> Option<FocusId> {
        if self.focusable.is_empty() {
            return None;
        }
        let next = match self.focus_index {
            Some(i) => (i + 1) % self.focusable.len(),
            None => 0,
        };
        let id = self.focusable[next];
        self.focus(id);
        Some(id)
    }

    pub fn focus_prev(&mut self) -> Option<FocusId> {
        if self.focusable.is_empty() {
            return None;
        }
        let prev = match self.focus_index {
            Some(0) => self.focusable.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        let id = self.focusable[prev];
        self.focus(id);
        Some(id)
    }

    pub fn hit_test(&self, hit_grid: &HitGrid, x: u32, y: u32) -> Option<u32> {
        hit_grid.test(x, y)
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventDispatcher {
    pub focus: FocusManager,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            focus: FocusManager::new(),
        }
    }

    pub fn dispatch_mouse(
        &mut self,
        event: &ot::MouseEvent,
        hit_grid: &HitGrid,
    ) -> MouseDispatchResult {
        MouseDispatchResult {
            hit_id: self.focus.hit_test(hit_grid, event.x, event.y),
            consumed: true,
        }
    }

    pub fn dispatch_key(&mut self, _event: &ot::KeyEvent) -> KeyDispatchResult {
        KeyDispatchResult {
            target: self.focus.focused(),
            consumed: self.focus.focused.is_some(),
        }
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct MouseDispatchResult {
    pub hit_id: Option<u32>,
    pub consumed: bool,
}

#[derive(Debug)]
pub struct KeyDispatchResult {
    pub target: Option<FocusId>,
    pub consumed: bool,
}
