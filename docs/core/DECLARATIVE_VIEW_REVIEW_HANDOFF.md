# Declarative View Review Handoff

## Context

The current commit series has implemented several items from the earlier declarative View review:

- `Node::Element` and `Node::Overlay` are boxed to avoid large enum variants.
- `Widget::render` uses `&mut self`.
- Overlay subtree layout and recursive rendering were added.
- Wide character continuation cells were added to `TextLineWidget`.
- `list()` was marked experimental and hidden from public re-exports.
- `InputProps.initial_value` was renamed to `default_value`.
- `ViewRuntime` now keeps a `WidgetId -> action` registry.
- `opentui-core` clippy issues from the previous review have been cleaned up.

This document records the latest review state and the recommended next work for another Agent.

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
- `cargo clippy -p opentui-core --all-targets --no-deps -- -D warnings`: passed.
- `cargo check --all-targets`: failed due to an existing root test issue in `tests/common/pty.rs`.

The repo-wide blocker:

```text
tests/common/pty.rs:314:35
libc::ioctl(slave_fd, libc::TIOCSCTTY, 0);
expected u64, found u32
```

Relevant file:

- `tests/common/pty.rs:314`

Suggested fix from rustc:

```rust
libc::ioctl(slave_fd, libc::TIOCSCTTY.into(), 0);
```

## Current Findings

### 1. `.on_action()` is not yet a real declarative interaction API

Severity: High

`ViewRuntime` stores actions:

```rust
actions: HashMap<WidgetId, String>
```

and `dispatch_mouse()` can return an action when `WidgetTree::dispatch_mouse()` finds a target:

```rust
let inner = self.tree.dispatch_mouse(mouse, hit_grid);
let action = inner.target.and_then(|id| self.actions.get(&id).cloned());
```

However, the declarative render path does not automatically register hit areas for action/focusable nodes. Callers must provide a `HitGrid`, but `ViewRuntime` currently gives them no complete way to populate it from declarative layout.

Impact:

- `.on_action("submit")` is stored metadata, not an end-to-end interaction.
- Mouse dispatch returns actions only if external code manually registers matching widget IDs.
- This undermines the intended declarative API.

Recommended implementation:

1. Add runtime-owned hit registration or expose a post-layout registration pass.
2. Register hit areas for nodes with:
   - `.on_action(...)`
   - `.focusable()`
   - future clickable/hoverable props
3. Use `WidgetId` as the hit ID.
4. Add tests:
   - render a `view().on_action("submit")`
   - synthesize a mouse event inside its layout
   - assert `ViewMouseDispatchResult.action == Some("submit")`

Candidate API:

```rust
impl ViewRuntime {
    pub fn register_hit_areas(&self, hit_grid: &mut HitGrid);
}
```

or:

```rust
pub struct ViewRuntime {
    tree: WidgetTree,
    actions: HashMap<WidgetId, String>,
    hit_grid: HitGrid,
}
```

### 2. `TextLineWidget` still loses multi-codepoint graphemes

Severity: High

The current implementation handles wide continuation cells, which fixes basic CJK width behavior, but it still writes only the first scalar of each grapheme:

```rust
if let Some(ch) = grapheme.chars().next() {
    ctx.buffer.set_blended(col, y, ot::Cell::new(ch, style));
}
```

Impact:

- ZWJ emoji sequences are truncated to the first scalar.
- Combining mark sequences such as `e\u{0301}` lose the combining mark.
- Grapheme pool support is not used, even when `RenderContext.grapheme_pool` is available.

Recommended implementation:

1. Compute alignment as now.
2. Delegate actual drawing to the buffer API:

   ```rust
   if let Some(pool) = ctx.grapheme_pool.as_deref_mut() {
       ctx.buffer.draw_text_with_pool(pool, start_x, y, &self.text, style);
   } else {
       ctx.buffer.draw_text(start_x, y, &self.text, style);
   }
   ```

3. Preserve background fill behavior.
4. Add tests for:
   - CJK wide continuation
   - combining mark text such as `e\u{0301}`
   - ZWJ emoji such as `👩‍💻`

### 3. Prelude does not expose the stable View API surface

Severity: Medium

`crates/core/src/view/mod.rs` exports:

- `fill`
- `overlay`
- `rich_text`
- `separator`
- `span`
- `ViewMouseDispatchResult`
- other stable builders

But `crates/core/src/prelude.rs` exports only a smaller subset:

```rust
ElementBuilder, ElementKind, Key, Node, Props, TextProps, ViewProps,
ViewRuntime, empty, fragment, panel, text, view, when
```

Impact:

- Examples such as `opencode_view.rs` need direct `opentui_core::view::{...}` imports.
- The intended ergonomic `use opentui_core::prelude::*;` experience is incomplete.

Recommended implementation:

- Add stable, non-experimental view builders to the prelude:
  - `fill`
  - `overlay`
  - `rich_text`
  - `separator`
  - `span`
  - `ViewMouseDispatchResult`
- Keep experimental `list()` out of the prelude for now.

### 4. Full workspace checks are blocked by `tests/common/pty.rs`

Severity: Medium

This is outside `opentui-core`, but it blocks the project-required gate:

```bash
cargo check --all-targets
```

Recommended implementation:

```rust
libc::ioctl(slave_fd, libc::TIOCSCTTY.into(), 0);
```

After fixing, rerun:

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Confirmed Improvements Since Previous Review

### Overlay subtree rendering

Status: Improved.

`WidgetTree::layout()` now calls `layout_overlays()`, and overlay subtrees are recursively rendered through `render_subtree()`.

Relevant files:

- `crates/core/src/widget.rs`
- `crates/core/tests/view_runtime.rs`

Current note:

- The test `test_overlay_child_text_is_rendered` confirms that overlay child text appears somewhere.
- The test could be stronger by checking exact position or using a child string that does not overlap with the title text.

### Wide character continuation cells

Status: Partially improved.

`TextLineWidget` now writes continuation cells for graphemes with display width greater than one.

Remaining gap:

- Multi-codepoint grapheme identity is still not preserved.
- Grapheme pool is still not used.

### Experimental list API

Status: Acceptable for now.

`list()` is marked doc-hidden/experimental and removed from the main `view` re-export/prelude path. This is reasonable until a real declarative list or `virtual_list()` API is designed.

## Recommended Next Work Order

1. Fix the repo-wide `tests/common/pty.rs` type mismatch so full checks can run.
2. Complete declarative action/hit-test integration.
3. Make `TextLineWidget` delegate to grapheme-aware buffer drawing.
4. Update prelude exports for the stable View API.
5. Strengthen overlay tests to assert exact child rendering behavior.
6. After those are stable, start keyed reconciliation planning/implementation.

## Suggested Acceptance Criteria For Next Agent

The next implementation pass should be considered complete when:

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test -p opentui-core
```

all pass.

Behavioral checks to add:

- Clicking a declarative node with `.on_action("submit")` returns `"submit"`.
- `text("e\u{0301}")` preserves the full grapheme when rendered with a grapheme pool.
- `text("👩‍💻")` renders through the grapheme pool rather than truncating to the first scalar.
- Overlay child text is asserted at a deterministic cell position.

## Worktree Note

At the time of this handoff, the following review document may still be untracked if it has not been added by the user or another Agent:

- `docs/core/DECLARATIVE_VIEW_IMPLEMENTATION_REVIEW.md`

This file is review documentation only and does not affect build behavior.
