use std::fmt;

use crate::layout::LayoutStyle;
use crate::view::event::EventBinding;
use crate::view::key::Key;
use crate::view::node::Node;
use crate::view::props::Props;

pub struct Element<M> {
    pub kind: ElementKind,
    pub key: Option<Key>,
    pub layout: LayoutStyle,
    pub props: Props,
    pub children: Vec<Node<M>>,
    pub events: Vec<EventBinding<M>>,
}

impl<M: fmt::Debug> fmt::Debug for Element<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element")
            .field("kind", &self.kind)
            .field("key", &self.key)
            .field("layout", &self.layout)
            .field("props", &self.props)
            .field("children", &self.children)
            .field("events", &self.events)
            .finish()
    }
}

impl<M: Clone> Clone for Element<M> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            key: self.key.clone(),
            layout: self.layout.clone(),
            props: self.props.clone(),
            children: self.children.clone(),
            events: self.events.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    View,
    Text,
    Input,
    List,
    Fill,
    Separator,
    Custom(&'static str),
}
