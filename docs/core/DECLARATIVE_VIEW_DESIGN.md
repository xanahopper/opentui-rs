# opentui-core 声明式 View API 设计

## 背景

`opentui_rust` 当前定位是高性能终端渲染引擎：它提供 `Renderer`、`OptimizedBuffer`、cell、style、text、input、terminal 等底层能力，但刻意不提供应用结构、组件树或事件循环。

`opentui-core` 已经在渲染层之上提供了一个更高层的 UI 层：

- `LayoutEngine` / `LayoutStyle`：基于 Taffy 的 flex/grid 布局包装
- `Widget` / `WidgetTree`：命令式 widget 树
- `RenderCommand`：渲染命令列表，用于 scissor/opacity 的两阶段执行
- `BoxWidget`、`TextWidget`、`InputWidget`、`ListWidget` 等具体 widget
- 焦点、overlay、scroll、theme、keybinding 等上层能力

但是当前使用方式仍然偏命令式。普通应用需要手动分配 `WidgetId`、手动 `add_child()`、手动更新 widget 字段。对于 OpenCode 这种界面，代码会很快变成大量重复的树构建样板。

本设计的目标是在现有 `WidgetTree` 之上增加一层 Rust 风格的声明式 builder API，达到接近 OpenTUI/OpenCode JSX 的表达力，但不使用 `tui!` 宏，也不引入 HTML-like 语法。

## 目标

1. 使用普通 Rust builder 表达 UI：

   ```rust
   use opentui_core::view::*;

   fn ui(app: &App) -> Node {
       view()
           .key("root")
           .row()
           .size_pct(1.0, 1.0)
           .bg(BG)
           .children([
               view()
                   .key("main")
                   .column()
                   .grow(1.0)
                   .children([
                       messages_view(app),
                       prompt_view(app),
                   ]),
               when(app.sidebar_visible, || sidebar_view(app)),
           ])
   }
   ```

2. `view()` 成为通用容器节点，替代 `box_()`/`container()` 这类命名。

3. 保留现有渲染层和 widget 层：声明式 API 只负责描述 UI 和同步 `WidgetTree`。

4. 第一阶段不追求复杂 keyed diff。先做 full rebuild，稳定 API 后再做 reconciliation。

5. 将普通应用示例从手动 `allocate_id()` / `add_child()` 迁移到 `fn ui(state) -> Node`。

## 非目标

1. 不设计 `tui! { ... }`、`rsx! { ... }` 或 HTML-like 宏语法。
2. 不把 `opentui_rust` 底层 renderer 改造成框架。
3. 不引入 React 式 hook 系统作为第一阶段目标。
4. 不让 `Node` 直接绘制到 `OptimizedBuffer`。绘制仍然由 widget 和 `WidgetTree` 完成。
5. 不在第一版实现完整 VDOM diff、生命周期回调、事件冒泡捕获模型。

## 当前 Core 的主要问题

### 1. WidgetTree 是命令式树

当前典型用法：

```rust
let mut tree = WidgetTree::new();
let root = tree.add(BoxWidget::new(tree.allocate_id(), LayoutStyle::row()));
let child = tree.add_child(
    root,
    TextLineWidget::with_text(tree.allocate_id(), LayoutStyle::default(), "hello"),
);
```

问题：

- UI 结构和 ID 管理混在一起
- 动态 UI 需要手动重建/移除/更新
- 示例代码被大量样板污染
- 很难从应用状态直接看出界面结构

声明式层应将应用代码改为：

```rust
fn ui(app: &App) -> Node {
    view()
        .row()
        .children([
            text("hello"),
            when(app.sidebar_visible, || sidebar(app)),
        ])
}
```

### 2. WidgetId 不适合声明式更新

`WidgetTree::allocate_id()` 只是单调递增，不知道外部是否手写了冲突 ID。当前 examples 里大量手动保存 ID。

声明式层应改用：

- `key("main")`：应用可读的稳定身份
- path key：未显式 key 时由树路径生成临时身份
- `WidgetId`：内部实现细节，由 reconciler 分配和复用

建议：

```rust
pub enum Key {
    Static(&'static str),
    Owned(String),
    IndexPath(Vec<u32>),
}
```

### 3. Widget::render(&self, ...) 限制有状态 widget

当前 trait：

```rust
fn render(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout);
```

这导致 `ListWidget` 无法在 `render()` 内正常调用 `VirtualList::render()`，因为后者需要 `&mut VirtualListState`。现在 `ListWidget::render()` 是空实现，另开 `render_with_renderer()`。

这说明 widget render 阶段需要可变访问：

```rust
fn render(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout);
```

影响：

- `InputWidget` 可以在渲染时更新 cursor position
- `ListWidget` 可以正常维护 scroll viewport
- `ScrollViewWidget` 可以更新 scroll state
- overlay 和 focus 状态更新更自然

### 4. BoxWidget 的 style 状态不一致

`BoxWidget` 同时维护：

- `user_style`
- `effective_style`
- `base_padding`
- border 派生 padding

`border()` 会调用 `recompute_effective_style()`，但 `WidgetTree::set_widget_style()` 直接写 `style_mut()`，不会触发重算。

声明式 props patch 会频繁更新 layout 和 border。如果不修复，这里会产生布局错误。

建议拆分：

```rust
pub struct ViewWidget {
    id: WidgetId,
    layout: LayoutStyle,
    decoration: ViewDecoration,
    state: WidgetState,
}

pub struct ViewDecoration {
    pub bg: Option<Rgba>,
    pub border: Option<BorderStyle>,
    pub title: Option<String>,
    pub title_align: TitleAlign,
}
```

`style()` 返回 layout 之前统一计算 effective layout，或者由声明式层提前生成 final `LayoutStyle`，避免 widget 内部藏双状态。

### 5. Hit testing 没有统一接入

`RenderContext` 已有：

```rust
pub hit_grid: Option<&mut ot::renderer::HitGrid>,
```

但多数 example 传 `None`。`WidgetTree::dispatch_mouse()` 又依赖 hit grid 查找目标。结果是 hit testing 的数据流没有闭合。

声明式层应在 layout 后自动注册交互节点：

```rust
view()
    .key("send-button")
    .focusable()
    .on_click(Action::Send)
```

内部行为：

1. layout 计算出 rect
2. 如果 node 可交互，注册 `WidgetId -> rect` 到 hit grid
3. mouse event 通过 hit grid 找 `WidgetId`
4. `WidgetTree` 调用目标 widget 的 handler，或返回声明式 action

### 6. Overlay 只能绘制单 widget

当前 `render_overlays()` 会按 z-order 找 overlay widget，并直接调用其 `render()`。这绕过了正常子树递归、layout 和 render command。

OpenCode 常见 overlay：

- command palette
- modal dialog
- tooltip
- dropdown
- backdrop

这些都需要完整 subtree。

建议 overlay 也由 `Node` 描述：

```rust
view()
    .children([...])
    .overlay(
        panel()
            .key("command-palette")
            .modal()
            .backdrop()
            .children([...])
    )
```

### 7. 文本绘制路径不一致

`TextWidget` 走 `TextBufferView`，但 `TextLineWidget` / `StyledTextWidget` 手动按 `char` 写 cell。这会带来：

- multi-codepoint grapheme 处理不一致
- wide char continuation cell 处理不完整
- `grapheme_pool` 无法统一使用

声明式文本节点应统一走底层 text/grapheme-aware API。

## 设计概览

整体分层：

```text
Application state
    |
    v
fn ui(state) -> Node
    |
    v
opentui_core::view
    - Node
    - Element
    - builders: view(), panel(), text(), input(), list(), when()
    |
    v
rebuild / reconcile
    |
    v
WidgetTree
    |
    v
LayoutEngine -> RenderCommand -> OptimizedBuffer
    |
    v
Renderer::present()
```

`Node` 是描述，不拥有 renderer，不直接绘制。

`WidgetTree` 是执行结构，负责 layout、render、focus、event dispatch。

`Renderer` 仍然负责底层 diff、ANSI 输出、layer、grapheme pool、hit grid。

## Public API 草案

### 基础节点类型

```rust
pub enum Node {
    Element(Element),
    Fragment(Vec<Node>),
    Empty,
}

pub struct Element {
    pub kind: ElementKind,
    pub key: Option<Key>,
    pub layout: LayoutStyle,
    pub props: Props,
    pub children: Vec<Node>,
}

pub enum ElementKind {
    View,
    Text,
    StyledText,
    Input,
    List,
    ScrollView,
    Progress,
    Tabs,
    Custom(&'static str),
}
```

### view()

`view()` 是通用容器。它可以只负责 layout，也可以有背景、边框、overflow、opacity、focusability。

```rust
view()
    .key("main")
    .column()
    .grow(1.0)
    .padding_all(1.0)
    .gap(1.0)
    .bg(BG_PANEL)
    .overflow_hidden()
    .children([
        text("Session"),
        text("abc123"),
    ])
```

### panel()

`panel()` 是 `view()` 的视觉便捷版本。它不是必须的底层 kind，也可以只是 builder preset。

```rust
panel()
    .key("logs")
    .title("Recent Events")
    .border_rounded(BORDER)
    .bg(BG_PANEL)
    .children([...])
```

等价于：

```rust
view()
    .border_rounded(BORDER)
    .bg(BG_PANEL)
    .title("Recent Events")
```

### text()

单行或简单文本节点：

```rust
text("OpenCode")
    .key("title")
    .fg(TEXT)
    .bg(BG_PANEL)
    .bold()
    .height(1.0)
```

建议第一阶段将 `text()` 映射到改造后的 grapheme-aware text widget，而不是继续使用当前 `TextLineWidget` 的 char-by-char 写法。

### rich_text()

带 inline segments：

```rust
rich_text([
    span("git: ").fg(TEXT_MUTED),
    span("main").fg(SUCCESS).bold(),
])
.height(1.0)
```

### input()

单行输入：

```rust
input(&app.input_text)
    .key("prompt-input")
    .placeholder("Ask anything")
    .focusable()
    .height(1.0)
```

第一阶段只负责渲染当前 value。后续再引入 controlled/uncontrolled 策略。

### list()

初版建议不要急着做泛型 closure renderer。先设计声明式 item：

```rust
list(app.commands.iter().enumerate().map(|(idx, cmd)| {
    view()
        .key(cmd.name)
        .height(1.0)
        .bg(if idx == app.selected { SELECTED } else { BG })
        .children([
            text(cmd.name).fg(TEXT),
            text(cmd.shortcut).fg(TEXT_MUTED),
        ])
}))
```

后续如果性能需要，再提供 `virtual_list()`。

### 条件和集合

```rust
when(app.sidebar_visible, || sidebar(app))

fragment(app.messages.iter().map(message_view))

empty()
```

建议 `children()` 接受 `IntoChildren`，兼容数组、Vec、iterator collect 后的 Vec。

## Builder 命名建议

### 推荐命名

| API | 语义 |
|-----|------|
| `view()` | 通用 layout/visual 容器 |
| `panel()` | 带边框/title 背景的视觉容器 |
| `text()` | 文本节点 |
| `span()` | rich text 片段 |
| `input()` | 单行输入 |
| `fragment()` | 多节点分组 |
| `when()` | 条件节点 |
| `empty()` | 空节点 |

### 避免命名

| API | 原因 |
|-----|------|
| `box()` | `box` 是 Rust 关键字 |
| `r#box()` | 调用点不好看 |
| `Box::new()` | 容易和 `std::boxed::Box` 混淆 |
| `container()` | 准确但偏长，视觉上不如 `view()` 简洁 |
| HTML-like tag | 不符合本设计的非宏目标 |

## Widget 映射

第一阶段映射表：

| ElementKind | Widget |
|-------------|--------|
| `View` | `BoxWidget` 或新 `ViewWidget` |
| `Text` | 新 grapheme-aware `TextLineWidget` |
| `StyledText` | 改造后的 `StyledTextWidget` |
| `Input` | `InputWidget` |
| `Progress` | `ProgressBarWidget` |
| `Tabs` | `TabsWidget` |
| `ScrollView` | `ScrollViewWidget` |
| `List` | 普通 child list 或 `ListWidget` |

建议新增 `ViewWidget`，然后逐步让 `BoxWidget` 成为兼容别名或迁移目标：

- `view()` 映射 `ViewWidget`
- `panel()` 映射 `ViewWidget + border/title props`
- 老 `BoxWidget` 暂时保留，避免一次性打断全部 tests/examples

## Rebuild 与 Reconcile

### Phase A: Full rebuild

每帧：

1. app 调用 `ui(&state) -> Node`
2. `build_widget_tree(node) -> WidgetTree`
3. `tree.layout(width, height)`
4. `tree.render(ctx)`
5. `renderer.present()`

优点：

- 实现简单
- API 可以快速验证
- 终端 UI 节点数通常很少，初期足够快
- 避免过早设计 diff 机制

缺点：

- widget 内部状态会丢失
- focus/scroll/input cursor 需要外部 state 或临时 state map
- 大列表性能一般

### Phase B: Keyed reconciliation

引入：

```rust
pub struct ViewRuntime {
    tree: WidgetTree,
    key_map: HashMap<NodeIdentity, WidgetId>,
    state: WidgetStateStore,
}
```

每帧：

1. app 生成新 `Node`
2. runtime 根据 key/path 复用 widget
3. props 更新已有 widget
4. 删除不存在节点
5. 插入新增节点
6. 重新 layout/render

核心身份：

```text
explicit key: parent_identity + key
implicit key: parent_identity + child_index + element_kind
```

建议规则：

- 动态列表必须显式 `.key(...)`
- 无 key 节点仅保证同一父节点下按 index 复用
- key 在同一 siblings 内必须唯一

## Props Patch

声明式层需要把 `Element.props` 应用到已有 widget。

初版可以用 enum：

```rust
pub enum Props {
    View(ViewProps),
    Text(TextProps),
    StyledText(StyledTextProps),
    Input(InputProps),
    Progress(ProgressProps),
    Empty,
}
```

每个 props 类型显式字段：

```rust
pub struct ViewProps {
    pub bg: Option<Rgba>,
    pub border: Option<BorderStyle>,
    pub title: Option<String>,
    pub title_align: TitleAlign,
    pub overflow: Overflow,
    pub opacity: f32,
    pub focusable: bool,
    pub visible: bool,
}
```

避免早期就引入 `HashMap<String, Value>`。Rust 里结构化 props 更可靠，也更容易通过 clippy。

## Event 模型

第一阶段建议事件仍然由应用处理，core 只提供 routing。

```rust
let result = runtime.dispatch_event(&event);
match result.action {
    Some(Action::Send) => app.send(),
    Some(Action::ToggleSidebar) => app.toggle_sidebar(),
    None => {}
}
```

后续可以让 builder 挂 action：

```rust
view()
    .key("send")
    .focusable()
    .on_click(Action::Send)
```

不要在第一阶段引入闭包回调存储在 `Node` 中。闭包会带来生命周期、trait object、clone、diff equality 等问题。更稳的模型是：

- UI 描述 action id
- runtime 返回 action id
- app 根据 action id 更新 state

## Focus 模型

建议：

- focusable 来自 node props
- `WidgetTree::build_focus_chain()` 仍按 DFS
- focused widget id 保存在 runtime
- rebuild 后通过 key 恢复 focused widget

示例：

```rust
input(&app.input)
    .key("prompt")
    .focusable()
    .focused(app.focus == FocusTarget::Prompt)
```

长期可以支持：

```rust
runtime.focus_key("prompt");
runtime.focus_next();
runtime.focus_prev();
```

## Layout API 改进

当前 `LayoutStyle::width_percent(pct)` 直接接收 f32，但旧文档有 `100.0` 风格，示例中又可能想用 `1.0` 表达 100%。需要统一。

建议：

- 底层 `LayoutStyle::width_percent(1.0)` 表示 100%
- 文档全部使用 `1.0`
- 可选新增 `width_pct(100.0)` 不推荐

`view()` builder 提供常用快捷方法：

```rust
view()
    .row()
    .column()
    .width(42.0)
    .height(1.0)
    .size(80.0, 24.0)
    .width_pct(1.0)
    .height_pct(1.0)
    .size_pct(1.0, 1.0)
    .grow(1.0)
    .shrink(0.0)
    .padding_all(1.0)
    .padding_x(2.0)
    .padding_y(1.0)
    .gap(1.0)
```

这些只是代理到 `LayoutStyle`，不改变底层布局模型。

## RenderContext 改进

当前：

```rust
pub struct RenderContext<'a> {
    pub buffer: &'a mut OptimizedBuffer,
    pub grapheme_pool: Option<&'a mut GraphemePool>,
    pub link_pool: Option<&'a mut LinkPool>,
    pub hit_grid: Option<&'a mut HitGrid>,
    pub theme: Option<&'a UiTheme>,
}
```

建议增加专门的 runtime render entry：

```rust
pub struct ViewRuntime;

impl ViewRuntime {
    pub fn render_to_renderer(&mut self, renderer: &mut Renderer, node: Node);
    pub fn render_to_buffer(&mut self, ctx: &mut RenderContext<'_>, node: Node, size: Size);
}
```

这样应用不需要手动拆 `renderer.buffer_with_pool()`。

## Overlay 设计

新增 node-level overlay：

```rust
pub struct OverlayNode {
    pub node: Node,
    pub z_order: OverlayZOrder,
    pub backdrop: bool,
    pub placement: OverlayPlacement,
}
```

Placement：

```rust
pub enum OverlayPlacement {
    Absolute { x: f32, y: f32, width: f32, height: f32 },
    Centered { width: f32, height: f32 },
    Fullscreen,
}
```

Builder：

```rust
panel()
    .key("palette")
    .overlay()
    .modal()
    .centered(72.0, 18.0)
    .backdrop()
```

第一阶段可以只做 `Absolute` 和 `Centered`。

## 迁移策略

### Step 1: 新增 view 模块

文件：

- `crates/core/src/view.rs`
- `crates/core/src/lib.rs`
- `crates/core/src/prelude.rs`

内容：

- `Node`
- `Element`
- `ElementKind`
- `Key`
- `Props`
- builders: `view()`, `panel()`, `text()`, `fragment()`, `when()`, `empty()`

### Step 2: 新增 full rebuild builder

文件：

- `crates/core/src/view/build.rs` 或 `crates/core/src/view_builder.rs`

功能：

- 遍历 `Node`
- 分配 `WidgetId`
- 创建对应 widget
- 调用 `WidgetTree::add()` / `add_child()`

### Step 3: 修改 Widget::render 为 &mut self

影响文件：

- `crates/core/src/widget.rs`
- `crates/core/src/widgets/*.rs`
- tests

收益：

- 修正 `ListWidget::render()` 空实现
- 简化 input/list/scroll 状态更新

### Step 4: 增加 ViewRuntime

第一版：

```rust
pub struct ViewRuntime {
    tree: WidgetTree,
}
```

方法：

```rust
pub fn rebuild(&mut self, node: Node);
pub fn layout(&mut self, width: f32, height: f32);
pub fn render(&mut self, ctx: &mut RenderContext<'_>);
```

### Step 5: 迁移 OpenCode 示例

迁移目标：

- `crates/core/examples/opencode_declarative.rs`

要求：

- 不再手动 `allocate_id()`
- 不再手动 `add_child()`
- UI 主体由 `fn ui(app: &App) -> Node` 表达

### Step 6: Keyed reconciliation

在 API 稳定后实现。

关键内容：

- `key_map`
- props patch
- child reorder
- removed widget cleanup
- focus restore
- scroll/input state preserve

## 测试计划

### Unit tests

1. `view()` creates `ElementKind::View`
2. `panel()` sets border/title props
3. `text()` stores text props
4. `when(false, ...)` returns `Node::Empty`
5. `fragment()` preserves child order
6. duplicate sibling keys report error or deterministic behavior

### Integration tests

1. build simple tree:

   ```rust
   view().children([text("hello")])
   ```

   renders `hello` at expected cell.

2. layout:

   ```rust
   view().row().children([
       view().width(10.0),
       view().grow(1.0),
   ])
   ```

   computed layout matches expected width.

3. focus:

   two focusable inputs cycle with Tab.

4. overlay:

   centered modal draws above background.

5. grapheme:

   `text("👩‍💻")` renders with correct width/continuation behavior.

### Example validation

Run:

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

For substantive code changes, also run targeted tests:

```bash
cargo test -p opentui-core
```

## Open Questions

1. Should `view()` map to existing `BoxWidget` first, or introduce `ViewWidget` immediately?
2. Should `text()` be single-line by default, with `paragraph()` for wrapped text?
3. Should action routing use typed enum supplied by app, or string/action id?
4. Should `Node` own all strings, or support borrowed strings with lifetimes?
5. Should full rebuild be accepted long-term for simple apps, or only as temporary implementation?

## Recommended Decisions

1. Introduce `ViewWidget` now, keep `BoxWidget` as compatibility.
2. Make `text()` single-line by default; use `paragraph()` or `text_block()` for multi-line/wrapped text later.
3. Use string/static action IDs first; typed app-specific actions can come later through generics.
4. Let `Node` own strings in v1. Simpler lifetimes matter more than avoiding small allocations.
5. Ship full rebuild first, then keyed reconciliation once `opencode_declarative` proves the API.

## Summary

The right next layer for `opentui-core` is not a macro and not HTML-like syntax. It is a Rust-native builder DSL centered on `view()`.

The proposed architecture keeps the current rendering engine intact, reuses `WidgetTree` as the execution model, and adds a declarative `Node` tree above it. The first milestone should prioritize API clarity and example migration over reconciliation complexity. Once the shape is proven, keyed reconciliation can preserve focus, scroll, and input state without changing application-facing code.
