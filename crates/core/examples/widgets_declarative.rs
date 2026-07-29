//! Widget Showcase — declarative API demo of all Phase 3 widgets.
//!
//! Run: cargo run -p opentui-core --example widgets_declarative
//!
//! Demonstrates: Slider, ScrollBar, Select, Checkbox, Spinner, Badge,
//! RadioGroup, Gauge — all built via the declarative View API.

#![allow(unsafe_code)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::missing_const_for_fn)]

use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opentui_core::input::{Event, InputParser, KeyCode, ParseError};
use opentui_core::prelude::*;
use opentui_core::terminal::{enable_raw_mode, terminal_size};
use opentui_core::view::{
    ViewRuntime, badge, checkbox, gauge, radio_group, select, slider, spinner, text, view,
};
use opentui_core::{Renderer, RendererOptions, Rgba};

const BG: Rgba = Rgba::new(0.05, 0.05, 0.07, 1.0);
const BG_PANEL: Rgba = Rgba::new(0.08, 0.08, 0.11, 1.0);
const TEXT: Rgba = Rgba::new(0.88, 0.88, 0.92, 1.0);
const TEXT_MUTED: Rgba = Rgba::new(0.50, 0.50, 0.55, 1.0);
const PRIMARY: Rgba = Rgba::new(0.29, 0.55, 0.90, 1.0);
const SUCCESS: Rgba = Rgba::new(0.35, 0.80, 0.50, 1.0);
const WARNING: Rgba = Rgba::new(0.78, 0.63, 0.16, 1.0);
const ERROR: Rgba = Rgba::new(0.78, 0.31, 0.31, 1.0);

struct App {
    slider_value: f32,
    gauge_value: f32,
    checkbox_checked: bool,
    selected_item: usize,
    radio_selected: usize,
    spinner_running: bool,
}

impl App {
    fn new() -> Self {
        Self {
            slider_value: 50.0,
            gauge_value: 65.0,
            checkbox_checked: true,
            selected_item: 1,
            radio_selected: 0,
            spinner_running: true,
        }
    }
}

fn ui_left_panel(app: &App) -> opentui_core::view::Node {
    view()
        .column()
        .grow(1.0)
        .padding(1.0, 2.0, 1.0, 2.0)
        .gap(1.0)
        .bg(BG_PANEL)
        .children([
            text("Controls")
                .fg(TEXT)
                .bg(BG_PANEL)
                .bold()
                .height(1.0)
                .build(),
            // ── Slider ──────────────────────────────────────
            view()
                .column()
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([
                    text("Volume")
                        .fg(TEXT_MUTED)
                        .bg(BG_PANEL)
                        .height(1.0)
                        .build(),
                    view()
                        .row()
                        .height(1.0)
                        .shrink(0.0)
                        .bg(BG_PANEL)
                        .children([slider()
                            .value(app.slider_value)
                            .range(0.0, 100.0)
                            .grow(1.0)
                            .height(1.0)
                            .build()])
                        .build(),
                ])
                .build(),
            // ── Gauge ───────────────────────────────────────
            view()
                .column()
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([
                    text("CPU Load")
                        .fg(TEXT_MUTED)
                        .bg(BG_PANEL)
                        .height(1.0)
                        .build(),
                    view()
                        .row()
                        .height(1.0)
                        .shrink(0.0)
                        .bg(BG_PANEL)
                        .children([gauge()
                            .value(app.gauge_value)
                            .range(0.0, 100.0)
                            .segments(15)
                            .show_label()
                            .grow(1.0)
                            .height(1.0)
                            .build()])
                        .build(),
                ])
                .build(),
            // ── Checkbox ────────────────────────────────────
            view()
                .column()
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([
                    checkbox("Enable notifications")
                        .checked(app.checkbox_checked)
                        .height(1.0)
                        .shrink(0.0)
                        .build(),
                    checkbox("Dark mode")
                        .checked(false)
                        .height(1.0)
                        .shrink(0.0)
                        .build(),
                    checkbox("Auto-save")
                        .checked(true)
                        .height(1.0)
                        .shrink(0.0)
                        .build(),
                ])
                .build(),
            // ── Spinner ─────────────────────────────────────
            view()
                .row()
                .height(1.0)
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([spinner()
                    .label("Processing...")
                    .running(app.spinner_running)
                    .grow(1.0)
                    .height(1.0)
                    .build()])
                .build(),
        ])
        .build()
}

fn ui_right_panel(app: &App) -> opentui_core::view::Node {
    view()
        .column()
        .grow(1.0)
        .padding(1.0, 2.0, 1.0, 2.0)
        .gap(1.0)
        .bg(BG_PANEL)
        .children([
            text("Selections")
                .fg(TEXT)
                .bg(BG_PANEL)
                .bold()
                .height(1.0)
                .build(),
            // ── Select ──────────────────────────────────────
            view()
                .column()
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([
                    text("Theme")
                        .fg(TEXT_MUTED)
                        .bg(BG_PANEL)
                        .height(1.0)
                        .build(),
                    select(vec![
                        "Dracula".into(),
                        "Nord".into(),
                        "Solarized".into(),
                        "Monokai".into(),
                        "Gruvbox".into(),
                    ])
                    .selected(app.selected_item)
                    .wrap()
                    .height(5.0)
                    .shrink(0.0)
                    .build(),
                ])
                .build(),
            // ── RadioGroup ──────────────────────────────────
            view()
                .column()
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([
                    text("Editor")
                        .fg(TEXT_MUTED)
                        .bg(BG_PANEL)
                        .height(1.0)
                        .build(),
                    radio_group(vec!["VS Code".into(), "Neovim".into(), "Emacs".into()])
                        .selected(app.radio_selected)
                        .height(3.0)
                        .shrink(0.0)
                        .build(),
                ])
                .build(),
            // ── Badges ──────────────────────────────────────
            view()
                .row()
                .height(1.0)
                .shrink(0.0)
                .gap(1.0)
                .bg(BG_PANEL)
                .children([
                    badge("PASS").height(1.0).width(10.0).shrink(0.0).build(),
                    badge("WARN")
                        .fg(Rgba::BLACK)
                        .bg(WARNING)
                        .height(1.0)
                        .width(10.0)
                        .shrink(0.0)
                        .build(),
                    badge("FAIL")
                        .fg(TEXT)
                        .bg(ERROR)
                        .height(1.0)
                        .width(10.0)
                        .shrink(0.0)
                        .build(),
                ])
                .build(),
            // ── ScrollBar ───────────────────────────────────
            view()
                .column()
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([
                    text("File Explorer")
                        .fg(TEXT_MUTED)
                        .bg(BG_PANEL)
                        .height(1.0)
                        .build(),
                    view()
                        .column()
                        .height(8.0)
                        .shrink(0.0)
                        .bg(BG)
                        .padding(0.0, 1.0, 0.0, 1.0)
                        .children([
                            text("src/main.rs").fg(TEXT).bg(BG).height(1.0).build(),
                            text("src/lib.rs").fg(TEXT).bg(BG).height(1.0).build(),
                            text("src/app.rs").fg(TEXT).bg(BG).height(1.0).build(),
                            text("src/ui.rs").fg(TEXT).bg(BG).height(1.0).build(),
                            text("src/widgets.rs").fg(TEXT).bg(BG).height(1.0).build(),
                            text("src/render.rs").fg(TEXT).bg(BG).height(1.0).build(),
                            text("src/input.rs").fg(TEXT).bg(BG).height(1.0).build(),
                            text("src/buffer.rs").fg(TEXT).bg(BG).height(1.0).build(),
                        ])
                        .build(),
                ])
                .build(),
        ])
        .build()
}

fn ui(app: &App, _w: u32, _h: u32) -> opentui_core::view::Node {
    let status_items = vec![
        badge("Showcase")
            .fg(TEXT)
            .bg(PRIMARY)
            .height(1.0)
            .width(12.0)
            .shrink(0.0)
            .build(),
        text(format!("slider={:.0}", app.slider_value))
            .fg(TEXT_MUTED)
            .bg(BG)
            .height(1.0)
            .shrink(0.0)
            .build(),
        text(format!("gauge={:.0}", app.gauge_value))
            .fg(TEXT_MUTED)
            .bg(BG)
            .height(1.0)
            .shrink(0.0)
            .build(),
        text(format!("check={}", app.checkbox_checked))
            .fg(if app.checkbox_checked {
                SUCCESS
            } else {
                TEXT_MUTED
            })
            .bg(BG)
            .height(1.0)
            .shrink(0.0)
            .build(),
    ];

    view()
        .column()
        .bg(BG)
        .overflow_hidden()
        .children([
            view()
                .row()
                .grow(1.0)
                .bg(BG)
                .children([ui_left_panel(app), ui_right_panel(app)])
                .build(),
            view()
                .row()
                .height(1.0)
                .shrink(0.0)
                .gap(2.0)
                .bg(BG)
                .padding(0.0, 2.0, 0.0, 2.0)
                .children(status_items)
                .build(),
        ])
        .build()
}

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

pub fn main() -> io::Result<()> {
    let (width, height) = terminal_size().unwrap_or((100, 30));
    let w = u32::from(width);
    let h = u32::from(height);

    let options = RendererOptions {
        use_alt_screen: true,
        hide_cursor: true,
        enable_mouse: false,
        query_capabilities: false,
    };
    let mut renderer = Renderer::new_with_options(w, h, options)?;
    let _raw_guard = enable_raw_mode()?;
    renderer.set_title("Widget Showcase")?;
    renderer.set_background(BG);

    let app = Rc::new(RefCell::new(App::new()));
    let running = Arc::new(AtomicBool::new(true));
    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut runtime = ViewRuntime::new();

    while running.load(Ordering::SeqCst) {
        let app_ref = app.clone();

        {
            let app_borrowed = app.borrow();
            let buffer = renderer.buffer();
            buffer.clear(Rgba::TRANSPARENT);

            let node = ui(&app_borrowed, w, h);
            let mut ctx = RenderContext {
                buffer,
                grapheme_pool: None,
                link_pool: None,
                hit_grid: None,
                theme: None,
            };
            runtime.render_to_buffer(&mut ctx, &node, w as f32, h as f32);
        }

        drop(app_ref);
        renderer.present()?;

        if let Ok(n) = read_with_timeout(&stdin, &mut read_buf, Duration::from_millis(100)) {
            if n == 0 {
                continue;
            }
            let mut offset = 0usize;
            while offset < n {
                match parser.parse(&read_buf[offset..n]) {
                    Ok((event, used)) => {
                        offset += used;
                        if let Event::Key(key) = event {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Escape => {
                                    running.store(false, Ordering::SeqCst);
                                }
                                KeyCode::Char(' ') => {
                                    let mut a = app.borrow_mut();
                                    a.spinner_running = !a.spinner_running;
                                }
                                KeyCode::Left => {
                                    let mut a = app.borrow_mut();
                                    a.slider_value = (a.slider_value - 5.0).max(0.0);
                                }
                                KeyCode::Right => {
                                    let mut a = app.borrow_mut();
                                    a.slider_value = (a.slider_value + 5.0).min(100.0);
                                }
                                KeyCode::Up => {
                                    let mut a = app.borrow_mut();
                                    if a.selected_item > 0 {
                                        a.selected_item -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    let mut a = app.borrow_mut();
                                    if a.selected_item < 4 {
                                        a.selected_item += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(ParseError::Incomplete) => break,
                    Err(_) => offset += 1,
                }
            }
        }
    }

    Ok(())
}
