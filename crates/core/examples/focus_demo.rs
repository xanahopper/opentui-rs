//! Focus Demo — interactive focus chain navigation with styled containers.
//!
//! Run: cargo run -p opentui-core --example focus_demo
//!
//! Keys:
//!   q / Ctrl+C — quit
//!   Tab — focus next
//!   Shift+Tab — focus prev
//!   1-5 — jump to panel by number

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

use opentui_core::input::{Event, InputParser, KeyCode};
use opentui_core::terminal::{enable_raw_mode, terminal_size};
use opentui_core::{Renderer, RendererOptions, Rgba};

use opentui_core::layout::LayoutStyle;
use opentui_core::prelude::RenderContext;
use opentui_core::theme::UiTheme;
use opentui_core::tree::RenderTree;
use opentui_core::widgets::{BoxWidget, TextWidget};

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
        enable_mouse: false,
        query_capabilities: true,
    };
    let mut renderer = Renderer::new_with_options(w, h, options)?;
    let _raw_guard = enable_raw_mode()?;
    renderer.set_title("OpenTUI Core — Focus Demo")?;
    renderer.set_background(Rgba::from_rgb_u8(15, 15, 20));

    let theme = UiTheme::dark_default();
    let mut tree = RenderTree::new();

    let border_normal = Rgba::from_rgb_u8(60, 60, 75);
    let border_focused = Rgba::from_rgb_u8(100, 180, 255);
    let panel_bg = Rgba::from_rgb_u8(22, 22, 30);
    let p = 1.0_f32;

    let root = tree.set_root(Box::new(
        BoxWidget::new(LayoutStyle::column().width(w as f32).height(h as f32))
            .background(Rgba::from_rgb_u8(15, 15, 20)),
    ));

    let title_bar = tree.add_child(
        root,
        Box::new(
            BoxWidget::new(LayoutStyle::row().height(1.0).flex_shrink(0.0))
                .background(Rgba::from_rgb_u8(30, 30, 45)),
        ),
    );
    let _title_text = tree.add_child(
        title_bar,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            " Focus Demo — Tab / Shift+Tab | 1-5 jump to panel ",
        )),
    );

    let body = tree.add_child(
        root,
        Box::new(
            BoxWidget::new(LayoutStyle::row().flex_grow(1.0))
                .background(Rgba::from_rgb_u8(15, 15, 20)),
        ),
    );

    let left_col = tree.add_child(
        body,
        Box::new(
            BoxWidget::new(
                LayoutStyle::column()
                    .width_percent(50.0)
                    .flex_shrink(0.0)
                    .padding(p, p, p, p),
            )
            .background(Rgba::from_rgb_u8(15, 15, 20)),
        ),
    );

    let p1 = tree.add_child(
        left_col,
        Box::new(
            BoxWidget::new(LayoutStyle::column().flex_grow(1.0).padding(p, p, p, p))
                .border_rounded(border_normal)
                .border_focused_color(border_focused)
                .title("1: Files")
                .background(panel_bg)
                .focusable(),
        ),
    );
    let _p1_text = tree.add_child(
        p1,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "  Cargo.toml\n  src/lib.rs\n  src/widget.rs\n  src/layout.rs",
        )),
    );

    let p2 = tree.add_child(
        left_col,
        Box::new(
            BoxWidget::new(LayoutStyle::column().flex_grow(1.0).padding(p, p, p, p))
                .border_rounded(border_normal)
                .border_focused_color(border_focused)
                .title("2: Git Status")
                .background(panel_bg)
                .focusable(),
        ),
    );
    let _p2_text = tree.add_child(
        p2,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "  M src/widget.rs\n  A crates/core/Cargo.toml\n  ?? examples/",
        )),
    );

    let right_col = tree.add_child(
        body,
        Box::new(
            BoxWidget::new(LayoutStyle::column().flex_grow(1.0).padding(p, p, p, p))
                .background(Rgba::from_rgb_u8(15, 15, 20)),
        ),
    );

    let p3 = tree.add_child(
        right_col,
        Box::new(
            BoxWidget::new(LayoutStyle::column().flex_grow(2.0).padding(p, p, p, p))
                .border_rounded(border_normal)
                .border_focused_color(border_focused)
                .title("3: Editor")
                .background(panel_bg)
                .focusable(),
        ),
    );
    let _p3_text = tree.add_child(
        p3,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "fn main() {\n    println!(\"Hello!\");\n}",
        )),
    );

    let p4 = tree.add_child(
        right_col,
        Box::new(
            BoxWidget::new(LayoutStyle::column().flex_grow(1.0).padding(p, p, p, p))
                .border_rounded(border_normal)
                .border_focused_color(border_focused)
                .title("4: Terminal")
                .background(panel_bg)
                .focusable(),
        ),
    );
    let _p4_text = tree.add_child(
        p4,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "$ cargo test\n  running 19 tests ...\n  test result: ok",
        )),
    );

    let p5 = tree.add_child(
        root,
        Box::new(
            BoxWidget::new(
                LayoutStyle::column()
                    .height(5.0)
                    .flex_shrink(0.0)
                    .padding(p, p, p, p),
            )
            .border_rounded(border_normal)
            .border_focused_color(border_focused)
            .title("5: Problems")
            .background(panel_bg)
            .focusable(),
        ),
    );
    let _p5_text = tree.add_child(
        p5,
        Box::new(TextWidget::with_text(
            LayoutStyle::default().flex_grow(1.0),
            "  No problems detected",
        )),
    );

    tree.focus(p1);

    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut running = true;

    let panels = [p1, p2, p3, p4, p5];

    while running {
        tree.run_layout(w as f32, h as f32);

        for &pid in &panels {
            let is_focused = tree.focused_node() == Some(pid);
            let border_color = if is_focused {
                border_focused
            } else {
                border_normal
            };
            if let Some(node) = tree.get_mut(pid) {
                if let Some(box_w) = node.behavior.as_any_mut().downcast_mut::<BoxWidget>() {
                    box_w.set_border_focused_color(border_color);
                }
            }
        }

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
            tree.run_render(&mut ctx, 0.0);
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

                    if key.modifiers.is_empty() {
                        if let KeyCode::Char(c) = key.code {
                            if let Some(digit) = c.to_digit(10) {
                                let idx = digit as usize;
                                if idx >= 1 && idx <= panels.len() {
                                    tree.focus(panels[idx - 1]);
                                    continue;
                                }
                            }
                        }
                    }

                    tree.dispatch_key(&key);
                }
            }
        }
    }

    Ok(())
}
