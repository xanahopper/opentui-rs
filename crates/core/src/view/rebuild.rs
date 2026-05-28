use crate::view::element::{Element, ElementKind};
use crate::view::node::Node;
use crate::view::props::Props;
use crate::widget::{Overlay, OverlayZOrder, WidgetTree};
use crate::widgets::{InputWidget, ListWidget, StyledTextWidget, TextLineWidget, ViewWidget};

pub fn build_tree(node: &Node) -> WidgetTree {
    let mut ctx = BuildContext { next_id: 1 };
    let mut tree = WidgetTree::new();
    build_recursive(node, &mut tree, None, &mut ctx);
    tree.build_focus_chain();
    tree
}

struct BuildContext {
    next_id: u64,
}

impl BuildContext {
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

fn build_recursive(
    node: &Node,
    tree: &mut WidgetTree,
    parent: Option<u64>,
    ctx: &mut BuildContext,
) {
    match node {
        Node::Element(elem) => {
            let id = ctx.alloc_id();
            let widget = create_widget(id, elem);
            if let Some(p) = parent {
                tree.add_child(p, widget);
            } else {
                tree.add(widget);
            }
            for child in &elem.children {
                build_recursive(child, tree, Some(id), ctx);
            }
        }
        Node::Overlay(overlay) => {
            let id = ctx.alloc_id();
            if let Node::Element(ref elem) = *overlay.content {
                let widget = create_widget(id, elem);
                tree.add(widget);
                tree.add_overlay(
                    Overlay::new(
                        id,
                        overlay.x as f32,
                        overlay.y as f32,
                        overlay.width as f32,
                        overlay.height as f32,
                    )
                    .z_order(OverlayZOrder::new(overlay.z_order))
                    .backdrop(overlay.backdrop),
                );
                for child in &elem.children {
                    build_recursive(child, tree, Some(id), ctx);
                }
            }
        }
        Node::Fragment(children) => {
            for child in children {
                build_recursive(child, tree, parent, ctx);
            }
        }
        Node::Empty => {}
    }
}

fn create_widget(id: u64, elem: &Element) -> Box<dyn crate::widget::Widget> {
    match elem.kind {
        ElementKind::View => Box::new(ViewWidget::from_element(id, elem)),
        ElementKind::Text => Box::new(TextLineWidget::from_element(id, elem)),
        ElementKind::StyledText => {
            let mut widget = StyledTextWidget::new(id, elem.layout.clone());
            if let Props::StyledText(ref props) = elem.props {
                widget.set_segments(props.segments.clone());
            }
            Box::new(widget)
        }
        ElementKind::Input => {
            let mut widget = InputWidget::new(id, elem.layout.clone());
            if let Props::Input(ref props) = elem.props {
                if let Some(ref ph) = props.placeholder {
                    widget = widget.placeholder(ph);
                }
                if props.password {
                    widget = widget.password_mode();
                }
                if let Some(ref val) = props.initial_value {
                    widget.set_value(val);
                }
            }
            Box::new(widget)
        }
        ElementKind::List => {
            let mut widget = ListWidget::new(id, elem.layout.clone());
            if let Props::List(ref props) = elem.props {
                widget = widget.scrollbar(props.scrollbar);
            }
            Box::new(widget)
        }
        ElementKind::Custom(_) => Box::new(ViewWidget::from_element(id, elem)),
    }
}
