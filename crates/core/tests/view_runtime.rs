#![allow(clippy::cast_precision_loss)]

use opentui_core::renderable::context::RenderContext;
use opentui_core::view::{
    ViewRuntime, badge, checkbox, gauge, overlay, panel, radio_group, select, slider, spinner,
    text, view,
};
use opentui_core::{OptimizedBuffer, Rgba};

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
    assert_eq!(runtime.tree().overlays().len(), 1);
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

    let mut runtime = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(20.0, 10.0);

    let mut buffer = OptimizedBuffer::new(20, 10);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
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

    let mut runtime = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(10.0, 3.0);

    let mut buffer = OptimizedBuffer::new(10, 3);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
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

    let mut runtime = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(20.0, 10.0);

    let mut buffer = OptimizedBuffer::new(20, 10);
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: None,
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
        .children([text("hello")
            .fg(TEXT)
            .height(1.0)
            .on_action("greet")
            .build()])
        .build();

    let mut runtime = ViewRuntime::new();
    let w = 20u32;
    let h = 5u32;
    runtime.rebuild(&node);
    runtime.layout(w as f32, h as f32);
    runtime.register_hit_areas(w, h);

    let mouse = opentui_core::terminal::MouseEvent::new(
        0,
        0,
        opentui_core::terminal::MouseButton::Left,
        opentui_core::terminal::MouseEventKind::Press,
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
        .on_action("overlay")
        .children([text("top").fg(TEXT).height(1.0).build()])
        .build();
    let node = view()
        .column()
        .size(20.0, 10.0)
        .bg(BG)
        .on_action("background")
        .children([overlay(overlay_content)
            .position(5, 2)
            .size(10, 3)
            .z_order(400)
            .build()])
        .build();

    let mut runtime = ViewRuntime::new();
    let w = 20u32;
    let h = 10u32;
    runtime.rebuild(&node);
    runtime.layout(w as f32, h as f32);
    runtime.register_hit_areas(w, h);

    let mouse = opentui_core::terminal::MouseEvent::new(
        5,
        2,
        opentui_core::terminal::MouseButton::Left,
        opentui_core::terminal::MouseEventKind::Press,
    );
    let result = runtime.dispatch_mouse(&mouse);
    assert_eq!(result.action.as_deref(), Some("overlay"));
}

#[test]
fn test_on_action_click_outside_returns_none() {
    let node = view()
        .column()
        .size(20.0, 5.0)
        .bg(BG)
        .children([text("hello")
            .fg(TEXT)
            .height(1.0)
            .on_action("greet")
            .build()])
        .build();

    let mut runtime = ViewRuntime::new();
    let w = 20u32;
    let h = 5u32;
    runtime.rebuild(&node);
    runtime.layout(w as f32, h as f32);
    runtime.register_hit_areas(w, h);

    let mouse = opentui_core::terminal::MouseEvent::new(
        0,
        4,
        opentui_core::terminal::MouseButton::Left,
        opentui_core::terminal::MouseEventKind::Press,
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

    let mut runtime = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(10.0, 3.0);

    let mut buffer = OptimizedBuffer::new(10, 3);
    let mut pool = opentui_core::GraphemePool::new();
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: Some(&mut pool),
        link_pool: None,
        hit_grid: None,
        theme: None,
    };
    runtime.render(&mut ctx);

    let cell = buffer.get(0, 0);
    assert!(
        cell.is_some(),
        "combining mark text should produce a cell at (0,0)"
    );
    if let Some(c) = cell {
        match c.content {
            opentui_core::CellContent::Grapheme(gid) => {
                let resolved = pool.get(gid);
                assert_eq!(
                    resolved,
                    Some(combined),
                    "grapheme pool should store full combining sequence"
                );
            }
            opentui_core::CellContent::Char(ch) => {
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

    let mut runtime = ViewRuntime::new();
    runtime.rebuild(&node);
    runtime.layout(10.0, 3.0);

    let mut buffer = OptimizedBuffer::new(10, 3);
    let mut pool = opentui_core::GraphemePool::new();
    let mut ctx = RenderContext {
        buffer: &mut buffer,
        grapheme_pool: Some(&mut pool),
        link_pool: None,
        hit_grid: None,
        theme: None,
    };
    runtime.render(&mut ctx);

    let cell = buffer.get(0, 0);
    assert!(cell.is_some(), "ZWJ emoji should produce a cell at (0,0)");
    if let Some(c) = cell {
        match c.content {
            opentui_core::CellContent::Grapheme(gid) => {
                let resolved = pool.get(gid);
                assert_eq!(
                    resolved,
                    Some(emoji),
                    "grapheme pool should store full ZWJ sequence"
                );
            }
            opentui_core::CellContent::Char(ch) => {
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

#[test]
fn test_rebuild_checkbox_declarative() {
    let node = view()
        .column()
        .size(30.0, 1.0)
        .children([checkbox("Accept terms").height(1.0).build()])
        .build();

    let buffer = render_node(&node, 30, 1);
    assert_eq!(
        buffer.get(0, 0).and_then(|c| c.content.as_char()),
        Some('[')
    );
    assert_eq!(
        buffer.get(1, 0).and_then(|c| c.content.as_char()),
        Some(' ')
    );
    assert_eq!(
        buffer.get(4, 0).and_then(|c| c.content.as_char()),
        Some('A')
    );
}

#[test]
fn test_rebuild_spinner_declarative() {
    let node = view()
        .column()
        .size(20.0, 1.0)
        .children([spinner().label("Loading").height(1.0).build()])
        .build();

    let buffer = render_node(&node, 20, 1);
    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    assert!(ch.is_some(), "spinner should render a frame char");
    assert_eq!(
        buffer.get(2, 0).and_then(|c| c.content.as_char()),
        Some('L')
    );
}

#[test]
fn test_rebuild_badge_declarative() {
    let node = view()
        .column()
        .size(20.0, 1.0)
        .children([badge("OK").height(1.0).build()])
        .build();

    let buffer = render_node(&node, 20, 1);
    assert_eq!(
        buffer.get(0, 0).and_then(|c| c.content.as_char()),
        Some(' ')
    );
    assert_eq!(
        buffer.get(2, 0).and_then(|c| c.content.as_char()),
        Some('O')
    );
    assert_eq!(
        buffer.get(3, 0).and_then(|c| c.content.as_char()),
        Some('K')
    );
}

#[test]
fn test_rebuild_slider_declarative() {
    let node = view()
        .column()
        .size(20.0, 1.0)
        .children([slider().height(1.0).build()])
        .build();

    let buffer = render_node(&node, 20, 1);
    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    assert!(ch.is_some(), "slider should render track");
}

#[test]
fn test_rebuild_select_declarative() {
    let node = view()
        .column()
        .size(20.0, 3.0)
        .children([select(vec!["One".into(), "Two".into(), "Three".into()])
            .height(3.0)
            .build()])
        .build();

    let buffer = render_node(&node, 20, 3);
    assert_eq!(
        buffer.get(2, 0).and_then(|c| c.content.as_char()),
        Some('O')
    );
    assert_eq!(
        buffer.get(2, 1).and_then(|c| c.content.as_char()),
        Some('T')
    );
}

#[test]
fn test_rebuild_radio_group_declarative() {
    let node = view()
        .column()
        .size(20.0, 3.0)
        .children([radio_group(vec!["Yes".into(), "No".into()])
            .height(3.0)
            .build()])
        .build();

    let buffer = render_node(&node, 20, 3);
    assert_eq!(
        buffer.get(1, 0).and_then(|c| c.content.as_char()),
        Some('\u{25CF}')
    );
    assert_eq!(
        buffer.get(4, 0).and_then(|c| c.content.as_char()),
        Some('Y')
    );
    assert_eq!(
        buffer.get(1, 1).and_then(|c| c.content.as_char()),
        Some('\u{25CB}')
    );
}

#[test]
fn test_rebuild_gauge_declarative() {
    let node = view()
        .column()
        .size(10.0, 1.0)
        .children([gauge().height(1.0).build()])
        .build();

    let buffer = render_node(&node, 10, 1);
    let ch = buffer.get(0, 0).and_then(|c| c.content.as_char());
    assert!(ch.is_some(), "gauge should render");
}
