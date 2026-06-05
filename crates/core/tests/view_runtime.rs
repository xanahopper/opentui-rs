#![allow(clippy::cast_precision_loss)]

use opentui_core::view::{ViewRuntime, overlay, panel, text, view};
use opentui_core::widget::RenderContext;
use opentui_rust::{OptimizedBuffer, Rgba};

const BG: Rgba = Rgba::new(0.0, 0.0, 0.0, 1.0);
const TEXT: Rgba = Rgba::new(1.0, 1.0, 1.0, 1.0);

fn render_node(node: &opentui_core::view::Node<()>, w: u32, h: u32) -> OptimizedBuffer {
    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
    runtime.rebuild(node);
    runtime.layout(w as f32, h as f32);

    let mut buffer = OptimizedBuffer::new(w, h);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
        hovered_id: None,
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
    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
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
        hovered_id: None,
    };
    runtime.render(&mut ctx);

    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    assert_eq!(ch, Some('s'));
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

    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(20.0, 10.0);
    assert!(runtime.tree().overlays().len() == 1);
}

#[test]
fn test_overlay_child_text_is_rendered() {
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
            overlay(content).position(5, 2).size(10, 3).build(),
        ])
        .build();

    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(20.0, 10.0);

    let mut buffer = OptimizedBuffer::new(20, 10);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
        hovered_id: None,
    };
    runtime.render(&mut ctx);

    let mut found = false;
    for y in 0..10 {
        for x in 0..20 {
            if let Some(ch) = buffer.get(x, y).and_then(|c| c.content.as_char()) {
                if ch == 'o' || ch == 'k' {
                    eprintln!("found '{ch}' at ({x},{y})");
                    found = true;
                }
            }
        }
    }
    assert!(
        found,
        "overlay child text 'ok' should be rendered somewhere in the buffer"
    );
}

#[test]
fn test_text_grapheme_wide_char_has_continuation() {
    let node = view()
        .column()
        .size(10.0, 3.0)
        .bg(BG)
        .children([text("\u{4E16}").fg(TEXT).height(1.0).build()])
        .build();

    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(10.0, 3.0);

    let mut buffer = OptimizedBuffer::new(10, 3);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
        hovered_id: None,
    };
    runtime.render(&mut ctx);

    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    let cont = buffer.get(1, 0);
    assert!(
        ch == Some('\u{4E16}'),
        "wide char should be at col 0, got {ch:?}"
    );
    assert!(
        cont.is_some_and(|c| c.content.is_continuation()),
        "col 1 should be a continuation cell, got {cont:?}"
    );
}

#[test]
fn test_overlay_child_at_deterministic_position() {
    let content = view()
        .column()
        .size(10.0, 3.0)
        .bg(Rgba::new(0.15, 0.15, 0.15, 1.0))
        .children([text("XY").fg(TEXT).height(1.0).build()])
        .build();
    let node = view()
        .column()
        .size(20.0, 10.0)
        .bg(BG)
        .children([
            text("bg").fg(TEXT).height(1.0).build(),
            overlay(content).position(5, 2).size(10, 3).build(),
        ])
        .build();

    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(20.0, 10.0);

    let mut buffer = OptimizedBuffer::new(20, 10);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
        hovered_id: None,
    };
    runtime.render(&mut ctx);

    let ch_x = buffer.get(5, 2).and_then(|c| c.content.as_char());
    let ch_y = buffer.get(6, 2).and_then(|c| c.content.as_char());
    assert_eq!(ch_x, Some('X'), "overlay child text 'X' at (5,2)");
    assert_eq!(ch_y, Some('Y'), "overlay child text 'Y' at (6,2)");
}

#[test]
fn test_on_action_click_returns_action() {
    let node = view()
        .column()
        .size(20.0, 5.0)
        .bg(BG)
        .children([text("hello").fg(TEXT).height(1.0).build()])
        .on_click("greet".to_string())
        .build();

    let mut runtime: ViewRuntime<String> = ViewRuntime::new();
    let w = 20u32;
    let h = 5u32;
    runtime.rebuild(&node);
    runtime.layout(w as f32, h as f32);
    runtime.register_hit_areas(w, h);

    let mouse = opentui_rust::terminal::MouseEvent::new(
        0,
        0,
        opentui_rust::terminal::MouseButton::Left,
        opentui_rust::terminal::MouseEventKind::Press,
    );
    let result = runtime.dispatch_mouse(&mouse);
    assert_eq!(
        result.action.as_deref(),
        Some("greet"),
        "clicking action node should return 'greet'"
    );
}

#[test]
fn test_overlay_action_wins_over_background_action() {
    let overlay_content = view()
        .column()
        .size(10.0, 3.0)
        .bg(Rgba::new(0.15, 0.15, 0.15, 1.0))
        .children([text("top").fg(TEXT).height(1.0).build()])
        .on_click("overlay".to_string())
        .build();
    let node = view()
        .column()
        .size(20.0, 10.0)
        .bg(BG)
        .on_click("background".to_string())
        .children([overlay(overlay_content)
            .position(5, 2)
            .size(10, 3)
            .z_order(400)
            .build()])
        .build();

    let mut runtime: ViewRuntime<String> = ViewRuntime::new();
    let w = 20u32;
    let h = 5u32;
    runtime.rebuild(&node);
    runtime.layout(w as f32, h as f32);
    runtime.register_hit_areas(w, h);

    let mouse = opentui_rust::terminal::MouseEvent::new(
        5,
        2,
        opentui_rust::terminal::MouseButton::Left,
        opentui_rust::terminal::MouseEventKind::Press,
    );
    let result = runtime.dispatch_mouse(&mouse);
    assert_eq!(result.action.as_deref(), Some("overlay"));
}

#[test]
fn test_on_action_click_outside_returns_none() {
    let text_node: opentui_core::view::Node<String> = text("hello")
        .fg(TEXT)
        .height(1.0)
        .on_click("greet".to_string())
        .build();
    let parent = view()
        .column()
        .size(20.0, 5.0)
        .bg(BG)
        .build()
        .map_msg(|()| String::new());

    let combined = match parent {
        opentui_core::view::Node::Element(mut e) => {
            e.children = vec![text_node];
            opentui_core::view::Node::Element(e)
        }
        _ => panic!("expected Element"),
    };

    let mut runtime: ViewRuntime<String> = ViewRuntime::new();
    let w = 20u32;
    let h = 5u32;
    runtime.rebuild(&combined);
    runtime.layout(w as f32, h as f32);
    runtime.register_hit_areas(w, h);

    let mouse = opentui_rust::terminal::MouseEvent::new(
        0,
        4,
        opentui_rust::terminal::MouseButton::Left,
        opentui_rust::terminal::MouseEventKind::Press,
    );
    let result = runtime.dispatch_mouse(&mouse);
    assert_eq!(
        result.action, None,
        "clicking outside action node should return no action"
    );
}

#[test]
fn test_grapheme_pool_combining_mark() {
    let combined = "e\u{0301}";
    let node = view()
        .column()
        .size(10.0, 3.0)
        .bg(BG)
        .children([text(combined).fg(TEXT).height(1.0).build()])
        .build();

    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(10.0, 3.0);

    let mut buffer = OptimizedBuffer::new(10, 3);
    let mut pool = opentui_rust::GraphemePool::new();
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: Some(&mut pool),
        link_pool: None,
        hit_grid: None,
        theme: None,
        hovered_id: None,
    };
    runtime.render(&mut ctx);

    let cell = buffer.get(0, 0);
    assert!(
        cell.is_some(),
        "combining mark text should produce a cell at (0,0)"
    );
    if let Some(c) = cell {
        match c.content {
            opentui_rust::CellContent::Grapheme(gid) => {
                let resolved = pool.get(gid);
                assert_eq!(
                    resolved,
                    Some(combined),
                    "grapheme pool should store full combining sequence"
                );
            }
            opentui_rust::CellContent::Char(ch) => {
                panic!(
                    "expected Grapheme content for combining mark, got Char('{ch}') — pool not used?"
                );
            }
            other => panic!("unexpected cell content: {other:?}"),
        }
    }
}

#[test]
fn test_grapheme_pool_zwj_emoji() {
    let emoji = "\u{1F469}\u{200D}\u{1F4BB}";
    let node = view()
        .column()
        .size(10.0, 3.0)
        .bg(BG)
        .children([text(emoji).fg(TEXT).height(1.0).build()])
        .build();

    let mut runtime: ViewRuntime<()> = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(10.0, 3.0);

    let mut buffer = OptimizedBuffer::new(10, 3);
    let mut pool = opentui_rust::GraphemePool::new();
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: Some(&mut pool),
        link_pool: None,
        hit_grid: None,
        theme: None,
        hovered_id: None,
    };
    runtime.render(&mut ctx);

    let cell = buffer.get(0, 0);
    assert!(cell.is_some(), "ZWJ emoji should produce a cell at (0,0)");
    if let Some(c) = cell {
        match c.content {
            opentui_rust::CellContent::Grapheme(gid) => {
                let resolved = pool.get(gid);
                assert_eq!(
                    resolved,
                    Some(emoji),
                    "grapheme pool should store full ZWJ sequence"
                );
            }
            opentui_rust::CellContent::Char(ch) => {
                panic!(
                    "expected Grapheme content for ZWJ emoji, got Char('{ch}') — pool not used?"
                );
            }
            other => panic!("unexpected cell content: {other:?}"),
        }
    }

    let cont = buffer.get(1, 0);
    assert!(
        cont.is_some_and(|c| c.content.is_continuation()),
        "ZWJ emoji (width 2) should have continuation cell at col 1"
    );
}
