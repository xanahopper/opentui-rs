//! Core Dashboard Example
//!
//! Demonstrates the opentui-core crate with:
//! - `BoxWidget` containers with borders and titles
//! - `TextWidget` for displaying text
//! - Flexbox layout via Taffy
//! - Focus management with Tab/Shift+Tab
//! - Theme-aware rendering

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]

use std::io::{self, Read};

use opentui_rust::input::{Event, InputParser, KeyCode};
use opentui_rust::terminal::{enable_raw_mode, terminal_size};
use opentui_rust::{Renderer, Rgba};

use opentui_core::layout::LayoutStyle;
use opentui_core::theme::UiTheme;
use opentui_core::widget::{RenderContext, WidgetTree};
use opentui_core::widgets::{BoxWidget, TextWidget};

fn main() -> io::Result<()> {
    let (width, height) = terminal_size().unwrap_or((80, 24));
    let mut renderer = Renderer::new(u32::from(width), u32::from(height))?;
    let _raw_guard = enable_raw_mode()?;

    let theme = UiTheme::dark_default();

    let (w, h) = renderer.size();

    // Build widget tree
    let mut tree = WidgetTree::new();

    // Root: full screen, column layout, dark background
    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(w as f32).height(h as f32))
            .background(theme.background),
    );

    // Header: 1 row
    let header = tree.add_child(
        root,
        BoxWidget::new(2, LayoutStyle::row().width(w as f32).height(1.0))
            .background(theme.background_panel),
    );
    let _header_title = tree.add_child(
        header,
        TextWidget::with_text(
            3,
            LayoutStyle::default().flex_grow(1.0),
            " OpenTUI Core Dashboard ",
        ),
    );

    // Body: fills remaining space, row layout
    let body = tree.add_child(
        root,
        BoxWidget::new(4, LayoutStyle::row().flex_grow(1.0))
            .border_rounded(theme.border)
            .border_focused_color(theme.border_active)
            .title("Panels")
            .overflow_hidden()
            .focusable(),
    );

    // Left panel
    let left = tree.add_child(
        body,
        BoxWidget::new(
            5,
            LayoutStyle::column()
                .width((w as f32 * 0.4).round())
                .flex_shrink(0.0),
        )
        .border_rounded(theme.border_subtle)
        .title("Files")
        .background(theme.background_panel),
    );
    let _file_list = tree.add_child(
        left,
        TextWidget::with_text(
            6,
            LayoutStyle::default().flex_grow(1.0),
            "  Cargo.toml\n  src/lib.rs\n  src/main.rs\n  README.md\n",
        ),
    );

    // Right panel
    let right = tree.add_child(
        body,
        BoxWidget::new(7, LayoutStyle::column().flex_grow(1.0))
            .border_rounded(theme.border_subtle)
            .title("Editor")
            .background(theme.background_panel)
            .focusable(),
    );
    let _editor_text = tree.add_child(
        right,
        TextWidget::with_text(
            8,
            LayoutStyle::default().flex_grow(1.0),
            "use opentui_core::prelude::*;\n\nfn main() {\n    println!(\"Hello!\");\n}\n",
        ),
    );

    // Status line: 1 row
    let status = tree.add_child(
        root,
        BoxWidget::new(9, LayoutStyle::row().width(w as f32).height(1.0))
            .background(theme.background_element),
    );
    let _status_text = tree.add_child(
        status,
        TextWidget::with_text(
            10,
            LayoutStyle::default().flex_grow(1.0),
            " Tab: switch focus | q: quit ",
        ),
    );

    // Build focus chain and focus the body panel
    tree.build_focus_chain();
    tree.set_focused_widget(Some(body));

    let mut input_parser = InputParser::new();
    let mut stdin = io::stdin();

    loop {
        // Layout
        tree.layout(w as f32, h as f32);

        // Render
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

        // Input
        let mut buf = [0u8; 64];
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            continue;
        }

        let mut offset = 0usize;
        while offset < n {
            let Ok((event, used)) = input_parser.parse(&buf[offset..n]) else {
                break;
            };
            offset += used;

            if let Event::Key(key) = event {
                if key.code == KeyCode::Char('q') || key.is_ctrl_c() {
                    return Ok(());
                }
                tree.dispatch_key(&key);
            }
        }
    }
}
