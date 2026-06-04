//! Key binding registry for mapping key combinations to named actions.
//!
//! Applications register `(modifiers + code) → action_name` bindings, then
//! resolve incoming `KeyEvent`s to action strings. This decouples key
//! handling from widget logic.

use std::collections::HashMap;

use opentui_rust::input::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct KeyCombo {
    modifiers: KeyModifiers,
    code: KeyCode,
}

impl KeyCombo {
    fn from_event(event: &KeyEvent) -> Self {
        Self {
            modifiers: event.modifiers,
            code: event.code,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId(u32);

impl ActionId {
    pub fn raw(self) -> u32 {
        self.0
    }
}

pub struct KeyBindingRegistry {
    bindings: HashMap<KeyCombo, ActionId>,
    actions: HashMap<ActionId, &'static str>,
    next_action: u32,
}

impl KeyBindingRegistry {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            actions: HashMap::new(),
            next_action: 0,
        }
    }

    pub fn bind(
        &mut self,
        modifiers: KeyModifiers,
        code: KeyCode,
        action: &'static str,
    ) -> ActionId {
        let id = ActionId(self.next_action);
        self.next_action += 1;
        self.bindings.insert(KeyCombo { modifiers, code }, id);
        self.actions.insert(id, action);
        id
    }

    pub fn unbind(&mut self, modifiers: KeyModifiers, code: KeyCode) {
        if let Some(id) = self.bindings.remove(&KeyCombo { modifiers, code }) {
            self.actions.remove(&id);
        }
    }

    pub fn resolve(&self, event: &KeyEvent) -> Option<&'static str> {
        let combo = KeyCombo::from_event(event);
        self.bindings
            .get(&combo)
            .and_then(|id| self.actions.get(id).copied())
    }

    pub fn resolve_id(&self, event: &KeyEvent) -> Option<ActionId> {
        self.bindings.get(&KeyCombo::from_event(event)).copied()
    }

    pub fn action_name(&self, id: ActionId) -> Option<&'static str> {
        self.actions.get(&id).copied()
    }

    pub fn has_binding(&self, event: &KeyEvent) -> bool {
        self.bindings.contains_key(&KeyCombo::from_event(event))
    }
}

impl Default for KeyBindingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_resolve() {
        let mut reg = KeyBindingRegistry::new();
        reg.bind(KeyModifiers::CTRL, KeyCode::Char('s'), "save");
        reg.bind(KeyModifiers::CTRL, KeyCode::Char('q'), "quit");

        assert_eq!(
            reg.resolve(&KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CTRL)),
            Some("save"),
        );
        assert_eq!(
            reg.resolve(&KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CTRL)),
            Some("quit"),
        );
    }

    #[test]
    fn test_unbind() {
        let mut reg = KeyBindingRegistry::new();
        reg.bind(KeyModifiers::empty(), KeyCode::Char('q'), "quit");
        assert!(reg.has_binding(&KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())));

        reg.unbind(KeyModifiers::empty(), KeyCode::Char('q'));
        assert!(!reg.has_binding(&KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty())));
    }

    #[test]
    fn test_no_binding() {
        let reg = KeyBindingRegistry::new();
        assert_eq!(
            reg.resolve(&KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty())),
            None,
        );
    }

    #[test]
    fn test_resolve_id() {
        let mut reg = KeyBindingRegistry::new();
        let id = reg.bind(KeyModifiers::empty(), KeyCode::Enter, "confirm");
        assert_eq!(
            reg.resolve_id(&KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())),
            Some(id)
        );
        assert_eq!(reg.action_name(id), Some("confirm"));
    }
}
