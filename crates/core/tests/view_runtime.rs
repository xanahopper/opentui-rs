use opentui_core::view::{ViewRuntime, overlay, panel, text, view};
use opentui_core::widget::RenderContext;
use opentui_rust::{OptimizedBuffer, Rgba};

const BG: Rgba = Rgba::new(0.0, 0.0, 0.0, 1.0);
const TEXT: Rgba = Rgba::new(1.0, 1.0, 1.0, 1.0);

fn render_node(node: &opentui_core::view::Node, w: u32, h: u32) -> OptimizedBuffer {
    let mut runtime = ViewRuntime::new();
    runtime.rebuild(node);
    runtime.layout(w as f32, h as f32);

    let mut buffer = OptimizedBuffer::new(w, h);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
    };
    runtime.render(&mut ctx);
    buffer
}

#[test]
fn test_rebuild_simple_text() {
    let node = view()
        .column()
        .size(20.0, 5.0)
        .bg(BG)
        .children([text("hello").fg(TEXT).height(1.0).build()])
        .build();

    let buffer = render_node(&node, 20, 5);
    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    assert_eq!(ch, Some('h'));
}

#[test]
fn test_rebuild_nested_layout() {
    let node = view()
        .row()
        .size(40.0, 5.0)
        .bg(BG)
        .children([
            view()
                .width(10.0)
                .height(5.0)
                .bg(Rgba::new(0.1, 0.1, 0.1, 1.0))
                .build(),
            text("right").fg(TEXT).height(1.0).grow(1.0).build(),
        ])
        .build();

    let buffer = render_node(&node, 40, 5);
    let ch = buffer.get(10, 0).and_then(|c| c.content.as_char());
    assert_eq!(ch, Some('r'));
}

#[test]
fn test_rebuild_then_rebuild_again() {
    let mut runtime = ViewRuntime::new();
    let w = 20u32;
    let h = 5u32;

    let node1 = view()
        .column()
        .size_pct(1.0, 1.0)
        .bg(BG)
        .children([text("first").fg(TEXT).height(1.0).build()])
        .build();
    runtime.rebuild(&node1);
    runtime.layout(w as f32, h as f32);
    assert!(runtime.tree().root().is_some());

    let node2 = view()
        .column()
        .size_pct(1.0, 1.0)
        .bg(BG)
        .children([text("second").fg(TEXT).height(1.0).build()])
        .build();
    runtime.rebuild(&node2);
    runtime.layout(w as f32, h as f32);

    let mut buffer = OptimizedBuffer::new(w, h);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
    };
    runtime.render(&mut ctx);

    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    assert_eq!(ch, Some('s'));
}

#[test]
fn test_empty_node_does_not_crash() {
    let node = opentui_core::view::empty();
    let mut runtime = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(20.0, 5.0);

    let mut buffer = OptimizedBuffer::new(20, 5);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
    };
    runtime.render(&mut ctx);
}

#[test]
fn test_when_conditional_rendering() {
    let node = view()
        .column()
        .size(20.0, 5.0)
        .bg(BG)
        .children([
            text("always").fg(TEXT).height(1.0).build(),
            opentui_core::view::when(false, || text("hidden").fg(TEXT).height(1.0).build()),
        ])
        .build();

    let buffer = render_node(&node, 20, 5);
    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    assert_eq!(ch, Some('a'));
}

#[test]
fn test_rebuild_with_overlay() {
    let content = panel()
        .title("Popup")
        .size(10.0, 3.0)
        .children([text("ok").fg(TEXT).height(1.0).build()])
        .build();
    let node = view()
        .column()
        .size(20.0, 10.0)
        .bg(BG)
        .children([
            text("bg").fg(TEXT).height(1.0).build(),
            overlay(content)
                .position(5, 2)
                .size(10, 3)
                .backdrop()
                .build(),
        ])
        .build();

    let mut runtime = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(20.0, 10.0);
    assert!(runtime.tree().overlays().len() == 1);
}
