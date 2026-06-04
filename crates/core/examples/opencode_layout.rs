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
//!   Ctrl+C           Quit
//!   Ctrl+X B         Toggle sidebar
//!   Ctrl+X N         New session
//!   Ctrl+X Q         Quit
//!   Escape           Close palette / quit (when input empty)
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

use opentui_rust::input::{Event, InputParser, KeyCode, KeyModifiers, MouseEventKind, ParseError};
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
                Message {
                    role: "user",
                    text: "Help me understand the layout of this application".into(),
                },
                Message {
                    role: "assistant",
                    text: "Sure! This is the OpenCode session view. The main area is a scrollable message list with an input prompt at the bottom. A sidebar on the right shows session info when the terminal is wide enough.".into(),
                },
                Message {
                    role: "user",
                    text: "What keyboard shortcuts are available?".into(),
                },
                Message {
                    role: "assistant",
                    text: "Key bindings:\n\n  Enter        Send message\n  Ctrl+P       Command palette\n  Ctrl+X B     Toggle sidebar\n  Ctrl+X N     New session\n  Page Up/Down Scroll messages\n\nIn the real opencode, Ctrl+X is the leader key for many more commands."
                        .into(),
                },
                Message {
                    role: "user",
                    text: "Show me a tool call".into(),
                },
                Message {
                    role: "assistant",
                    text: "Here's how a Bash tool call renders:\n\n  \u{25B8} Bash: cargo test --lib\n    running 32 tests ...\n    test result: ok. 32 passed; 0 failed"
                        .into(),
                },
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
        let margin_top = i32::from(idx != 0);

        if msg.role == "user" {
            let padding_top: i32 = 1;
            let padding_bot: i32 = 1;
            let padding_left: u32 = 2;
            let total_h = lines.len() as i32 + padding_top + padding_bot;
            let draw_y = cur_y + margin_top;

            if draw_y + total_h > first_visible && draw_y < last_visible {
                let border_style = Style::builder().fg(PRIMARY).bg(BG_PANEL).build();
                let text_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();

                for row_off in 0..total_h {
                    let row_y = draw_y + row_off;
                    if row_y < first_visible || row_y >= last_visible {
                        continue;
                    }
                    buf.set(msg_x - 1, row_y as u32, Cell::new('\u{2503}', border_style));
                    clear_area(buf, msg_x, row_y as u32, msg_w, 1, BG_PANEL);

                    let line_idx = row_off - padding_top;
                    if line_idx >= 0 && line_idx < lines.len() as i32 {
                        buf.draw_text(
                            msg_x + padding_left,
                            row_y as u32,
                            lines[line_idx as usize],
                            text_style,
                        );
                    }
                }
            }
            cur_y = draw_y + total_h;
        } else {
            let padding_left: u32 = 3;
            let line_count = lines.len() as i32;
            let draw_y = cur_y + margin_top;

            if draw_y + line_count > first_visible && draw_y < last_visible {
                let text_style = Style::builder().fg(TEXT).bg(BG).build();
                let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG).build();

                for row_off in 0..line_count {
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
                        buf.draw_text(msg_x + padding_left - 2, row_y as u32, line, s);
                    }
                }
            }
            cur_y = draw_y + line_count;
        }
    }
}

/// OpenCode-style bottom prompt panel.
///
/// Matches the layout from `opencode/packages/opencode/src/cli/cmd/tui/component/prompt/index.tsx`:
///
/// The outer session box has `paddingLeft=2`, which we account for by offsetting the
/// border and content to align with the message area.
///
/// ```text
/// Col:  0  1   2   3  4  5 ...              w-3  w-2  w-1
///       _  ┃  ░░  ░░  ░  <input text>  ...  ░░   ░░   ░░     Row 0: paddingTop
///       _  ┃  ░░  ░░  ░  <input text>  ...  ░░   ░░   ░░     Row 1: textarea
///       _  ┃  ░░  ░░  ░                      ░░   ░░   ░░     Row 2: metadata pad
///       _  ┃  ░░  ░░  Code·model·provider    ░░   ░░   ░░     Row 3: metadata
///       _  ╹  ▀▀  ▀▀  ▀  ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀  ▀▀   ▀▀   ▀▀   Row 4: decoration
///       _  ░░ ░░  ░   tab agents · ctrl+p    ░░   ░░   ░░     Row 5: hint bar
/// ```
fn draw_input(buf: &mut OptimizedBuffer, app: &App, x: u32, y: u32, w: u32) {
    let border_x = x + 1;
    let border_color = BORDER_ACTIVE;
    let border_style = Style::builder().fg(border_color).bg(BG).build();
    let text_style = Style::builder().fg(TEXT).bg(BG_ELEMENT).build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG_ELEMENT).build();
    let agent_style = Style::builder()
        .fg(border_color)
        .bg(BG_ELEMENT)
        .bold()
        .build();
    let decor_fg = Style::builder().fg(BG_ELEMENT).bg(BG).build();
    let hint_key_style = Style::builder().fg(TEXT).bg(BG).build();
    let hint_label_style = Style::builder().fg(TEXT_MUTED).bg(BG).build();

    let el_x = border_x + 1;
    let el_w = w.saturating_sub(border_x - x + 1);
    let content_x = el_x + 2;
    let content_right = el_x + el_w.saturating_sub(2);

    // Row 0: paddingTop — border + BG_ELEMENT fill (empty)
    buf.set(border_x, y, Cell::new('\u{2503}', border_style));
    clear_area(buf, el_x, y, el_w, 1, BG_ELEMENT);

    // Row 1: textarea row — border + BG_ELEMENT + input text
    buf.set(border_x, y + 1, Cell::new('\u{2503}', border_style));
    clear_area(buf, el_x, y + 1, el_w, 1, BG_ELEMENT);

    if app.input_text.is_empty() {
        buf.draw_text(
            content_x,
            y + 1,
            "Ask anything... \"help me get started\"",
            muted_style,
        );
    } else {
        buf.draw_text(content_x, y + 1, &app.input_text, text_style);
    }

    let cx = content_x + app.input_text.chars().count() as u32;
    if cx < content_right {
        let cursor_style = Style::builder().fg(TEXT).bg(BG_ELEMENT).build();
        buf.set(cx, y + 1, Cell::new('\u{2588}', cursor_style));
    }

    // Row 2: metadata paddingTop — empty border + BG_ELEMENT fill
    buf.set(border_x, y + 2, Cell::new('\u{2503}', border_style));
    clear_area(buf, el_x, y + 2, el_w, 1, BG_ELEMENT);

    // Row 3: metadata — border + BG_ELEMENT + agent · model · provider
    buf.set(border_x, y + 3, Cell::new('\u{2503}', border_style));
    clear_area(buf, el_x, y + 3, el_w, 1, BG_ELEMENT);

    buf.draw_text(content_x, y + 3, "Code", agent_style);
    buf.draw_text(
        content_x + 5,
        y + 3,
        "\u{00B7} claude-sonnet-4-20250514 anthropic",
        muted_style,
    );

    // Row 4: decoration row — ╹ left + ▀▀▀ bottom (BG_ELEMENT colored)
    buf.set(border_x, y + 4, Cell::new('\u{2575}', border_style));
    clear_area(buf, el_x, y + 4, el_w, 1, BG);
    for col in el_x..(el_x + el_w) {
        buf.set(col, y + 4, Cell::new('\u{2580}', decor_fg));
    }

    // Row 5: hint bar — full width, no border
    //   "tab agents  ctrl+p commands"
    //   key=TEXT color, label=textMuted, gap=2 between groups
    clear_area(buf, x, y + 5, w, 1, BG);
    let hint_x = x + 2;
    buf.draw_text(hint_x, y + 5, "tab ", hint_key_style);
    buf.draw_text(hint_x + 4, y + 5, "agents", hint_label_style);
    buf.draw_text(hint_x + 12, y + 5, "ctrl+p ", hint_key_style);
    buf.draw_text(hint_x + 19, y + 5, "commands", hint_label_style);
}

fn draw_sidebar(buf: &mut OptimizedBuffer, w: u32, h: u32, term_w: u32) {
    let x = term_w - w;
    let title_style = Style::builder().fg(TEXT).bg(BG_PANEL).bold().build();
    let muted_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let success_style = Style::builder().fg(SUCCESS).bg(BG_PANEL).build();

    clear_area(buf, x, 0, w, h, BG_PANEL);

    let cx = x + 2;
    buf.draw_text(cx, 1, "OpenCode", title_style);
    buf.draw_text(cx, 3, "Session", muted_style);
    buf.draw_text(cx, 4, "abc123def456", muted_style);

    buf.draw_text(cx, 6, "\u{25CF} git: main", success_style);

    let sep_style = Style::builder().fg(BORDER).bg(BG_PANEL).build();
    for col in 2..(w - 2) {
        buf.set(x + col, 8, Cell::new('\u{2500}', sep_style));
    }

    buf.draw_text(cx, 10, "Share URL", muted_style);
    buf.draw_text(cx, 11, "Not shared", muted_style);

    let ver_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    buf.draw_text(cx, h - 2, "\u{25CF} OpenCode v0.1.0", ver_style);
}

fn draw_palette(buf: &mut OptimizedBuffer, app: &App, w: u32, h: u32) {
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

    let filter_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let filter_placeholder = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let item_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let selected_style = Style::builder().fg(TEXT).bg(PRIMARY).bold().build();
    let shortcut_muted = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let shortcut_sel = Style::builder().fg(TEXT).bg(PRIMARY).build();

    clear_area(buf, dialog_x, dialog_y, dialog_w, dialog_h, BG_PANEL);

    let inner_x = dialog_x + 4;
    let inner_right = dialog_x + dialog_w - 4;

    // Title row: "Commands" (left) + "esc" (right)
    let title_style = Style::builder().fg(TEXT).bg(BG_PANEL).bold().build();
    let esc_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    buf.draw_text(inner_x, dialog_y + 1, "Commands", title_style);
    buf.draw_text(inner_right - 3, dialog_y + 1, "esc", esc_style);

    // Search input row
    if app.palette_filter.is_empty() {
        buf.draw_text(inner_x, dialog_y + 2, "Search...", filter_placeholder);
    } else {
        buf.draw_text(inner_x, dialog_y + 2, &app.palette_filter, filter_style);
        let cx = inner_x + app.palette_filter.chars().count() as u32;
        let cursor_style = Style::builder().fg(PRIMARY).bg(BG_PANEL).build();
        if cx < inner_right {
            buf.set(cx, dialog_y + 2, Cell::new('\u{2588}', cursor_style));
        }
    }

    // Separator below search
    let sep_style = Style::builder().fg(BORDER).bg(BG_PANEL).build();
    for col in 4..(dialog_w - 4) {
        buf.set(
            dialog_x + col,
            dialog_y + 3,
            Cell::new('\u{2500}', sep_style),
        );
    }

    // Items list
    let indices = app.filtered_indices();
    let cmds = App::commands();
    let list_y = dialog_y + 4;
    let list_h = dialog_h.saturating_sub(6);

    let scroll = app.palette_scroll.min(indices.len().saturating_sub(1));
    let visible = indices.iter().skip(scroll).take(list_h as usize);

    for (vi, &oi) in visible.enumerate() {
        let row_y = list_y + vi as u32;
        if row_y >= dialog_y + dialog_h - 1 {
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
            for col in 0..dialog_w {
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

        buf.draw_text(dialog_x + 4, row_y, item.name, name_style);

        let shortcut_x =
            (dialog_x + dialog_w - 4).saturating_sub(item.shortcut.chars().count() as u32);
        buf.draw_text(shortcut_x, row_y, item.shortcut, key_style);
    }

    // Footer hints
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
        dialog_x + 4,
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

    // Buffer for accumulating incomplete escape sequences across reads
    let mut pending: Vec<u8> = Vec::new();

    while running {
        let sidebar_w = if app.sidebar_visible {
            SIDEBAR_WIDTH
        } else {
            0
        };
        let main_w = w.saturating_sub(sidebar_w);
        // 4 rows for bottom panel: input + metadata + decoration + hint
        let bottom_h = 6u32;
        let msg_h = h.saturating_sub(bottom_h);

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
                // No data — if we have a pending escape byte, treat as Esc key
                if pending.len() == 1 && pending[0] == 0x1b {
                    handle_key_event(
                        KeyCode::Escape,
                        KeyModifiers::empty(),
                        &mut app,
                        &mut running,
                    );
                }
                pending.clear();
                continue;
            }

            // Prepend any pending bytes from previous read
            if !pending.is_empty() {
                pending.extend_from_slice(&read_buf[..n]);
            }
            let all_data = if pending.is_empty() {
                read_buf[..n].to_vec()
            } else {
                pending.clone()
            };
            let data = &all_data[..];

            let mut offset = 0usize;
            while offset < data.len() {
                match parser.parse(&data[offset..]) {
                    Ok((event, used)) => {
                        offset += used;
                        match event {
                            Event::Key(key) => {
                                handle_key_event(key.code, key.modifiers, &mut app, &mut running);
                            }
                            Event::Mouse(mouse) => {
                                app.mouse_x = mouse.x;
                                app.mouse_y = mouse.y;

                                if app.palette_open {
                                    match mouse.kind {
                                        MouseEventKind::Move => {
                                            app.palette_mouse_mode = true;
                                            let dialog_w = 60_u32.min(w.saturating_sub(4));
                                            let dialog_x = (w.saturating_sub(dialog_w)) / 2;
                                            let dialog_y = h / 4;
                                            let list_y = dialog_y + 4;
                                            if mouse.x > dialog_x
                                                && mouse.x < dialog_x + dialog_w
                                                && mouse.y >= list_y
                                            {
                                                let row = (mouse.y - list_y) as usize;
                                                let indices = app.filtered_indices();
                                                let idx = app.palette_scroll + row;
                                                if idx < indices.len() {
                                                    app.palette_selected = idx;
                                                }
                                            }
                                        }
                                        MouseEventKind::Press => {
                                            let indices = app.filtered_indices();
                                            if let Some(&oi) = indices.get(app.palette_selected) {
                                                let item = &App::commands()[oi];
                                                let name = item.name;
                                                app.palette_open = false;
                                                app.palette_filter.clear();
                                                match name {
                                                    "Toggle Sidebar" => {
                                                        app.sidebar_visible = !app.sidebar_visible;
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
                    Err(ParseError::Incomplete) => {
                        // Stash remaining bytes for next read
                        pending = data[offset..].to_vec();
                        offset = data.len();
                    }
                    Err(_) => {
                        // Unrecognized / too long — skip one byte
                        offset += 1;
                    }
                }
            }

            // If all bytes were consumed, clear pending
            if offset >= data.len() && !pending.is_empty() && offset >= pending.len() {
                // pending was already consumed in the loop
            }
            if offset >= data.len() {
                pending.clear();
            }
        }
    }

    Ok(())
}

fn handle_key_event(code: KeyCode, modifiers: KeyModifiers, app: &mut App, running: &mut bool) {
    if app.palette_open {
        match code {
            KeyCode::Escape | KeyCode::Char('c') if modifiers.contains(KeyModifiers::CTRL) => {
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
                    app.palette_selected = (app.palette_selected + 1) % indices.len();
                    let visible_rows = 8;
                    if app.palette_selected >= app.palette_scroll + visible_rows {
                        app.palette_scroll = app.palette_selected - visible_rows + 1;
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
                        "Toggle Sidebar" => app.sidebar_visible = !app.sidebar_visible,
                        "New Session" => {
                            app.messages.clear();
                            app.input_text.clear();
                        }
                        "Quit" => *running = false,
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
        return;
    }

    // Leader key (Ctrl+X)
    if app.leader_pending {
        app.leader_pending = false;
        match code {
            KeyCode::Char('b') => app.sidebar_visible = !app.sidebar_visible,
            KeyCode::Char('q') => *running = false,
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
        return;
    }

    if modifiers.contains(KeyModifiers::CTRL) && code == KeyCode::Char('x') {
        app.leader_pending = true;
        return;
    }
    if modifiers.contains(KeyModifiers::CTRL) && code == KeyCode::Char('p') {
        app.palette_open = true;
        app.palette_filter.clear();
        app.palette_selected = 0;
        app.palette_scroll = 0;
        return;
    }

    // Ctrl+C always quits
    if modifiers.contains(KeyModifiers::CTRL) && code == KeyCode::Char('c') {
        *running = false;
        return;
    }

    match code {
        // Esc quits only when input is empty (matching opencode behavior)
        KeyCode::Escape => {
            if app.input_text.is_empty() {
                *running = false;
            } else {
                app.input_text.clear();
            }
        }
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
