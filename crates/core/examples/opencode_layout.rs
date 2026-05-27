//! OpenCode-style TUI layout clone.
//!
//! Mimics the opencode session view:
//!   - Left: scrollable message area + input prompt at bottom
//!   - Right: sidebar (42 cols, auto-show when width > 120)
//!   - No header bar
//!   - Tab: switch focus (messages / input / sidebar)
//!   - Escape / q: quit
//!   - Ctrl+X B: toggle sidebar
//!   - Enter (in input): send message
//!   - Ctrl+P: command palette overlay
//!
//! Run: cargo run -p opentui-core --example opencode_layout

#![allow(unsafe_code)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::single_match)]
#![allow(dead_code)]

use std::io::{self, Read};
use std::time::Duration;

use opentui_rust::input::{Event, InputParser, KeyCode, KeyModifiers};
use opentui_rust::terminal::{enable_raw_mode, terminal_size};
use opentui_rust::{OptimizedBuffer, Renderer, RendererOptions, Rgba, Style};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusArea {
    Messages,
    Input,
    Sidebar,
}

struct Message {
    role: &'static str,
    text: String,
}

struct App {
    messages: Vec<Message>,
    input_text: String,
    focus: FocusArea,
    sidebar_visible: bool,
    command_palette_open: bool,
    scroll_offset: usize,
}

impl App {
    fn new() -> Self {
        Self {
            messages: vec![
                Message {
                    role: "user",
                    text: "Help me understand the layout of this application".into(),
                },
                Message {
                    role: "assistant",
                    text: "Sure! This is the OpenCode session view layout. The main area is split into:\n\n  1. Message scroll area (top, flex-grow)\n  2. Input prompt (bottom, auto-expanding)\n  3. Sidebar (right, 42 cols, auto when width > 120)".into(),
                },
                Message {
                    role: "user",
                    text: "What keyboard shortcuts are available?".into(),
                },
                Message {
                    role: "assistant",
                    text: "Key bindings:\n\n  Tab          Cycle focus area\n  Ctrl+X B     Toggle sidebar\n  Ctrl+P       Command palette\n  Enter        Send message (in input)\n  Escape       Quit / close dialog\n  Page Up/Down Scroll messages\n\nIn the real opencode, Ctrl+X is the leader key\nfor many more commands.".into(),
                },
                Message {
                    role: "user",
                    text: "Show me a tool call example".into(),
                },
                Message {
                    role: "assistant",
                    text: "Here's how a tool call renders:\n\n  \u{25B8} Bash: cargo test --lib\n    running 32 tests ...\n    test result: ok. 32 passed; 0 failed\n\n  \u{25B8} Edit: src/widget.rs\n    -  let x = 0;\n    +  let x = 1;\n\n  \u{25B8} Read: Cargo.toml (23 lines)".into(),
                },
            ],
            input_text: String::new(),
            focus: FocusArea::Input,
            sidebar_visible: false,
            command_palette_open: false,
            scroll_offset: 0,
        }
    }

    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        self.messages.push(Message {
            role: "user",
            text: std::mem::take(&mut self.input_text),
        });
        self.messages.push(Message {
            role: "assistant",
            text: "I received your message. In the real opencode, this would\nbe sent to an LLM provider for processing.".into(),
        });
    }
}

const SIDEBAR_WIDTH: u32 = 42;

// Colors (dark theme, opencode-style)
const BG: Rgba = Rgba::new(0.06, 0.06, 0.09, 1.0);
const BG_PANEL: Rgba = Rgba::new(0.08, 0.08, 0.12, 1.0);
const BG_ELEMENT: Rgba = Rgba::new(0.10, 0.10, 0.14, 1.0);
const BORDER: Rgba = Rgba::new(0.18, 0.18, 0.22, 1.0);
const BORDER_ACTIVE: Rgba = Rgba::new(0.30, 0.55, 0.90, 1.0);
const TEXT: Rgba = Rgba::new(0.88, 0.88, 0.92, 1.0);
const TEXT_MUTED: Rgba = Rgba::new(0.50, 0.50, 0.55, 1.0);
const PRIMARY: Rgba = Rgba::new(0.40, 0.60, 0.95, 1.0);
const ACCENT: Rgba = Rgba::new(0.65, 0.45, 0.95, 1.0);
const SUCCESS: Rgba = Rgba::new(0.35, 0.80, 0.50, 1.0);
const WARNING: Rgba = Rgba::new(0.90, 0.65, 0.20, 1.0);

fn draw_user_message(buf: &mut OptimizedBuffer, x: u32, y: u32, w: u32, text: &str) -> u32 {
    let border_style = Style::builder().fg(PRIMARY).bg(BG_PANEL).build();
    let text_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let lines: Vec<&str> = text.split('\n').collect();
    let height = lines.len() as u32;

    for row in 0..height {
        buf.set(
            x,
            y + row,
            opentui_rust::Cell::new('\u{2503}', border_style),
        );
    }

    for (i, line) in lines.iter().enumerate() {
        buf.draw_text(x + 2, y + i as u32, line, text_style);
        // Clear rest of line
        let used = line.chars().count() as u32;
        let clear_style = Style::builder().bg(BG_PANEL).build();
        for col in (x + 2 + used)..(x + w) {
            buf.set(col, y + i as u32, opentui_rust::Cell::new(' ', clear_style));
        }
    }

    height
}

fn draw_assistant_message(buf: &mut OptimizedBuffer, x: u32, y: u32, _w: u32, text: &str) -> u32 {
    let text_style = Style::builder().fg(TEXT).bg(BG).build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG).build();
    let lines: Vec<&str> = text.split('\n').collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("  \u{25B8}") {
            buf.draw_text(x, y + i as u32, line, muted_style);
        } else {
            buf.draw_text(x, y + i as u32, line, text_style);
        }
    }

    lines.len() as u32
}

fn draw_messages(buf: &mut OptimizedBuffer, app: &App, x: u32, y: u32, w: u32, h: u32) {
    let clear_style = Style::builder().bg(BG).build();
    for row in 0..h {
        for col in 0..w {
            buf.set(x + col, y + row, opentui_rust::Cell::new(' ', clear_style));
        }
    }

    let mut cur_y = y as i32;
    for msg in &app.messages {
        if cur_y >= (y + h) as i32 {
            break;
        }
        if cur_y < y as i32 - 20 {
            cur_y += msg.text.split('\n').count() as i32 + 1;
            continue;
        }
        let rendered = if msg.role == "user" {
            draw_user_message(buf, x, cur_y as u32, w, &msg.text)
        } else {
            draw_assistant_message(buf, x + 2, cur_y as u32, w - 2, &msg.text)
        };
        cur_y += rendered as i32 + 1;
    }
}

fn draw_input_area(buf: &mut OptimizedBuffer, app: &App, x: u32, y: u32, w: u32, focused: bool) {
    let border_color = if focused { BORDER_ACTIVE } else { BORDER };
    let border_style = Style::builder().fg(border_color).bg(BG_PANEL).build();
    let text_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let clear_style = Style::builder().bg(BG_PANEL).build();
    let agent_style = Style::builder().fg(PRIMARY).bg(BG_PANEL).bold().build();

    // Clear area (3 rows: input + metadata + hint)
    for row in 0..3 {
        for col in 0..w {
            buf.set(x + col, y + row, opentui_rust::Cell::new(' ', clear_style));
        }
    }

    // Left border
    buf.set(x, y, opentui_rust::Cell::new('\u{2503}', border_style));

    // Input text or placeholder
    if app.input_text.is_empty() {
        buf.draw_text(x + 2, y, "Type a message...", muted_style);
    } else {
        buf.draw_text(x + 2, y, &app.input_text, text_style);
    }

    // Cursor blink (simple: always show when focused)
    if focused {
        let cursor_x = x + 2 + app.input_text.chars().count() as u32;
        if cursor_x < x + w {
            let cursor_style = Style::builder().fg(PRIMARY).bg(BG_PANEL).build();
            buf.set(
                cursor_x,
                y,
                opentui_rust::Cell::new('\u{2588}', cursor_style),
            );
        }
    }

    // Bottom-left corner
    buf.set(x, y + 1, opentui_rust::Cell::new('\u{2571}', border_style));

    // Metadata row: agent name + model
    buf.draw_text(x + 2, y + 1, "Code", agent_style);
    buf.draw_text(
        x + 7,
        y + 1,
        "\u{00B7} claude-sonnet-4-20250514 anthropic",
        muted_style,
    );

    // Hint row
    if focused {
        buf.draw_text(
            x + 2,
            y + 2,
            "tab agents  \u{00B7}  ctrl+p commands",
            muted_style,
        );
    }
}

fn draw_sidebar(buf: &mut OptimizedBuffer, w: u32, h: u32, term_w: u32) {
    let x = term_w - w;
    let clear_style = Style::builder().bg(BG_ELEMENT).build();
    let border_style = Style::builder().fg(BORDER).bg(BG_ELEMENT).build();
    let title_style = Style::builder().fg(TEXT).bg(BG_ELEMENT).bold().build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG_ELEMENT).build();
    let success_style = Style::builder().fg(SUCCESS).bg(BG_ELEMENT).build();
    let version_style = Style::builder().fg(TEXT_MUTED).bg(BG_ELEMENT).build();

    for row in 0..h {
        buf.set(x, row, opentui_rust::Cell::new('\u{2502}', border_style));
        for col in 1..w {
            buf.set(x + col, row, opentui_rust::Cell::new(' ', clear_style));
        }
    }

    buf.draw_text(x + 2, 0, "OpenCode", title_style);
    buf.draw_text(x + 2, 2, "Session", muted_style);
    buf.draw_text(x + 2, 3, "abc123def456", muted_style);

    // Workspace status
    buf.draw_text(x + 2, 5, "\u{25CF} git: main", success_style);

    // Spacer line
    let sep_style = Style::builder().fg(BORDER).bg(BG_ELEMENT).build();
    for col in 1..w {
        buf.set(x + col, 7, opentui_rust::Cell::new('\u{2500}', sep_style));
    }

    buf.draw_text(x + 2, 9, "Share URL", muted_style);
    buf.draw_text(x + 2, 10, "Not shared", muted_style);

    // Version at bottom
    buf.draw_text(x + 2, h - 1, "\u{25CF} OpenCode v0.1.0", version_style);
}

fn draw_command_palette(buf: &mut OptimizedBuffer, w: u32, h: u32) {
    let dialog_w = 60_u32.min(w.saturating_sub(4));
    let dialog_h = 12_u32.min(h.saturating_sub(4));
    let dialog_x = (w.saturating_sub(dialog_w)) / 2;
    let dialog_y = h / 4;

    // Backdrop
    let backdrop = Style::builder().bg(Rgba::new(0.0, 0.0, 0.0, 0.6)).build();
    for row in 0..h {
        for col in 0..w {
            buf.set(col, row, opentui_rust::Cell::new(' ', backdrop));
        }
    }

    // Dialog box
    let border_style = Style::builder().fg(BORDER_ACTIVE).bg(BG_ELEMENT).build();
    let clear_style = Style::builder().bg(BG_ELEMENT).build();
    let title_style = Style::builder().fg(TEXT).bg(BG_ELEMENT).bold().build();
    let item_style = Style::builder().fg(TEXT).bg(BG_ELEMENT).build();
    let hint_style = Style::builder().fg(PRIMARY).bg(BG_ELEMENT).bold().build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG_ELEMENT).build();

    for row in 0..dialog_h {
        for col in 0..dialog_w {
            buf.set(
                dialog_x + col,
                dialog_y + row,
                opentui_rust::Cell::new(' ', clear_style),
            );
        }
    }

    // Border
    buf.set(
        dialog_x,
        dialog_y,
        opentui_rust::Cell::new('\u{250C}', border_style),
    );
    buf.set(
        dialog_x + dialog_w - 1,
        dialog_y,
        opentui_rust::Cell::new('\u{2510}', border_style),
    );
    buf.set(
        dialog_x,
        dialog_y + dialog_h - 1,
        opentui_rust::Cell::new('\u{2514}', border_style),
    );
    buf.set(
        dialog_x + dialog_w - 1,
        dialog_y + dialog_h - 1,
        opentui_rust::Cell::new('\u{2518}', border_style),
    );
    for col in 1..(dialog_w - 1) {
        buf.set(
            dialog_x + col,
            dialog_y,
            opentui_rust::Cell::new('\u{2500}', border_style),
        );
        buf.set(
            dialog_x + col,
            dialog_y + dialog_h - 1,
            opentui_rust::Cell::new('\u{2500}', border_style),
        );
    }
    for row in 1..(dialog_h - 1) {
        buf.set(
            dialog_x,
            dialog_y + row,
            opentui_rust::Cell::new('\u{2502}', border_style),
        );
        buf.set(
            dialog_x + dialog_w - 1,
            dialog_y + row,
            opentui_rust::Cell::new('\u{2502}', border_style),
        );
    }

    // Title
    buf.draw_text(dialog_x + 2, dialog_y, "Commands", title_style);

    // Filter hint
    buf.draw_text(dialog_x + 2, dialog_y + 1, "Type to filter...", muted_style);

    // Command items
    let commands = [
        ("New Session", "ctrl+x n"),
        ("Session List", "ctrl+x l"),
        ("Toggle Sidebar", "ctrl+x b"),
        ("Model Picker", "ctrl+x m"),
        ("Agent Picker", "ctrl+x a"),
        ("Theme", "ctrl+x t"),
        ("Timeline", "ctrl+x g"),
        ("Help", "ctrl+x h"),
        ("Export Session", "ctrl+x x"),
        ("Status", "ctrl+x s"),
    ];
    for (i, (name, _key)) in commands.iter().enumerate() {
        let row_y = dialog_y + 2 + i as u32;
        if row_y >= dialog_y + dialog_h - 1 {
            break;
        }
        let style = if i == 0 { hint_style } else { item_style };
        buf.draw_text(dialog_x + 2, row_y, name, style);
    }
}

fn main() -> io::Result<()> {
    let (width, height) = terminal_size().unwrap_or((100, 30));
    let w = u32::from(width);
    let h = u32::from(height);

    let options = RendererOptions {
        use_alt_screen: true,
        hide_cursor: true,
        enable_mouse: true,
        query_capabilities: true,
    };
    let mut renderer = Renderer::new_with_options(w, h, options)?;
    let _raw_guard = enable_raw_mode()?;
    renderer.set_title("OpenCode")?;
    renderer.set_background(BG);

    let mut app = App::new();
    if w > 120 {
        app.sidebar_visible = true;
    }

    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut running = true;
    let mut leader_pending = false;

    while running {
        let sidebar_w = if app.sidebar_visible {
            SIDEBAR_WIDTH
        } else {
            0
        };
        let main_w = w.saturating_sub(sidebar_w);

        // Layout: no header
        // Message area: top, height = h - 4 (3 for input + 1 gap)
        let msg_y = 0u32;
        let msg_h = h.saturating_sub(4);
        let input_y = msg_h;

        {
            let buffer = renderer.buffer();
            buffer.clear(Rgba::TRANSPARENT);

            // Draw messages
            draw_messages(buffer, &app, 0, msg_y, main_w, msg_h);

            // Draw input
            let input_focused = app.focus == FocusArea::Input;
            draw_input_area(buffer, &app, 0, input_y, main_w, input_focused);

            // Draw sidebar
            if app.sidebar_visible {
                draw_sidebar(buffer, SIDEBAR_WIDTH, h, w);
            }

            // Command palette overlay
            if app.command_palette_open {
                draw_command_palette(buffer, w, h);
            }
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
                    // Command palette open?
                    if app.command_palette_open {
                        if key.code == KeyCode::Esc || key.is_ctrl_c() {
                            app.command_palette_open = false;
                        }
                        continue;
                    }

                    // Leader key handling (Ctrl+X)
                    if leader_pending {
                        leader_pending = false;
                        match key.code {
                            KeyCode::Char('b') => {
                                app.sidebar_visible = !app.sidebar_visible;
                            }
                            KeyCode::Char('q') => {
                                running = false;
                            }
                            KeyCode::Char('n') => {
                                app.messages.clear();
                                app.input_text.clear();
                            }
                            KeyCode::Char('l') => {
                                app.command_palette_open = true;
                            }
                            _ => {}
                        }
                        continue;
                    }

                    if key.modifiers.contains(KeyModifiers::CTRL) && key.code == KeyCode::Char('x')
                    {
                        leader_pending = true;
                        continue;
                    }

                    // Ctrl+P: command palette
                    if key.modifiers.contains(KeyModifiers::CTRL) && key.code == KeyCode::Char('p')
                    {
                        app.command_palette_open = true;
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc if app.focus != FocusArea::Input => {
                            running = false;
                        }
                        KeyCode::Esc => {
                            if app.focus == FocusArea::Input && !app.input_text.is_empty() {
                                app.input_text.clear();
                            } else {
                                running = false;
                            }
                        }
                        KeyCode::Tab => {
                            app.focus = match app.focus {
                                FocusArea::Messages => FocusArea::Input,
                                FocusArea::Input => {
                                    if app.sidebar_visible {
                                        FocusArea::Sidebar
                                    } else {
                                        FocusArea::Messages
                                    }
                                }
                                FocusArea::Sidebar => FocusArea::Messages,
                            };
                        }
                        KeyCode::BackTab => {
                            app.focus = match app.focus {
                                FocusArea::Messages => {
                                    if app.sidebar_visible {
                                        FocusArea::Sidebar
                                    } else {
                                        FocusArea::Input
                                    }
                                }
                                FocusArea::Input => FocusArea::Messages,
                                FocusArea::Sidebar => FocusArea::Input,
                            };
                        }
                        KeyCode::Enter if app.focus == FocusArea::Input => {
                            app.send_message();
                        }
                        KeyCode::Backspace if app.focus == FocusArea::Input => {
                            app.input_text.pop();
                        }
                        KeyCode::Char(c)
                            if app.focus == FocusArea::Input && app.input_text.len() < 200 =>
                        {
                            app.input_text.push(c);
                        }
                        KeyCode::PageUp => {
                            app.scroll_offset = app.scroll_offset.saturating_sub(5);
                        }
                        KeyCode::PageDown => {
                            app.scroll_offset = app.scroll_offset.saturating_add(5);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
