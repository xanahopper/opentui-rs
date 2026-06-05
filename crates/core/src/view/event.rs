use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Click,
    RightClick,
    MiddleClick,
    Hover,
    Scroll,
}

#[derive(Clone)]
pub struct EventBinding<M> {
    pub kind: EventKind,
    pub message: M,
}

impl<M: fmt::Debug> fmt::Debug for EventBinding<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventBinding")
            .field("kind", &self.kind)
            .field("message", &self.message)
            .finish()
    }
}
