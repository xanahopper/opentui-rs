//! OpenCode-style TUI — using the declarative View API.
//!
//! This example uses `view()`, `text()`, `fill()`, `separator()`, `rich_text()`,
//! `when()`, `overlay()` etc. to build the entire UI as a `Node` tree.
//! The `ViewRuntime` handles rebuild + layout + render each frame.
//!
//! Compare with `opencode_declarative.rs` which uses the imperative WidgetTree API.
//!
//! Run: cargo run -p opentui-core --example opencode_view

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
#![allow(clippy::range_minus_one)]

use std::cell::RefCell;
use std::io::{self, Read};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opentui_core::input::{Event, InputParser, KeyCode, KeyModifiers, MouseEventKind, ParseError};
use opentui_core::prelude::*;
use opentui_core::terminal::{enable_raw_mode, terminal_size};
use opentui_core::view::{
    ViewRuntime, fill, overlay, panel, rich_text, separator, span, text, view,
};
use opentui_core::{Renderer, RendererOptions, Rgba};

use opentui_core::widgets::{BorderChars, BorderSides};

const SIDEBAR_WIDTH: f32 = 42.0;

const BG: Rgba = Rgba::new(0.039, 0.039, 0.039, 1.0);
const BG_PANEL: Rgba = Rgba::new(0.078, 0.078, 0.078, 1.0);
const BG_ELEMENT: Rgba = Rgba::new(0.118, 0.118, 0.118, 1.0);
const BORDER_ACTIVE: Rgba = Rgba::new(0.294, 0.549, 0.902, 1.0);
const TEXT: Rgba = Rgba::new(0.878, 0.878, 0.922, 1.0);
const TEXT_MUTED: Rgba = Rgba::new(0.498, 0.498, 0.549, 1.0);
const PRIMARY: Rgba = Rgba::new(0.294, 0.549, 0.902, 1.0);
const SUCCESS: Rgba = Rgba::new(0.349, 0.796, 0.498, 1.0);
const MAX_INPUT_LINES: usize = 6;
const MAX_INPUT_CHARS: usize = 1_000;

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
                    text: "Sure! This is the OpenCode session view built with the declarative View API. The main area is a scrollable message list with an input prompt at the bottom. A sidebar on the right shows session info when the terminal is wide enough.".into(),
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

fn ui_sidebar() -> opentui_core::view::Node {
    view()
        .column()
        .width(SIDEBAR_WIDTH)
        .padding(0.0, 2.0, 0.0, 2.0)
        .bg(BG_PANEL)
        .children([
            fill(BG_PANEL).height(1.0).shrink(0.0).build(),
            text("OpenCode")
                .fg(TEXT)
                .bg(BG_PANEL)
                .bold()
                .height(1.0)
                .shrink(0.0)
                .build(),
            fill(BG_PANEL).height(1.0).shrink(0.0).build(),
            text("Session")
                .fg(TEXT_MUTED)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
            text("abc123def456")
                .fg(TEXT_MUTED)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
            fill(BG_PANEL).height(1.0).shrink(0.0).build(),
            text("\u{25CF} git: main")
                .fg(SUCCESS)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
            fill(BG_PANEL).height(1.0).shrink(0.0).build(),
            separator()
                .height(1.0)
                .shrink(0.0)
                .fg(Rgba::new(0.176, 0.176, 0.216, 1.0))
                .bg(BG_PANEL)
                .build(),
            fill(BG_PANEL).height(1.0).shrink(0.0).build(),
            text("Share URL")
                .fg(TEXT_MUTED)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
            text("Not shared")
                .fg(TEXT_MUTED)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
            fill(BG_PANEL).grow(1.0).build(),
            text("\u{25CF} OpenCode v0.1.0")
                .fg(TEXT_MUTED)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
        ])
        .build()
}

fn ui_prompt(app: &App) -> Vec<opentui_core::view::Node> {
    let mut rows = vec![
        view()
            .row()
            .height(1.0)
            .shrink(0.0)
            .bg(BG_ELEMENT)
            .border(BorderStyle {
                chars: BorderChars::split_left(),
                color: BORDER_ACTIVE,
                focused_color: None,
                sides: BorderSides::left_only(),
            })
            .build(),
    ];

    if app.input_text.is_empty() {
        rows.push(
            view()
                .row()
                .height(1.0)
                .shrink(0.0)
                .bg(BG_ELEMENT)
                .padding(0.0, 0.0, 0.0, 2.0)
                .border(BorderStyle {
                    chars: BorderChars::split_left(),
                    color: BORDER_ACTIVE,
                    focused_color: None,
                    sides: BorderSides::left_only(),
                })
                .children([rich_text(vec![
                    span("\u{2588}", TEXT),
                    span("Ask anything... \"help me get started\"", TEXT_MUTED),
                ])
                .grow(1.0)
                .shrink(0.0)
                .build()])
                .build(),
        );
    } else {
        let input_lines: Vec<&str> = app.input_text.split('\n').collect();
        let first_visible = input_lines.len().saturating_sub(MAX_INPUT_LINES);
        for (idx, line) in input_lines.iter().skip(first_visible).enumerate() {
            let is_last_visible = first_visible + idx + 1 == input_lines.len();
            let display = if is_last_visible {
                format!("{line}\u{2588}")
            } else {
                (*line).to_string()
            };
            rows.push(
                view()
                    .row()
                    .height(1.0)
                    .shrink(0.0)
                    .bg(BG_ELEMENT)
                    .padding(0.0, 0.0, 0.0, 2.0)
                    .border(BorderStyle {
                        chars: BorderChars::split_left(),
                        color: BORDER_ACTIVE,
                        focused_color: None,
                        sides: BorderSides::left_only(),
                    })
                    .children([text(display)
                        .fg(TEXT)
                        .bg(BG_ELEMENT)
                        .grow(1.0)
                        .shrink(0.0)
                        .build()])
                    .build(),
            );
        }
    }

    rows.extend([
        view()
            .row()
            .height(1.0)
            .shrink(0.0)
            .bg(BG_ELEMENT)
            .border(BorderStyle {
                chars: BorderChars::split_left(),
                color: BORDER_ACTIVE,
                focused_color: None,
                sides: BorderSides::left_only(),
            })
            .build(),
        view()
            .row()
            .height(1.0)
            .shrink(0.0)
            .bg(BG_ELEMENT)
            .padding(0.0, 0.0, 0.0, 2.0)
            .border(BorderStyle {
                chars: BorderChars::split_left(),
                color: BORDER_ACTIVE,
                focused_color: None,
                sides: BorderSides::left_only(),
            })
            .children([rich_text(vec![
                span("Code", BORDER_ACTIVE).bold(),
                span(" \u{00B7} claude-sonnet-4-20250514 anthropic", TEXT_MUTED),
            ])
            .grow(1.0)
            .shrink(0.0)
            .build()])
            .build(),
        view()
            .row()
            .height(1.0)
            .shrink(0.0)
            .bg(BG)
            .border(BorderStyle {
                chars: BorderChars {
                    vertical: '\u{2579}',
                    ..BorderChars::empty()
                },
                color: BORDER_ACTIVE,
                focused_color: None,
                sides: BorderSides::left_only(),
            })
            .children([view()
                .row()
                .height(1.0)
                .grow(1.0)
                .shrink(0.0)
                .border(BorderStyle {
                    chars: BorderChars {
                        horizontal: '\u{2580}',
                        ..BorderChars::empty()
                    },
                    color: BG_ELEMENT,
                    focused_color: None,
                    sides: BorderSides {
                        top: false,
                        right: false,
                        bottom: true,
                        left: false,
                    },
                })
                .build()])
            .build(),
    ]);

    rows
}

fn ui_messages(app: &App) -> Vec<opentui_core::view::Node> {
    app.messages
        .iter()
        .enumerate()
        .flat_map(|(idx, msg)| {
            let lines: Vec<&str> = msg.text.split('\n').collect();
            let margin_top = if idx == 0 { 0.0 } else { 1.0 };

            let msg_children: Vec<opentui_core::view::Node> = lines
                .iter()
                .map(|line| {
                    let is_tool = line.starts_with("  \u{25B8}") || line.starts_with("  \u{25CF}");
                    if msg.role == "user" {
                        text(*line)
                            .fg(TEXT)
                            .bg(BG_PANEL)
                            .height(1.0)
                            .shrink(0.0)
                            .build()
                    } else {
                        text(*line)
                            .fg(if is_tool { TEXT_MUTED } else { TEXT })
                            .bg(BG)
                            .height(1.0)
                            .shrink(0.0)
                            .build()
                    }
                })
                .collect();

            if msg.role == "user" {
                vec![
                    view()
                        .column()
                        .shrink(0.0)
                        .margin(margin_top, 0.0, 0.0, 0.0)
                        .bg(BG_PANEL)
                        .padding(1.0, 0.0, 1.0, 2.0)
                        .border(BorderStyle {
                            chars: BorderChars::split_left(),
                            color: PRIMARY,
                            focused_color: None,
                            sides: BorderSides::left_only(),
                        })
                        .children(msg_children)
                        .build(),
                ]
            } else {
                vec![
                    view()
                        .column()
                        .shrink(0.0)
                        .margin(margin_top, 0.0, 0.0, 0.0)
                        .bg(BG)
                        .padding(0.0, 0.0, 0.0, 3.0)
                        .children(msg_children)
                        .build(),
                ]
            }
        })
        .collect()
}

fn ui(app: &App, w: u32, h: u32) -> opentui_core::view::Node {
    let msg_area = view()
        .column()
        .grow(1.0)
        .padding(0.0, 2.0, 0.0, 2.0)
        .gap(1.0)
        .bg(BG)
        .overflow_hidden()
        .children(ui_messages(app))
        .build();

    let prompt_box = view()
        .column()
        .shrink(0.0)
        .bg(BG)
        .children(ui_prompt(app))
        .build();

    let hint_bar = view()
        .row()
        .height(1.0)
        .shrink(0.0)
        .children([
            fill(BG).grow(1.0).build(),
            rich_text(vec![
                span("tab ", TEXT),
                span("agents", TEXT_MUTED),
                span("  ctrl+p ", TEXT),
                span("commands", TEXT_MUTED),
            ])
            .width(27.0)
            .height(1.0)
            .shrink(0.0)
            .build(),
        ])
        .build();

    let bottom_panel = view()
        .column()
        .shrink(0.0)
        .margin(0.0, 2.0, 1.0, 2.0)
        .bg(BG)
        .children([prompt_box, hint_bar])
        .build();

    let main_area = view()
        .column()
        .grow(1.0)
        .bg(BG)
        .overflow_hidden()
        .children([msg_area, bottom_panel])
        .build();

    let mut root_children: Vec<opentui_core::view::Node> = vec![main_area];

    if app.sidebar_visible {
        root_children.push(ui_sidebar());
    }
    if app.palette_open {
        root_children.push(ui_palette(app, w, h));
    }

    view()
        .row()
        .width(w as f32)
        .height(h as f32)
        .bg(BG)
        .overflow_hidden()
        .children(root_children)
        .build()
}

fn ui_palette(app: &App, w: u32, h: u32) -> opentui_core::view::Node {
    let dialog_w = 60_u32.min(w.saturating_sub(4));
    let dialog_h = 14_u32.min(h.saturating_sub(4));
    let dialog_x = w.saturating_sub(dialog_w) / 2;
    let dialog_y = h / 4;

    let indices = app.filtered_indices();
    let cmds = App::commands();
    let list_h = dialog_h.saturating_sub(6);
    let scroll = app.palette_scroll.min(indices.len().saturating_sub(1));

    let mut rows: Vec<opentui_core::view::Node> = indices
        .iter()
        .enumerate()
        .skip(scroll)
        .take(list_h as usize)
        .map(|(idx, &oi)| {
            let item = &cmds[oi];
            let is_selected = indices.get(app.palette_selected) == Some(&oi);
            let row_bg = if is_selected { PRIMARY } else { BG_PANEL };
            let name_fg = TEXT;
            let shortcut_fg = if is_selected { TEXT } else { TEXT_MUTED };
            let shortcut_width = 14.0_f32;

            view()
                .row()
                .height(1.0)
                .shrink(0.0)
                .bg(row_bg)
                .on_action(format!("palette:select:{idx}"))
                .children([
                    text(item.name)
                        .fg(name_fg)
                        .bg(row_bg)
                        .grow(1.0)
                        .shrink(1.0)
                        .height(1.0)
                        .build(),
                    text(item.shortcut)
                        .fg(shortcut_fg)
                        .bg(row_bg)
                        .width(shortcut_width)
                        .height(1.0)
                        .align_right()
                        .build(),
                ])
                .build()
        })
        .collect();

    if rows.is_empty() {
        rows.push(
            text("No commands found")
                .fg(TEXT_MUTED)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
        );
    }

    let filter_text = if app.palette_filter.is_empty() {
        "Search...".to_string()
    } else {
        format!("{}\u{2588}", app.palette_filter)
    };
    let filter_fg = if app.palette_filter.is_empty() {
        TEXT_MUTED
    } else {
        TEXT
    };

    let content = panel()
        .column()
        .size(dialog_w as f32, dialog_h as f32)
        .padding(1.0, 4.0, 1.0, 4.0)
        .bg(BG_PANEL)
        .children([
            view()
                .row()
                .height(1.0)
                .shrink(0.0)
                .bg(BG_PANEL)
                .children([
                    text("Commands")
                        .fg(TEXT)
                        .bg(BG_PANEL)
                        .bold()
                        .grow(1.0)
                        .height(1.0)
                        .build(),
                    text("esc")
                        .fg(TEXT_MUTED)
                        .bg(BG_PANEL)
                        .width(3.0)
                        .height(1.0)
                        .align_right()
                        .build(),
                ])
                .build(),
            text(filter_text)
                .fg(filter_fg)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
            separator()
                .height(1.0)
                .shrink(0.0)
                .fg(Rgba::new(0.176, 0.176, 0.216, 1.0))
                .bg(BG_PANEL)
                .build(),
            view()
                .column()
                .height(list_h as f32)
                .shrink(0.0)
                .bg(BG_PANEL)
                .overflow_hidden()
                .children(rows)
                .build(),
            text("\u{2191}\u{2193} navigate  \u{00B7}  enter select  \u{00B7}  esc close")
                .fg(TEXT_MUTED)
                .bg(BG_PANEL)
                .height(1.0)
                .shrink(0.0)
                .build(),
        ])
        .build();

    overlay(content)
        .position(dialog_x, dialog_y)
        .size(dialog_w, dialog_h)
        .backdrop()
        .z_order(400)
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

fn parse_palette_select_action(action: &str) -> Option<usize> {
    action.strip_prefix("palette:select:")?.parse().ok()
}

fn activate_selected_palette_command(app: &mut App, running: &Arc<AtomicBool>) {
    let indices = app.filtered_indices();
    let Some(&oi) = indices.get(app.palette_selected) else {
        return;
    };

    let name = App::commands()[oi].name;
    app.palette_open = false;
    app.palette_filter.clear();
    app.palette_selected = 0;
    app.palette_scroll = 0;

    match name {
        "Toggle Sidebar" => app.sidebar_visible = !app.sidebar_visible,
        "New Session" => {
            app.messages.clear();
            app.input_text.clear();
        }
        "Quit" => running.store(false, Ordering::SeqCst),
        _ => {}
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn run() -> io::Result<()> {
    let (width, height) = terminal_size().unwrap_or((100, 30));
    let w = u32::from(width);
    let h = u32::from(height);

    let options = RendererOptions {
        use_alt_screen: true,
        hide_cursor: true,
        enable_mouse: true,
        // This example reads stdin directly. Capability query responses also arrive
        // on stdin, so leave probing off to avoid treating them as prompt text.
        query_capabilities: false,
    };
    let mut renderer = Renderer::new_with_options(w, h, options)?;
    let _raw_guard = enable_raw_mode()?;
    renderer.set_title("OpenCode (View API)")?;
    renderer.set_background(BG);

    let app = Rc::new(RefCell::new(App::new(w)));
    let running = Arc::new(AtomicBool::new(true));
    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut pending: Vec<u8> = Vec::new();
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

        if let Ok(n) = read_with_timeout(&stdin, &mut read_buf, Duration::from_millis(50)) {
            if n == 0 {
                if pending.len() == 1 && pending[0] == 0x1b {
                    let mut app_mut = app.borrow_mut();
                    handle_key_event(
                        KeyCode::Escape,
                        KeyModifiers::empty(),
                        &mut app_mut,
                        &running,
                    );
                }
                pending.clear();
                continue;
            }

            if !pending.is_empty() {
                pending.extend_from_slice(&read_buf[..n]);
            }
            let all_data = if pending.is_empty() {
                read_buf[..n].to_vec()
            } else {
                pending.clone()
            };
            let data = &all_data[..];

            for &b in data {
                if b == 0x03 {
                    running.store(false, Ordering::SeqCst);
                }
            }

            let mut offset = 0usize;
            while offset < data.len() {
                match parser.parse(&data[offset..]) {
                    Ok((event, used)) => {
                        offset += used;
                        match event {
                            Event::Key(key) => {
                                let mut app_mut = app.borrow_mut();
                                handle_key_event(key.code, key.modifiers, &mut app_mut, &running);
                            }
                            Event::Mouse(mouse) => {
                                let mut app_mut = app.borrow_mut();
                                app_mut.mouse_x = mouse.x;
                                app_mut.mouse_y = mouse.y;
                                if app_mut.palette_open {
                                    let dispatch = runtime.dispatch_mouse(&mouse);
                                    if let Some(action) = dispatch.action {
                                        if let Some(idx) = parse_palette_select_action(&action) {
                                            let indices = app_mut.filtered_indices();
                                            app_mut.palette_mouse_mode = true;
                                            if idx < indices.len() {
                                                app_mut.palette_selected = idx;
                                            }
                                            if mouse.kind == MouseEventKind::Press {
                                                activate_selected_palette_command(
                                                    &mut app_mut,
                                                    &running,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(ParseError::Incomplete) => {
                        pending = data[offset..].to_vec();
                        offset = data.len();
                    }
                    Err(_) => {
                        offset += 1;
                    }
                }
            }

            if offset >= data.len() {
                pending.clear();
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    run()
}

fn handle_key_event(
    code: KeyCode,
    modifiers: KeyModifiers,
    app: &mut App,
    running: &Arc<AtomicBool>,
) {
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
                activate_selected_palette_command(app, running);
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

    if app.leader_pending {
        app.leader_pending = false;
        match code {
            KeyCode::Char('b') => app.sidebar_visible = !app.sidebar_visible,
            KeyCode::Char('q') => running.store(false, Ordering::SeqCst),
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
    if modifiers.contains(KeyModifiers::CTRL) && code == KeyCode::Char('c') {
        running.store(false, Ordering::SeqCst);
        return;
    }

    match code {
        KeyCode::Escape => {
            if app.input_text.is_empty() {
                running.store(false, Ordering::SeqCst);
            } else {
                app.input_text.clear();
            }
        }
        KeyCode::Enter if modifiers.intersects(KeyModifiers::SHIFT | KeyModifiers::CTRL) => {
            if app.input_text.chars().count() < MAX_INPUT_CHARS {
                app.input_text.push('\n');
            }
        }
        KeyCode::Enter => app.send_message(),
        KeyCode::Backspace => {
            app.input_text.pop();
        }
        KeyCode::Char(c)
            if !modifiers.intersects(KeyModifiers::CTRL | KeyModifiers::ALT)
                && app.input_text.chars().count() < MAX_INPUT_CHARS =>
        {
            app.input_text.push(c);
        }
        _ => {}
    }
}
