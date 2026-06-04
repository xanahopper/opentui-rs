# Declarative View Implementation Review

## Scope

This review covers the current `opentui-core` declarative View API implementation:

- `crates/core/src/view/*`
- `ViewRuntime`
- `ViewWidget`
- declarative examples and tests
- integration points with `WidgetTree`, rendering, overlays, focus, lists, and events

The review compares the current implementation against the intended direction described in `docs/core/DECLARATIVE_VIEW_DESIGN.md`: a Rust-native builder API centered on `view()`, without macro or HTML-like syntax.

## High-Level Status

The implementation has landed a solid Phase A scaffold.

Implemented:

- `view` module with `Node`, `Element`, `ElementKind`, `Key`, props, builders, rebuild, and runtime.
- `view()`, `panel()`, `text()`, `rich_text()`, `span()`, `input()`, `list()`, `fill()`, `separator()`, `fragment()`, `when()`, `empty()`, and `overlay()` builders.
- `ViewRuntime` with rebuild, layout, render, and event forwarding.
- `ViewWidget` as the new general-purpose visual/layout container.
- `Widget::render` changed from `&self` to `&mut self`, which removes a major blocker for stateful widgets.
- Declarative examples:
  - `crates/core/examples/declarative_hello.rs`
  - `crates/core/examples/opencode_view.rs`
- Tests:
  - `crates/core/tests/view_builder.rs`
  - `crates/core/tests/view_runtime.rs`

Not complete:

- Overlay subtree rendering is incomplete.
- Declarative `list()` does not render items.
- Action and hit-test integration is not wired.
- Keys are stored but not used for reconciliation or state preservation.
- Text rendering is still not fully grapheme-safe.
- `opentui-core` does not pass clippy with `-D warnings`.

## Verification Results

Commands run:

```bash
cargo fmt --check
cargo check -p opentui-core --all-targets
cargo test -p opentui-core
cargo clippy -p opentui-core --all-targets --no-deps -- -D warnings
cargo check --all-targets
```

Results:

- `cargo fmt --check`: passed.
- `cargo check -p opentui-core --all-targets`: passed.
- `cargo test -p opentui-core`: passed.
- `cargo clippy -p opentui-core --all-targets --no-deps -- -D warnings`: failed on core lint issues.
- `cargo check --all-targets`: failed before completing because of an existing root crate test issue in `tests/common/pty.rs:314`.

The root crate failure:

```text
tests/common/pty.rs:314:35
libc::ioctl(slave_fd, libc::TIOCSCTTY, 0);
expected u64, found u32
```

This is not specific to the declarative View implementation.

## Findings

### 1. Overlay subtree rendering is incomplete

Severity: High

Current behavior:

- `Node::Overlay` accepts content.
- `build_recursive()` creates the overlay root widget and registers it with `WidgetTree::add_overlay()`.
- Children of the overlay root are added as children in `WidgetTree`.
- `WidgetTree::render_overlays()` only calls `render()` on the overlay root widget.
- Overlay children are not traversed or rendered.

Relevant code:

- `crates/core/src/view/rebuild.rs:51`
- `crates/core/src/widget.rs:532`

Example impact:

```rust
overlay(
    panel()
        .title("Popup")
        .children([text("ok").build()])
        .build()
)
```

The panel can render, but `text("ok")` is skipped.

Recommended fix:

- Treat overlay content as a subtree, not a single widget.
- Add an overlay render command path that recursively renders overlay children with the overlay root as coordinate origin.
- Alternatively, create a temporary layout/render pass for each overlay subtree.

Minimum acceptable next step:

- Add a failing test asserting overlay child text is visible.
- Update `render_overlays()` to recursively render the overlay subtree.

### 2. Declarative `list()` is non-functional

Severity: High

Current behavior:

- `list(item_count)` creates `ElementKind::List`.
- `rebuild.rs` creates a `ListWidget`.
- `ListWidget::render()` is empty.
- Actual list rendering still requires `ListWidget::render_with_renderer(...)`, which the declarative runtime never calls.

Relevant code:

- `crates/core/src/view/builder.rs:45`
- `crates/core/src/view/rebuild.rs:107`
- `crates/core/src/widgets/list_widget.rs:85`

Recommended fix:

- Do not expose `list()` as a functional declarative builder until item rendering is specified.
- Either:
  - remove/hide `list()` from the prelude for now, or
  - define `list(children)` as a normal declarative child list, or
  - implement a real `virtual_list()` API with item data and a renderer strategy.

Pragmatic recommendation:

- Keep `list()` internal or mark it experimental.
- Use regular `view().children(items.map(...))` for Phase A.
- Add `virtual_list()` later when keyed reconciliation and state preservation exist.

### 3. TextLineWidget is not fully grapheme-safe

Severity: High

Current behavior:

- `TextLineWidget` now iterates `split_graphemes_with_widths()`.
- It still writes only `grapheme.chars().next()`.
- It does not write continuation cells for wide graphemes.
- It does not use `GraphemePool`.

Relevant code:

- `crates/core/src/widgets/text_line_widget.rs:193`

Impact:

- Multi-codepoint graphemes can collapse to the first scalar.
- Emoji ZWJ sequences can render incorrectly.
- Wide characters may not mark continuation cells, which can corrupt diff rendering and hit positioning.

Recommended fix:

- Prefer `OptimizedBuffer::draw_text_with_pool()` when `RenderContext.grapheme_pool` is available.
- Fall back to `draw_text()` only when no pool is available.
- Apply alignment by computing `start_x`, then delegate actual rendering to the buffer text API.

### 4. `on_action()` is stored but not wired

Severity: Medium-High

Current behavior:

- `Element.action` exists.
- Builder supports `.on_action(...)`.
- No runtime system maps widget IDs to actions.
- No hit registration is performed for action nodes.
- `dispatch_mouse()` returns only `MouseDispatchResult { target, consumed }`.

Relevant code:

- `crates/core/src/view/element.rs:13`
- `crates/core/src/view/builder.rs:387`
- `crates/core/src/view/runtime.rs:47`
- `crates/core/src/widget.rs:763`

Recommended fix:

Add an action registry to `ViewRuntime`:

```rust
pub struct ViewRuntime {
    tree: WidgetTree,
    actions: HashMap<WidgetId, String>,
}
```

During rebuild:

- Assign `WidgetId`.
- If `Element.action.is_some()`, store `WidgetId -> action`.
- Mark the widget focusable/clickable or register a hit area after layout.

Dispatch should return action information:

```rust
pub struct ViewMouseDispatchResult {
    pub target: Option<WidgetId>,
    pub action: Option<String>,
    pub consumed: bool,
}
```

### 5. Hit-test integration is not closed

Severity: Medium-High

Current behavior:

- `RenderContext` has `hit_grid`.
- `WidgetTree::dispatch_mouse()` can consume a hit grid.
- Declarative rendering does not automatically register hit areas for interactive nodes.
- Examples pass `hit_grid: None`.

Relevant code:

- `crates/core/src/widget.rs:36`
- `crates/core/src/widget.rs:763`
- `crates/core/examples/declarative_hello.rs:123`
- `crates/core/examples/opencode_view.rs:682`

Recommended fix:

- Add a post-layout pass that registers hit areas for focusable/action nodes.
- Use `WidgetId` as hit ID where possible.
- Integrate with `Renderer::register_hit_area()` or a `HitGrid` owned by the runtime.

### 6. Keys are parsed but unused

Severity: Medium

Current behavior:

- `Key` exists.
- `.key(...)` works at the builder level.
- `build_tree()` ignores keys and allocates fresh sequential IDs every rebuild.

Relevant code:

- `crates/core/src/view/key.rs`
- `crates/core/src/view/builder.rs:176`
- `crates/core/src/view/rebuild.rs:12`

This is acceptable for a full-rebuild Phase A, but the limitation should be explicit.

Impact:

- Focus state is lost across rebuilds.
- Input cursor/value state is lost unless controlled by app state.
- Scroll/list state is lost.
- Dynamic lists cannot preserve identity.

Recommended fix:

- Document that keys are currently reserved for future reconciliation.
- Implement `ViewRuntime` keyed reconciliation after the Phase A API settles.

### 7. `InputProps.initial_value` is a rebuild hazard

Severity: Medium

Current behavior:

- `input().value(...)` maps to `InputProps.initial_value`.
- Full rebuild creates a fresh `InputWidget` and calls `set_value()`.

Impact:

- This acts like a controlled input only if the app updates the value every frame.
- Without keyed reconciliation, internal input edits cannot survive rebuild.
- The name `initial_value` suggests uncontrolled semantics, but rebuild makes it effectively per-frame value.

Recommended fix:

- Rename to `value` if controlled.
- Or explicitly split:
  - `value(...)` for controlled input
  - `default_value(...)` for future uncontrolled input with keyed state preservation

### 8. `Node` has a large enum variant

Severity: Medium

Clippy reports:

```text
large size difference between variants
Node::Element(Element) is at least 776 bytes
```

Relevant code:

- `crates/core/src/view/node.rs:4`

Recommended fix:

Box the large variant:

```rust
pub enum Node {
    Element(Box<Element>),
    Overlay(Box<OverlayNode>),
    Fragment(Vec<Node>),
    Empty,
}
```

This will require small changes to builder/tests/rebuild, but it is the right shape for a declarative tree.

### 9. Core clippy currently fails

Severity: Medium

Command:

```bash
cargo clippy -p opentui-core --all-targets --no-deps -- -D warnings
```

Failures:

- `single_match` in `view/builder.rs`
- `use_self` in `view/builder.rs` and `view/node.rs`
- `large_enum_variant` in `view/node.rs`
- `match_same_arms` in `view/rebuild.rs`
- `assigning_clones` in `text_line_widget.rs` and `view_widget.rs`

Recommended fix:

These are straightforward cleanup items and should be handled before considering the implementation complete.

### 10. Full workspace checks are blocked by unrelated root crate issues

Severity: Medium

`cargo check --all-targets` fails in root crate tests because of:

```rust
libc::ioctl(slave_fd, libc::TIOCSCTTY, 0);
```

on this platform, `ioctl` expects `c_ulong` and `TIOCSCTTY` is `u32`.

Recommended fix:

Use the suggested conversion:

```rust
libc::ioctl(slave_fd, libc::TIOCSCTTY.into(), 0);
```

This is outside the declarative View implementation, but it blocks the project-mandated full check.

## Positive Notes

The implementation made several good structural choices:

1. It avoided macro syntax and kept the public API as normal Rust builder calls.
2. It introduced `ViewWidget`, avoiding further overloading of `BoxWidget`.
3. It changed `Widget::render` to `&mut self`, which is the right direction for input/list/scroll widgets.
4. It added targeted tests for builder behavior and runtime rebuild/render basics.
5. It added real examples, including an OpenCode-style declarative example.

## Recommended Next Work

Recommended order:

1. Fix core clippy issues.
2. Add failing tests for overlay child rendering and grapheme text rendering.
3. Fix overlay subtree rendering.
4. Make `text()` delegate to grapheme-aware buffer drawing.
5. Either remove/hide declarative `list()` or implement meaningful rendering.
6. Add runtime action registry and hit-test registration.
7. Clarify controlled vs uncontrolled input naming.
8. Start keyed reconciliation only after the above behavior is stable.

## Current Readiness

The current code is ready to treat as an experimental Phase A implementation.

It is not yet ready to present as a complete declarative UI layer because several public APIs exist without complete behavior:

- `overlay()` accepts children but does not render them.
- `list()` accepts item count but does not render items.
- `on_action()` stores action data but does not dispatch it.
- `.key()` stores identity data but does not preserve state.

The API direction is sound. The next milestone should focus on closing these behavior gaps rather than adding more builder surface.
