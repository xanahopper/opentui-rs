use crate::layout::LayoutStyle;
use crate::view::key::Key;
use crate::view::node::Node;
use crate::view::props::Props;

#[derive(Debug, Clone)]
pub struct Element {
    pub kind: ElementKind,
    pub key: Option<Key>,
    pub layout: LayoutStyle,
    pub props: Props,
    pub children: Vec<Node>,
    pub action: Option<String>,
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
