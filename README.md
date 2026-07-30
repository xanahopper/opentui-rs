# OpenTUI Rust

[![License: MIT](https://img.shields.io/badge/License-MIT%2BOpenAI%2FAnthropic%20Rider-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Edition](https://img.shields.io/badge/edition-2024-purple.svg)](https://doc.rust-lang.org/edition-guide/)
[![codecov](https://codecov.io/gh/xanahopper/opentui-rs/graph/badge.svg)](https://codecov.io/gh/xanahopper/opentui-rs)

OpenTUI Rust is a native Rust terminal UI rendering engine inspired by
[anomalyco/opentui](https://github.com/anomalyco/opentui). It combines the
low-level rendering primitives of the original Zig core with Rust-native
layout, widgets, input parsing, text editing, and an optional declarative View
API.

This is an independent Rust implementation, not a binding to the upstream Zig
or TypeScript packages. Applications use Cargo only; Zig, Bun, Node.js, React,
and Solid are not required.

```bash
cargo add opentui-core
```

---

## What It Provides

- A low-level `OptimizedBuffer` API for applications that want direct control
- A render tree with Taffy flexbox/grid/block layout
- A declarative `view()` / `text()` builder API with `ViewRuntime`
- Built-in widgets, including text, editor, input, list, select, tabs, slider,
  scrollbar, checkbox, radio group, gauge, spinner, progress, and status line
- Cell-based buffers with **real RGBA alpha blending**
- **Scissor clipping** for nested viewports
- **Double-buffered rendering** with diff detection (only changed cells update)
- **Rope-based text editing** with undo/redo
- Unicode grapheme, width, bidi, normalization, wrapping, and selection support
- Raw keyboard, mouse, paste, focus, and terminal response parsing
- Renderable mouse dispatch with bubbling, hover, drag capture, drop, and
  independent `stop_propagation` / `prevent_default` controls
- OSC 8 hyperlinks, capability detection, synchronized output, and threaded
  rendering

OpenTUI Rust does not impose an application runtime. You own the event loop and
state model, and can choose between direct buffer drawing, imperative widgets,
or the declarative View layer.

## Differences From Upstream OpenTUI

| Area | OpenTUI Rust | Upstream OpenTUI |
|------|--------------|------------------|
| Runtime | Native Rust crate | Zig core with TypeScript bindings |
| Toolchain | Cargo + Rust nightly | Bun/Node.js + Zig |
| UI APIs | Buffers, render tree, widgets, declarative builders | Imperative core plus React/Solid reconcilers |
| Layout | Taffy flexbox, grid, and block layout | Upstream layout engine |
| Text storage | Rope-backed Rust text/edit buffers | Upstream text/renderable model |
| Input | Rust byte parser and render-tree dispatch | TypeScript renderer dispatch |
| Ecosystem packages | One `opentui-core` crate | `@opentui/core`, React, Solid, Three.js, examples |
| API compatibility | Rust-native; not source-compatible | Canonical TypeScript API |
| Maturity | API under active development | Powers OpenCode in production |

---

## Quick Example

```rust
use opentui_core::{Renderer, Rgba, Style};

fn main() -> std::io::Result<()> {
    let mut renderer = Renderer::new(80, 24)?;

    renderer.clear();
    renderer
        .buffer()
        .draw_text(10, 5, "Hello, OpenTUI!", Style::fg(Rgba::GREEN));

    renderer.present()?;
    Ok(())
}
```

### Declarative View

The optional View API builds a render tree backed by Taffy layout. Rebuild it
from application state, run layout, render through `ViewRuntime`, and dispatch
parsed key or mouse events back into the runtime.

```rust
use opentui_core::Rgba;
use opentui_core::view::{Node, text, view};

fn ui(count: u32) -> Node {
    view()
        .column()
        .size_pct(1.0, 1.0)
        .padding_all(2.0)
        .gap(1.0)
        .children([
            text("OpenTUI Rust").bold().height(1.0).build(),
            text(format!("Count: {count}"))
                .fg(Rgba::GREEN)
                .height(1.0)
                .on_mouse_over("counter:hover")
                .on_mouse_out("counter:out")
                .build(),
        ])
        .build()
}
```

See [`declarative_hello.rs`](crates/core/examples/declarative_hello.rs) for a
complete render/input loop and
[`opencode_view.rs`](crates/core/examples/opencode_view.rs) for a larger stateful
application with layout, overlays, mouse hover, drag selection, and actions.

### Alpha Blending

```rust
use opentui_core::{Cell, Rgba, Style};

// 50% transparent red over blue background
let bg = Rgba::BLUE;
let overlay = Rgba::RED.with_alpha(0.5);

buffer.clear(bg);
buffer.set_blended(10, 5, Cell::new('X', Style::fg(overlay)));
// Result: purple-ish cell with proper Porter-Duff compositing
```

### Scissor Clipping

```rust
use opentui_core::buffer::ClipRect;

// Only draw within this rectangle
buffer.push_scissor(ClipRect::new(10, 10, 20, 10));

// This text is clipped to the scissor rect
buffer.draw_text(0, 0, "This won't appear outside the clip region", Style::default());

buffer.pop_scissor();
```

### Opacity Stacks

```rust
// Everything drawn at 50% opacity
buffer.push_opacity(0.5);
buffer.draw_text(0, 0, "Semi-transparent", Style::fg(Rgba::WHITE));
buffer.pop_opacity();
```

---

## Design Philosophy

### 1. Layered APIs, Not A Forced Runtime

OpenTUI Rust provides buffers, render trees, widgets, and declarative builders,
but does not own your event loop or application state. Use only the layers your
application needs.

### 2. Correctness Over Convenience

- **Real alpha blending** using Porter-Duff "over" compositing
- **Proper grapheme handling** via `unicode-segmentation`
- **Accurate character widths** via `unicode-width`
- **Immutable rope** for text that doesn't corrupt on edits

### 3. Performance by Default

- **Diff rendering**: Only changed cells generate ANSI output
- **Synchronized output**: Uses `\x1b[?2026h` to eliminate flicker
- **Zero allocations** on hot paths (cell updates, blending)
- **SIMD-friendly** memory layout (contiguous cell arrays)

### 4. Terminal Respect

- Automatic cleanup on drop (restores cursor, exits alt screen)
- Proper mouse protocol handling
- True color support with graceful fallback
- Works in SSH, tmux, and embedded terminals

### 5. Rust-Native Implementation

The rendering model follows upstream OpenTUI, while the public API, text
storage, layout integration, input parser, and tests are implemented in Rust.
This project does not claim production parity with upstream OpenTUI.

---

## Comparison

| Library | Abstraction | Alpha | Scissor | Diff | Text Edit | Use Case |
|---------|-------------|-------|---------|------|-----------|----------|
| **OpenTUI Rust** | Layered rendering engine | Yes | Yes | Yes | Yes | Custom Rust TUIs |
| ratatui | Widget framework | No | No | Partial | No | Standard TUIs |
| crossterm | Terminal I/O | No | No | No | No | Low-level control |
| termion | Terminal I/O | No | No | No | No | Low-level control |
| cursive | Dialog framework | No | No | Yes | Partial | Form-based apps |
| tui-rs | Widget framework | No | No | Partial | No | Dashboards |

**Choose OpenTUI Rust when you need:**
- Compositing layers with transparency
- Pixel-perfect control over rendering
- High-performance text editing
- Direct buffers and an optional widget/View layer in one crate

**Choose ratatui when you need:**
- Quick prototyping with widgets
- Standard TUI patterns (tables, lists, tabs)
- Large community and examples

---

## Installation

### From crates.io

```bash
cargo add opentui-core
```

### From Source

```bash
git clone https://github.com/xanahopper/opentui-rs
cd opentui-rs
cargo build --release
```

### Cargo.toml

```toml
[dependencies]
opentui-core = "0.2.1"
```

---

## Quick Start

### 1. Create a Renderer

```rust
use opentui_core::Renderer;
use std::io;

fn main() -> io::Result<()> {
    // Automatically enters alt screen, hides cursor, enables mouse
    let mut renderer = Renderer::new(80, 24)?;

    // Your rendering loop here...

    Ok(())
    // Renderer::drop() cleans up automatically
}
```

### 2. Draw to the Buffer

```rust
use opentui_core::{Rgba, Style};
use opentui_core::buffer::BoxStyle;

let buffer = renderer.buffer();

// Clear with background
buffer.clear(Rgba::from_hex("#1a1a2e").unwrap());

// Draw styled text
buffer.draw_text(5, 2, "Title", Style::bold().fg(Rgba::WHITE));
buffer.draw_text(5, 4, "Normal text", Style::fg(Rgba::from_hex("#888888").unwrap()));

// Draw a box
buffer.draw_box(2, 1, 40, 10, BoxStyle::single());
```

### 3. Present Frame

```rust
// Diff-based update (fast)
renderer.present()?;

// Force full redraw
renderer.invalidate();
renderer.present()?;
```

### 4. Parse Input

OpenTUI Rust includes an ANSI input parser but leaves polling and the event loop
to the application:

```rust
use opentui_core::{Event, InputParser, KeyCode};

let mut parser = InputParser::new();
let bytes = b"q"; // Read bytes from stdin in your event loop.

if let Ok((Event::Key(key), consumed)) = parser.parse(bytes) {
    if key.code == KeyCode::Char('q') {
        // Exit the application loop.
    }
    assert_eq!(consumed, 1);
}
```

---

## Demo Showcase

The `demo_showcase` binary demonstrates OpenTUI's full capability set in an interactive terminal application.

### Running the Demo

```bash
# Interactive mode (explore with keyboard/mouse)
cargo run --bin demo_showcase

# Guided tour mode (auto-plays through features)
cargo run --bin demo_showcase -- --tour

# Tour mode with auto-exit (for scripting/CI)
cargo run --bin demo_showcase -- --tour --exit-after-tour
```

### What It Demonstrates

The demo showcases every major OpenTUI feature:

- **Alpha blending** — Glass-like overlays, semi-transparent panels
- **Scissor clipping** — Nested scroll regions, viewport masking
- **Opacity stacks** — Hierarchical transparency
- **Diff rendering** — Only changed cells update (watch the stats panel)
- **Grapheme handling** — CJK, emoji, ZWJ sequences rendered correctly
- **OSC 8 hyperlinks** — Clickable URLs in supported terminals
- **Hit testing** — Mouse hover/click detection
- **Pixel buffers** — Animated graphics using block characters

### Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus between panels |
| `↑/↓` | Navigate sidebar sections |
| `Enter` | Select section |
| `F1` | Toggle help overlay |
| `Ctrl+P` | Command palette |
| `Ctrl+D` | Debug/inspector panel |
| `T` | Start/restart tour |
| `Esc` | Close overlay / exit tour |
| `Q` | Quit |

### CLI Flags

**Interactive:**

| Flag | Description |
|------|-------------|
| `--fps <N>` | Target frame rate (default: 60) |
| `--tour` | Start in guided tour mode |
| `--exit-after-tour` | Exit when tour completes |
| `--max-frames <N>` | Hard frame limit (safety bound) |
| `--seed <N>` | Random seed for deterministic behavior |
| `--threaded` | Use threaded renderer |

**Headless/Testing:**

| Flag | Description |
|------|-------------|
| `--headless-smoke` | Run without TTY (for CI) |
| `--headless-dump-json` | Output frame stats as JSON |
| `--headless-size <WxH>` | Set virtual terminal size (e.g., `80x24`) |

**Terminal Behavior:**

| Flag | Description |
|------|-------------|
| `--no-mouse` | Disable mouse tracking |
| `--no-alt-screen` | Don't use alternate screen buffer |
| `--no-cap-queries` | Skip terminal capability detection |
| `--cap-preset <name>` | Force capability preset (`minimal`, `no_hyperlinks`, etc.) |

### Recommended Terminals

For the best visual experience, use a terminal that supports:

- **True color** (24-bit RGB)
- **Synchronized output** (eliminates flicker)
- **OSC 8 hyperlinks** (clickable URLs)
- **Unicode** (grapheme clusters, emoji)

**Recommended:** kitty, WezTerm, Ghostty, Alacritty, iTerm2

### Verification Scripts

```bash
# Quick validation (format + clippy + tests)
./scripts/demo_showcase_verify.sh

# Fast check (skip headless tests)
./scripts/demo_showcase_verify.sh --quick

# Run PTY E2E tests with artifact collection
./scripts/demo_showcase_e2e_pty.sh
```

---

## API Reference

### Core Types

| Type | Purpose |
|------|---------|
| `Rgba` | RGBA color with f32 components, alpha blending |
| `Style` | Foreground, background, and text attributes |
| `TextAttributes` | Bold, italic, underline, etc. (bitflags) |
| `Cell` | Single terminal cell (char + colors + attributes) |
| `CellContent` | Char, grapheme cluster, empty, or continuation |
| `GraphemeId` | Packed grapheme ID + width encoding for cells |
| `GraphemePool` | Interned grapheme storage with ref counting |
| `LinkPool` | Hyperlink URL storage for OSC 8 output |

### Buffer Operations

| Method | Description |
|--------|-------------|
| `OptimizedBuffer::new(w, h)` | Create buffer with dimensions |
| `buffer.set(x, y, cell)` | Write cell (respects scissor/opacity) |
| `buffer.set_blended(x, y, cell)` | Write with alpha blending |
| `buffer.get(x, y)` | Read cell at position |
| `buffer.clear(bg)` | Fill entire buffer |
| `buffer.fill_rect(x, y, w, h, bg)` | Fill rectangle |
| `buffer.draw_text(x, y, text, style)` | Draw UTF-8 string |
| `buffer.draw_box(x, y, w, h, style)` | Draw box border |
| `buffer.draw_buffer(x, y, src)` | Composite another buffer |
| `buffer.push_scissor(rect)` | Push clipping rectangle |
| `buffer.pop_scissor()` | Pop clipping rectangle |
| `buffer.push_opacity(f32)` | Push opacity multiplier |
| `buffer.pop_opacity()` | Pop opacity multiplier |

### Renderer Operations

| Method | Description |
|--------|-------------|
| `Renderer::new(w, h)` | Create renderer, setup terminal |
| `renderer.buffer()` | Get back buffer for drawing |
| `renderer.present()` | Swap buffers, render diff |
| `renderer.present_force()` | Force full redraw |
| `renderer.resize(w, h)` | Handle terminal resize |
| `renderer.set_cursor(x, y, visible)` | Position/show cursor |
| `renderer.set_title(title)` | Set terminal title |
| `renderer.register_hit_area(...)` | Register mouse hit zone |
| `renderer.hit_test(x, y)` | Test mouse position |

### Threaded Renderer

Use the threaded renderer when you want terminal I/O off the main thread:

```rust
use opentui_core::renderer::ThreadedRenderer;

let mut renderer = ThreadedRenderer::new(80, 24)?;
renderer.buffer().draw_text(1, 1, "Threaded!", Style::fg(Rgba::GREEN));
renderer.present()?;
renderer.shutdown()?;
```

### Grapheme Pools and Hyperlinks

Use the grapheme pool for multi-codepoint graphemes so they can be resolved
back to their full UTF-8 sequence during rendering:

```rust
let (buffer, pool) = renderer.buffer_with_pool();
let grapheme = "\u{0061}\u{0301}"; // "a" + combining acute accent
buffer.draw_text_with_pool(pool, 0, 0, grapheme, Style::fg(Rgba::WHITE));
```

Hyperlinks are stored in a link pool and referenced by link ID in text styles:

```rust
let link_id = renderer.link_pool().alloc("https://example.com");
let style = Style::fg(Rgba::BLUE).with_underline().with_link(link_id);
renderer.buffer().draw_text(0, 1, "example.com", style);
```

### Color Operations

```rust
// Creation
Rgba::new(1.0, 0.0, 0.0, 1.0)  // f32 RGBA
Rgba::rgb(1.0, 0.0, 0.0)       // f32 RGB (opaque)
Rgba::from_rgb_u8(255, 0, 0)   // u8 RGB
Rgba::from_hex("#FF0000")      // Hex string
Rgba::from_hsv(0.0, 1.0, 1.0)  // HSV (h: 0-360)

// Operations
color.blend_over(other)         // Porter-Duff "over"
color.with_alpha(0.5)           // Set alpha
color.multiply_alpha(0.5)       // Multiply alpha
color.lerp(other, 0.5)          // Linear interpolation
color.to_rgb_u8()               // Convert to (u8, u8, u8)
```

### Text Module

| Type | Purpose |
|------|---------|
| `TextBuffer` | Styled text storage with rope backend |
| `TextBufferView` | Viewport with wrapping and selection |
| `EditBuffer` | Editable text with cursor and undo/redo |
| `WrapMode` | None, Char, or Word wrapping |
| `HighlightedBuffer` | Text buffer wrapper with syntax highlighting |
| `SyntaxStyleRegistry` | Style registry for token kinds |
| `TokenizerRegistry` | Language tokenizer registry |
| `Theme` | TokenKind → Style mapping |

---

## Architecture

```
                    ┌─────────────────────────────────────┐
                    │           Your Application          │
                    └─────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              OpenTUI                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │   Renderer   │───▶│    Buffer    │◀───│    Text      │                  │
│  │              │    │              │    │              │                  │
│  │ • Double buf │    │ • Cells      │    │ • Rope       │                  │
│  │ • Diff detect│    │ • Scissor    │    │ • Segments   │                  │
│  │ • Hit grid   │    │ • Opacity    │    │ • Highlights │                  │
│  │ • Sync output│    │ • Drawing    │    │ • Edit/Undo  │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│         │                   │                   │                          │
│         ▼                   ▼                   ▼                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │   Terminal   │    │    Cell      │    │   Unicode    │                  │
│  │              │    │              │    │              │                  │
│  │ • ANSI codes │    │ • Char/Graph │    │ • Graphemes  │                  │
│  │ • Mouse      │    │ • Style      │    │ • Width calc │                  │
│  │ • Cursor     │    │ • Blending   │    │ • Segmentat. │                  │
│  └──────────────┘    └──────────────┘    └──────────────┘                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
                    ┌─────────────────────────────────────┐
                    │         stdout (ANSI TTY)           │
                    └─────────────────────────────────────┘
```

### Module Breakdown

```
opentui-rs/
├── Cargo.toml       # Workspace configuration
└── crates/core/
    ├── Cargo.toml   # opentui-core package
    └── src/
├── lib.rs          # Public API exports
├── color.rs        # RGBA type, blending, conversions
├── style.rs        # TextAttributes, Style builder
├── cell.rs         # Cell type, CellContent enum
├── grapheme_pool.rs# Grapheme pool + ID encoding
├── link.rs         # Hyperlink pool (OSC 8)
├── ansi/           # ANSI escape sequence generation
│   ├── mod.rs
│   ├── sequences.rs
│   └── output.rs   # Buffered ANSI writer
├── buffer/         # OptimizedBuffer
│   ├── mod.rs
│   ├── scissor.rs  # ClipRect, ScissorStack
│   ├── opacity.rs  # OpacityStack
│   └── drawing.rs  # Text/box drawing
├── text/           # Text editing
│   ├── mod.rs
│   ├── rope.rs     # Rope wrapper (ropey)
│   ├── segment.rs  # StyledSegment
│   ├── buffer.rs   # TextBuffer
│   ├── view.rs     # TextBufferView
│   ├── edit.rs     # EditBuffer
│   └── editor.rs   # EditorView
├── renderer/       # Display rendering
│   ├── mod.rs
│   ├── diff.rs     # Buffer diffing
│   ├── hitgrid.rs  # Mouse hit testing
│   └── threaded.rs # Threaded renderer
├── terminal/       # Terminal abstraction
│   ├── mod.rs
│   ├── capabilities.rs
│   ├── cursor.rs
│   └── mouse.rs
├── input/          # Keyboard, mouse, paste, focus, and response parser
├── renderable/     # Render tree, Taffy layout, View API, and widgets
│   ├── view/       # Declarative builders and ViewRuntime
│   └── widgets/    # Concrete widget implementations
├── unicode/        # Unicode handling
│   ├── mod.rs
│   ├── grapheme.rs
│   └── width.rs
└── highlight/      # Syntax highlighting
    ├── mod.rs
    ├── highlighted_buffer.rs
    ├── languages/  # Language tokenizers
    ├── syntax.rs
    ├── theme.rs
    ├── token.rs
    └── tokenizer.rs
```

---

## Troubleshooting

### Terminal doesn't restore after crash

If your program panics, the terminal may be left in a bad state:

```bash
# Reset terminal
reset
# Or
stty sane
```

`Renderer`, `Terminal`, and `RawModeGuard` restore managed terminal state when
they are dropped during normal return or panic unwinding. Applications that use
`panic = "abort"` should install a panic hook appropriate to their process and
terminal lifecycle.

### Characters display with wrong width

Some terminals report incorrect widths for certain Unicode characters. Try:

```rust
// Use wcwidth-based calculation (POSIX compatible)
unicode::set_width_method(WidthMethod::WcWidth);

// Or Unicode Standard Annex #11 (more accurate for CJK)
unicode::set_width_method(WidthMethod::Unicode);
```

### Flickering on slow terminals

OpenTUI uses synchronized output (`\x1b[?2026h`) which most modern terminals support. If you see flicker:

1. Update your terminal emulator
2. Try a different terminal (kitty, alacritty, wezterm)
3. Reduce frame rate

### Colors look wrong

Ensure your terminal supports true color:

```bash
echo $COLORTERM  # Should be "truecolor" or "24bit"
```

If not supported, colors will be approximated to 256-color palette.

### High CPU usage

Check that you're not calling `present()` in a tight loop:

```rust
// Bad: spins CPU
loop {
    renderer.present()?;
}

// Good: wait for events
loop {
    renderer.present()?;
    if !event::poll(Duration::from_millis(16))? {
        continue;  // ~60 FPS max
    }
    // Handle event...
}
```

---

## Limitations

- **No built-in event loop**: input parsing and dispatch are included, but the
  application owns byte polling, scheduling, and state updates
- **Nightly toolchain currently pinned**: see `rust-toolchain.toml`; the crate
  declares Rust 1.85 as its minimum language version
- **Unix terminal backend**: raw mode currently uses `libc` termios and Unix
  file descriptors; native Windows support is not implemented
- **Not API-compatible with upstream**: upstream TypeScript, React, Solid, and
  Three.js code cannot be used directly
- **API stability**: the Rust widget and declarative APIs are under active
  development
- **Text-only**: No image protocols (sixel, kitty graphics) yet

---

## FAQ

**Q: Why not just use ratatui?**

A: ratatui is excellent for standard TUI patterns. OpenTUI Rust is useful when
you need true RGBA composition, precise clipping, editable rope-backed text, or
both direct-buffer and retained/declarative rendering in the same engine.

**Q: Is this production-ready?**

A: The rendering core has broad unit, integration, property, snapshot, and PTY
coverage, but the Rust API is still stabilizing. Pin your version and expect
some churn.

**Q: Why f32 for colors instead of u8?**

A: Alpha blending math is more accurate with floats. Final output converts to u8 for ANSI codes. The performance difference is negligible.

**Q: Can I use this with async?**

A: Yes, but `Renderer` isn't `Send`. Keep it on one thread and send drawing commands via channels.

**Q: Why require nightly Rust?**

A: The repository currently pins nightly so contributors and CI use one known
toolchain with the required rustfmt and clippy components. Edition 2024 itself
is stable in Rust 1.85.

**Q: How do I handle terminal resize?**

A: Detect `SIGWINCH` or consume a parsed `Event::Resize`, then call
`renderer.resize(width, height)` and rerun layout before rendering.

---

## About Contributions

Bug reports and focused pull requests are welcome. Before submitting a change,
run `cargo fmt --check`, `cargo check --all-targets`,
`cargo clippy --all-targets -- -D warnings`, and the relevant tests.

---

## License

MIT License with OpenAI/Anthropic Rider. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

- [OpenTUI](https://github.com/anomalyco/opentui) for the original rendering
  model and ongoing reference behavior
- [Taffy](https://github.com/DioxusLabs/taffy) for layout
- [ropey](https://crates.io/crates/ropey) for the rope data structure
- [unicode-segmentation](https://crates.io/crates/unicode-segmentation) for grapheme clustering
- [unicode-width](https://crates.io/crates/unicode-width) for display width calculation
