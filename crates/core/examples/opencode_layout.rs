//! OpenCode-style TUI — faithful clone of the session view.
//!
//! Layout (no header):
//!   - Message area (flex-grow, padded)
//!   - Input prompt at bottom (left border, agent color, metadata row)
//!   - Sidebar (42 cols, auto when width > 120)
//!
//! Keys:
//!   Enter           Send message
//!   Backspace        Delete char
//!   Ctrl+P           Command palette
//!   Ctrl+X B         Toggle sidebar
//!   Ctrl+X N         New session
//!   Ctrl+X Q         Quit
//!   Escape           Close palette / quit
//!   PageUp/PageDown  Scroll messages
//!   Up/Down          Navigate palette items
//!
//! Mouse:
//!   Hover            Highlight palette items
//!   Click            Select palette item
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
#![allow(clippy::many_single_char_names)]
#![allow(clippy::unnested_or_patterns)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::collapsible_match)]

use std::io::{self, Read};
use std::time::Duration;

use opentui_rust::input::{Event, InputParser, KeyCode, KeyModifiers, MouseEventKind};
use opentui_rust::terminal::{enable_raw_mode, terminal_size};
use opentui_rust::{Cell, OptimizedBuffer, Renderer, RendererOptions, Rgba, Style};

const SIDEBAR_WIDTH: u32 = 42;

const BG: Rgba = Rgba::new(0.059, 0.059, 0.086, 1.0);
const BG_PANEL: Rgba = Rgba::new(0.078, 0.078, 0.118, 1.0);
const BG_ELEMENT: Rgba = Rgba::new(0.098, 0.098, 0.137, 1.0);
const BORDER: Rgba = Rgba::new(0.176, 0.176, 0.216, 1.0);
const BORDER_ACTIVE: Rgba = Rgba::new(0.294, 0.549, 0.902, 1.0);
const TEXT: Rgba = Rgba::new(0.878, 0.878, 0.922, 1.0);
const TEXT_MUTED: Rgba = Rgba::new(0.498, 0.498, 0.549, 1.0);
const PRIMARY: Rgba = Rgba::new(0.294, 0.549, 0.902, 1.0);
const SUCCESS: Rgba = Rgba::new(0.349, 0.796, 0.498, 1.0);

struct Message {
    role: &'static str,
    text: String,
}

struct CommandItem {
    name: &'static str,
    shortcut: &'static str,
    category: &'static str,
}

struct App {
    messages: Vec<Message>,
    input_text: String,
    sidebar_visible: bool,
    palette_open: bool,
    palette_filter: String,
    palette_selected: usize,
    palette_scroll: usize,
    palette_mouse_mode: bool,
    scroll_offset: usize,
    mouse_x: u32,
    mouse_y: u32,
    leader_pending: bool,
}

impl App {
    fn new(w: u32) -> Self {
        Self {
            messages: vec![
                Message { role: "user", text: "Help me understand the layout of this application".into() },
                Message { role: "assistant", text: "Sure! This is the OpenCode session view. The main area is a scrollable message list with an input prompt at the bottom. A sidebar on the right shows session info when the terminal is wide enough.".into() },
                Message { role: "user", text: "What keyboard shortcuts are available?".into() },
                Message { role: "assistant", text: "Key bindings:\n\n  Enter        Send message\n  Ctrl+P       Command palette\n  Ctrl+X B     Toggle sidebar\n  Ctrl+X N     New session\n  Page Up/Down Scroll messages\n\nIn the real opencode, Ctrl+X is the leader key for many more commands.".into() },
                Message { role: "user", text: "Show me a tool call".into() },
                Message { role: "assistant", text: "Here's how a Bash tool call renders:\n\n  \u{25B8} Bash: cargo test --lib\n    running 32 tests ...\n    test result: ok. 32 passed; 0 failed".into() },
            ],
            input_text: String::new(),
            sidebar_visible: w > 120,
            palette_open: false,
            palette_filter: String::new(),
            palette_selected: 0,
            palette_scroll: 0,
            palette_mouse_mode: false,
            scroll_offset: 0,
            mouse_x: 0,
            mouse_y: 0,
            leader_pending: false,
        }
    }

    fn commands() -> &'static [CommandItem] {
        &[
            CommandItem {
                name: "New Session",
                shortcut: "ctrl+x n",
                category: "Session",
            },
            CommandItem {
                name: "Session List",
                shortcut: "ctrl+x l",
                category: "Session",
            },
            CommandItem {
                name: "Toggle Sidebar",
                shortcut: "ctrl+x b",
                category: "View",
            },
            CommandItem {
                name: "Model Picker",
                shortcut: "ctrl+x m",
                category: "Agent",
            },
            CommandItem {
                name: "Agent Picker",
                shortcut: "ctrl+x a",
                category: "Agent",
            },
            CommandItem {
                name: "Theme",
                shortcut: "ctrl+x t",
                category: "Settings",
            },
            CommandItem {
                name: "Timeline",
                shortcut: "ctrl+x g",
                category: "Session",
            },
            CommandItem {
                name: "Help",
                shortcut: "ctrl+x h",
                category: "General",
            },
            CommandItem {
                name: "Export Session",
                shortcut: "ctrl+x x",
                category: "Session",
            },
            CommandItem {
                name: "Status",
                shortcut: "ctrl+x s",
                category: "General",
            },
            CommandItem {
                name: "Quit",
                shortcut: "ctrl+x q",
                category: "General",
            },
        ]
    }

    fn filtered_indices(&self) -> Vec<usize> {
        let cmds = Self::commands();
        if self.palette_filter.is_empty() {
            (0..cmds.len()).collect()
        } else {
            let f = self.palette_filter.to_lowercase();
            cmds.iter()
                .enumerate()
                .filter(|(_, c)| c.name.to_lowercase().contains(&f))
                .map(|(i, _)| i)
                .collect()
        }
    }

    fn send_message(&mut self) {
        if self.input_text.trim().is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.input_text);
        self.messages.push(Message { role: "user", text });
        self.messages.push(Message {
            role: "assistant",
            text: "I received your message. In the real opencode, this would\nbe sent to an LLM for processing.".into(),
        });
    }
}

fn clear_area(buf: &mut OptimizedBuffer, x: u32, y: u32, w: u32, h: u32, bg: Rgba) {
    let style = Style::builder().bg(bg).build();
    for row in 0..h {
        for col in 0..w {
            buf.set(x + col, y + row, Cell::new(' ', style));
        }
    }
}

fn draw_messages(buf: &mut OptimizedBuffer, app: &App, x: u32, y: u32, w: u32, h: u32) {
    clear_area(buf, x, y, w, h, BG);

    let msg_x = x + 2;
    let msg_w = w.saturating_sub(4);
    let mut cur_y = y as i32;
    let first_visible = y as i32;
    let last_visible = (y + h) as i32;

    for (idx, msg) in app.messages.iter().enumerate() {
        let lines: Vec<&str> = msg.text.split('\n').collect();
        let block_h = lines.len() as i32 + 1;

        if msg.role == "user" {
            let draw_y = cur_y + 1;
            if draw_y + block_h > first_visible && cur_y < last_visible {
                let border_style = Style::builder().fg(PRIMARY).bg(BG_PANEL).build();
                let text_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
                let bg_style = Style::builder().bg(BG_PANEL).build();

                for row_off in 0..block_h {
                    let row_y = draw_y + row_off;
                    if row_y < first_visible || row_y >= last_visible {
                        continue;
                    }
                    buf.set(msg_x - 1, row_y as u32, Cell::new('\u{2502}', border_style));
                    clear_area(buf, msg_x, row_y as u32, msg_w, 1, BG_PANEL);

                    if let Some(line) = lines.get(row_off as usize) {
                        buf.draw_text(msg_x + 2, row_y as u32, line, text_style);
                    }
                }
            }
            cur_y += block_h + 2;
        } else {
            let draw_y = cur_y + 1;
            if draw_y + block_h > first_visible && cur_y < last_visible {
                let text_style = Style::builder().fg(TEXT).bg(BG).build();
                let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG).build();

                for row_off in 0..(block_h - 1) {
                    let row_y = draw_y + row_off;
                    if row_y < first_visible || row_y >= last_visible {
                        continue;
                    }
                    if let Some(line) = lines.get(row_off as usize) {
                        let s = if line.starts_with("  \u{25B8}") || line.starts_with("  \u{25CF}")
                        {
                            muted_style
                        } else {
                            text_style
                        };
                        buf.draw_text(msg_x + 1, row_y as u32, line, s);
                    }
                }
            }
            cur_y += block_h + 1;
        }

        if idx < app.messages.len() - 1 && cur_y < last_visible {
            // tiny gap
        }
    }
}

fn draw_input(buf: &mut OptimizedBuffer, app: &App, x: u32, y: u32, w: u32) {
    let border_style = Style::builder().fg(BORDER_ACTIVE).build();
    let text_style = Style::builder().fg(TEXT).bg(BG_ELEMENT).build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG_ELEMENT).build();
    let agent_style = Style::builder().fg(PRIMARY).bg(BG_ELEMENT).bold().build();
    let bg_style = Style::builder().bg(BG_ELEMENT).build();

    // Row 0: left border + input content (with padding)
    // Row 1: bottom-left corner + metadata
    // Row 2: hint bar

    // Input row
    buf.set(x, y, Cell::new('\u{2503}', border_style));
    clear_area(buf, x + 1, y, w.saturating_sub(1), 1, BG_ELEMENT);

    if app.input_text.is_empty() {
        buf.draw_text(x + 3, y, "Type a message...", muted_style);
    } else {
        buf.draw_text(x + 3, y, &app.input_text, text_style);
    }

    // Cursor
    let cx = x + 3 + app.input_text.chars().count() as u32;
    if cx < x + w - 1 {
        let cursor_style = Style::builder().fg(PRIMARY).bg(BG_ELEMENT).build();
        buf.set(cx, y, Cell::new('\u{2588}', cursor_style));
    }

    // Bottom-left corner
    buf.set(x, y + 1, Cell::new('\u{2571}', border_style));
    clear_area(buf, x + 1, y + 1, w.saturating_sub(1), 1, BG_ELEMENT);

    // Metadata row: agent name + model
    buf.draw_text(x + 3, y + 1, "Code", agent_style);
    buf.draw_text(
        x + 8,
        y + 1,
        "\u{00B7} claude-sonnet-4-20250514 anthropic",
        muted_style,
    );

    // Hint row
    clear_area(buf, x, y + 2, w, 1, BG);
    buf.draw_text(
        x + 3,
        y + 2,
        "tab agents  \u{00B7}  ctrl+p commands",
        muted_style,
    );
}

fn draw_sidebar(buf: &mut OptimizedBuffer, w: u32, h: u32, term_w: u32) {
    let x = term_w - w;
    let border_style = Style::builder().fg(BORDER).bg(BG_PANEL).build();
    let title_style = Style::builder().fg(TEXT).bg(BG_PANEL).bold().build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let success_style = Style::builder().fg(SUCCESS).bg(BG_PANEL).build();

    clear_area(buf, x, 0, w, h, BG_PANEL);

    // Left border
    for row in 0..h {
        buf.set(x, row, Cell::new('\u{2502}', border_style));
    }

    // Content with padding
    let cx = x + 2;
    let cw = w.saturating_sub(4);

    buf.draw_text(cx, 1, "OpenCode", title_style);
    buf.draw_text(cx, 3, "Session", muted_style);
    buf.draw_text(cx, 4, "abc123def456", muted_style);

    // Workspace
    buf.draw_text(cx, 6, "\u{25CF} git: main", success_style);

    // Separator
    let sep_style = Style::builder().fg(BORDER).bg(BG_PANEL).build();
    for col in 1..(w - 1) {
        buf.set(x + col, 8, Cell::new('\u{2500}', sep_style));
    }

    buf.draw_text(cx, 10, "Share URL", muted_style);
    buf.draw_text(cx, 11, "Not shared", muted_style);

    // Version at bottom
    let ver_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    buf.draw_text(cx, h - 1, "\u{25CF} OpenCode v0.1.0", ver_style);
}

fn draw_palette(buf: &mut OptimizedBuffer, app: &App, w: u32, h: u32) {
    // Dim backdrop (RGBA 0,0,0,150 ~ alpha 0.59)
    let backdrop = Style::builder().bg(Rgba::new(0.0, 0.0, 0.0, 0.586)).build();
    for row in 0..h {
        for col in 0..w {
            buf.set(col, row, Cell::new(' ', backdrop));
        }
    }

    let dialog_w = 60_u32.min(w.saturating_sub(4));
    let dialog_h = 14_u32.min(h.saturating_sub(4));
    let dialog_x = (w.saturating_sub(dialog_w)) / 2;
    let dialog_y = h / 4;

    // Dialog panel
    let border_style = Style::builder().fg(BORDER).bg(BG_PANEL).build();
    let clear_style = Style::builder().bg(BG_PANEL).build();
    let title_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let filter_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let filter_placeholder = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let item_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let selected_style = Style::builder().fg(TEXT).bg(PRIMARY).bold().build();
    let shortcut_muted = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let shortcut_sel = Style::builder().fg(TEXT).bg(PRIMARY).build();

    clear_area(buf, dialog_x, dialog_y, dialog_w, dialog_h, BG_PANEL);

    // Rounded border
    buf.set(dialog_x, dialog_y, Cell::new('\u{250C}', border_style));
    buf.set(
        dialog_x + dialog_w - 1,
        dialog_y,
        Cell::new('\u{2510}', border_style),
    );
    buf.set(
        dialog_x,
        dialog_y + dialog_h - 1,
        Cell::new('\u{2514}', border_style),
    );
    buf.set(
        dialog_x + dialog_w - 1,
        dialog_y + dialog_h - 1,
        Cell::new('\u{2518}', border_style),
    );
    for col in 1..(dialog_w - 1) {
        buf.set(
            dialog_x + col,
            dialog_y,
            Cell::new('\u{2500}', border_style),
        );
        buf.set(
            dialog_x + col,
            dialog_y + dialog_h - 1,
            Cell::new('\u{2500}', border_style),
        );
    }
    for row in 1..(dialog_h - 1) {
        buf.set(
            dialog_x,
            dialog_y + row,
            Cell::new('\u{2502}', border_style),
        );
        buf.set(
            dialog_x + dialog_w - 1,
            dialog_y + row,
            Cell::new('\u{2502}', border_style),
        );
    }

    // Filter input row
    if app.palette_filter.is_empty() {
        buf.draw_text(dialog_x + 2, dialog_y + 1, "Search...", filter_placeholder);
    } else {
        buf.draw_text(
            dialog_x + 2,
            dialog_y + 1,
            &app.palette_filter,
            filter_style,
        );
        let cx = dialog_x + 2 + app.palette_filter.chars().count() as u32;
        let cursor_style = Style::builder().fg(PRIMARY).bg(BG_PANEL).build();
        if cx < dialog_x + dialog_w - 2 {
            buf.set(cx, dialog_y + 1, Cell::new('\u{2588}', cursor_style));
        }
    }

    // Separator below filter
    let sep_style = Style::builder().fg(BORDER).bg(BG_PANEL).build();
    for col in 1..(dialog_w - 1) {
        buf.set(
            dialog_x + col,
            dialog_y + 2,
            Cell::new('\u{2500}', sep_style),
        );
    }

    // Items
    let indices = app.filtered_indices();
    let cmds = App::commands();
    let list_y = dialog_y + 3;
    let list_h = dialog_h.saturating_sub(5);

    let scroll = app.palette_scroll.min(indices.len().saturating_sub(1));
    let visible = indices.iter().skip(scroll).take(list_h as usize);

    for (vi, &oi) in visible.enumerate() {
        let row_y = list_y + vi as u32;
        if row_y >= dialog_y + dialog_h - 2 {
            break;
        }

        let item = &cmds[oi];
        let is_selected = indices.get(app.palette_selected) == Some(&oi);
        let is_hovered = app.palette_mouse_mode
            && row_y == app.mouse_y
            && app.mouse_x > dialog_x
            && app.mouse_x < dialog_x + dialog_w;
        let highlighted = is_selected || is_hovered;

        if highlighted {
            let hl_bg = Style::builder().bg(PRIMARY).build();
            for col in 1..(dialog_w - 1) {
                buf.set(dialog_x + col, row_y, Cell::new(' ', hl_bg));
            }
        }

        let name_style = if highlighted {
            selected_style
        } else {
            item_style
        };
        let key_style = if highlighted {
            shortcut_sel
        } else {
            shortcut_muted
        };

        buf.draw_text(dialog_x + 3, row_y, item.name, name_style);

        let shortcut_x = dialog_x + dialog_w - 3 - item.shortcut.chars().count() as u32;
        buf.draw_text(shortcut_x, row_y, item.shortcut, key_style);
    }

    // Footer: keybind hints
    let footer_y = dialog_y + dialog_h - 2;
    let footer_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    clear_area(
        buf,
        dialog_x + 1,
        footer_y,
        dialog_w.saturating_sub(2),
        1,
        BG_PANEL,
    );
    buf.draw_text(
        dialog_x + 2,
        footer_y,
        "\u{2191}\u{2193} navigate  \u{00B7}  enter select  \u{00B7}  esc close",
        footer_style,
    );
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

    let mut app = App::new(w);
    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut running = true;

    while running {
        let sidebar_w = if app.sidebar_visible {
            SIDEBAR_WIDTH
        } else {
            0
        };
        let main_w = w.saturating_sub(sidebar_w);
        let msg_h = h.saturating_sub(3);

        {
            let buffer = renderer.buffer();
            buffer.clear(Rgba::TRANSPARENT);

            draw_messages(buffer, &app, 0, 0, main_w, msg_h);
            draw_input(buffer, &app, 0, msg_h, main_w);

            if app.sidebar_visible {
                draw_sidebar(buffer, SIDEBAR_WIDTH, h, w);
            }

            if app.palette_open {
                draw_palette(buffer, &app, w, h);
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

                match event {
                    Event::Key(key) => {
                        if app.palette_open {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('c')
                                    if key.modifiers.contains(KeyModifiers::CTRL) =>
                                {
                                    app.palette_open = false;
                                    app.palette_filter.clear();
                                    app.palette_selected = 0;
                                    app.palette_scroll = 0;
                                }
                                KeyCode::Up => {
                                    app.palette_mouse_mode = false;
                                    let indices = app.filtered_indices();
                                    if !indices.is_empty() {
                                        app.palette_selected = if app.palette_selected == 0 {
                                            indices.len() - 1
                                        } else {
                                            app.palette_selected - 1
                                        };
                                        if app.palette_selected < app.palette_scroll {
                                            app.palette_scroll = app.palette_selected;
                                        }
                                    }
                                }
                                KeyCode::Down => {
                                    app.palette_mouse_mode = false;
                                    let indices = app.filtered_indices();
                                    if !indices.is_empty() {
                                        app.palette_selected =
                                            (app.palette_selected + 1) % indices.len();
                                        let visible_rows = 8;
                                        if app.palette_selected >= app.palette_scroll + visible_rows
                                        {
                                            app.palette_scroll =
                                                app.palette_selected - visible_rows + 1;
                                        }
                                    }
                                }
                                KeyCode::Enter => {
                                    let indices = app.filtered_indices();
                                    if let Some(&oi) = indices.get(app.palette_selected) {
                                        let item = &App::commands()[oi];
                                        let name = item.name;
                                        app.palette_open = false;
                                        app.palette_filter.clear();
                                        match name {
                                            "Toggle Sidebar" => {
                                                app.sidebar_visible = !app.sidebar_visible
                                            }
                                            "New Session" => {
                                                app.messages.clear();
                                                app.input_text.clear();
                                            }
                                            "Quit" => running = false,
                                            _ => {}
                                        }
                                    }
                                }
                                KeyCode::Backspace => {
                                    app.palette_filter.pop();
                                    app.palette_selected = 0;
                                    app.palette_scroll = 0;
                                }
                                KeyCode::Char(c) => {
                                    app.palette_filter.push(c);
                                    app.palette_selected = 0;
                                    app.palette_scroll = 0;
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // Leader key
                        if app.leader_pending {
                            app.leader_pending = false;
                            match key.code {
                                KeyCode::Char('b') => app.sidebar_visible = !app.sidebar_visible,
                                KeyCode::Char('q') => running = false,
                                KeyCode::Char('n') => {
                                    app.messages.clear();
                                    app.input_text.clear();
                                }
                                KeyCode::Char('l') | KeyCode::Char('p') => {
                                    app.palette_open = true;
                                    app.palette_filter.clear();
                                    app.palette_selected = 0;
                                    app.palette_scroll = 0;
                                }
                                _ => {}
                            }
                            continue;
                        }

                        if key.modifiers.contains(KeyModifiers::CTRL)
                            && key.code == KeyCode::Char('x')
                        {
                            app.leader_pending = true;
                            continue;
                        }
                        if key.modifiers.contains(KeyModifiers::CTRL)
                            && key.code == KeyCode::Char('p')
                        {
                            app.palette_open = true;
                            app.palette_filter.clear();
                            app.palette_selected = 0;
                            app.palette_scroll = 0;
                            continue;
                        }

                        match key.code {
                            KeyCode::Esc => running = false,
                            KeyCode::Enter => app.send_message(),
                            KeyCode::Backspace => {
                                app.input_text.pop();
                            }
                            KeyCode::Char(c) => {
                                if app.input_text.len() < 200 {
                                    app.input_text.push(c);
                                }
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
                    Event::Mouse(mouse) => {
                        app.mouse_x = mouse.x;
                        app.mouse_y = mouse.y;

                        if app.palette_open {
                            match mouse.kind {
                                MouseEventKind::Move => {
                                    app.palette_mouse_mode = true;
                                    // Highlight item under cursor
                                    let dialog_w = 60_u32.min(w.saturating_sub(4));
                                    let dialog_x = (w.saturating_sub(dialog_w)) / 2;
                                    let dialog_y = h / 4;
                                    let list_y = dialog_y + 3;
                                    if mouse.x > dialog_x
                                        && mouse.x < dialog_x + dialog_w
                                        && mouse.y >= list_y
                                    {
                                        let row = mouse.y - list_y;
                                        let indices = app.filtered_indices();
                                        let idx = app.palette_scroll + row as usize;
                                        if idx < indices.len() {
                                            app.palette_selected = idx;
                                        }
                                    }
                                }
                                MouseEventKind::Press => {
                                    // Click selects
                                    let indices = app.filtered_indices();
                                    if let Some(&oi) = indices.get(app.palette_selected) {
                                        let item = &App::commands()[oi];
                                        let name = item.name;
                                        app.palette_open = false;
                                        app.palette_filter.clear();
                                        match name {
                                            "Toggle Sidebar" => {
                                                app.sidebar_visible = !app.sidebar_visible
                                            }
                                            "New Session" => {
                                                app.messages.clear();
                                                app.input_text.clear();
                                            }
                                            "Quit" => running = false,
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
