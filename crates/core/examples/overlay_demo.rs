//! Overlay Demo — demonstrates modal dialogs and overlay system.
//!
//! Run: cargo run -p opentui-core --example overlay_demo
//!
//! Keys:
//!   q / Ctrl+C — quit
//!   m — toggle modal overlay
//!   d — toggle dropdown overlay

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
use opentui_core::theme::UiTheme;
use opentui_core::widget::{Overlay, OverlayZOrder, RenderContext, WidgetTree};
use opentui_core::widgets::{BoxWidget, StatusLineWidget, TextWidget};

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
    renderer.set_title("OpenTUI Core — Overlay Demo")?;
    renderer.set_background(Rgba::from_rgb_u8(15, 15, 20));

    let theme = UiTheme::dark_default();
    let mut tree = WidgetTree::new();

    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(w as f32).height(h as f32))
            .background(Rgba::from_rgb_u8(15, 15, 20)),
    );

    let header = tree.add_child(
        root,
        BoxWidget::new(2, LayoutStyle::row().height(1.0).flex_shrink(0.0))
            .background(Rgba::from_rgb_u8(30, 30, 45)),
    );
    let _header_text = tree.add_child(
        header,
        TextWidget::with_text(
            3,
            LayoutStyle::default().flex_grow(1.0),
            " Overlay Demo — m: modal | d: dropdown | q: quit ",
        ),
    );

    let content = tree.add_child(
        root,
        BoxWidget::new(
            4,
            LayoutStyle::column()
                .flex_grow(1.0)
                .padding_x(2.0)
                .padding_y(1.0),
        )
        .background(Rgba::from_rgb_u8(15, 15, 20)),
    );
    let _content_text = tree.add_child(
        content,
        TextWidget::with_text(
            5,
            LayoutStyle::default().flex_grow(1.0),
            "This is the main application content.\n\n\
             Press 'm' to open a modal dialog.\n\
             Press 'd' to open a dropdown overlay.\n\
             Press 'q' to quit.\n\n\
             Overlays render on top of this content\n\
             with optional backdrop dimming.",
        ),
    );

    let _status = tree.add_child(
        root,
        StatusLineWidget::new(50, LayoutStyle::default().height(1.0))
            .left("overlay_demo")
            .center("NORMAL")
            .right("m: modal | d: dropdown"),
    );

    let modal_widget = tree.add(
        BoxWidget::new(
            60,
            LayoutStyle::column()
                .width(40.0)
                .height(8.0)
                .padding(1.0, 2.0, 1.0, 2.0),
        )
        .border_rounded(Rgba::from_rgb_u8(100, 180, 255))
        .title("Confirm")
        .background(Rgba::from_rgb_u8(35, 35, 50)),
    );
    let _modal_text = tree.add_child(
        modal_widget,
        TextWidget::with_text(
            61,
            LayoutStyle::default().flex_grow(1.0),
            "  Are you sure you want to continue?\n\n  Press 'm' or 'q' to close.",
        ),
    );

    let dropdown_widget = tree.add(
        BoxWidget::new(
            70,
            LayoutStyle::column()
                .width(20.0)
                .height(6.0)
                .padding(1.0, 2.0, 1.0, 2.0),
        )
        .border_rounded(Rgba::from_rgb_u8(200, 160, 60))
        .title("Menu")
        .background(Rgba::from_rgb_u8(40, 38, 28)),
    );
    let _dropdown_text = tree.add_child(
        dropdown_widget,
        TextWidget::with_text(
            71,
            LayoutStyle::default().flex_grow(1.0),
            "  New File\n  Open File\n  Save\n  Close",
        ),
    );

    let mut modal_visible = false;
    let mut dropdown_visible = false;

    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut running = true;

    while running {
        if modal_visible {
            let mw = 42.0_f32;
            let mh = 10.0_f32;
            let mx = (w as f32 - mw) / 2.0;
            let my = (h as f32 - mh) / 2.0;
            if !tree.has_overlay(modal_widget) {
                tree.add_overlay(
                    Overlay::new(modal_widget, mx, my, mw, mh)
                        .z_order(OverlayZOrder::MODAL)
                        .backdrop(true),
                );
            }
        } else {
            tree.remove_overlay(modal_widget);
        }

        if dropdown_visible {
            let dw = 22.0_f32;
            let dh = 8.0_f32;
            if !tree.has_overlay(dropdown_widget) {
                tree.add_overlay(
                    Overlay::new(dropdown_widget, 5.0, 4.0, dw, dh).z_order(OverlayZOrder::TOOLTIP),
                );
            }
        } else {
            tree.remove_overlay(dropdown_widget);
        }

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
                    if key.is_ctrl_c() {
                        running = false;
                        break;
                    }
                    match key.code {
                        KeyCode::Char('q') => {
                            if modal_visible {
                                modal_visible = false;
                            } else {
                                running = false;
                            }
                        }
                        KeyCode::Char('m') => {
                            modal_visible = !modal_visible;
                            if modal_visible {
                                dropdown_visible = false;
                            }
                        }
                        KeyCode::Char('d') => {
                            dropdown_visible = !dropdown_visible;
                            if dropdown_visible {
                                modal_visible = false;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
