use opentui_core::view::{
    Element, ElementKind, Key, Node, Props, empty, fragment, input, overlay, panel, rich_text,
    span, text, view, when,
};

fn unwrap_element(node: &Node) -> &Element {
    match node {
        Node::Element(e) => e,
        _ => panic!("expected Element, got {:?}", node),
    }
}

#[test]
fn test_view_creates_element_kind_view() {
    let node = view().build();
    let elem = unwrap_element(&node);
    assert_eq!(elem.kind, ElementKind::View);
}

#[test]
fn test_panel_sets_border() {
    let node = panel().build();
    let elem = unwrap_element(&node);
    assert_eq!(elem.kind, ElementKind::View);
    if let Props::View(ref vp) = elem.props {
        assert!(vp.border.is_some());
    } else {
        panic!("expected ViewProps");
    }
}

#[test]
fn test_text_stores_content() {
    let node = text("hello").build();
    let elem = unwrap_element(&node);
    assert_eq!(elem.kind, ElementKind::Text);
    if let Props::Text(ref tp) = elem.props {
        assert_eq!(tp.content, "hello");
    } else {
        panic!("expected TextProps");
    }
}

#[test]
fn test_text_fg_bg_bold() {
    let node = text("x")
        .fg(opentui_rust::Rgba::new(1.0, 0.0, 0.0, 1.0))
        .bg(opentui_rust::Rgba::new(0.0, 0.0, 1.0, 1.0))
        .bold()
        .italic()
        .underline()
        .build();
    let elem = unwrap_element(&node);
    if let Props::Text(ref tp) = elem.props {
        assert!(tp.bold);
        assert!(tp.italic);
        assert!(tp.underline);
        assert!(tp.fg.r > 0.9);
        assert!(tp.bg.is_some());
    } else {
        panic!("expected TextProps");
    }
}

#[test]
fn test_when_false_returns_empty() {
    let node = when(false, || text("hidden").build());
    assert!(matches!(node, Node::Empty));
}

#[test]
fn test_when_true_returns_inner() {
    let node = when(true, || text("visible").build());
    assert!(matches!(node, Node::Element(_)));
}

#[test]
fn test_fragment_preserves_children() {
    let node = fragment(vec![text("a").build(), text("b").build()]);
    match node {
        Node::Fragment(children) => assert_eq!(children.len(), 2),
        _ => panic!("expected Fragment"),
    }
}

#[test]
fn test_empty_returns_empty() {
    assert!(matches!(empty(), Node::Empty));
}

#[test]
fn test_key_static() {
    let node = view().key("root").build();
    let elem = unwrap_element(&node);
    assert_eq!(elem.key, Some(Key::Static("root")));
}

#[test]
fn test_key_owned() {
    let node = view().key(String::from("dynamic")).build();
    let elem = unwrap_element(&node);
    assert!(matches!(elem.key, Some(Key::Owned(_))));
}

#[test]
fn test_layout_row_column() {
    let row = view().row().build();
    let col = view().column().build();
    let row_elem = unwrap_element(&row);
    let col_elem = unwrap_element(&col);
    assert!(
        row_elem.layout.inner.flex_direction
            == opentui_core::layout::taffy_style::FlexDirection::Row
    );
    assert!(
        col_elem.layout.inner.flex_direction
            == opentui_core::layout::taffy_style::FlexDirection::Column
    );
}

#[test]
fn test_children_with_array() {
    let node = view()
        .children([text("a").build(), text("b").build()])
        .build();
    let elem = unwrap_element(&node);
    assert_eq!(elem.children.len(), 2);
}

#[test]
fn test_view_bg_opacity_focusable() {
    let node = view()
        .bg(opentui_rust::Rgba::new(0.1, 0.1, 0.1, 1.0))
        .opacity(0.5)
        .focusable()
        .build();
    let elem = unwrap_element(&node);
    if let Props::View(ref vp) = elem.props {
        assert!(vp.bg.is_some());
        assert!((vp.opacity - 0.5).abs() < 0.01);
        assert!(vp.focusable);
    } else {
        panic!("expected ViewProps");
    }
}

#[test]
fn test_view_overflow_hidden() {
    let node = view().overflow_hidden().build();
    let elem = unwrap_element(&node);
    if let Props::View(ref vp) = elem.props {
        assert!(matches!(
            vp.overflow,
            opentui_core::widget::Overflow::Hidden
        ));
    }
}

#[test]
fn test_panel_title() {
    let node = panel().title("My Panel").build();
    let elem = unwrap_element(&node);
    if let Props::View(ref vp) = elem.props {
        assert_eq!(vp.title.as_deref(), Some("My Panel"));
    }
}

#[test]
fn test_rich_text_with_segments() {
    let seg1 = span("Hello ", opentui_rust::Rgba::WHITE);
    let seg2 = span("World", opentui_rust::Rgba::new(1.0, 0.0, 0.0, 1.0)).bold();
    let node = rich_text(vec![seg1, seg2]).build();
    let elem = unwrap_element(&node);
    assert_eq!(elem.kind, ElementKind::StyledText);
    if let Props::StyledText(ref sp) = elem.props {
        assert_eq!(sp.segments.len(), 2);
        assert_eq!(sp.segments[1].bold, true);
    } else {
        panic!("expected StyledTextProps");
    }
}

#[test]
fn test_input_builder() {
    let node = input()
        .placeholder("Enter text...")
        .password()
        .value("secret")
        .build();
    let elem = unwrap_element(&node);
    assert_eq!(elem.kind, ElementKind::Input);
    if let Props::Input(ref ip) = elem.props {
        assert_eq!(ip.placeholder.as_deref(), Some("Enter text..."));
        assert!(ip.password);
        assert_eq!(ip.initial_value.as_deref(), Some("secret"));
    } else {
        panic!("expected InputProps");
    }
}

#[test]
fn test_overlay_builder() {
    let content = panel().title("Modal").build();
    let node = overlay(content)
        .position(10, 5)
        .size(40, 10)
        .backdrop()
        .z_order(400)
        .build();
    match &node {
        Node::Overlay(ov) => {
            assert_eq!(ov.x, 10);
            assert_eq!(ov.y, 5);
            assert_eq!(ov.width, 40);
            assert_eq!(ov.height, 10);
            assert!(ov.backdrop);
            assert_eq!(ov.z_order, 400);
        }
        _ => panic!("expected Overlay, got {:?}", node),
    }
}
