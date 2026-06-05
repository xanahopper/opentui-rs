use std::fmt;

use crate::view::element::Element;

pub enum Node<M> {
    Element(Box<Element<M>>),
    Overlay(Box<OverlayNode<M>>),
    Fragment(Vec<Self>),
    Empty,
}

pub struct OverlayNode<M> {
    pub content: Box<Node<M>>,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub backdrop: bool,
    pub z_order: u16,
}

impl<M: fmt::Debug> fmt::Debug for Node<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Element(e) => f.debug_tuple("Element").field(e).finish(),
            Self::Overlay(o) => f.debug_tuple("Overlay").field(o).finish(),
            Self::Fragment(children) => f.debug_tuple("Fragment").field(children).finish(),
            Self::Empty => write!(f, "Empty"),
        }
    }
}

impl<M: fmt::Debug> fmt::Debug for OverlayNode<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OverlayNode")
            .field("content", &self.content)
            .field("x", &self.x)
            .field("y", &self.y)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("backdrop", &self.backdrop)
            .field("z_order", &self.z_order)
            .finish()
    }
}

impl<M: Clone> Clone for Node<M> {
    fn clone(&self) -> Self {
        match self {
            Self::Element(e) => Self::Element(e.clone()),
            Self::Overlay(o) => Self::Overlay(Box::new(OverlayNode {
                content: o.content.clone(),
                x: o.x,
                y: o.y,
                width: o.width,
                height: o.height,
                backdrop: o.backdrop,
                z_order: o.z_order,
            })),
            Self::Fragment(children) => Self::Fragment(children.clone()),
            Self::Empty => Self::Empty,
        }
    }
}

impl<M: Clone> Node<M> {
    pub fn map_msg<N: Clone>(self, f: impl Fn(M) -> N + Clone + 'static) -> Node<N> {
        match self {
            Self::Element(elem) => {
                let mapped_children: Vec<Node<N>> = elem
                    .children
                    .into_iter()
                    .map(|c| c.map_msg(f.clone()))
                    .collect();
                let mapped_events: Vec<crate::view::event::EventBinding<N>> = elem
                    .events
                    .into_iter()
                    .map(|e| crate::view::event::EventBinding {
                        kind: e.kind,
                        message: (f.clone())(e.message),
                    })
                    .collect();
                Node::Element(Box::new(Element {
                    kind: elem.kind,
                    key: elem.key,
                    layout: elem.layout,
                    props: elem.props,
                    children: mapped_children,
                    events: mapped_events,
                }))
            }
            Self::Overlay(overlay) => {
                let mapped_content = overlay.content.map_msg(f);
                Node::Overlay(Box::new(OverlayNode {
                    content: Box::new(mapped_content),
                    x: overlay.x,
                    y: overlay.y,
                    width: overlay.width,
                    height: overlay.height,
                    backdrop: overlay.backdrop,
                    z_order: overlay.z_order,
                }))
            }
            Self::Fragment(children) => {
                Node::Fragment(children.into_iter().map(|c| c.map_msg(f.clone())).collect())
            }
            Self::Empty => Node::Empty,
        }
    }
}
