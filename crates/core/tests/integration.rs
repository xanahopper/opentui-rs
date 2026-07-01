//! Integration tests for full tree → layout → render pipeline.

#![allow(clippy::float_cmp)]
#![allow(clippy::missing_const_for_fn)]

use opentui_core::{OptimizedBuffer, Rgba};

use opentui_core::layout::LayoutStyle;
use opentui_core::theme::UiTheme;
use opentui_core::widget::{Overlay, OverlayZOrder, RenderContext, WidgetTree};
use opentui_core::widgets::{
    BoxWidget, ProgressBarWidget, StatusLineWidget, Tab, TabsWidget, TextWidget,
};

fn make_ctx<'a>(buf: &'a mut OptimizedBuffer, theme: &'a UiTheme) -> RenderContext<'a> {
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

#[test]
fn test_text_widget_renders() {
    let mut buf = OptimizedBuffer::new(40, 5);
    let theme = UiTheme::dark_default();
    let mut tree = WidgetTree::new();

    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(40.0).height(5.0)).background(Rgba::BLACK),
    );
    let _text = tree.add_child(
        root,
        TextWidget::with_text(2, LayoutStyle::default().flex_grow(1.0), "Hello World"),
    );

    tree.layout(40.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('H'));
    assert_eq!(cell_char(&buf, 1, 0), Some('e'));
    assert_eq!(cell_char(&buf, 4, 0), Some('o'));
}

#[test]
fn test_status_line_renders_segments() {
    let mut buf = OptimizedBuffer::new(40, 1);
    let theme = UiTheme::dark_default();
    let mut tree = WidgetTree::new();

    let _sl = tree.add(
        StatusLineWidget::new(1, LayoutStyle::default().width(40.0).height(1.0))
            .left("LEFT")
            .center("MID")
            .right("RIGHT"),
    );

    tree.layout(40.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('L'));
    // Right segment at end
    assert_eq!(cell_char(&buf, 35, 0), Some('R'));
}

#[test]
fn test_progress_bar_renders_fill() {
    let mut buf = OptimizedBuffer::new(22, 1);
    let theme = UiTheme::dark_default();
    let mut tree = WidgetTree::new();

    let _bar = tree.add(
        ProgressBarWidget::new(1, LayoutStyle::default().width(22.0).height(1.0)).progress(0.5),
    );

    tree.layout(22.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
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
    let mut tree = WidgetTree::new();

    let _tabs = tree.add(
        TabsWidget::new(1, LayoutStyle::default().width(40.0).height(3.0))
            .tabs(vec![Tab::new("File"), Tab::new("Edit")]),
    );

    tree.layout(40.0, 3.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
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
    let mut tree = WidgetTree::new();

    let root = tree
        .add(BoxWidget::new(1, LayoutStyle::row().width(40.0).height(5.0)).background(Rgba::BLACK));

    let left = tree.add_child(
        root,
        BoxWidget::new(2, LayoutStyle::column().width(20.0).height(5.0)).background(Rgba::BLACK),
    );
    let _left_text = tree.add_child(
        left,
        TextWidget::with_text(3, LayoutStyle::default().flex_grow(1.0), "L"),
    );

    let right = tree.add_child(
        root,
        BoxWidget::new(4, LayoutStyle::column().flex_grow(1.0)).background(Rgba::BLACK),
    );
    let _right_text = tree.add_child(
        right,
        TextWidget::with_text(5, LayoutStyle::default().flex_grow(1.0), "R"),
    );

    tree.layout(40.0, 5.0);

    let left_layout = tree.computed_layout(left).unwrap();
    let right_layout = tree.computed_layout(right).unwrap();
    assert_eq!(left_layout.width, 20.0);
    assert_eq!(right_layout.x, 20.0);
    assert_eq!(right_layout.width, 20.0);

    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('L'));
    assert_eq!(cell_char(&buf, 20, 0), Some('R'));
}

#[test]
fn test_overlay_renders_on_top() {
    let mut buf = OptimizedBuffer::new(40, 10);
    let theme = UiTheme::dark_default();
    let mut tree = WidgetTree::new();

    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(40.0).height(10.0)).background(Rgba::BLACK),
    );
    let _bg = tree.add_child(
        root,
        TextWidget::with_text(2, LayoutStyle::default().flex_grow(1.0), "BBBBBBBBBB"),
    );

    let overlay_widget = tree.add(TextWidget::with_text(
        3,
        LayoutStyle::default().width(5.0).height(1.0),
        "OVER",
    ));
    tree.add_overlay(Overlay::new(overlay_widget, 0.0, 0.0, 5.0, 1.0));

    tree.layout(40.0, 10.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
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
    let mut tree = WidgetTree::new();

    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(20.0).height(5.0)).background(Rgba::BLACK),
    );
    let _bg = tree.add_child(
        root,
        TextWidget::with_text(
            2,
            LayoutStyle::default().flex_grow(1.0),
            "XXXXXXXXXXXXXXXXXXXX",
        ),
    );

    let modal = tree.add(TextWidget::with_text(
        3,
        LayoutStyle::default().width(5.0).height(1.0),
        "MODAL",
    ));
    tree.add_overlay(
        Overlay::new(modal, 5.0, 2.0, 5.0, 1.0)
            .z_order(OverlayZOrder::MODAL)
            .backdrop(true),
    );

    tree.layout(20.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
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
    let mut tree = WidgetTree::new();

    let border_n = Rgba::from_rgb_u8(60, 60, 60);
    let border_f = Rgba::from_rgb_u8(100, 200, 255);

    let root = tree
        .add(BoxWidget::new(1, LayoutStyle::row().width(40.0).height(5.0)).background(Rgba::BLACK));

    let a = tree.add_child(
        root,
        BoxWidget::new(2, LayoutStyle::column().width(20.0))
            .border_rounded(border_n)
            .border_focused_color(border_f)
            .title("A")
            .focusable(),
    );
    let _ta = tree.add_child(a, TextWidget::with_text(3, LayoutStyle::default(), "AA"));

    let b = tree.add_child(
        root,
        BoxWidget::new(4, LayoutStyle::column().flex_grow(1.0))
            .border_rounded(border_n)
            .border_focused_color(border_f)
            .title("B")
            .focusable(),
    );
    let _tb = tree.add_child(b, TextWidget::with_text(5, LayoutStyle::default(), "BB"));

    tree.build_focus_chain();

    // Focus A
    tree.focus_next();
    assert_eq!(tree.focused_id(), Some(a));
    assert!(tree.has_focused_descendant(root));

    // Focus B
    tree.focus_next();
    assert_eq!(tree.focused_id(), Some(b));

    // Wrap back to A
    tree.focus_next();
    assert_eq!(tree.focused_id(), Some(a));

    // Render succeeds without panic
    tree.layout(40.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.render(&mut ctx);
    }

    // Title "A" renders somewhere in the box
    assert!(cell_char(&buf, 1, 0) == Some('A') || cell_char(&buf, 2, 0) == Some('A'));
}

#[test]
fn test_keybinding_in_render_loop() {
    use opentui_core::keybinding::KeyBindingRegistry;
    use opentui_core::input::{KeyCode, KeyEvent, KeyModifiers};

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
