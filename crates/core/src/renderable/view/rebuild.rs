use std::collections::HashMap;

use crate::Rgba;
use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::node::NodeId;
use crate::renderable::tree::RenderTree;
use crate::view::element::{Element, ElementKind};
use crate::view::node::Node;
use crate::view::props::{Props, SpinnerPreset};

pub fn build_tree(node: &Node) -> RenderTree {
    build_tree_with_actions(node).0
}

pub fn build_tree_with_actions(node: &Node) -> (RenderTree, HashMap<NodeId, String>) {
    let mut tree = RenderTree::new();
    let mut actions = HashMap::new();
    build_recursive(node, &mut tree, None, &mut actions);
    (tree, actions)
}

fn build_recursive(
    node: &Node,
    tree: &mut RenderTree,
    parent: Option<NodeId>,
    actions: &mut HashMap<NodeId, String>,
) {
    match node {
        Node::Element(elem) => {
            let id = add_element(tree, parent, elem);
            if let Some(ref action) = elem.action {
                actions.insert(id, action.clone());
            }
            for child in &elem.children {
                build_recursive(child, tree, Some(id), actions);
            }
        }
        Node::Overlay(overlay) => {
            if let Node::Element(ref elem) = *overlay.content {
                let id = add_element(tree, None, elem);
                if let Some(ref action) = elem.action {
                    actions.insert(id, action.clone());
                }
                for child in &elem.children {
                    build_recursive(child, tree, Some(id), actions);
                }
                tree.add_overlay(crate::renderable::tree::Overlay {
                    node: id,
                    x: overlay.x as f32,
                    y: overlay.y as f32,
                    width: overlay.width as f32,
                    height: overlay.height as f32,
                    z_order: overlay.z_order,
                    backdrop: overlay.backdrop,
                });
            }
        }
        Node::Fragment(children) => {
            for child in children {
                build_recursive(child, tree, parent, actions);
            }
        }
        Node::Empty => {}
    }
}

fn add_element(tree: &mut RenderTree, parent: Option<NodeId>, elem: &Element) -> NodeId {
    let (behavior, defaults): (Box<dyn Behavior>, FrameworkDefaults) = create_behavior(elem);

    let id = match parent {
        Some(p) => tree.add_child(p, behavior),
        None => tree.set_root(behavior),
    };

    tree.set_focusable(id, defaults.focusable);
    tree.set_overflow(id, defaults.overflow);
    tree.set_visible(id, defaults.visible);
    tree.set_opacity(id, defaults.opacity);
    tree.set_style(id, elem.layout.clone());

    id
}

#[allow(clippy::too_many_lines)]
fn create_behavior(elem: &Element) -> (Box<dyn Behavior>, FrameworkDefaults) {
    match elem.kind {
        ElementKind::Text => {
            let w = crate::renderable::widgets::TextLineWidget::from_element(elem);
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::StyledText => {
            let mut w = crate::renderable::widgets::StyledTextWidget::new(elem.layout.clone());
            if let Props::StyledText(ref props) = elem.props {
                w.set_segments(props.segments.clone());
            }
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Input => {
            let mut w = crate::renderable::widgets::InputWidget::new(elem.layout.clone());
            if let Props::Input(ref props) = elem.props {
                if let Some(ref ph) = props.placeholder {
                    w = w.placeholder(ph);
                }
                if props.password {
                    w = w.password_mode();
                }
                if let Some(ref val) = props.default_value {
                    w.set_value(val);
                }
            }
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::List => {
            let mut w = crate::renderable::widgets::ListWidget::new(elem.layout.clone());
            if let Props::List(ref props) = elem.props {
                w = w.scrollbar(props.scrollbar);
            }
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Fill => {
            let color = if let Props::Fill(ref props) = elem.props {
                props.color
            } else {
                Rgba::TRANSPARENT
            };
            let w = crate::renderable::widgets::FillWidget::new(elem.layout.clone(), color);
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Separator => {
            let mut w = crate::renderable::widgets::SeparatorWidget::new(elem.layout.clone());
            if let Props::Separator(ref props) = elem.props {
                w = w.char_(props.char).fg(props.fg);
            }
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Checkbox => {
            let mut w = crate::renderable::widgets::CheckboxWidget::new(elem.layout.clone());
            if let Props::Checkbox(ref props) = elem.props {
                if props.checked {
                    w = w.checked(true);
                }
                if let Some(ref label) = props.label {
                    w = w.label(label);
                }
            }
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Spinner => {
            let mut w = crate::renderable::widgets::SpinnerWidget::new(elem.layout.clone());
            if let Props::Spinner(ref props) = elem.props {
                let frames = match props.preset {
                    SpinnerPreset::Braille => crate::renderable::widgets::SpinnerFrames::braille(),
                    SpinnerPreset::Dots => crate::renderable::widgets::SpinnerFrames::dots(),
                    SpinnerPreset::Arrow => crate::renderable::widgets::SpinnerFrames::arrow(),
                    SpinnerPreset::Line => crate::renderable::widgets::SpinnerFrames::line(),
                    SpinnerPreset::Bounce => crate::renderable::widgets::SpinnerFrames::bounce(),
                    SpinnerPreset::Ascii => crate::renderable::widgets::SpinnerFrames::ascii(),
                };
                w = w.frames(frames).running(props.running);
                if let Some(ref label) = props.label {
                    w = w.label(label);
                }
            }
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Badge => {
            let mut w = if let Props::Badge(ref props) = elem.props {
                crate::renderable::widgets::BadgeWidget::new(elem.layout.clone(), &props.text)
                    .badge_style(crate::renderable::widgets::BadgeStyle {
                        shape: props.shape,
                        fg: props.fg,
                        bg: props.bg,
                        ..Default::default()
                    })
            } else {
                crate::renderable::widgets::BadgeWidget::new(elem.layout.clone(), "")
            };
            let _ = &mut w;
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Slider => {
            let mut w = if let Props::Slider(ref props) = elem.props {
                let s = if props.horizontal {
                    crate::renderable::widgets::SliderWidget::horizontal(elem.layout.clone())
                } else {
                    crate::renderable::widgets::SliderWidget::vertical(elem.layout.clone())
                };
                s.range(props.min, props.max)
                    .value(props.value)
                    .viewport_size(props.viewport_size)
            } else {
                crate::renderable::widgets::SliderWidget::horizontal(elem.layout.clone())
            };
            let _ = &mut w;
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Select => {
            let mut w = crate::renderable::widgets::SelectWidget::new(elem.layout.clone());
            if let Props::Select(ref props) = elem.props {
                let items: Vec<_> = props
                    .items
                    .iter()
                    .map(|s| crate::renderable::widgets::SelectItem::new(s.as_str()))
                    .collect();
                w = w
                    .items(items)
                    .wrap_selection(props.wrap)
                    .show_description(props.show_description);
                w.set_selected(props.selected);
            }
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::RadioGroup => {
            let mut w = if let Props::RadioGroup(ref props) = elem.props {
                let opts: Vec<_> = props
                    .options
                    .iter()
                    .map(|s| crate::renderable::widgets::RadioOption::new(s.as_str()))
                    .collect();
                if props.horizontal {
                    crate::renderable::widgets::RadioGroupWidget::horizontal(elem.layout.clone())
                        .options(opts)
                        .selected(props.selected)
                } else {
                    crate::renderable::widgets::RadioGroupWidget::vertical(elem.layout.clone())
                        .options(opts)
                        .selected(props.selected)
                }
            } else {
                crate::renderable::widgets::RadioGroupWidget::vertical(elem.layout.clone())
            };
            let _ = &mut w;
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Gauge => {
            let mut w = if let Props::Gauge(ref props) = elem.props {
                let g = if props.horizontal {
                    crate::renderable::widgets::GaugeWidget::horizontal(elem.layout.clone())
                } else {
                    crate::renderable::widgets::GaugeWidget::vertical(elem.layout.clone())
                };
                g.range(props.min, props.max)
                    .value(props.value)
                    .segments(props.segments)
                    .show_label(props.show_label)
            } else {
                crate::renderable::widgets::GaugeWidget::horizontal(elem.layout.clone())
            };
            let _ = &mut w;
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::ScrollBar => {
            let mut w = if let Props::ScrollBar(ref props) = elem.props {
                let sb = if props.horizontal {
                    crate::renderable::widgets::ScrollBarWidget::horizontal(elem.layout.clone())
                } else {
                    crate::renderable::widgets::ScrollBarWidget::vertical(elem.layout.clone())
                };
                sb.scroll_size(props.scroll_size)
                    .viewport_size(props.viewport_size)
                    .scroll_position(props.scroll_position)
                    .show_arrows(props.show_arrows)
            } else {
                crate::renderable::widgets::ScrollBarWidget::vertical(elem.layout.clone())
            };
            let _ = &mut w;
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
        ElementKind::Custom(_) | ElementKind::View => {
            let w = crate::renderable::widgets::ViewWidget::from_element(elem);
            let d = w.framework_defaults();
            (Box::new(w), d)
        }
    }
}
