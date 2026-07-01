//! OpenCode-style TUI — declarative View API with full mouse interaction.
//!
//! Replicates OpenCode's terminal session view:
//!   - Full-screen row layout
//!   - Main area (left): scrollable conversation + input prompt
//!   - Sidebar (right): session info panel
//!   - Command palette overlay (Ctrl+P)
//!
//! Mouse interactions: hover highlights, click selection, scroll.
//! Keyboard: Ctrl+Q/Ctrl+C quit, Ctrl+B toggle sidebar, Ctrl+P palette.
//!
//! Run: cargo run -p opentui-core --example opencode

#![allow(unsafe_code)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::branches_sharing_code)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::range_minus_one)]
#![allow(clippy::redundant_pub_crate)]
#![allow(dead_code)]

use std::cell::RefCell;
use std::io;
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use opentui_core::prelude::*;
use opentui_core::view::{overlay, panel, rich_text, separator, span, text, view};
use opentui_core::input::{Event, InputParser, KeyCode, KeyModifiers, ParseError};
use opentui_core::terminal::{MouseEventKind, enable_raw_mode, terminal_size};
use opentui_core::{Renderer, RendererOptions, Rgba};

// ── Colour palette ──────────────────────────────────────────────────────

const BG: Rgba = Rgba::new(0.039, 0.039, 0.039, 1.0);
const BG_PANEL: Rgba = Rgba::new(0.067, 0.067, 0.067, 1.0);
const BG_ELEMENT: Rgba = Rgba::new(0.098, 0.098, 0.098, 1.0);
const BG_HOVER: Rgba = Rgba::new(0.137, 0.137, 0.137, 1.0);
const BG_SELECTED: Rgba = Rgba::new(0.157, 0.157, 0.176, 1.0);

const TEXT: Rgba = Rgba::new(0.878, 0.878, 0.922, 1.0);
const TEXT_BRIGHT: Rgba = Rgba::new(0.937, 0.937, 0.957, 1.0);
const TEXT_MUTED: Rgba = Rgba::new(0.498, 0.498, 0.549, 1.0);
const TEXT_DIM: Rgba = Rgba::new(0.349, 0.349, 0.388, 1.0);

const PRIMARY: Rgba = Rgba::new(0.980, 0.698, 0.514, 1.0);
const ACCENT: Rgba = Rgba::new(0.616, 0.486, 0.847, 1.0);
const SUCCESS: Rgba = Rgba::new(0.498, 0.847, 0.561, 1.0);

const BORDER: Rgba = Rgba::new(0.176, 0.176, 0.216, 1.0);
const BORDER_SUBTLE: Rgba = Rgba::new(0.118, 0.118, 0.137, 1.0);
const BORDER_ACTIVE: Rgba = Rgba::new(0.294, 0.549, 0.902, 1.0);

const SIDEBAR_WIDTH: f32 = 36.0;

// ── Domain model ────────────────────────────────────────────────────────

#[derive(Clone)]
struct Message {
    role: &'static str,
    text: String,
}

#[derive(Clone)]
struct CommandItem {
    name: &'static str,
    shortcut: &'static str,
    category: &'static str,
}

impl CommandItem {
    fn all() -> &'static [Self] {
        &[
            Self {
                name: "New Session",
                shortcut: "ctrl+x n",
                category: "Session",
            },
            Self {
                name: "Session List",
                shortcut: "ctrl+x l",
                category: "Session",
            },
            Self {
                name: "Toggle Sidebar",
                shortcut: "ctrl+x b",
                category: "View",
            },
            Self {
                name: "Model Picker",
                shortcut: "ctrl+x m",
                category: "Agent",
            },
            Self {
                name: "Agent Picker",
                shortcut: "ctrl+x a",
                category: "Agent",
            },
            Self {
                name: "Theme",
                shortcut: "ctrl+x t",
                category: "Settings",
            },
            Self {
                name: "Timeline",
                shortcut: "ctrl+x g",
                category: "Session",
            },
            Self {
                name: "Help",
                shortcut: "ctrl+x h",
                category: "General",
            },
            Self {
                name: "Export Session",
                shortcut: "ctrl+x x",
                category: "Session",
            },
            Self {
                name: "Status",
                shortcut: "ctrl+x s",
                category: "General",
            },
            Self {
                name: "Clear Buffer",
                shortcut: "ctrl+x k",
                category: "Buffer",
            },
            Self {
                name: "Quit",
                shortcut: "ctrl+x q",
                category: "General",
            },
        ]
    }
}

impl Message {
    fn fg(&self) -> Rgba {
        if self.role == "assistant" {
            TEXT
        } else {
            TEXT_BRIGHT
        }
    }
}

// ── App state ───────────────────────────────────────────────────────────

struct App {
    messages: Vec<Message>,
    input_text: String,
    sidebar_visible: bool,
    palette_open: bool,
    palette_filter: String,
    palette_selected: usize,
    palette_scroll: usize,
    palette_mouse_mode: bool,
    hovered_palette: Option<usize>,
    mouse_x: u32,
    mouse_y: u32,
}

impl App {
    fn new(w: u32) -> Self {
        Self {
            messages: vec![
                Message { role: "user", text: "Help me understand the layout of this TUI application".into() },
                Message {
                    role: "assistant",
                    text: "This is a terminal UI built with the opentui-core declarative View API. The layout has a main conversation area on the left, an optional sidebar on the right, and a command palette overlay accessible via Ctrl+P. Mouse hover, click, and scroll are all handled through the hit grid system."
                        .into(),
                },
                Message { role: "user", text: "What mouse interactions are supported?".into() },
                Message {
                    role: "assistant",
                    text: "Mouse interactions supported:\n\n  \u{25B8} Hover  \u{2014} palette items highlight on mouse-over\n  \u{25B8} Click  \u{2014} select palette items and send button\n  \u{25B8} Scroll \u{2014} scroll the conversation with mouse wheel\n  \u{25B8} Focus  \u{2014} click anywhere to activate relevant actions\n\nThe hit grid maps screen positions to widget actions via ViewRuntime."
                        .into(),
                },
                Message { role: "user", text: "Show me what the rendering pipeline looks like".into() },
                Message {
                    role: "assistant",
                    text: "The rendering pipeline:\n\n  1. Build declarative Node tree from app state\n  2. ViewRuntime::rebuild() \u{2014} creates WidgetTree\n  3. ViewRuntime::layout() \u{2014} runs Taffy flexbox\n  4. ViewRuntime::register_hit_areas() \u{2014} builds hit grid\n  5. ViewRuntime::render() \u{2014} executes render commands\n  6. Renderer::present() \u{2014} diffs and outputs ANSI\n\nEach frame, hover state is detected from the hit grid and baked into the next frame's Node tree."
                        .into(),
                },
                Message { role: "user", text: "What about keyboard input?".into() },
                Message {
                    role: "assistant",
                    text: "Keyboard input is parsed by InputParser which handles ANSI escape sequences, kitty keyboard protocol, mouse SGR/X11, paste, and focus events. The parsed events are dispatched through ViewRuntime which routes them to the focused widget."
                        .into(),
                },
            ],
            input_text: String::new(),
            sidebar_visible: w > 100,
            palette_open: false,
            palette_filter: String::new(),
            palette_selected: 0,
            palette_scroll: 0,
            palette_mouse_mode: false,
            hovered_palette: None,
            mouse_x: 0,
            mouse_y: 0,
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
            text: "I received your message! In the real OpenCode, this would be sent to an LLM for processing."
                .into(),
        });
    }

    fn filtered_indices(&self) -> Vec<usize> {
        let cmds = CommandItem::all();
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

    fn activate_selected_palette(&mut self, running: &AtomicBool) {
        let indices = self.filtered_indices();
        if let Some(&cmd_idx) = indices.get(self.palette_selected) {
            let cmd = &CommandItem::all()[cmd_idx];
            match cmd.name {
                "Toggle Sidebar" => self.sidebar_visible = !self.sidebar_visible,
                "Quit" => running.store(false, Ordering::SeqCst),
                _ => {}
            }
        }
        self.palette_open = false;
    }
}

// ── Read with timeout ───────────────────────────────────────────────────

fn read_with_timeout(stdin: &io::Stdin, buf: &mut [u8], timeout: Duration) -> io::Result<usize> {
    use std::os::fd::AsRawFd;

    let fd = stdin.as_raw_fd();
    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let timeout_ms = timeout.as_millis() as libc::c_int;
    let ret = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    if ret == 0 {
        return Err(io::Error::new(io::ErrorKind::WouldBlock, "timeout"));
    }
    stdin.read(buf)
}

// ── UI builders ─────────────────────────────────────────────────────────

fn ui_sidebar() -> opentui_core::view::Node {
    view()
        .column()
        .width(SIDEBAR_WIDTH)
        .padding(0.0, 2.0, 0.0, 2.0)
        .bg(BG_PANEL)
        .children([
            view().height(1.0).build(),
            text("OpenCode").fg(TEXT).bold().height(1.0).build(),
            view().height(1.0).build(),
            text("Session").fg(TEXT_MUTED).height(1.0).build(),
            text("abc123def456").fg(TEXT_MUTED).height(1.0).build(),
            view().height(1.0).build(),
            text("\u{25CF} git: main").fg(SUCCESS).height(1.0).build(),
            view().grow(1.0).build(),
            separator().fg(BORDER).height(1.0).build(),
            view().height(1.0).build(),
            text("Share URL").fg(TEXT_MUTED).height(1.0).build(),
            text("Not shared").fg(TEXT_MUTED).height(1.0).build(),
            view().height(1.0).build(),
            text("\u{25CF} opentui-core v0.1")
                .fg(TEXT_MUTED)
                .height(1.0)
                .build(),
        ])
        .build()
}

fn ui_palette(app: &App, w: u32, h: u32) -> opentui_core::view::Node {
    let dialog_w = 60_u32.min(w.saturating_sub(4));
    let dialog_h = 14_u32.min(h.saturating_sub(4));
    let x = (w.saturating_sub(dialog_w) / 2) as u32;
    let y = (h / 4) as u32;

    let indices = app.filtered_indices();
    let cmds = CommandItem::all();

    let list_h = dialog_h.saturating_sub(5) as usize;
    let scroll = app.palette_scroll.min(indices.len().saturating_sub(1));

    let palette_items: Vec<opentui_core::view::Node> = indices
        .iter()
        .enumerate()
        .skip(scroll)
        .take(list_h)
        .map(|(display_idx, &cmd_idx)| {
            let cmd = &cmds[cmd_idx];
            let selected = indices.get(app.palette_selected) == Some(&cmd_idx);
            let hovered = app.hovered_palette == Some(display_idx);
            let row_bg = if selected || hovered {
                BG_HOVER
            } else {
                BG_PANEL
            };
            let name_fg = if selected { TEXT_BRIGHT } else { TEXT };
            let cat_fg = TEXT_MUTED;

            view()
                .row()
                .height(1.0)
                .bg(row_bg)
                .on_action(format!("palette:{display_idx}"))
                .padding_x(1.0)
                .children([
                    text(cmd.name).fg(name_fg).bg(row_bg).grow(1.0).build(),
                    text(cmd.category).fg(cat_fg).bg(row_bg).width(12.0).build(),
                    text(cmd.shortcut).fg(cat_fg).bg(row_bg).width(14.0).build(),
                ])
                .build()
        })
        .collect();

    let filter_text = if app.palette_filter.is_empty() {
        "Type to filter...".to_string()
    } else {
        app.palette_filter.clone()
    };
    let filter_fg = if app.palette_filter.is_empty() {
        TEXT_MUTED
    } else {
        TEXT
    };

    overlay(
        panel()
            .column()
            .padding(0.5, 0.5, 0.5, 0.5)
            .bg(BG_PANEL)
            .border_rounded(BORDER)
            .children([
                view()
                    .row()
                    .height(1.0)
                    .padding_x(0.5)
                    .bg(BG_ELEMENT)
                    .children([
                        text("> ").fg(PRIMARY).build(),
                        text(filter_text).fg(filter_fg).grow(1.0).build(),
                    ])
                    .build(),
                separator().fg(BORDER_SUBTLE).height(1.0).build(),
                view().column().children(palette_items).build(),
            ])
            .build(),
    )
    .position(x, y)
    .size(dialog_w, dialog_h)
    .z_order(400)
    .build()
}

fn ui_prompt(app: &App) -> opentui_core::view::Node {
    let text = if app.input_text.is_empty() {
        "Type a message...".to_string()
    } else {
        app.input_text.clone()
    };
    let fg = if app.input_text.is_empty() {
        TEXT_MUTED
    } else {
        TEXT
    };

    view()
        .row()
        .height(3.0)
        .bg(BG_ELEMENT)
        .border_rounded(if app.input_text.is_empty() {
            BORDER
        } else {
            BORDER_ACTIVE
        })
        .on_action("click:prompt")
        .children([
            text(format!("  {text}")).fg(fg).grow(1.0).build(),
            text("\u{23CE} Send  ")
                .fg(if app.input_text.is_empty() {
                    TEXT_DIM
                } else {
                    PRIMARY
                })
                .on_action("click:send")
                .build(),
        ])
        .build()
}

fn ui_messages(app: &App) -> opentui_core::view::Node {
    let mut nodes: Vec<opentui_core::view::Node> = Vec::new();

    for msg in &app.messages {
        let (label, label_fg) = if msg.role == "user" {
            ("You", PRIMARY)
        } else {
            ("OpenCode", ACCENT)
        };

        nodes.push(rich_text(vec![span(label, label_fg).bold()]).build());

        for line in msg.text.lines() {
            nodes.push(text(format!("  {line}")).fg(msg.fg()).build());
        }

        nodes.push(view().height(1.0).build());
    }

    view()
        .column()
        .grow(1.0)
        .bg(BG)
        .overflow_hidden()
        .children(nodes)
        .build()
}

fn ui(app: &App, w: u32, h: u32) -> opentui_core::view::Node {
    let main_area = view()
        .column()
        .grow(1.0)
        .bg(BG)
        .padding(0.0, 1.0, 1.0, 1.0)
        .children([ui_messages(app), ui_prompt(app)])
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
        .size(w as f32, h as f32)
        .bg(BG)
        .overflow_hidden()
        .children(root_children)
        .build()
}

// ── Hover + action ──────────────────────────────────────────────────────

fn parse_palette_select(action: &str) -> Option<usize> {
    action.strip_prefix("palette:")?.parse().ok()
}

fn detect_palette_hover(app: &mut App, runtime: &ViewRuntime) {
    app.hovered_palette = None;

    if !app.palette_open {
        return;
    }

    let hit_id = match runtime.hit_grid().test(app.mouse_x, app.mouse_y) {
        Some(id) => id,
        None => return,
    };

    let action = match runtime.action_for_widget(hit_id as u64) {
        Some(a) => a,
        None => return,
    };

    if let Some(idx) = parse_palette_select(action) {
        app.hovered_palette = Some(idx);
    }
}

// ── Keyboard handler ────────────────────────────────────────────────────

fn handle_key(code: KeyCode, modifiers: KeyModifiers, app: &mut App, running: &AtomicBool) {
    if modifiers.contains(KeyModifiers::CTRL) {
        match code {
            KeyCode::Char('q') | KeyCode::Char('c') => {
                if app.palette_open {
                    app.palette_open = false;
                    app.palette_filter.clear();
                    app.palette_selected = 0;
                    app.palette_scroll = 0;
                } else {
                    running.store(false, Ordering::SeqCst);
                }
            }
            KeyCode::Char('b') => app.sidebar_visible = !app.sidebar_visible,
            KeyCode::Char('p') => {
                app.palette_open = !app.palette_open;
                if !app.palette_open {
                    app.palette_filter.clear();
                    app.palette_selected = 0;
                    app.palette_scroll = 0;
                }
            }
            _ => {}
        }
        return;
    }

    if app.palette_open {
        match code {
            KeyCode::Escape => {
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
                    if app.palette_selected >= app.palette_scroll + 10 {
                        app.palette_scroll = app.palette_selected.saturating_sub(9);
                    }
                }
            }
            KeyCode::Enter => {
                app.activate_selected_palette(running);
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

    match code {
        KeyCode::Enter => app.send_message(),
        KeyCode::Escape => app.input_text.clear(),
        KeyCode::Backspace => {
            app.input_text.pop();
        }
        KeyCode::Char(c) => {
            app.input_text.push(c);
        }
        _ => {}
    }
}

// ── Main ────────────────────────────────────────────────────────────────

fn run() -> io::Result<()> {
    let (term_w, term_h) = terminal_size().unwrap_or((80, 24));
    let mut w = term_w as u32;
    let mut h = term_h as u32;

    let mut renderer = Renderer::new_with_options(
        w,
        h,
        RendererOptions {
            use_alt_screen: true,
            hide_cursor: true,
            enable_mouse: true,
            query_capabilities: false,
        },
    )?;
    let _raw_guard = enable_raw_mode()?;
    renderer.set_title("OpenCode")?;
    renderer.set_background(BG);

    let app = Rc::new(RefCell::new(App::new(w)));
    let running = Arc::new(AtomicBool::new(true));
    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 256];
    let mut pending: Vec<u8> = Vec::new();
    let mut runtime = ViewRuntime::new();

    while running.load(Ordering::SeqCst) {
        {
            let app_ref = app.borrow();

            let node = ui(&app_ref, w, h);

            let mut ctx = RenderContext {
                buffer: renderer.buffer(),
                grapheme_pool: None,
                link_pool: None,
                hit_grid: None,
                theme: None,
            };

            runtime.render_to_buffer(&mut ctx, &node, w as f32, h as f32);

            // Detect hover state for next frame
            drop(app_ref);
            let mut app_mut = app.borrow_mut();
            detect_palette_hover(&mut app_mut, &runtime);
            drop(app_mut);
        }

        renderer.present()?;

        match read_with_timeout(&stdin, &mut read_buf, Duration::from_millis(50)) {
            Ok(n) if n > 0 => {
                if !pending.is_empty() {
                    pending.extend_from_slice(&read_buf[..n]);
                }
                let data = if pending.is_empty() {
                    read_buf[..n].to_vec()
                } else {
                    std::mem::take(&mut pending)
                };

                let mut offset = 0usize;
                while offset < data.len() {
                    match parser.parse(&data[offset..]) {
                        Ok((Event::Key(key), used)) => {
                            offset += used;
                            handle_key(key.code, key.modifiers, &mut app.borrow_mut(), &running);
                        }
                        Ok((Event::Mouse(mouse), used)) => {
                            offset += used;
                            let mut app_mut = app.borrow_mut();
                            app_mut.mouse_x = mouse.x;
                            app_mut.mouse_y = mouse.y;

                            match mouse.kind {
                                MouseEventKind::Press => {
                                    let dispatch = runtime.dispatch_mouse(&mouse);
                                    if let Some(action) = dispatch.action {
                                        if let Some(idx) = parse_palette_select(&action) {
                                            app_mut.palette_mouse_mode = true;
                                            let indices = app_mut.filtered_indices();
                                            let cmd_indices: Vec<usize> = indices
                                                .iter()
                                                .enumerate()
                                                .filter(|(di, _)| {
                                                    *di >= app_mut.palette_scroll
                                                        && *di < app_mut.palette_scroll + 10
                                                })
                                                .map(|(_, ci)| *ci)
                                                .collect();
                                            app_mut.palette_selected =
                                                *cmd_indices.get(idx).unwrap_or(&0);
                                            app_mut.activate_selected_palette(&running);
                                        } else if action == "click:send" {
                                            app_mut.send_message();
                                        }
                                    }
                                }
                                MouseEventKind::ScrollUp => {
                                    // Scrolling handled by overflow_hidden + render_offset would need ScrollState integration
                                }
                                MouseEventKind::ScrollDown => {}
                                _ => {}
                            }
                        }
                        Ok((Event::Resize(resize), used)) => {
                            offset += used;
                            w = resize.width as u32;
                            h = resize.height as u32;
                            renderer.resize(w, h)?;
                        }
                        Ok((_, used)) => {
                            offset += used;
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
            _ => {}
        }
    }

    renderer.cleanup()?;
    Ok(())
}

fn main() -> io::Result<()> {
    run()
}
