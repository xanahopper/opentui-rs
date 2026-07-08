//! Integration tests for full tree → layout → render pipeline.

#![allow(clippy::float_cmp)]

use opentui_core::buffer::TitleAlign;
use opentui_core::renderable::context::RenderContext;
use opentui_core::renderable::layout::ComputedLayout;
use opentui_core::renderable::node::NodeId;
use opentui_core::renderable::tree::{Overlay, RenderTree};
use opentui_core::widgets::{
    BadgeStyle, BadgeWidget, BoxWidget, CheckboxWidget, ProgressBarWidget, ScrollBarWidget,
    ScrollViewWidget, SelectItem, SelectWidget, SliderWidget, SpinnerFrames, SpinnerWidget,
    StatusLineWidget, Tab, TabsWidget, TextLineWidget, TextWidget,
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
fn test_slider_clamps_value_and_viewport() {
    let mut slider = SliderWidget::horizontal(LayoutStyle::default().width(20.0).height(1.0))
        .range(0.0, 100.0)
        .value(150.0)
        .viewport_size(150.0);

    assert_eq!(slider.value_value(), 100.0);
    assert_eq!(slider.viewport_size_value(), 100.0);

    slider.set_value(-10.0);
    assert_eq!(slider.value_value(), 0.0);

    slider.set_min(20.0);
    assert_eq!(slider.value_value(), 20.0);

    slider.set_max(80.0);
    slider.set_value(90.0);
    assert_eq!(slider.value_value(), 80.0);
}

#[test]
fn test_slider_renders_smooth_horizontal_thumb() {
    let mut buf = OptimizedBuffer::new(20, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _slider = tree.set_root(Box::new(
        SliderWidget::horizontal(LayoutStyle::default().width(20.0).height(1.0))
            .range(0.0, 100.0)
            .value(50.0)
            .viewport_size(10.0),
    ));

    tree.run_layout(20.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 9, 0), Some('▐'));
    assert_eq!(cell_char(&buf, 10, 0), Some('█'));
}

#[test]
fn test_slider_keyboard_updates_focused_value() {
    let mut tree = RenderTree::new();
    let slider = tree.set_root(Box::new(
        SliderWidget::horizontal(LayoutStyle::default().width(20.0).height(1.0))
            .range(0.0, 100.0)
            .value(50.0),
    ));
    tree.focus(slider);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Right,)));
    let node = tree.get(slider).unwrap();
    let slider_widget = node
        .behavior
        .as_any()
        .downcast_ref::<SliderWidget>()
        .unwrap();
    assert_eq!(slider_widget.value_value(), 51.0);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Home,)));
    let node = tree.get(slider).unwrap();
    let slider_widget = node
        .behavior
        .as_any()
        .downcast_ref::<SliderWidget>()
        .unwrap();
    assert_eq!(slider_widget.value_value(), 0.0);
}

#[test]
fn test_slider_mouse_press_updates_from_position() {
    let mut buf = OptimizedBuffer::new(20, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();
    let slider = tree.set_root(Box::new(
        SliderWidget::horizontal(LayoutStyle::default().width(20.0).height(1.0))
            .range(0.0, 100.0)
            .value(0.0),
    ));

    tree.run_layout(20.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert!(tree.dispatch_mouse_to(
        slider,
        &opentui_core::terminal::MouseEvent::press(
            10,
            0,
            opentui_core::terminal::MouseButton::Left,
        ),
    ));
    let node = tree.get(slider).unwrap();
    let slider_widget = node
        .behavior
        .as_any()
        .downcast_ref::<SliderWidget>()
        .unwrap();
    assert_eq!(slider_widget.value_value(), 50.0);
}

#[test]
fn test_box_title_color_and_bottom_title_render() {
    let mut buf = OptimizedBuffer::new(20, 4);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();
    let title_color = Rgba::from_rgb_u8(255, 180, 80);

    let _root = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::column().width(20.0).height(4.0))
            .border_rounded(Rgba::from_rgb_u8(60, 60, 60))
            .title("Top")
            .bottom_title("Bottom")
            .title_align(TitleAlign::Left)
            .title_color(title_color),
    ));

    tree.run_layout(20.0, 4.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 2, 0), Some('T'));
    assert_eq!(buf.get(2, 0).unwrap().fg, title_color);
    assert_eq!(cell_char(&buf, 2, 3), Some('B'));
    assert_eq!(buf.get(2, 3).unwrap().fg, title_color);
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
fn test_scrollbar_clamps_scroll_position() {
    let mut scrollbar = ScrollBarWidget::vertical(LayoutStyle::default().width(1.0).height(10.0))
        .scroll_size(100.0)
        .viewport_size(20.0)
        .scroll_position(200.0);

    assert_eq!(scrollbar.scroll_position_value(), 80.0);

    scrollbar.set_viewport_size(90.0);
    assert_eq!(scrollbar.scroll_position_value(), 10.0);

    scrollbar.set_scroll_position(-10.0);
    assert_eq!(scrollbar.scroll_position_value(), 0.0);
}

#[test]
fn test_scrollbar_renders_vertical_thumb() {
    let mut buf = OptimizedBuffer::new(1, 10);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _scrollbar = tree.set_root(Box::new(
        ScrollBarWidget::vertical(LayoutStyle::default().width(1.0).height(10.0))
            .scroll_size(100.0)
            .viewport_size(20.0)
            .scroll_position(40.0),
    ));

    tree.run_layout(1.0, 10.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 4), Some('█'));
    assert_eq!(cell_char(&buf, 0, 5), Some('█'));
}

#[test]
fn test_scrollbar_keyboard_updates_focused_position() {
    let mut tree = RenderTree::new();
    let scrollbar = tree.set_root(Box::new(
        ScrollBarWidget::vertical(LayoutStyle::default().width(1.0).height(10.0))
            .scroll_size(100.0)
            .viewport_size(20.0),
    ));
    tree.focus(scrollbar);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Down,)));
    let node = tree.get(scrollbar).unwrap();
    let scrollbar_widget = node
        .behavior
        .as_any()
        .downcast_ref::<ScrollBarWidget>()
        .unwrap();
    assert_eq!(scrollbar_widget.scroll_position_value(), 4.0);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::End,)));
    let node = tree.get(scrollbar).unwrap();
    let scrollbar_widget = node
        .behavior
        .as_any()
        .downcast_ref::<ScrollBarWidget>()
        .unwrap();
    assert_eq!(scrollbar_widget.scroll_position_value(), 80.0);
}

#[test]
fn test_scrollbar_mouse_track_and_arrows_update_position() {
    let mut buf = OptimizedBuffer::new(1, 10);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();
    let scrollbar = tree.set_root(Box::new(
        ScrollBarWidget::vertical(LayoutStyle::default().width(1.0).height(10.0))
            .scroll_size(100.0)
            .viewport_size(20.0)
            .show_arrows(true),
    ));

    tree.run_layout(1.0, 10.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }
    assert_eq!(cell_char(&buf, 0, 0), Some('▲'));
    assert_eq!(cell_char(&buf, 0, 9), Some('▼'));

    assert!(
        tree.dispatch_mouse_to(
            scrollbar,
            &opentui_core::terminal::MouseEvent::press(
                0,
                5,
                opentui_core::terminal::MouseButton::Left,
            ),
        )
    );
    let node = tree.get(scrollbar).unwrap();
    let scrollbar_widget = node
        .behavior
        .as_any()
        .downcast_ref::<ScrollBarWidget>()
        .unwrap();
    assert_eq!(scrollbar_widget.scroll_position_value(), 40.0);

    assert!(
        tree.dispatch_mouse_to(
            scrollbar,
            &opentui_core::terminal::MouseEvent::press(
                0,
                9,
                opentui_core::terminal::MouseButton::Left,
            ),
        )
    );
    let node = tree.get(scrollbar).unwrap();
    let scrollbar_widget = node
        .behavior
        .as_any()
        .downcast_ref::<ScrollBarWidget>()
        .unwrap();
    assert_eq!(scrollbar_widget.scroll_position_value(), 50.0);
}

#[test]
fn test_spinner_renders_current_frame() {
    let mut buf = OptimizedBuffer::new(20, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _spinner = tree.set_root(Box::new(
        SpinnerWidget::new(LayoutStyle::default().width(20.0).height(1.0)).label("Loading..."),
    ));

    tree.run_layout(20.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    let ch = buf.get(0, 0).and_then(|c| c.content.as_char());
    assert!(ch.is_some());
    assert_eq!(cell_char(&buf, 2, 0), Some('L'));
    assert_eq!(cell_char(&buf, 3, 0), Some('o'));
}

#[test]
fn test_spinner_advances_on_update() {
    let mut tree = RenderTree::new();
    let spinner = tree.set_root(Box::new(
        SpinnerWidget::new(LayoutStyle::default().width(10.0).height(1.0))
            .frames(SpinnerFrames::line()),
    ));

    tree.run_layout(10.0, 1.0);

    let node = tree.get(spinner).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SpinnerWidget>()
        .unwrap();
    let initial = sw.current_char();

    tree.set_live(spinner, true);
    tree.run_render(
        &mut RenderContext {
            buffer: &mut OptimizedBuffer::new(10, 1),
            grapheme_pool: None,
            link_pool: None,
            hit_grid: None,
            theme: None,
        },
        0.13,
    );

    let node = tree.get(spinner).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SpinnerWidget>()
        .unwrap();
    assert_ne!(sw.current_char(), initial);
}

#[test]
fn test_spinner_stops_when_not_running() {
    let mut tree = RenderTree::new();
    let spinner = tree.set_root(Box::new(
        SpinnerWidget::new(LayoutStyle::default().width(10.0).height(1.0))
            .frames(SpinnerFrames::line())
            .running(false),
    ));

    tree.run_layout(10.0, 1.0);

    let node = tree.get(spinner).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SpinnerWidget>()
        .unwrap();
    let before = sw.current_char();

    tree.set_live(spinner, true);
    tree.run_render(
        &mut RenderContext {
            buffer: &mut OptimizedBuffer::new(10, 1),
            grapheme_pool: None,
            link_pool: None,
            hit_grid: None,
            theme: None,
        },
        1.0,
    );

    let node = tree.get(spinner).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SpinnerWidget>()
        .unwrap();
    assert_eq!(sw.current_char(), before);
}

#[test]
fn test_badge_renders_padded_text() {
    let mut buf = OptimizedBuffer::new(20, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _badge = tree.set_root(Box::new(
        BadgeWidget::new(LayoutStyle::default().width(20.0).height(1.0), "PASS")
            .badge_style(BadgeStyle::success()),
    ));

    tree.run_layout(20.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some(' '));
    assert_eq!(cell_char(&buf, 1, 0), Some(' '));
    assert_eq!(cell_char(&buf, 2, 0), Some('P'));
    assert_eq!(cell_char(&buf, 3, 0), Some('A'));
    assert_eq!(cell_char(&buf, 5, 0), Some('S'));
    assert_eq!(cell_char(&buf, 6, 0), Some(' '));
    assert_eq!(cell_char(&buf, 7, 0), Some(' '));
}

#[test]
fn test_badge_bracketed_shape() {
    let mut buf = OptimizedBuffer::new(20, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _badge = tree.set_root(Box::new(
        BadgeWidget::new(LayoutStyle::default().width(20.0).height(1.0), "FAIL").badge_style(
            BadgeStyle {
                shape: opentui_core::widgets::BadgeShape::Bracketed,
                ..BadgeStyle::error()
            },
        ),
    ));

    tree.run_layout(20.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('['));
    assert_eq!(cell_char(&buf, 1, 0), Some('F'));
    assert_eq!(cell_char(&buf, 5, 0), Some(']'));
}

#[test]
fn test_checkbox_renders_checked_and_unchecked() {
    let mut buf = OptimizedBuffer::new(20, 2);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _cb = tree.set_root(Box::new(
        CheckboxWidget::new(LayoutStyle::default().width(20.0).height(2.0))
            .checked(true)
            .label("Enable notifications"),
    ));

    tree.run_layout(20.0, 2.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('['));
    assert_eq!(cell_char(&buf, 1, 0), Some('x'));
    assert_eq!(cell_char(&buf, 2, 0), Some(']'));
    assert_eq!(cell_char(&buf, 4, 0), Some('E'));
}

#[test]
fn test_checkbox_keyboard_toggles() {
    let mut tree = RenderTree::new();
    let cb = tree.set_root(Box::new(
        CheckboxWidget::new(LayoutStyle::default().width(20.0).height(1.0)).label("Test"),
    ));
    tree.focus(cb);

    let node = tree.get(cb).unwrap();
    let cw = node
        .behavior
        .as_any()
        .downcast_ref::<CheckboxWidget>()
        .unwrap();
    assert!(!cw.is_checked());

    assert!(
        tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Char(
            ' '
        ),))
    );
    let node = tree.get(cb).unwrap();
    let cw = node
        .behavior
        .as_any()
        .downcast_ref::<CheckboxWidget>()
        .unwrap();
    assert!(cw.is_checked());

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Enter,)));
    let node = tree.get(cb).unwrap();
    let cw = node
        .behavior
        .as_any()
        .downcast_ref::<CheckboxWidget>()
        .unwrap();
    assert!(!cw.is_checked());
}

#[test]
fn test_checkbox_mouse_click_toggles() {
    let mut buf = OptimizedBuffer::new(20, 1);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();
    let cb = tree.set_root(Box::new(
        CheckboxWidget::new(LayoutStyle::default().width(20.0).height(1.0)).label("Click me"),
    ));

    tree.run_layout(20.0, 1.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    let node = tree.get(cb).unwrap();
    let cw = node
        .behavior
        .as_any()
        .downcast_ref::<CheckboxWidget>()
        .unwrap();
    assert!(!cw.is_checked());

    assert!(
        tree.dispatch_mouse_to(
            cb,
            &opentui_core::terminal::MouseEvent::press(
                1,
                0,
                opentui_core::terminal::MouseButton::Left,
            ),
        )
    );
    let node = tree.get(cb).unwrap();
    let cw = node
        .behavior
        .as_any()
        .downcast_ref::<CheckboxWidget>()
        .unwrap();
    assert!(cw.is_checked());

    assert!(tree.dispatch_mouse_to(
        cb,
        &opentui_core::terminal::MouseEvent::press(
            10,
            0,
            opentui_core::terminal::MouseButton::Left,
        ),
    ));
    let node = tree.get(cb).unwrap();
    let cw = node
        .behavior
        .as_any()
        .downcast_ref::<CheckboxWidget>()
        .unwrap();
    assert!(!cw.is_checked());
}

#[test]
fn test_select_renders_items_and_selection() {
    let mut buf = OptimizedBuffer::new(20, 5);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let _select = tree.set_root(Box::new(
        SelectWidget::new(LayoutStyle::default().width(20.0).height(5.0)).items(vec![
            SelectItem::new("Apple"),
            SelectItem::new("Banana"),
            SelectItem::new("Cherry"),
        ]),
    ));
    tree.focus(tree.root().unwrap());

    tree.run_layout(20.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 0, 0), Some('\u{25B8}'));
    assert_eq!(cell_char(&buf, 2, 0), Some('A'));
    assert_eq!(cell_char(&buf, 3, 0), Some('p'));
    assert_eq!(cell_char(&buf, 0, 1), Some(' '));
    assert_eq!(cell_char(&buf, 2, 1), Some('B'));
}

#[test]
fn test_select_keyboard_navigation() {
    let mut tree = RenderTree::new();
    let select = tree.set_root(Box::new(
        SelectWidget::new(LayoutStyle::default().width(20.0).height(5.0)).items(vec![
            SelectItem::new("One"),
            SelectItem::new("Two"),
            SelectItem::new("Three"),
        ]),
    ));
    tree.focus(select);

    let node = tree.get(select).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SelectWidget>()
        .unwrap();
    assert_eq!(sw.selected_index(), 0);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Down)));
    let node = tree.get(select).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SelectWidget>()
        .unwrap();
    assert_eq!(sw.selected_index(), 1);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Down,)));
    let node = tree.get(select).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SelectWidget>()
        .unwrap();
    assert_eq!(sw.selected_index(), 2);

    assert!(!tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Down,)));

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Home)));
    let node = tree.get(select).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SelectWidget>()
        .unwrap();
    assert_eq!(sw.selected_index(), 0);
}

#[test]
fn test_select_wrap_around() {
    let mut tree = RenderTree::new();
    let select = tree.set_root(Box::new(
        SelectWidget::new(LayoutStyle::default().width(20.0).height(5.0))
            .items(vec![
                SelectItem::new("A"),
                SelectItem::new("B"),
                SelectItem::new("C"),
            ])
            .wrap_selection(true),
    ));
    tree.focus(select);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Up)));
    let node = tree.get(select).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SelectWidget>()
        .unwrap();
    assert_eq!(sw.selected_index(), 2);

    assert!(tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Down)));
    let node = tree.get(select).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SelectWidget>()
        .unwrap();
    assert_eq!(sw.selected_index(), 0);
}

#[test]
fn test_select_mouse_click_selects_item() {
    let mut buf = OptimizedBuffer::new(20, 5);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();
    let select = tree.set_root(Box::new(
        SelectWidget::new(LayoutStyle::default().width(20.0).height(5.0)).items(vec![
            SelectItem::new("Alpha"),
            SelectItem::new("Beta"),
            SelectItem::new("Gamma"),
        ]),
    ));

    tree.run_layout(20.0, 5.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert!(
        tree.dispatch_mouse_to(
            select,
            &opentui_core::terminal::MouseEvent::press(
                2,
                2,
                opentui_core::terminal::MouseButton::Left,
            ),
        )
    );
    let node = tree.get(select).unwrap();
    let sw = node
        .behavior
        .as_any()
        .downcast_ref::<SelectWidget>()
        .unwrap();
    assert_eq!(sw.selected_index(), 2);
}

#[test]
fn test_select_scrolls_to_keep_selected_visible() {
    let mut buf = OptimizedBuffer::new(10, 2);
    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();
    let select = tree.set_root(Box::new(
        SelectWidget::new(LayoutStyle::default().width(10.0).height(2.0)).items(vec![
            SelectItem::new("0"),
            SelectItem::new("1"),
            SelectItem::new("2"),
            SelectItem::new("3"),
            SelectItem::new("4"),
        ]),
    ));
    tree.focus(select);

    tree.run_layout(10.0, 2.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }
    assert_eq!(cell_char(&buf, 2, 0), Some('0'));

    for _ in 0..4 {
        tree.dispatch_key(&opentui_core::KeyEvent::key(opentui_core::KeyCode::Down));
    }

    tree.run_layout(10.0, 2.0);
    {
        let mut ctx = make_ctx(&mut buf, &theme);
        tree.run_render(&mut ctx, 0.0);
    }

    assert_eq!(cell_char(&buf, 2, 1), Some('4'));
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
