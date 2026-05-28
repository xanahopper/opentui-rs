use crate::view::element::{Element, ElementKind};
use crate::view::node::Node;
use crate::widget::WidgetTree;
use crate::widgets::{TextLineWidget, ViewWidget};

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
        ElementKind::Custom(_) => Box::new(ViewWidget::from_element(id, elem)),
    }
}
