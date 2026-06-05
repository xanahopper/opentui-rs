# Event System Implementation Plan

> Full specification for implementing OpenTUI's event system in the Rust port,
> using **Message Delegation (Elm Architecture + iced pattern)**.

## Architecture Overview

```
Terminal (ANSI bytes)
    |
    v
InputParser (src/input/)
    |  Parses raw bytes into typed Event variants
    v
Event::Mouse(MouseEvent) / Event::Key(KeyEvent) / Event::Paste(PasteEvent)
    |
    v
ViewRuntime<M>::process_mouse_event() / process_key_event()
    |  Hit grid test + event dispatch + bubbling
    |  Returns Vec<M> (typed messages for the app)
    v
App::update(app, msg)
    |  Message delegation to sub-modules
    v
App::view(app) -> Node<M>
    |  Pure function, rebuilds declarative tree
    v
ViewRuntime::render_to_buffer()
    |  Rebuild + layout + hit grid + render + recheck_hover
    v
Renderer::present()
```

### Two-Layer Event Model

| Layer | Responsibility | Examples |
|-------|---------------|---------|
| **Runtime (automatic)** | Hover tracking, scroll dispatch, focus management | `hover_bg()` auto-applied, scroll to ScrollViewWidget |
| **App (messages)** | Semantic events via typed enum | `on_click(AppMsg::SidebarClick)`, `on_scroll(AppMsg::ListScroll)` |

Runtime-managed interactions do NOT produce App messages.
Only explicitly registered event bindings produce `Vec<M>`.

---

## Phase 1: MouseEvent Model Supplement

### File: `src/terminal/mouse.rs`

#### 1.1 Add missing MouseEventKind variants

```rust
pub enum MouseEventKind {
    Press,
    Release,
    Move,
    Drag,
    DragEnd,    // already exists
    Drop,       // NEW: synthesized on release over different widget during capture
    Over,       // NEW: synthesized on hover enter
    Out,        // NEW: synthesized on hover leave
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
}
```

#### 1.2 Add fields to MouseEvent

```rust
pub struct MouseEvent {
    pub x: u32,
    pub y: u32,
    pub button: MouseButton,
    pub kind: MouseEventKind,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub scroll_delta: f64,
    // NEW fields:
    pub target_id: Option<u64>,        // Hit widget ID (set by ViewRuntime)
    pub source_id: Option<u64>,        // Drag source ID (set for Drop events)
    pub propagation_stopped: bool,     // Stop bubbling
    pub default_prevented: bool,       // Prevent auto-focus
    pub is_dragging: bool,             // Selection drag in progress
}
```

Add methods:
```rust
pub fn stop_propagation(&mut self) { self.propagation_stopped = true; }
pub fn prevent_default(&mut self) { self.default_prevented = true; }
```

#### 1.3 Add ScrollDirection helper

```rust
pub enum ScrollDirection { Up, Down, Left, Right }

impl MouseEvent {
    pub fn scroll_direction(&self) -> Option<ScrollDirection> { ... }
}
```

---

## Phase 2: Declarative Event Binding (Node<M>)

### 2.1 New file: `crates/core/src/view/event.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Click,
    RightClick,
    MiddleClick,
    Hover,
    Scroll,
}

#[derive(Debug, Clone)]
pub struct EventBinding<M> {
    pub kind: EventKind,
    pub message: M,
}
```

### 2.2 Genericize `Node<M>` — `crates/core/src/view/node.rs`

Before:
```rust
#[derive(Debug, Clone)]
pub enum Node { ... }
```

After:
```rust
#[derive(Debug)]
pub enum Node<M> {
    Element(Box<Element<M>>),
    Overlay(Box<OverlayNode<M>>),
    Fragment(Vec<Self>),
    Empty,
}

impl<M: Clone> Node<M> {
    pub fn map_msg<N: Clone>(self, f: impl Fn(M) -> N + Clone + 'static) -> Node<N> { ... }
}
```

Note: Remove `Clone` derive from `Node`. `Node<M>` is `Clone` when `M: Clone`
(via manual impl).

### 2.3 Genericize `Element<M>` — `crates/core/src/view/element.rs`

```rust
#[derive(Debug)]
pub struct Element<M> {
    pub kind: ElementKind,
    pub key: Option<Key>,
    pub layout: LayoutStyle,
    pub props: Props,
    pub children: Vec<Node<M>>,
    pub events: Vec<EventBinding<M>>,   // replaces action: Option<String>
}
```

### 2.4 Genericize `ElementBuilder<M>` — `crates/core/src/view/builder.rs`

```rust
// Factory functions return ElementBuilder<()>
pub fn view() -> ElementBuilder<()> { ... }
pub fn text(content: impl Into<String>) -> ElementBuilder<()> { ... }
// etc.

pub struct ElementBuilder<M> {
    kind: ElementKind,
    key: Option<Key>,
    layout: LayoutStyle,
    props: Props,
    children: Vec<Node<M>>,
    text_content: Option<String>,
    events: Vec<EventBinding<M>>,   // replaces action: Option<String>
}

impl<M: Clone> ElementBuilder<M> {
    // Type-preserving event registrations (M stays the same)
    pub fn on_click(mut self, msg: M) -> Self { ... }
    pub fn on_right_click(mut self, msg: M) -> Self { ... }
    pub fn on_hover(mut self, msg: M) -> Self { ... }
    pub fn on_scroll(mut self, msg: M) -> Self { ... }
    
    pub fn interactive(mut self) -> Self {
        // Registers to hit grid for hover tracking (no message)
        // Sets a flag on ViewProps
    }
    
    pub fn build(self) -> Node<M> { ... }
}

impl<M: Clone> ElementBuilder<M> {
    /// Map messages to a different type. Core of message delegation.
    /// Usage: sidebar::view(&state).map_msg(AppMsg::Sidebar)
    pub fn map_msg<N: Clone>(self, f: impl Fn(M) -> N + Clone + 'static) -> ElementBuilder<N> {
        ElementBuilder {
            kind: self.kind,
            key: self.key,
            layout: self.layout,
            props: self.props,
            children: self.children.into_iter().map(|c| c.map_msg(f.clone())).collect(),
            text_content: self.text_content,
            events: self.events.into_iter().map(|e| EventBinding {
                kind: e.kind,
                message: (f.clone())(e.message),
            }).collect(),
        }
    }
}

impl ElementBuilder<()> {
    /// Convert from no-events builder to typed builder by registering first event
    fn typed<N: Clone>(self, binding: EventBinding<N>) -> ElementBuilder<N> {
        ElementBuilder {
            kind: self.kind,
            key: self.key,
            layout: self.layout,
            props: self.props,
            children: self.children.into_iter().map(|c| c.map_msg(|()| unreachable!())).collect(),
            // Actually need a different strategy...
        }
    }
}
```

**Type transition strategy**: Factory functions return `ElementBuilder<()>`.
When you call `.on_click(msg)`, the builder transitions to `ElementBuilder<AppMsg>`.
Implementation: `on_click` is only available on `ElementBuilder<()>` initially,
then subsequent `.on_hover(msg)` / `.on_scroll(msg)` work on `ElementBuilder<M>`.

Actually, simpler approach: Make all `on_*` methods generic over the return type:

```rust
impl<M> ElementBuilder<M> {
    /// Add event binding. Type M must already be set (via first on_* call or map_msg).
    pub fn on_click(mut self, msg: M) -> Self
    where M: Clone
    {
        self.events.push(EventBinding { kind: EventKind::Click, message: msg });
        self
    }
}

// Separate impl block for the initial type transition
impl ElementBuilder<()> {
    pub fn click<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.typed(EventBinding { kind: EventKind::Click, message: msg })
    }
    pub fn hover<N: Clone>(self, msg: N) -> ElementBuilder<N> { ... }
    pub fn scroll<N: Clone>(self, msg: N) -> ElementBuilder<N> { ... }
}
```

**Final decision**: Use `on_*` methods that work on any `ElementBuilder<M>` where
`M` is the message type. The initial type `()` comes from factory functions.
Users call `.on_click(AppMsg::Foo)` which requires the builder to already be
typed. Use a separate `.click(msg)` on `ElementBuilder<()>` for the initial
transition. Or, more pragmatically, make all factory functions return
`ElementBuilder<!>` (never type) or just use a `PhantomData<M>` approach.

**Simplest viable approach**: Factory functions return `ElementBuilder<()>`.
First `on_*` call consumes self and returns `ElementBuilder<M>`. Example:

```rust
// ElementBuilder<()> — no message type yet
let builder = view().bg(BG_PANEL);

// First on_* call transitions to typed builder
let builder: ElementBuilder<AppMsg> = builder.on_click(AppMsg::SidebarClick);

// Subsequent calls preserve type
let builder = builder.on_hover(AppMsg::SidebarHover);
let node = builder.build();  // Node<AppMsg>
```

This works because `()` doesn't have the same type as `AppMsg`, so `on_click`
can't be called on `ElementBuilder<()>` with the `M: Clone` bound impl.
We need two impl blocks:

```rust
// Impl for typed builders (M is already set)
impl<M: Clone> ElementBuilder<M> {
    pub fn on_click(mut self, msg: M) -> Self { ... }
    pub fn on_hover(mut self, msg: M) -> Self { ... }
    pub fn on_scroll(mut self, msg: M) -> Self { ... }
}

// Impl for untyped builders — transitions to typed
impl ElementBuilder<()> {
    pub fn on_click<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::Click, msg)
    }
    pub fn on_hover<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::Hover, msg)
    }
    pub fn on_scroll<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::Scroll, msg)
    }
}
```

The `transition` method converts children `Node<()>` to `Node<N>` (trivial since
`()` has no event bindings in typical usage) and creates the typed builder.

### 2.5 `ViewProps` additions — `crates/core/src/view/props.rs`

```rust
pub struct ViewProps {
    // ... existing fields ...
    pub interactive: bool,        // NEW: register to hit grid for hover
    pub hover_bg: Option<Rgba>,   // NEW: auto-applied when hovered
    pub hover_fg: Option<Rgba>,   // NEW: auto-applied when hovered
}
```

Builder methods:
```rust
pub fn interactive(mut self) -> Self { ... }
pub fn hover_bg(mut self, color: Rgba) -> Self { ... }
pub fn hover_fg(mut self, color: Rgba) -> Self { ... }
```

---

## Phase 3: ViewRuntime<M> Event Dispatch Engine

### File: `crates/core/src/view/runtime.rs`

```rust
pub struct ViewRuntime<M> {
    tree: WidgetTree,
    events: HashMap<WidgetId, Vec<EventBinding<M>>>,
    hit_grid: HitGrid,
    hovered_id: Option<WidgetId>,
    captured_id: Option<WidgetId>,
    pointer_pos: Option<(u32, u32)>,
    pointer_modifiers: (bool, bool, bool),
}

pub struct DispatchResult<M> {
    pub messages: Vec<M>,
    pub consumed: bool,
}
```

### 3.1 Main dispatch entry point

```rust
impl<M: Clone> ViewRuntime<M> {
    pub fn process_mouse_event(&mut self, mouse: &MouseEvent) -> DispatchResult<M> {
        // Store pointer position
        self.pointer_pos = Some((mouse.x, mouse.y));
        self.pointer_modifiers = (mouse.shift, mouse.ctrl, mouse.alt);

        // Captured mode: route all events to captured widget
        if self.captured_id.is_some() {
            return self.process_captured(mouse);
        }

        // Hit test
        let hit_id = self.hit_grid.test(mouse.x, mouse.y);
        let hit_wid = hit_id.map(|id| id as WidgetId);

        match mouse.kind {
            MouseEventKind::Press => self.process_down(mouse, hit_wid),
            MouseEventKind::Release => self.process_up(mouse, hit_wid),
            MouseEventKind::Move => self.process_move(mouse, hit_wid),
            MouseEventKind::Drag => self.process_drag(mouse, hit_wid),
            MouseEventKind::ScrollUp | ScrollDown | ScrollLeft | ScrollRight
                => self.process_scroll(mouse, hit_wid),
            _ => DispatchResult::default(),
        }
    }
}
```

### 3.2 Down (click)

```rust
fn process_down(&mut self, mouse: &MouseEvent, hit: Option<WidgetId>) -> DispatchResult<M> {
    let mut msgs = Vec::new();
    if let Some(wid) = hit {
        // Auto-focus: walk parent chain for focusable widget
        if mouse.button == MouseButton::Left && !mouse.default_prevented {
            self.auto_focus(wid);
        }
        // Collect click messages (bubbling)
        let kind = match mouse.button {
            MouseButton::Left => EventKind::Click,
            MouseButton::Right => EventKind::RightClick,
            MouseButton::Middle => EventKind::MiddleClick,
            MouseButton::None => return DispatchResult::default(),
        };
        self.collect_bubbling(wid, kind, &mut msgs);
    }
    DispatchResult { messages: msgs, consumed: !msgs.is_empty() }
}
```

### 3.3 Move (hover over/out)

```rust
fn process_move(&mut self, mouse: &MouseEvent, hit: Option<WidgetId>) -> DispatchResult<M> {
    self.update_hover(hit)
}

fn update_hover(&mut self, new_hovered: Option<WidgetId>) -> DispatchResult<M> {
    let mut msgs = Vec::new();
    if new_hovered != self.hovered_id {
        // Out on old
        if let Some(old) = self.hovered_id {
            self.collect_events(old, EventKind::Hover, &mut msgs);
        }
        // Over on new
        if let Some(new) = new_hovered {
            self.collect_events(new, EventKind::Hover, &mut msgs);
        }
        self.hovered_id = new_hovered;
    }
    DispatchResult { messages: msgs, consumed: !msgs.is_empty() }
}
```

### 3.4 Drag (initiate capture on left button)

```rust
fn process_drag(&mut self, mouse: &MouseEvent, hit: Option<WidgetId>) -> DispatchResult<M> {
    let mut msgs = Vec::new();
    // Left drag initiates capture
    if mouse.button == MouseButton::Left {
        self.captured_id = hit;
    }
    // Hover tracking during drag
    let drag_hover = if self.captured_id.is_some() { None } else { hit };
    msgs.extend(self.update_hover(drag_hover).messages);
    DispatchResult { messages: msgs, consumed: !msgs.is_empty() }
}
```

### 3.5 Up (release)

```rust
fn process_up(&mut self, mouse: &MouseEvent, hit: Option<WidgetId>) -> DispatchResult<M> {
    // Not captured — simple release
    DispatchResult::default()
}
```

### 3.6 Captured event routing

```rust
fn process_captured(&mut self, mouse: &MouseEvent) -> DispatchResult<M> {
    let mut msgs = Vec::new();
    let captured = self.captured_id.unwrap();

    match mouse.kind {
        MouseEventKind::Release => {
            let hit = self.hit_grid.test(mouse.x, mouse.y).map(|id| id as WidgetId);

            // 1. drag-end on captured (collect bubbling from captured)
            self.collect_bubbling(captured, EventKind::Click, &mut msgs);

            // 2. If hit target != captured, generate drop on target
            if let Some(target) = hit {
                if target != captured {
                    // Drop event — future: collect drop-specific events
                }
            }

            // 3. Release capture
            self.captured_id = None;

            // 4. Re-evaluate hover
            msgs.extend(self.update_hover(hit).messages);
        }
        MouseEventKind::Move | MouseEventKind::Drag => {
            // Hover tracking during capture (skip captured widget)
            let hit = self.hit_grid.test(mouse.x, mouse.y).map(|id| id as WidgetId);
            let hover_target = hit.filter(|id| *id != captured);
            msgs.extend(self.update_hover(hover_target).messages);
        }
        _ => {}
    }

    DispatchResult { messages: msgs, consumed: true }
}
```

### 3.7 Scroll dispatch

```rust
fn process_scroll(&mut self, mouse: &MouseEvent, hit: Option<WidgetId>) -> DispatchResult<M> {
    let mut msgs = Vec::new();

    // Phase 1: hit test target
    // Phase 2: fallback to focused widget
    let target = hit.or(self.tree.focused_id());

    if let Some(wid) = target {
        // Let scrollable widgets handle scroll internally
        self.tree.dispatch_scroll_to_widget(wid, mouse);
        // Collect on_scroll messages (bubbling)
        self.collect_bubbling(wid, EventKind::Scroll, &mut msgs);
    }

    DispatchResult { messages: msgs, consumed: !msgs.is_empty() }
}
```

### 3.8 Event collection helpers

```rust
/// Collect events for a single widget (no bubbling)
fn collect_events(&self, wid: WidgetId, kind: EventKind, msgs: &mut Vec<M>) {
    if let Some(bindings) = self.events.get(&wid) {
        for b in bindings {
            if b.kind == kind {
                msgs.push(b.message.clone());
            }
        }
    }
}

/// Collect events along parent chain (bubbling)
fn collect_bubbling(&self, wid: WidgetId, kind: EventKind, msgs: &mut Vec<M>) {
    let mut current = Some(wid);
    while let Some(id) = current {
        self.collect_events(id, kind, msgs);
        current = self.tree.parent(id);
    }
}

/// Auto-focus: walk parent chain, focus first focusable widget
fn auto_focus(&mut self, wid: WidgetId) {
    let mut current = Some(wid);
    while let Some(id) = current {
        if self.tree.get(id).is_some_and(|w| w.focusable()) {
            self.tree.set_focused_widget(Some(id));
            return;
        }
        current = self.tree.parent(id);
    }
}
```

### 3.9 recheckHoverState

```rust
/// Call after render if hit grid changed. Checks if layout moved under cursor.
pub fn recheck_hover(&mut self) -> DispatchResult<M> {
    let Some((x, y)) = self.pointer_pos else { return DispatchResult::default() };
    if self.captured_id.is_some() { return DispatchResult::default(); }
    let hit = self.hit_grid.test(x, y).map(|id| id as WidgetId);
    self.update_hover(hit)
}
```

### 3.10 Render pipeline update

```rust
pub fn render_to_buffer(&mut self, ctx: &mut RenderContext, node: &Node<M>, w: f32, h: f32)
where M: Clone
{
    self.rebuild(node);
    self.layout(w, h);
    ctx.hovered_id = self.hovered_id;   // Inject hover state
    self.register_hit_areas(w as u32, h as u32);
    self.render(ctx);
}
```

---

## Phase 4: Hover + Scroll + Focus Pipeline

### 4.1 RenderContext — `crates/core/src/widget.rs`

```rust
pub struct RenderContext<'a> {
    pub buffer: &'a mut OptimizedBuffer,
    pub grapheme_pool: Option<&'a mut ot::GraphemePool>,
    pub link_pool: Option<&'a mut ot::LinkPool>,
    pub hit_grid: Option<&'a mut ot::renderer::HitGrid>,
    pub theme: Option<&'a crate::theme::UiTheme>,
    pub hovered_id: Option<WidgetId>,   // NEW
}
```

### 4.2 ViewWidget hover rendering

In `ViewWidget::render()`, check `ctx.hovered_id` and apply `hover_bg`/`hover_fg`:

```rust
fn render(&mut self, ctx: &mut RenderContext, layout: &ComputedLayout) {
    let is_hovered = ctx.hovered_id == Some(self.id());
    let bg = if is_hovered {
        self.props.hover_bg.or(self.props.bg).unwrap_or(TRANSPARENT)
    } else {
        self.props.bg.unwrap_or(TRANSPARENT)
    };
    // ... draw background ...
}
```

### 4.3 ScrollViewWidget scroll dispatch

Add to `WidgetTree`:
```rust
pub fn dispatch_scroll_to_widget(&mut self, wid: WidgetId, mouse: &MouseEvent) {
    // Walk from wid up parent chain
    let mut current = Some(wid);
    while let Some(id) = current {
        if let Some(node) = self.nodes.get_mut(&id) {
            // Check if widget is scrollable (ScrollViewWidget)
            // If so, call handle_mouse and break if consumed
            if node.widget.handle_mouse(mouse) {
                return;
            }
        }
        current = self.parent(id);
    }
}
```

### 4.4 Hit area registration update

In `ViewRuntime::register_hit_areas()`, also register `interactive` widgets
(not just focusable + action-bearing):

```rust
for (id, layout, focusable, interactive) in self.tree.hit_registration_order() {
    let has_events = events.contains_key(&id);
    if has_events || focusable || interactive {
        self.hit_grid.register(x, y, w, h, id as u32);
    }
}
```

---

## Phase 5: Drag Capture + Drop

Already specified in Phase 3.6. Key behaviors:
- Left-button `Drag` initiates capture
- Captured widget excluded from hit grid during capture
- `Release` while captured: generate `DragEnd`, `Up`, and `Drop` (if target != source)
- `Move`/`Drag` during capture: hover tracking continues (skipping captured widget)

---

## Phase 6: Example Migration

### opencode.rs — Before

```rust
// Manual string action system
.on_action(format!("palette:{display_idx}"))

// Manual hover detection
fn detect_palette_hover(app: &mut App, runtime: &ViewRuntime) { ... }

// Manual string parsing in event loop
if let Some(idx) = parse_palette_select(&action) { ... }
```

### opencode.rs — After

```rust
// Sub-module pattern
mod sidebar {
    #[derive(Debug, Clone)]
    pub enum Msg { Clicked, HoverIn, HoverOut }
    
    pub fn view(state: &SidebarState) -> Node<Msg> {
        let bg = if state.hovered { BG_HOVER } else { BG_PANEL };
        view()
            .bg(bg)
            .interactive(true)
            .hover_bg(BG_HOVER)
            .on_hover(Msg::HoverIn)
            .on_click(Msg::Clicked)
            .children([...])
            .build()
    }
    
    pub fn update(state: &mut SidebarState, msg: Msg) {
        match msg {
            Msg::HoverIn => state.hovered = true,
            Msg::HoverOut => state.hovered = false,
            Msg::Clicked => { /* ... */ }
        }
    }
}

// App-level message (stays small)
#[derive(Debug, Clone)]
enum AppMsg {
    Sidebar(sidebar::Msg),
    Palette(palette::Msg),
    SendMessage,
}

fn update(app: &mut App, msg: AppMsg) {
    match msg {
        AppMsg::Sidebar(m) => sidebar::update(&mut app.sidebar, m),
        AppMsg::Palette(m) => palette::update(&mut app.palette, m),
        AppMsg::SendMessage => app.send_message(),
    }
}

fn view(app: &App) -> Node<AppMsg> {
    view()
        .children([
            sidebar::view(&app.sidebar).map_msg(AppMsg::Sidebar),
            palette::view(&app.palette).map_msg(AppMsg::Palette),
            messages_view(app),
        ])
        .build()
}

// Main loop — much simpler
loop {
    match event {
        Event::Mouse(mouse) => {
            let result = runtime.process_mouse_event(&mouse);
            for msg in result.messages {
                update(&mut app, msg);
            }
        }
        Event::Key(key) => { ... }
        _ => {}
    }
    let node = view(&app);
    runtime.render_to_buffer(&mut ctx, &node, w, h);
    let hover_msgs = runtime.recheck_hover();
    for msg in hover_msgs.messages { update(&mut app, msg); }
    renderer.present();
}
```

---

## Phase 7: Cleanup

- Delete duplicate `EventDispatcher` / `FocusManager` in `crates/core/src/event.rs`
- Delete unused `KeyBindingRegistry` in `crates/core/src/keybinding.rs`
- Merge duplicate `MouseDispatchResult` / `KeyDispatchResult` types
- Update `crates/core/src/prelude.rs` exports
- Update `crates/core/src/view/mod.rs` exports
- Update all tests in `crates/core/tests/`

---

## File Change Summary

| File | Change | Phase |
|------|--------|-------|
| `src/terminal/mouse.rs` | Add `Over`, `Out`, `Drop` to `MouseEventKind`; add fields to `MouseEvent` | 1 |
| `crates/core/src/view/event.rs` | **New**: `EventKind`, `EventBinding<M>` | 2 |
| `crates/core/src/view/node.rs` | Genericize to `Node<M>`, add `map_msg()` | 2 |
| `crates/core/src/view/element.rs` | Genericize to `Element<M>`, replace `action` with `events` | 2 |
| `crates/core/src/view/builder.rs` | Genericize to `ElementBuilder<M>`, add `on_click/on_hover/on_scroll`, `map_msg()` | 2 |
| `crates/core/src/view/props.rs` | Add `interactive`, `hover_bg`, `hover_fg` to `ViewProps` | 2 |
| `crates/core/src/view/rebuild.rs` | Rewrite `build_tree_with_events<M>()` | 2 |
| `crates/core/src/view/runtime.rs` | Complete rewrite: `ViewRuntime<M>`, dispatch engine | 3 |
| `crates/core/src/view/mod.rs` | Update exports | 2 |
| `crates/core/src/widget.rs` | Add `hovered_id` to `RenderContext` | 4 |
| `crates/core/src/widgets/view_widget.rs` | Apply hover styles in render | 4 |
| `crates/core/src/prelude.rs` | Update exports | 7 |
| `crates/core/src/lib.rs` | Update exports | 7 |
| `crates/core/examples/opencode.rs` | Migrate to new event system | 6 |
| `crates/core/tests/*.rs` | Update to new API | 7 |
| `crates/core/src/event.rs` | Remove duplicate dispatch system | 7 |
| `crates/core/src/keybinding.rs` | Remove unused registry | 7 |

---

## Future Phases (out of scope for initial implementation)

| Feature | Description |
|---------|------------|
| Selection system | Mouse drag selection with anchor/focus model |
| Mouse pointer style | `setMousePointer("pointer"/"text"/...)` via CSI sequences |
| Scroll acceleration | Variable multiplier based on scroll velocity |
| on_size_change | Layout change callbacks |
| render_before/after | Custom draw hooks |
| on_key / on_paste | Keyboard/paste events on declarative nodes |
| Focus events | onFocus / onBlur declarative bindings |
