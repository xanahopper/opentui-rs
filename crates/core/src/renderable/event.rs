//! Higher-level event dispatch, focus management, and hit testing.
//!
//! This module provides an abstraction over raw `crate::Event` that
//! handles focus tracking, keyboard dispatch to the focused widget, and
//! mouse hit testing via the `HitGrid`.

use crate as ot;
use crate::renderer::HitGrid;

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
