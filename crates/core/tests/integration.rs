//! Integration tests for full tree → layout → render pipeline.

#![allow(clippy::float_cmp)]

use opentui_core::renderable::context::RenderContext;
use opentui_core::renderable::layout::ComputedLayout;
use opentui_core::renderable::node::NodeId;
use opentui_core::renderable::tree::{Overlay, RenderTree};
use opentui_core::widgets::{
    BoxWidget, ProgressBarWidget, ScrollViewWidget, StatusLineWidget, Tab, TabsWidget,
    TextLineWidget, TextWidget,
};
use opentui_core::{OptimizedBuffer, Rgba, Style};

use opentui_core::layout::LayoutStyle;
use opentui_core::theme::UiTheme;

const fn make_ctx<'a>(buf: &'a mut OptimizedBuffer, theme: &'a UiTheme) -> RenderContext<'a> {
    RenderContext {
        buffer: buf,
        grapheme_pool: None,
        link_pool: None,
        hit_grid: None,
        theme: Some(theme),
    }
}

fn cell_char(buf: &OptimizedBuffer, x: u32, y: u32) -> Option<char> {
    buf.get(x, y).and_then(|c| c.content.as_char())
}

// The new `RenderTree` stores overlays but does not draw them itself (overlay
// rendering is currently driven by the higher-level View layer). To keep these
// pipeline tests meaningful, we render the registered overlay nodes manually
// on top of the main tree, in z-order, with optional backdrop clearing —
// mirroring the old `WidgetTree::render` behavior.
fn render_overlays(tree: &mut RenderTree, ctx: &mut RenderContext<'_>) {
    let mut overlays: Vec<Overlay> = tree.overlays().to_vec();
    overlays.sort_by_key(|o| o.z_order);

    let (w, h) = (ctx.buffer.width(), ctx.buffer.height());
    for ov in overlays {
        if ov.backdrop {
            let spaces = " ".repeat(w as usize);
            for y in 0..h {
                ctx.buffer.draw_text(0, y, &spaces, Style::NONE);
            }
        }
        let layout = ComputedLayout {
            x: ov.x,
            y: ov.y,
            width: ov.width,
            height: ov.height,
        };
        if let Some(node) = tree.get_mut(ov.node) {
            node.behavior.render_self(ctx, &layout);
        }
    }
}

#[test]
fn test_text_widget_renders() {
    let mut buf = OptimizedBuffer::new(40, 5);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let root = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::column().width(40.0).height(5.0)).background(Rgba::BLACK),
    ));
    let _text = tree.add_child(
        root,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "Hello World",
        )),
    );

    tree.run_layout(40.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('H'));
    assert_eq!(cell_char(&buf, 1, 0), Some('e'));
    assert_eq!(cell_char(&buf, 4, 0), Some('o'));
}

#[test]
fn test_status_line_renders_segments() {
    let mut buf = OptimizedBuffer::new(40, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _sl = tree.set_root(Box::new(
        StatusLineWidget::new(LayoutStyle::default().width(40.0).height(1.0))
            .left("LEFT")
            .center("MID")
            .right("RIGHT"),
    ));

    tree.run_layout(40.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('L'));
    // Right segment at end
    assert_eq!(cell_char(&buf, 35, 0), Some('R'));
}

#[test]
fn test_progress_bar_renders_fill() {
    let mut buf = OptimizedBuffer::new(22, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _bar = tree.set_root(Box::new(
        ProgressBarWidget::new(LayoutStyle::default().width(22.0).height(1.0)).progress(0.5),
    ));

    tree.run_layout(22.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    // left cap at col 0
    assert_eq!(cell_char(&buf, 0, 0), Some('['));
    // right cap at col 21
    assert_eq!(cell_char(&buf, 21, 0), Some(']'));
    // filled char at col 1 (50% of 20 inner = 10 filled)
    assert_eq!(cell_char(&buf, 1, 0), Some('█'));
    // empty char at col 19
    assert_eq!(cell_char(&buf, 19, 0), Some('░'));
}

#[test]
fn test_tabs_renders_titles() {
    let mut buf = OptimizedBuffer::new(40, 3);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _tabs = tree.set_root(Box::new(
        TabsWidget::new(LayoutStyle::default().width(40.0).height(3.0))
            .tabs(vec![Tab::new("File"), Tab::new("Edit")]),
    ));

    tree.run_layout(40.0, 3.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 1, 0), Some('F'));
    assert_eq!(cell_char(&buf, 2, 0), Some('i'));
    assert_eq!(cell_char(&buf, 3, 0), Some('l'));
    assert_eq!(cell_char(&buf, 4, 0), Some('e'));
}

#[test]
fn test_nested_layout_row_column() {
    let mut buf = OptimizedBuffer::new(40, 5);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let root: NodeId = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::row().width(40.0).height(5.0)).background(Rgba::BLACK),
    ));

    let left: NodeId = tree.add_child(
        root,
        Box::new(
            BoxWidget::new(LayoutStyle::column().width(20.0).height(5.0)).background(Rgba::BLACK),
        ),
    );
    let _left_text = tree.add_child(
        left,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "L",
        )),
    );

    let right = tree.add_child(
        root,
        Box::new(BoxWidget::new(LayoutStyle::column().flex_grow(1.0)).background(Rgba::BLACK)),
    );
    let _right_text = tree.add_child(
        right,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "R",
        )),
    );

    tree.run_layout(40.0, 5.0);

    let left_layout = tree.computed_layout(left).unwrap();
    let right_layout = tree.computed_layout(right).unwrap();
    assert_eq!(left_layout.width, 20.0);
    assert_eq!(right_layout.x, 20.0);
    assert_eq!(right_layout.width, 20.0);

    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('L'));
    assert_eq!(cell_char(&buf, 20, 0), Some('R'));
}

#[test]
fn test_scroll_view_offsets_child_rendering() {
    let mut buf = OptimizedBuffer::new(10, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let scroll = tree.set_root(Box::new(
        ScrollViewWidget::new(LayoutStyle::column().width(10.0).height(1.0))
            .content_height(3.0)
            .scrollbar(false)
            .focusable(),
    ));
    tree.set_focusable(scroll, true);
    let content = tree.add_child(
        scroll,
        Box::new(BoxWidget::new(
            LayoutStyle::column().width(10.0).height(3.0),
        )),
    );
    let _a = tree.add_child(
        content,
        Box::new(TextLineWidget::with_text(
            LayoutStyle::default().height(1.0),
            "A",
        )),
    );
    let _b = tree.add_child(
        content,
        Box::new(TextLineWidget::with_text(
            LayoutStyle::default().height(1.0),
            "B",
        )),
    );
    let _c = tree.add_child(
        content,
        Box::new(TextLineWidget::with_text(
            LayoutStyle::default().height(1.0),
            "C",
        )),
    );

    tree.focus(scroll);
    tree.run_layout(10.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }
    assert_eq!(cell_char(&buf, 0, 0), Some('A'));

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Down,)));
    let mut buf = OptimizedBuffer::new(10, 1);
    tree.run_layout(10.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('B'));
}

#[test]
fn test_overlay_renders_on_top() {
    let mut buf = OptimizedBuffer::new(40, 10);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let root = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::column().width(40.0).height(10.0)).background(Rgba::BLACK),
    ));
    let _bg = tree.add_child(
        root,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "BBBBBBBBBB",
        )),
    );

    // Overlay nodes must live in the arena. Attach as an invisible child of the
    // root so the tree's own render pass skips it; the overlay helper draws it
    // at the explicit overlay coordinates instead.
    let overlay_widget = tree.add_child(
        root,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().width(5.0).height(1.0),
            "OVER",
        )),
    );
    tree.set_visible(overlay_widget, false);
    tree.add_overlay(Overlay {
        node: overlay_widget,
        x: 0.0,
        y: 0.0,
        width: 5.0,
        height: 1.0,
        z_order: 0,
        backdrop: false,
    });

    tree.run_layout(40.0, 10.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
        render_overlays(&mut tree, &mut ctx);
    }

    // Overlay overwrites the background text
    assert_eq!(cell_char(&buf, 0, 0), Some('O'));
    assert_eq!(cell_char(&buf, 3, 0), Some('R'));
    // Beyond overlay, background text remains
    assert_eq!(cell_char(&buf, 5, 0), Some('B'));
}

#[test]
fn test_overlay_backdrop_clears() {
    let mut buf = OptimizedBuffer::new(20, 5);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let root = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::column().width(20.0).height(5.0)).background(Rgba::BLACK),
    ));
    let _bg = tree.add_child(
        root,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "XXXXXXXXXXXXXXXXXXXX",
        )),
    );

    let modal = tree.add_child(
        root,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().width(5.0).height(1.0),
            "MODAL",
        )),
    );
    tree.set_visible(modal, false);
    tree.add_overlay(Overlay {
        node: modal,
        x: 5.0,
        y: 2.0,
        width: 5.0,
        height: 1.0,
        z_order: 1000,
        backdrop: true,
    });

    tree.run_layout(20.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
        render_overlays(&mut tree, &mut ctx);
    }

    // Backdrop clears all cells to spaces (not X)
    assert_eq!(cell_char(&buf, 0, 0), Some(' '));
    // Modal renders at (5,2)
    assert_eq!(cell_char(&buf, 5, 2), Some('M'));
}

#[test]
fn test_focus_cycle_with_render() {
    let mut buf = OptimizedBuffer::new(40, 5);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let border_n = Rgba::from_rgb_u8(60, 60, 60);
    let border_f = Rgba::from_rgb_u8(100, 200, 255);

    let root = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::row().width(40.0).height(5.0)).background(Rgba::BLACK),
    ));

    let a = tree.add_child(
        root,
        Box::new(
            BoxWidget::new(LayoutStyle::column().width(20.0))
                .border_rounded(border_n)
                .border_focused_color(border_f)
                .title("A")
                .focusable(),
        ),
    );
    // `.focusable()` on the widget is only propagated to the node by the View
    // rebuild path; for direct RenderTree usage we must mark the node explicitly.
    tree.set_focusable(a, true);
    let _ta = tree.add_child(
        a,
        Box::new(TextWidget::with_text(LayoutStyle::default(), "AA")),
    );

    let b = tree.add_child(
        root,
        Box::new(
            BoxWidget::new(LayoutStyle::column().flex_grow(1.0))
                .border_rounded(border_n)
                .border_focused_color(border_f)
                .title("B")
                .focusable(),
        ),
    );
    tree.set_focusable(b, true);
    let _tb = tree.add_child(
        b,
        Box::new(TextWidget::with_text(LayoutStyle::default(), "BB")),
    );

    // Focus A
    tree.focus_next();
    assert_eq!(tree.focused_node(), Some(a));
    assert!(tree.get(root).is_some_and(|n| n.has_focused_descendant));

    // Focus B
    tree.focus_next();
    assert_eq!(tree.focused_node(), Some(b));

    // Wrap back to A
    tree.focus_next();
    assert_eq!(tree.focused_node(), Some(a));

    // Render succeeds without panic
    tree.run_layout(40.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    // Title "A" renders somewhere in the box
    assert!(cell_char(&buf, 1, 0) == Some('A') || cell_char(&buf, 2, 0) == Some('A'));
    assert_eq!(buf.get(0, 0).unwrap().fg, border_f);
}

#[test]
fn test_box_uses_focused_border_for_focused_descendant() {
    let mut buf = OptimizedBuffer::new(20, 5);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let border_n = Rgba::from_rgb_u8(60, 60, 60);
    let border_f = Rgba::from_rgb_u8(100, 200, 255);

    let root = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::column().width(20.0).height(5.0)).background(Rgba::BLACK),
    ));
    let parent = tree.add_child(
        root,
        Box::new(
            BoxWidget::new(LayoutStyle::column().width(20.0).height(5.0))
                .border_rounded(border_n)
                .border_focused_color(border_f),
        ),
    );
    let child = tree.add_child(
        parent,
        Box::new(BoxWidget::new(LayoutStyle::column().width(10.0).height(2.0)).focusable()),
    );
    tree.set_focusable(child, true);
    tree.focus(child);

    tree.run_layout(20.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(buf.get(0, 0).unwrap().fg, border_f);
}

#[test]
fn test_keybinding_in_render_loop() {
    use opentui_core::input::{KeyCode, KeyEvent, KeyModifiers};
    use opentui_core::keybinding::KeyBindingRegistry;

    let mut reg = KeyBindingRegistry::new();
    reg.bind(KeyModifiers::empty(), KeyCode::Char('q'), "quit");
    reg.bind(KeyModifiers::CTRL, KeyCode::Char('s'), "save");

    let quit_key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
    assert_eq!(reg.resolve(&quit_key), Some("quit"));

    let save_key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CTRL);
    assert_eq!(reg.resolve(&save_key), Some("save"));

    let unknown_key = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty());
    assert_eq!(reg.resolve(&unknown_key), None);
}
