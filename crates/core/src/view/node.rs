use crate::view::element::Element;

#[derive(Debug, Clone)]
pub enum Node {
    Element(Element),
    Overlay(OverlayNode),
    Fragment(Vec<Node>),
    Empty,
}

#[derive(Debug, Clone)]
pub struct OverlayNode {
    pub content: Box<Node>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub backdrop: bool,
    pub z_order: u16,
}
