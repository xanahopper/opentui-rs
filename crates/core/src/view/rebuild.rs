use std::collections::HashMap;

use opentui_rust::Rgba;

use crate::view::element::{Element, ElementKind};
use crate::view::event::EventBinding;
use crate::view::node::Node;
use crate::view::props::Props;
use crate::widget::{Overlay, OverlayZOrder, WidgetId, WidgetTree};
use crate::widgets::{
    FillWidget, InputWidget, ListWidget, SeparatorWidget, TextWidget, ViewWidget,
};

pub fn build_tree<M: Clone>(node: &Node<M>) -> WidgetTree {
    build_tree_with_events(node).0
}

pub fn build_tree_with_events<M: Clone>(
    node: &Node<M>,
) -> (WidgetTree, HashMap<WidgetId, Vec<EventBinding<M>>>) {
    let mut ctx = BuildContext { next_id: 1 };
    let mut tree = WidgetTree::new();
    let mut events = HashMap::new();
    build_recursive(node, &mut tree, None, &mut ctx, &mut events);
    tree.build_focus_chain();
    (tree, events)
}

pub fn build_tree_with_actions(node: &Node<String>) -> (WidgetTree, HashMap<WidgetId, String>) {
    let (tree, events) = build_tree_with_events(node);
    let actions = events
        .into_iter()
        .filter_map(|(id, bindings)| {
            bindings
                .into_iter()
                .find(|b| b.kind == crate::view::event::EventKind::Click)
                .map(|b| (id, b.message))
        })
        .collect();
    (tree, actions)
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

fn build_recursive<M: Clone>(
    node: &Node<M>,
    tree: &mut WidgetTree,
    parent: Option<u64>,
    ctx: &mut BuildContext,
    events: &mut HashMap<WidgetId, Vec<EventBinding<M>>>,
) {
    match node {
        Node::Element(elem) => {
            let id = ctx.alloc_id();
            if !elem.events.is_empty() {
                events.insert(id, elem.events.clone());
            }
            let widget = create_widget(id, elem);
            if let Some(p) = parent {
                tree.add_child(p, widget);
            } else {
                tree.add(widget);
            }
            for child in &elem.children {
                build_recursive(child, tree, Some(id), ctx, events);
            }
        }
        Node::Overlay(overlay) => {
            let id = ctx.alloc_id();
            if let Node::Element(ref elem) = *overlay.content {
                if !elem.events.is_empty() {
                    events.insert(id, elem.events.clone());
                }
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
                    build_recursive(child, tree, Some(id), ctx, events);
                }
            }
        }
        Node::Fragment(children) => {
            for child in children {
                build_recursive(child, tree, parent, ctx, events);
            }
        }
        Node::Empty => {}
    }
}

fn create_widget<M>(id: u64, elem: &Element<M>) -> Box<dyn crate::widget::Widget> {
    match elem.kind {
        ElementKind::Text => Box::new(TextWidget::from_element(id, elem)),
        ElementKind::Input => {
            let mut widget = InputWidget::new(id, elem.layout.clone());
            if let Props::Input(ref props) = elem.props {
                if let Some(ref ph) = props.placeholder {
                    widget = widget.placeholder(ph);
                }
                if props.password {
                    widget = widget.password_mode();
                }
                if let Some(ref val) = props.default_value {
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
        ElementKind::Fill => {
            let color = if let Props::Fill(ref props) = elem.props {
                props.color
            } else {
                Rgba::TRANSPARENT
            };
            Box::new(FillWidget::new(id, elem.layout.clone(), color))
        }
        ElementKind::Separator => {
            let mut widget = SeparatorWidget::new(id, elem.layout.clone());
            if let Props::Separator(ref props) = elem.props {
                widget = widget.char_(props.char).fg(props.fg);
            }
            Box::new(widget)
        }
        ElementKind::Custom(_) | ElementKind::View => Box::new(ViewWidget::from_element(id, elem)),
    }
}
