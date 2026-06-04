# Unified TextWidget Implementation Plan

## Overview

Consolidate `TextWidget`, `TextLineWidget`, and `StyledTextWidget` into a single unified `TextWidget` that handles all text rendering scenarios: single-line, multi-line, plain, and styled.

## Design Decisions

1. **TextWidget is the sole text widget** — absorbs TextLineWidget + StyledTextWidget
2. **No padding/margin** — box model is the container/layout layer's responsibility (Taffy)
3. **BgFill three modes**: `None` (transparent) / `Text` (text cells only) / `Block` (entire rect)
4. **StyledSegment retained** as builder input format for `rich_text()` DSL
5. **Highlights stored in TextProps** as `Vec<(usize, usize, Style)>` — `from_element()` converts to `TextBuffer::add_highlight()`
6. **ElementKind::StyledText removed** — `rich_text()` internally generates `ElementKind::Text`
7. **Imperative API preserved** (`TextWidget::with_text()`) for backward compatibility with existing examples

## Architecture

### Data Flow

```
User Code                    Builder Layer              Props / Rebuild           TextWidget
─────────                    ─────────────              ──────────────           ──────────

text("hello")           →   ElementBuilder             →  TextProps {           → TextBuffer
  .fg(WHITE)                  kind: Text                   content: "hello",       + default_style
  .bold()                     text_content: "hello"        fg: WHITE,              from highlights
  .wrap(Word)                                              bold: true,
                                                           wrap: Word,
                                                           highlights: []
                                                         }

rich_text([              →   ElementBuilder             →  TextProps {           → TextBuffer
  span("hello ", W),          kind: Text                   content: "hello world", + default_style(W)
  span("world", RED).bold()                                highlights: [           + add_highlight(6..11, RED+bold)
])                                                           (6, 11, RED+bold)
                                                         ]
                                                         }
```

### TextWidget Internal Structure

```rust
pub struct TextWidget {
    id: WidgetId,
    style: LayoutStyle,
    buffer: TextBuffer,       // rope-backed text storage
    wrap: WrapMode,           // None (default), Char, Word
    bg_fill: BgFill,          // None, Text, Block
    bg_color: Option<Rgba>,
    overflow: Overflow,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}
```

### Render Logic

```
1. if w == 0 || h == 0: return
2. if bg_fill == Block: ctx.buffer.fill_rect(x, y, w, h, bg_color)
3. if bg_fill == Text:  set bg in buffer's default_style (TextBufferView applies per-cell)
4. Create TextBufferView::new(&buffer).viewport(0, 0, w, h).wrap_mode(self.wrap)
5. If grapheme_pool available: view.render_to_with_pool(...)
   Else: view.render_to(...)
```

### BgFill Modes

| Mode | Effect | Implementation | Use Case |
|------|--------|----------------|----------|
| `None` | No bg fill | default_style has no bg | Overlay text, transparent layers |
| `Text` | Only cells with content get bg | Set default_style.bg | Paragraph text, inline styled text |
| `Block` | Fill entire allocated rect | `fill_rect()` then render text | Sidebar rows, list items, tabs |

### Intrinsic Size

- `WrapMode::None`: `(max_line_display_width, line_count)`
- `WrapMode::Word/Char`: Falls back to unwrapped dimensions (available width unknown at measure time)

## Files to Change

### Core Pipeline (Phase A — sequential, has dependencies)

| # | File | Action |
|---|------|--------|
| 1 | `crates/core/src/view/props.rs` | Add `BgFill` enum, extend `TextProps` (wrap, bg_fill, highlights), remove `StyledTextProps` and `Props::StyledText` |
| 2 | `crates/core/src/view/element.rs` | Remove `ElementKind::StyledText` |
| 3 | `crates/core/src/widgets/text_widget.rs` | Complete rewrite — unified TextWidget |
| 4 | `crates/core/src/view/builder.rs` | Update `rich_text()` to compute highlights, add `.wrap()`, `.bg_fill()`, move `StyledSegment` definition here |
| 5 | `crates/core/src/view/rebuild.rs` | `ElementKind::Text` → `TextWidget::from_element()` |
| 6 | `crates/core/src/widgets/mod.rs` | Remove `text_line_widget` and `styled_text_widget` modules/exports |
| 7 | Delete `text_line_widget.rs` and `styled_text_widget.rs` | Physical file deletion |
| 8 | `crates/core/src/prelude.rs` | Update exports |

### Migration (Phase B — parallel)

| # | File | Changes |
|---|------|---------|
| 9 | `examples/opencode_declarative.rs` | TextLineWidget → TextWidget (10 places), StyledTextWidget → TextWidget (2 places) |
| 10 | `examples/core_dashboard.rs` | Import update |
| 11 | `examples/focus_demo.rs` | Import update |
| 12 | `examples/overlay_demo.rs` | Import update |
| 13 | `examples/widgets_showcase.rs` | Import update |
| 14 | `tests/integration.rs` | Import update |
| 15 | Other files using `text()`/`rich_text()` builders | Verify compatibility |

### Verification (Phase C)

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo test -p opentui-core
```

## Detailed Changes Per File

### props.rs

```rust
// ADD:
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BgFill {
    #[default]
    None,
    Text,
    Block,
}

// MODIFY TextProps — add fields:
pub wrap: WrapMode,                              // default: WrapMode::None
pub bg_fill: BgFill,                             // default: BgFill::None
pub highlights: Vec<(usize, usize, Style)>,      // default: vec![]

// REMOVE:
// - StyledTextProps struct
// - Props::StyledText variant
```

### element.rs

```rust
// REMOVE:
// StyledText variant
pub enum ElementKind {
    View,
    Text,
    // StyledText,  ← REMOVED
    Input,
    List,
    Fill,
    Separator,
    Custom(&'static str),
}
```

### text_widget.rs — New Unified Widget

**Constructors:**
- `new(id: WidgetId, style: LayoutStyle) -> Self` — empty buffer
- `with_text(id: WidgetId, style: LayoutStyle, text: &str) -> Self` — imperative
- `from_element(id: WidgetId, elem: &Element) -> Self` — declarative

**from_element logic:**
1. Extract `TextProps` from `elem.props`
2. Create `TextBuffer::with_text(&props.content)`
3. Build `default_style` from props.fg/bg/bold/italic/underline
4. Call `buffer.set_default_style(default_style)`
5. For each `(start, end, style)` in props.highlights: `buffer.add_highlight(start..end, style, 0)`
6. Set wrap and bg_fill from props

**Imperative style setters (for command-style usage):**
- `fg(color)`, `bg(color)`, `bold()`, `italic()`, `underline()` — rebuild default_style on buffer
- `wrap(mode)`, `bg_fill(mode)` — set modes

### builder.rs

**StyledSegment stays here** (builder DSL input format):
```rust
pub struct StyledSegment {
    pub text: String,
    pub fg: Rgba,
    pub bg: Option<Rgba>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}
```

**rich_text() implementation:**
1. Concatenate all segment texts → content string
2. Track byte offsets per segment
3. Use first segment's style as base (or first explicitly set default)
4. For each subsequent segment: compute style diff → `(byte_start, byte_end, diff_style)`
5. Build `TextProps { content, highlights, ... }`

**New builder methods on ElementBuilder:**
```rust
pub fn wrap(mut self, mode: WrapMode) -> Self    // sets TextProps.wrap
pub fn bg_fill(mut self, mode: BgFill) -> Self   // sets TextProps.bg_fill
```

**Updated ElementBuilder::new():**
- Remove `ElementKind::StyledText` branch
- `ElementKind::Text` initializes `TextProps` with `wrap: WrapMode::None, bg_fill: BgFill::None, highlights: vec![]`

**Updated ElementBuilder::build():**
- `Props::Text` branch: respect wrap/bg_fill
- Remove `Props::StyledText` branch

### rebuild.rs

```rust
// Only one text branch:
ElementKind::Text => Box::new(TextWidget::from_element(id, elem)),

// REMOVE: ElementKind::StyledText branch
```

Import: remove `StyledTextWidget`, `TextLineWidget`; add `TextWidget`.

## Key Relationships

### TextBuffer Highlight Model

`TextBuffer` supports per-range style overlays via `add_highlight(range, style, priority)`:
- `default_style` applies to all text
- `style_at(pos)` merges: default_style + all overlapping highlights (by priority)
- This is more powerful than segment model — supports overlapping, priority, dynamic add/remove

### StyledSegment → Highlights Conversion

```
Input:
  span("hello ", WHITE)
  span("world", RED).bold()
  span("!", GREEN)

Processing:
  content = "hello world!"       (concatenated)
  byte offsets: [0, 6, 11]
  first segment style → default (WHITE)
  segment 2 diff: fg=RED, bold=true → highlight (6, 11, RED+bold)
  segment 3 diff: fg=GREEN → highlight (11, 12, GREEN)

Output:
  TextProps {
    content: "hello world!",
    fg: WHITE,               // default
    highlights: [(6, 11, RED+bold), (11, 12, GREEN)]
  }
```

## Notes

- TextWidget does NOT own padding/margin — this is handled by Taffy layout nodes (containers)
- The `ComputedLayout` passed to `render()` is the node's content area (Taffy already subtracted parent padding)
- For block bg fill: `fill_rect()` fills the allocated rect before rendering text
- TextBufferView has a line_cache that's valid as long as buffer revision doesn't change — zero overhead on hot render path
- `TextBuffer` rope overhead for short labels is negligible compared to frame rendering cost
