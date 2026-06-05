//! Core Dashboard Example
//!
//! Demonstrates the opentui-core crate with:
//! - `BoxWidget` containers with borders and titles
//! - `TextWidget` for displaying text
//! - `ProgressBarWidget` for progress indicators
//! - `StatusLineWidget` for status bar
//! - Flexbox layout via Taffy
//! - Focus management with Tab/Shift+Tab
//! - Theme-aware rendering
//!
//! Run: cargo run -p opentui-core --example core_dashboard

#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]
#![allow(unsafe_code)]
#![allow(clippy::borrow_as_ptr)]

use std::io::{self, Read};
use std::time::Duration;

use opentui_rust::input::{Event, InputParser, KeyCode};
use opentui_rust::terminal::{enable_raw_mode, terminal_size};
use opentui_rust::{Renderer, Rgba};

use opentui_core::layout::LayoutStyle;
use opentui_core::theme::UiTheme;
use opentui_core::widget::{RenderContext, WidgetTree};
use opentui_core::widgets::{
    BoxWidget, ProgressBarStyle, ProgressBarWidget, ProgressChars, StatusLineWidget, TextWidget,
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

    let mut renderer = Renderer::new(w, h)?;
    let _raw_guard = enable_raw_mode()?;

    let theme = UiTheme::dark_default();

    let mut tree = WidgetTree::new();

    // Root: full screen, column layout
    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(w as f32).height(h as f32))
            .background(theme.background),
    );

    // Header: 1 row
    let header = tree.add_child(
        root,
        BoxWidget::new(2, LayoutStyle::row().height(1.0).flex_shrink(0.0))
            .background(theme.background_panel),
    );
    let _header_title = tree.add_child(
        header,
        TextWidget::with_text(
            3,
            LayoutStyle::default().flex_grow(1.0),
            " OpenTUI Core Dashboard ",
        ),
    );

    // Body: fills remaining space, row layout
    let body = tree.add_child(
        root,
        BoxWidget::new(4, LayoutStyle::row().flex_grow(1.0)).background(theme.background),
    );

    // Left panel: file list
    let left = tree.add_child(
        body,
        BoxWidget::new(
            5,
            LayoutStyle::column().width_percent(40.0).flex_shrink(0.0),
        )
        .border_rounded(theme.border)
        .border_focused_color(theme.border_active)
        .title("Files")
        .background(theme.background_panel)
        .focusable(),
    );
    let _file_list = tree.add_child(
        left,
        TextWidget::with_text(
            6,
            LayoutStyle::default().flex_grow(1.0),
            "  Cargo.toml\n  src/lib.rs\n  src/main.rs\n  README.md\n  benches/buffer.rs\n",
        ),
    );

    // Right panel: editor area + progress
    let right = tree.add_child(
        body,
        BoxWidget::new(7, LayoutStyle::column().flex_grow(1.0))
            .border_rounded(theme.border)
            .border_focused_color(theme.border_active)
            .title("Build Status")
            .background(theme.background_panel)
            .focusable(),
    );

    let _build_text = tree.add_child(
        right,
        TextWidget::with_text(
            8,
            LayoutStyle::default().flex_grow(1.0).height(3.0),
            "  Compiling opentui-core v0.1.0\n  Running tests...",
        ),
    );

    // Progress bars in right panel
    let _compile_pb = tree.add_child(
        right,
        ProgressBarWidget::new(20, LayoutStyle::default().height(1.0).flex_grow(0.0))
            .progress(0.85)
            .label("compile"),
    );

    let _test_pb = tree.add_child(
        right,
        ProgressBarWidget::new(21, LayoutStyle::default().height(1.0).flex_grow(0.0))
            .progress(0.62)
            .bar_style(ProgressBarStyle {
                filled_fg: Rgba::from_rgb_u8(80, 160, 255),
                filled_bg: Rgba::from_rgb_u8(30, 60, 120),
                empty_fg: Rgba::from_rgb_u8(50, 50, 60),
                empty_bg: Rgba::from_rgb_u8(25, 25, 32),
                label_fg: Rgba::from_rgb_u8(200, 220, 255),
                chars: ProgressChars::blocks(),
            })
            .label("tests"),
    );

    let _lint_pb = tree.add_child(
        right,
        ProgressBarWidget::new(22, LayoutStyle::default().height(1.0).flex_grow(0.0))
            .progress(1.0)
            .bar_style(ProgressBarStyle {
                filled_fg: Rgba::from_rgb_u8(100, 220, 120),
                filled_bg: Rgba::from_rgb_u8(30, 80, 40),
                empty_fg: Rgba::from_rgb_u8(50, 50, 60),
                empty_bg: Rgba::from_rgb_u8(25, 25, 32),
                label_fg: Rgba::WHITE,
                chars: ProgressChars::ascii(),
            })
            .label("clippy"),
    );

    // Status line at bottom
    let _status = tree.add_child(
        root,
        StatusLineWidget::new(50, LayoutStyle::default().height(1.0).flex_shrink(0.0))
            .left("core_dashboard")
            .center("NORMAL")
            .right("Tab: focus | q: quit"),
    );

    tree.build_focus_chain();
    tree.set_focused_widget(Some(left));

    let mut input_parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];

    loop {
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
                hovered_id: None,
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
                let Ok((event, used)) = input_parser.parse(&read_buf[offset..n]) else {
                    break;
                };
                offset += used;

                if let Event::Key(key) = event {
                    if key.code == KeyCode::Char('q') || key.is_ctrl_c() {
                        return Ok(());
                    }
                    tree.dispatch_key(&key);
                }
            }
        }
    }
}
