use crate::view::element::Element;

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Fragment(Vec<Node>),
    Empty,
}
