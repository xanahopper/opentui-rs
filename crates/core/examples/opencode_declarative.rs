/*
//! Legacy OpenCode-style TUI — low-level WidgetTree construction.
//!
//! This version uses only BoxWidget, TextLineWidget, FillWidget, SeparatorWidget,
//! and StyledTextWidget — zero custom Widget implementations. All layout is
//! driven by Taffy via LayoutStyle declarations.
//!
//! ```text
//! root (row, width=w, height=h)
//! ├── main_area (column, flex_grow=1)
//! │   ├── messages (column, flex_grow=1, padding=2, gap=1)
//! │   │   └── [per-message BoxWidgets — rebuilt each frame]
//! │   ├── prompt (column, flex_shrink=0)
//! │   │   └── [5 rows with border_left=┃]
//! │   └── hint_bar (StyledTextWidget, height=1)
//! └── sidebar (column, width=42, bg=BG_PANEL, padding=2)
//!     └── [TextLineWidget / FillWidget / SeparatorWidget rows]
//! ```
//!
//! Run: cargo run -p opentui-core --example opencode_declarative

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

use opentui_core::prelude::*;
use opentui_core::input::{Event, InputParser, KeyCode, KeyModifiers, MouseEventKind, ParseError};
use opentui_core::terminal::{enable_raw_mode, terminal_size};
use opentui_core::{Cell, OptimizedBuffer, Renderer, RendererOptions, Rgba, Style};

use opentui_core::widgets::{
    BorderChars, BorderSides, BoxWidget, FillWidget, SeparatorWidget, StyledSegment,
    StyledTextWidget, TextLineWidget,
};

const SIDEBAR_WIDTH: f32 = 42.0;

const BG: Rgba = Rgba::new(0.059, 0.059, 0.086, 1.0);
const BG_PANEL: Rgba = Rgba::new(0.078, 0.078, 0.118, 1.0);
const BG_ELEMENT: Rgba = Rgba::new(0.098, 0.098, 0.137, 1.0);
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
*/

//! Compatibility entry for the `OpenCode` declarative example.
//!
//! The executable implementation lives in `opencode_view.rs`, which builds the
//! UI through the View DSL (`view()`, `text()`, `overlay()`, actions, and
//! `ViewRuntime`). Keeping this entry point preserves the old example command
//! while avoiding a second, lower-level `WidgetTree` version.

#[path = "opencode_view.rs"]
mod opencode_view;

fn main() -> std::io::Result<()> {
    opencode_view::run()
}

/*
fn build_sidebar(tree: &mut WidgetTree, parent: WidgetId) {
    let spacer1 = tree.allocate_id();
    let title = tree.allocate_id();
    let spacer2 = tree.allocate_id();
    let label_session = tree.allocate_id();
    let val_session = tree.allocate_id();
    let spacer3 = tree.allocate_id();
    let git_status = tree.allocate_id();
    let spacer4 = tree.allocate_id();
    let sep = tree.allocate_id();
    let spacer5 = tree.allocate_id();
    let label_share = tree.allocate_id();
    let val_share = tree.allocate_id();
    let flex_spacer = tree.allocate_id();
    let version = tree.allocate_id();

    tree.add_child(
        parent,
        FillWidget::new(
            spacer1,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            BG_PANEL,
        ),
    );
    tree.add_child(
        parent,
        TextLineWidget::with_text(
            title,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            "OpenCode",
        )
        .fg(TEXT)
        .bg(BG_PANEL)
        .bold(),
    );
    tree.add_child(
        parent,
        FillWidget::new(
            spacer2,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            BG_PANEL,
        ),
    );
    tree.add_child(
        parent,
        TextLineWidget::with_text(
            label_session,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            "Session",
        )
        .fg(TEXT_MUTED)
        .bg(BG_PANEL),
    );
    tree.add_child(
        parent,
        TextLineWidget::with_text(
            val_session,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            "abc123def456",
        )
        .fg(TEXT_MUTED)
        .bg(BG_PANEL),
    );
    tree.add_child(
        parent,
        FillWidget::new(
            spacer3,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            BG_PANEL,
        ),
    );
    tree.add_child(
        parent,
        TextLineWidget::with_text(
            git_status,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            "\u{25CF} git: main",
        )
        .fg(SUCCESS)
        .bg(BG_PANEL),
    );
    tree.add_child(
        parent,
        FillWidget::new(
            spacer4,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            BG_PANEL,
        ),
    );
    tree.add_child(
        parent,
        SeparatorWidget::new(sep, LayoutStyle::column().height(1.0).flex_shrink(0.0))
            .fg(Rgba::new(0.176, 0.176, 0.216, 1.0))
            .bg(BG_PANEL),
    );
    tree.add_child(
        parent,
        FillWidget::new(
            spacer5,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            BG_PANEL,
        ),
    );
    tree.add_child(
        parent,
        TextLineWidget::with_text(
            label_share,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            "Share URL",
        )
        .fg(TEXT_MUTED)
        .bg(BG_PANEL),
    );
    tree.add_child(
        parent,
        TextLineWidget::with_text(
            val_share,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            "Not shared",
        )
        .fg(TEXT_MUTED)
        .bg(BG_PANEL),
    );
    tree.add_child(
        parent,
        FillWidget::new(flex_spacer, LayoutStyle::column().flex_grow(1.0), BG_PANEL),
    );
    tree.add_child(
        parent,
        TextLineWidget::with_text(
            version,
            LayoutStyle::column().height(1.0).flex_shrink(0.0),
            "\u{25CF} OpenCode v0.1.0",
        )
        .fg(TEXT_MUTED)
        .bg(BG_PANEL),
    );
}

fn build_prompt_rows(tree: &mut WidgetTree, parent: WidgetId, app: &App) {
    let row1 = tree.allocate_id();
    let row2 = tree.allocate_id();
    let row3 = tree.allocate_id();
    let row4 = tree.allocate_id();
    let row5 = tree.allocate_id();

    let border_style = LayoutStyle::row().height(1.0).flex_shrink(0.0);

    tree.add_child(
        parent,
        BoxWidget::new(row1, border_style.clone())
            .background(BG_ELEMENT)
            .border_custom(
                BorderChars::split_left(),
                BORDER_ACTIVE,
                BorderSides::left_only(),
            ),
    );

    let input_text = if app.input_text.is_empty() {
        "Ask anything... \"help me get started\"".to_string()
    } else {
        app.input_text.clone()
    };
    let input_fg = if app.input_text.is_empty() {
        TEXT_MUTED
    } else {
        TEXT
    };
    let cursor = if app.input_text.is_empty() {
        ""
    } else {
        "\u{2588}"
    };

    tree.add_child(
        parent,
        BoxWidget::new(row2, border_style.clone())
            .background(BG_ELEMENT)
            .border_custom(
                BorderChars::split_left(),
                BORDER_ACTIVE,
                BorderSides::left_only(),
            )
            .base_padding(0.0, 0.0, 0.0, 2.0),
    );

    if let Some(text_id) = {
        let id = tree.allocate_id();
        tree.add_child(
            row2,
            TextLineWidget::with_text(
                id,
                LayoutStyle::row().flex_grow(1.0).flex_shrink(0.0),
                format!("{input_text}{cursor}"),
            )
            .fg(input_fg)
            .bg(BG_ELEMENT),
        );
        Some(id)
    } {
        let _ = text_id;
    }

    tree.add_child(
        parent,
        BoxWidget::new(row3, border_style.clone())
            .background(BG_ELEMENT)
            .border_custom(
                BorderChars::split_left(),
                BORDER_ACTIVE,
                BorderSides::left_only(),
            ),
    );

    tree.add_child(
        parent,
        BoxWidget::new(row4, border_style.clone())
            .background(BG_ELEMENT)
            .border_custom(
                BorderChars::split_left(),
                BORDER_ACTIVE,
                BorderSides::left_only(),
            )
            .base_padding(0.0, 0.0, 0.0, 2.0),
    );

    let meta_id = tree.allocate_id();
    tree.add_child(
        row4,
        StyledTextWidget::from_segments(
            meta_id,
            LayoutStyle::row().flex_grow(1.0).flex_shrink(0.0),
            vec![
                StyledSegment::new("Code", BORDER_ACTIVE).bold(),
                StyledSegment::new(" \u{00B7} claude-sonnet-4-20250514 anthropic", TEXT_MUTED),
            ],
        ),
    );

    tree.add_child(
        parent,
        BoxWidget::new(row5, border_style).border_custom(
            BorderChars::split_left_no_bottom(),
            BORDER_ACTIVE,
            BorderSides::left_only(),
        ),
    );
    let deco_id = tree.allocate_id();
    tree.add_child(
        row5,
        SeparatorWidget::new(deco_id, LayoutStyle::row().flex_grow(1.0))
            .char_('\u{2580}')
            .fg(BG_ELEMENT)
            .bg(BG),
    );
}

fn build_messages(tree: &mut WidgetTree, parent: WidgetId, app: &App) {
    for (idx, msg) in app.messages.iter().enumerate() {
        let lines: Vec<&str> = msg.text.split('\n').collect();
        let msg_id = tree.allocate_id();

        if msg.role == "user" {
            tree.add_child(
                parent,
                BoxWidget::new(
                    msg_id,
                    LayoutStyle::column().flex_shrink(0.0).margin(
                        if idx == 0 { 0.0 } else { 1.0 },
                        0.0,
                        0.0,
                        0.0,
                    ),
                )
                .background(BG_PANEL)
                .border_custom(BorderChars::split_left(), PRIMARY, BorderSides::left_only())
                .base_padding(1.0, 0.0, 1.0, 2.0),
            );
            for line in &lines {
                let line_id = tree.allocate_id();
                tree.add_child(
                    msg_id,
                    TextLineWidget::with_text(
                        line_id,
                        LayoutStyle::column().height(1.0).flex_shrink(0.0),
                        *line,
                    )
                    .fg(TEXT)
                    .bg(BG_PANEL),
                );
            }
        } else {
            tree.add_child(
                parent,
                BoxWidget::new(
                    msg_id,
                    LayoutStyle::column().flex_shrink(0.0).margin(
                        if idx == 0 { 0.0 } else { 1.0 },
                        0.0,
                        0.0,
                        0.0,
                    ),
                )
                .background(BG)
                .base_padding(0.0, 0.0, 0.0, 3.0),
            );
            for line in &lines {
                let line_id = tree.allocate_id();
                let is_tool = line.starts_with("  \u{25B8}") || line.starts_with("  \u{25CF}");
                tree.add_child(
                    msg_id,
                    TextLineWidget::with_text(
                        line_id,
                        LayoutStyle::column().height(1.0).flex_shrink(0.0),
                        *line,
                    )
                    .fg(if is_tool { TEXT_MUTED } else { TEXT })
                    .bg(BG),
                );
            }
        }
    }
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
    let dialog_x = w.saturating_sub(dialog_w) / 2;
    let dialog_y = h / 4;

    let filter_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let filter_placeholder = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let item_style = Style::builder().fg(TEXT).bg(BG_PANEL).build();
    let selected_style = Style::builder().fg(TEXT).bg(PRIMARY).bold().build();
    let shortcut_muted = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let shortcut_sel = Style::builder().fg(TEXT).bg(PRIMARY).build();

    let panel_bg = Style::builder().bg(BG_PANEL).build();
    for row in dialog_y..(dialog_y + dialog_h) {
        for col in dialog_x..(dialog_x + dialog_w) {
            buf.set(col, row, Cell::new(' ', panel_bg));
        }
    }

    let inner_x = dialog_x + 4;
    let inner_right = dialog_x + dialog_w - 4;

    let title_style = Style::builder().fg(TEXT).bg(BG_PANEL).bold().build();
    let esc_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    buf.draw_text(inner_x, dialog_y + 1, "Commands", title_style);
    buf.draw_text(inner_right - 3, dialog_y + 1, "esc", esc_style);

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

    let sep_style = Style::builder()
        .fg(Rgba::new(0.176, 0.176, 0.216, 1.0))
        .bg(BG_PANEL)
        .build();
    for col in 4..(dialog_w - 4) {
        buf.set(
            dialog_x + col,
            dialog_y + 3,
            Cell::new('\u{2500}', sep_style),
        );
    }

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

    let footer_y = dialog_y + dialog_h - 2;
    let footer_style = Style::builder().fg(TEXT_MUTED).bg(BG_PANEL).build();
    let footer_bg = Style::builder().bg(BG_PANEL).build();
    for col in (dialog_x + 1)..(dialog_x + dialog_w - 1) {
        buf.set(col, footer_y, Cell::new(' ', footer_bg));
    }
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
    renderer.set_title("OpenCode (Declarative)")?;
    renderer.set_background(BG);

    let app = Rc::new(RefCell::new(App::new(w)));
    let running = Arc::new(AtomicBool::new(true));
    let mut parser = InputParser::new();
    let stdin = io::stdin();
    let mut read_buf = [0u8; 1024];
    let mut pending: Vec<u8> = Vec::new();

    while running.load(Ordering::SeqCst) {
        let app_ref = app.clone();

        {
            let app_borrowed = app.borrow();
            let buffer = renderer.buffer();
            buffer.clear(Rgba::TRANSPARENT);

            let sidebar_w = if app_borrowed.sidebar_visible {
                SIDEBAR_WIDTH as u32
            } else {
                0
            };
            let main_w = w.saturating_sub(sidebar_w);

            let mut tree = WidgetTree::new();

            let root_id = tree.allocate_id();
            let main_id = tree.allocate_id();
            let msg_id = tree.allocate_id();
            let prompt_id = tree.allocate_id();
            let hint_id = tree.allocate_id();

            tree.add(
                BoxWidget::new(root_id, LayoutStyle::row().width(w as f32).height(h as f32))
                    .background(BG)
                    .overflow_hidden(),
            );

            tree.add_child(
                root_id,
                BoxWidget::new(main_id, LayoutStyle::column().flex_grow(1.0))
                    .background(BG)
                    .overflow_hidden(),
            );

            tree.add_child(
                main_id,
                BoxWidget::new(
                    msg_id,
                    LayoutStyle::column()
                        .flex_grow(1.0)
                        .padding(0.0, 2.0, 0.0, 2.0)
                        .gap(1.0),
                )
                .background(BG)
                .overflow_hidden(),
            );
            build_messages(&mut tree, msg_id, &app_borrowed);

            let prompt_box = tree.add_child(
                main_id,
                BoxWidget::new(prompt_id, LayoutStyle::column().flex_shrink(0.0)).background(BG),
            );
            build_prompt_rows(&mut tree, prompt_box, &app_borrowed);

            tree.add_child(
                main_id,
                StyledTextWidget::from_segments(
                    hint_id,
                    LayoutStyle::column()
                        .height(1.0)
                        .flex_shrink(0.0)
                        .padding(0.0, 0.0, 0.0, 2.0),
                    vec![
                        StyledSegment::new("tab ", TEXT),
                        StyledSegment::new("agents", TEXT_MUTED),
                        StyledSegment::new("  ctrl+p ", TEXT),
                        StyledSegment::new("commands", TEXT_MUTED),
                    ],
                ),
            );

            if app_borrowed.sidebar_visible {
                let sb_id = tree.allocate_id();
                let sb = tree.add_child(
                    root_id,
                    BoxWidget::new(
                        sb_id,
                        LayoutStyle::column()
                            .width(SIDEBAR_WIDTH)
                            .padding(0.0, 0.0, 0.0, 2.0),
                    )
                    .background(BG_PANEL),
                );
                build_sidebar(&mut tree, sb);
            }

            tree.layout(w as f32, h as f32);

            let mut ctx = RenderContext {
                buffer,
                grapheme_pool: None,
                link_pool: None,
                hit_grid: None,
                theme: None,
            };
            tree.render(&mut ctx);

            if app_borrowed.palette_open {
                draw_palette(buffer, &app_borrowed, w, h);
            }
        }

        drop(app_ref);
        renderer.present()?;

        if let Ok(n) = read_with_timeout(&stdin, &mut read_buf, Duration::from_millis(50)) {
            if n == 0 {
                if pending.len() == 1 && pending[0] == 0x1b {
                    let mut app_mut = app.borrow_mut();
                    handle_key_event(KeyCode::Escape, KeyModifiers::empty(), &mut app_mut, &running);
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
                                    match mouse.kind {
                                        MouseEventKind::Move => {
                                            app_mut.palette_mouse_mode = true;
                                            let dialog_w = 60_u32.min(w.saturating_sub(4));
                                            let dialog_x = w.saturating_sub(dialog_w) / 2;
                                            let dialog_y = h / 4;
                                            let list_y = dialog_y + 4;
                                            if mouse.x > dialog_x
                                                && mouse.x < dialog_x + dialog_w
                                                && mouse.y >= list_y
                                            {
                                                let row = (mouse.y - list_y) as usize;
                                                let indices = app_mut.filtered_indices();
                                                let idx = app_mut.palette_scroll + row;
                                                if idx < indices.len() {
                                                    app_mut.palette_selected = idx;
                                                }
                                            }
                                        }
                                        MouseEventKind::Press => {
                                            let indices = app_mut.filtered_indices();
                                            if let Some(&oi) = indices.get(app_mut.palette_selected)
                                            {
                                                let name = App::commands()[oi].name;
                                                app_mut.palette_open = false;
                                                app_mut.palette_filter.clear();
                                                match name {
                                                    "Toggle Sidebar" => {
                                                        app_mut.sidebar_visible =
                                                            !app_mut.sidebar_visible;
                                                    }
                                                    "New Session" => {
                                                        app_mut.messages.clear();
                                                        app_mut.input_text.clear();
                                                    }
                                                    "Quit" => {
                                                        running.store(false, Ordering::SeqCst);
                                                    }
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
                let indices = app.filtered_indices();
                if let Some(&oi) = indices.get(app.palette_selected) {
                    let name = App::commands()[oi].name;
                    app.palette_open = false;
                    app.palette_filter.clear();
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
        KeyCode::Enter => app.send_message(),
        KeyCode::Backspace => {
            app.input_text.pop();
        }
        KeyCode::Char(c) => {
            if app.input_text.len() < 200 {
                app.input_text.push(c);
            }
        }
        _ => {}
    }
}
*/
