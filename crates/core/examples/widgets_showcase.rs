//! Widgets Showcase — demonstrates all opentui-core widgets in one screen.
//!
//! Run: cargo run -p opentui-core --example widgets_showcase
//!
//! Keys:
//!   q / Ctrl+C — quit
//!   Tab / Shift+Tab — cycle focus
//!   Alt+Left/Right — switch tabs

#![allow(unsafe_code)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]

use std::io::{self, Read};
use std::time::Duration;

use opentui_rust::input::{Event, InputParser, KeyCode};
use opentui_rust::terminal::terminal_size;
use opentui_rust::{Renderer, RendererOptions, Rgba};

use opentui_core::layout::LayoutStyle;
use opentui_core::theme::UiTheme;
use opentui_core::widget::{RenderContext, WidgetTree};
use opentui_core::widgets::{
    BoxWidget, ProgressBarStyle, ProgressBarWidget, ProgressChars, StatusLineWidget, Tab,
    TabsWidget, TextWidget,
};

fn read_with_timeout(stdin: &io::Stdin, buf: &mut [u8], timeout: Duration) -> io::Result<usize> {
    use std::os::unix::io::AsRawFd;

    let fd = stdin.as_raw_fd();
    let mut read_fds = std::mem::MaybeUninit::<libc::fd_set>::uninit();
    unsafe {
        libc::FD_ZERO(read_fds.as_mut_ptr());
        libc::FD_SET(fd, read_fds.as_mut_ptr());
    }
    let mut tv = libc::timeval {
        tv_sec: timeout.as_secs() as libc::time_t,
        tv_usec: timeout.subsec_micros() as libc::suseconds_t,
    };
    let result = unsafe {
        libc::select(
            fd + 1,
            read_fds.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut tv,
        )
    };
    if result > 0 {
        stdin.lock().read(buf)
    } else {
        Ok(0)
    }
}

fn main() -> io::Result<()> {
    let (width, height) = terminal_size().unwrap_or((80, 24));
    let w = u32::from(width);
    let h = u32::from(height);

    let options = RendererOptions {
        use_alt_screen: true,
        hide_cursor: true,
        enable_mouse: true,
        query_capabilities: true,
    };
    let mut renderer = Renderer::new_with_options(w, h, options)?;
    renderer.set_title("OpenTUI Core — Widgets Showcase")?;
    renderer.set_background(Rgba::from_rgb_u8(18, 18, 24));

    let theme = UiTheme::dark_default();
    let mut tree = WidgetTree::new();

    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(w as f32).height(h as f32))
            .background(Rgba::from_rgb_u8(18, 18, 24)),
    );

    let header = tree.add_child(
        root,
        BoxWidget::new(2, LayoutStyle::row().height(1.0).flex_shrink(0.0))
            .background(Rgba::from_rgb_u8(40, 40, 55)),
    );
    let _title = tree.add_child(
        header,
        TextWidget::with_text(
            3,
            LayoutStyle::default().flex_grow(1.0),
            " OpenTUI Core — Widgets Showcase ",
        ),
    );

    let tabs_id = tree.add_child(
        root,
        TabsWidget::new(
            10,
            LayoutStyle::default()
                .height(2.0)
                .width(w as f32)
                .flex_shrink(0.0),
        )
        .tabs(vec![
            Tab::new("Progress"),
            Tab::new("Layout"),
            Tab::new("Info"),
        ])
        .focusable(),
    );

    let content = tree.add_child(
        root,
        BoxWidget::new(4, LayoutStyle::column().flex_grow(1.0))
            .background(Rgba::from_rgb_u8(22, 22, 30)),
    );

    let progress_section = tree.add_child(
        content,
        BoxWidget::new(
            20,
            LayoutStyle::column()
                .flex_grow(1.0)
                .padding_x(2.0)
                .padding_y(1.0),
        )
        .background(Rgba::from_rgb_u8(22, 22, 30)),
    );

    let _pb_label1 = tree.add_child(
        progress_section,
        TextWidget::with_text(21, LayoutStyle::default().height(1.0), "Downloads:"),
    );
    let _pb1 = tree.add_child(
        progress_section,
        ProgressBarWidget::new(22, LayoutStyle::default().height(1.0).width(50.0))
            .progress(0.75)
            .label("opentui-core"),
    );

    let _spacer1 = tree.add_child(
        progress_section,
        TextWidget::with_text(23, LayoutStyle::default().height(1.0), ""),
    );

    let _pb_label2 = tree.add_child(
        progress_section,
        TextWidget::with_text(24, LayoutStyle::default().height(1.0), "Upload:"),
    );
    let _pb2 = tree.add_child(
        progress_section,
        ProgressBarWidget::new(25, LayoutStyle::default().height(1.0).width(50.0))
            .progress(0.33)
            .bar_style(ProgressBarStyle {
                filled_fg: Rgba::from_rgb_u8(80, 160, 255),
                filled_bg: Rgba::from_rgb_u8(30, 60, 120),
                empty_fg: Rgba::from_rgb_u8(50, 50, 60),
                empty_bg: Rgba::from_rgb_u8(25, 25, 32),
                label_fg: Rgba::from_rgb_u8(200, 220, 255),
                chars: ProgressChars::blocks(),
            }),
    );

    let _spacer2 = tree.add_child(
        progress_section,
        TextWidget::with_text(26, LayoutStyle::default().height(1.0), ""),
    );

    let _pb_label3 = tree.add_child(
        progress_section,
        TextWidget::with_text(27, LayoutStyle::default().height(1.0), "Tests:"),
    );
    let _pb3 = tree.add_child(
        progress_section,
        ProgressBarWidget::new(28, LayoutStyle::default().height(1.0).width(50.0))
            .progress(1.0)
            .bar_style(ProgressBarStyle {
                filled_fg: Rgba::from_rgb_u8(100, 220, 120),
                filled_bg: Rgba::from_rgb_u8(30, 80, 40),
                empty_fg: Rgba::from_rgb_u8(50, 50, 60),
                empty_bg: Rgba::from_rgb_u8(25, 25, 32),
                label_fg: Rgba::WHITE,
                chars: ProgressChars::ascii(),
            })
            .label("passed"),
    );

    let _spacer3 = tree.add_child(
        progress_section,
        TextWidget::with_text(29, LayoutStyle::default().height(1.0), ""),
    );

    let _pb_label4 = tree.add_child(
        progress_section,
        TextWidget::with_text(
            30,
            LayoutStyle::default().height(1.0),
            "Disk Usage (multi-row):",
        ),
    );
    let _pb4 = tree.add_child(
        progress_section,
        ProgressBarWidget::new(31, LayoutStyle::default().height(3.0).width(50.0))
            .progress(0.58)
            .bar_style(ProgressBarStyle {
                filled_fg: Rgba::from_rgb_u8(255, 180, 60),
                filled_bg: Rgba::from_rgb_u8(100, 60, 15),
                empty_fg: Rgba::from_rgb_u8(50, 50, 60),
                empty_bg: Rgba::from_rgb_u8(25, 25, 32),
                label_fg: Rgba::from_rgb_u8(255, 240, 200),
                chars: ProgressChars::smooth(),
            }),
    );

    let _status = tree.add_child(
        root,
        StatusLineWidget::new(50, LayoutStyle::default().height(1.0).width(w as f32))
            .left("widgets_showcase")
            .center("NORMAL")
            .right("Tab: focus | q: quit"),
    );

    tree.build_focus_chain();
    tree.set_focused_widget(Some(tabs_id));

    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut running = true;

    while running {
        tree.layout(w as f32, h as f32);

        {
            let buffer = renderer.buffer();
            buffer.clear(Rgba::TRANSPARENT);

            let mut ctx = RenderContext {
                buffer,
                grapheme_pool: None,
                link_pool: None,
                hit_grid: None,
                theme: Some(&theme),
            };
            tree.render(&mut ctx);
        }
        renderer.present()?;

        if let Ok(n) = read_with_timeout(&stdin, &mut read_buf, Duration::from_millis(50)) {
            if n == 0 {
                continue;
            }
            let mut offset = 0usize;
            while offset < n {
                let Ok((event, used)) = parser.parse(&read_buf[offset..n]) else {
                    break;
                };
                offset += used;

                if let Event::Key(key) = event {
                    if key.code == KeyCode::Char('q') || key.is_ctrl_c() {
                        running = false;
                        break;
                    }
                    tree.dispatch_key(&key);
                }
            }
        }
    }

    Ok(())
}
