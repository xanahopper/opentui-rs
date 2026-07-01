//! Declarative Hello — minimal example using the view builder API.
//!
//! Demonstrates `view()`, `text()`, `when()`, and `ViewRuntime` in a
//! simple render loop.
//!
//! Run: cargo run -p opentui-core --example `declarative_hello`

#![allow(unsafe_code)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]

use std::io::{self, Read};
use std::time::Duration;

use opentui_core::view::{Node, ViewRuntime, text, view, when};
use opentui_core::input::{Event, InputParser, KeyCode};
use opentui_core::terminal::{enable_raw_mode, terminal_size};
use opentui_core::{Renderer, Rgba};

const BG: Rgba = Rgba::new(0.059, 0.059, 0.086, 1.0);
const TEXT: Rgba = Rgba::new(0.878, 0.878, 0.922, 1.0);
const TEXT_MUTED: Rgba = Rgba::new(0.498, 0.498, 0.549, 1.0);
const SUCCESS: Rgba = Rgba::new(0.349, 0.796, 0.498, 1.0);

struct AppState {
    counter: u32,
    show_extra: bool,
    quit: bool,
}

fn ui(state: &AppState) -> Node {
    view()
        .column()
        .size_pct(1.0, 1.0)
        .bg(BG)
        .padding_all(2.0)
        .gap(1.0)
        .children([
            text("OpenTUI Declarative Hello")
                .fg(TEXT)
                .bold()
                .height(1.0)
                .build(),
            text(format!("Counter: {}", state.counter))
                .fg(SUCCESS)
                .height(1.0)
                .build(),
            when(state.show_extra, || {
                text("Extra line is visible! Press 't' to toggle.")
                    .fg(TEXT_MUTED)
                    .height(1.0)
                    .build()
            }),
            text("Press +/- to change counter, 't' to toggle, 'q' to quit")
                .fg(TEXT_MUTED)
                .height(1.0)
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

fn main() -> io::Result<()> {
    let (width, height) = terminal_size().unwrap_or((80, 24));
    let w = u32::from(width);
    let h = u32::from(height);

    let mut renderer = Renderer::new(w, h)?;
    let _raw_guard = enable_raw_mode()?;
    let mut input_parser = InputParser::new();
    let mut runtime = ViewRuntime::new();

    let mut state = AppState {
        counter: 0,
        show_extra: true,
        quit: false,
    };

    let stdin = io::stdin();
    let mut read_buf = [0u8; 64];

    while !state.quit {
        let node = ui(&state);
        runtime.rebuild(&node);
        runtime.layout(w as f32, h as f32);

        {
            let buffer = renderer.buffer();
            buffer.clear(Rgba::TRANSPARENT);
            let mut ctx = opentui_core::widget::RenderContext {
                buffer,
                grapheme_pool: None,
                link_pool: None,
                hit_grid: None,
                theme: None,
            };
            runtime.render(&mut ctx);
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
                    match key.code {
                        KeyCode::Char('+') => state.counter = state.counter.saturating_add(1),
                        KeyCode::Char('-') => state.counter = state.counter.saturating_sub(1),
                        KeyCode::Char('t') => state.show_extra = !state.show_extra,
                        KeyCode::Char('q') | KeyCode::Escape => state.quit = true,
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
