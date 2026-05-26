# opentui-core 实现计划

## 总览

将 `opentui-core` 从"基础骨架"演进为"可用的 UI 框架层"。

```
Phase 0: 修复 + 清理          ← 当前
Phase 1: Widget 生态完善
Phase 2: 事件系统深度集成
Phase 3: 高级 Widget
Phase 4: 示例 + 测试
Phase 5: 打磨 + 文档
```

## Phase 0: 修复 + 清理（~1 session）

### 0.1 修复 BoxWidget children 管理

**问题：** `BoxWidget.children: Vec<WidgetId>` 字段从未使用。WidgetTree 内部维护 children，Widget::children() 应委托给 WidgetTree。

**方案：**
- 移除 BoxWidget 的 `children` 字段
- WidgetTree 中的 `WidgetNode.children` 是唯一的 children 来源
- Widget::children() 的默认实现改为 `&[]`，WidgetTree 在 render 时忽略它
- 或者：给 Widget::children() 加一个 `children_override` 机制

**文件：** `widgets/box_widget.rs`, `widget.rs`

### 0.2 ScrollState 统一精度

**问题：** ScrollState 用 f64，opentui_rust 全部用 f32。

**方案：** 将 `ScrollState.offset_y`、`content_height` 改为 f32。f64 的精度对终端行号完全不需要。

**文件：** `scroll.rs`, `list.rs`

### 0.3 Theme 命名消歧

**问题：** `opentui_core::Theme` vs `opentui_rust::highlight::Theme` 同名不同义。

**方案：** 在 `opentui-core` 中重命名为 `UiTheme` / `UiThemeRegistry`，或在 re-export 时标注用途。

**文件：** `theme.rs`, `lib.rs`

## Phase 1: Widget 生态完善（2-3 sessions）

### 1.1 ScrollViewWidget

将 `ScrollView` 从静态函数重构为 Widget：

```rust
pub struct ScrollViewWidget {
    id: WidgetId,
    style: LayoutStyle,
    state: ScrollState,
    scrollbar: bool,
    scrollbar_style: ScrollBarStyle,
    // child content 在 render callback 中绘制
}
```

**要点：**
- 实现 Widget trait
- render() 中 push_scissor → render_children → pop_scissor → draw_scrollbar
- handle_key/handle_mouse 委托给 ScrollState
- overflow 自动设为 Hidden

**文件：** `widgets/scroll_view_widget.rs`（新建）

### 1.2 ListWidget

将 VirtualList 包装为 Widget：

```rust
pub struct ListWidget<R: ItemRenderer> {
    id: WidgetId,
    style: LayoutStyle,
    state: VirtualListState,
    renderer: R,
    scrollbar: bool,
}
```

**要点：**
- 泛型参数 `R: ItemRenderer`
- render() 内部调用 VirtualList::render
- handle_key/handle_mouse 委托给 VirtualListState
- HitGrid 集成

**文件：** `widgets/list_widget.rs`（新建）

### 1.3 InputWidget

单行文本输入：

```rust
pub struct InputWidget {
    id: WidgetId,
    style: LayoutStyle,
    buffer: EditBuffer,
    placeholder: Option<String>,
    cursor_visible: bool,
    // ...
}
```

**要点：**
- 包装 EditBuffer（单行模式，最大一行）
- handle_key 处理字符输入、Backspace、Delete、Home/End、Ctrl+A/U/K
- render 时显示光标（toggle blink state）
- 可选 placeholder（空内容时显示灰色提示文字）
- 可选 password 模式（显示 `***`）

**文件：** `widgets/input_widget.rs`（新建）

### 1.4 EditorWidget

多行编辑器：

```rust
pub struct EditorWidget {
    id: WidgetId,
    style: LayoutStyle,
    editor: EditorView,
    line_numbers: bool,
    // ...
}
```

**要点：**
- 包装 EditorView + EditBuffer
- handle_key 委托给 EditorView
- render 调用 `buffer.draw_editor_view()`
- 行号、状态行可选
- ScrollBar 集成

**文件：** `widgets/editor_widget.rs`（新建）

### 1.5 ProgressBarWidget

```rust
pub struct ProgressBarWidget {
    id: WidgetId,
    style: LayoutStyle,
    progress: f32,       // 0.0 - 1.0
    label: Option<String>,
    style_chars: ProgressStyle,
}
```

**要点：**
- 水平进度条（`[████░░░░░] 42%`）
- 可自定义填充/空白/边界字符
- 可选百分比标签

**文件：** `widgets/progress_widget.rs`（新建）

## Phase 2: 事件系统深度集成（1-2 sessions）

### 2.1 WidgetTree + EventDispatcher 统一

**当前问题：** EventDispatcher 和 WidgetTree 是两个独立系统，没有连接。

**方案：**

```rust
impl WidgetTree {
    pub fn dispatch_key(&mut self, key: &KeyEvent) -> Option<WidgetId> {
        // 找到 focused widget
        // 调用 widget.handle_key()
        // 如果 consumed，返回 widget id
        // 否则尝试 Tab/Shift+Tab 焦点切换
    }

    pub fn dispatch_mouse(&mut self, mouse: &MouseEvent, hit_grid: &HitGrid) -> Option<WidgetId> {
        // hit_test 找到目标
        // 调用 widget.handle_mouse()
        // 如果 focusable，设置焦点
    }

    pub fn focus_next(&mut self) { /* Tab */ }
    pub fn focus_prev(&mut self) { /* Shift+Tab */ }
}
```

**文件：** `widget.rs`（扩展）

### 2.2 FocusChain

当前 FocusManager 用 Vec<FocusId> + index 管理 Tab 顺序。改为自动从 WidgetTree 提取：

```rust
impl WidgetTree {
    pub fn build_focus_chain(&mut self) {
        // DFS 遍历 tree
        // 收集所有 focusable() == true 的 widget
        // 按 DFS 顺序排列
    }

    pub fn focus_next(&mut self) -> Option<WidgetId> { ... }
    pub fn focus_prev(&mut self) -> Option<WidgetId> { ... }
}
```

**文件：** `widget.rs`（扩展）

### 2.3 KeyBindings 系统（可选）

```rust
pub struct KeyBindingRegistry {
    bindings: HashMap<(KeyModifiers, KeyCode), &'static str>,
}

impl KeyBindingRegistry {
    pub fn bind(&mut self, mods: KeyModifiers, code: KeyCode, action: &'static str);
    pub fn resolve(&self, key: &KeyEvent) -> Option<&str>;
}
```

**文件：** `keybinding.rs`（新建）

## Phase 3: 高级 Widget（1-2 sessions）

### 3.1 TabsWidget

```rust
pub struct TabsWidget {
    id: WidgetId,
    style: LayoutStyle,
    tabs: Vec<Tab>,
    active: usize,
}
```

- 水平标签栏 + 内容区
- 可键盘切换（Ctrl+Tab / Ctrl+Shift+Tab）
- 标签溢出处理（截断或滚动）

### 3.2 StatusLineWidget

```rust
pub struct StatusLineWidget {
    id: WidgetId,
    style: LayoutStyle,
    left: String,
    center: String,
    right: String,
}
```

- 固定高度 1 行
- 左/中/右三段式布局
- 典型用途：文件名、模式、行列号

### 3.3 Modal / Overlay

```rust
pub struct Overlay {
    x: f32, y: f32,
    width: f32, height: f32,
    widget_id: WidgetId,
    z_order: u16,
}
```

- 绝对定位，脱离正常布局流
- Z-order 排序
- 典型用途：下拉菜单、弹窗、tooltip

## Phase 4: 示例 + 测试（2 sessions）

### 4.1 示例

| 示例 | 展示 |
|------|------|
| `core_hello.rs` | BoxWidget + TextWidget + 基础布局 |
| `core_dashboard.rs` | 多面板 dashboard，focus 切换 |
| `core_list.rs` | VirtualList + 键盘导航 |
| `core_editor.rs` | EditorWidget 完整编辑器 |
| `core_theme.rs` | 主题切换 |

### 4.2 测试

| 测试类别 | 覆盖范围 |
|----------|----------|
| LayoutEngine 单元测试 | column/row/nesting/padding/gap/percent |
| WidgetTree 单元测试 | add/remove/layout/render |
| BoxWidget 单元测试 | border/bg/title/focus-color |
| TextWidget 单元测试 | wrap/truncate/scroll |
| ScrollViewWidget 测试 | scroll state + viewport culling |
| ListWidget 测试 | select/scroll/hit-test |
| InputWidget 测试 | type/delete/cursor |
| FocusManager 测试 | register/focus/next/prev |
| EventDispatcher 测试 | mouse hit-test / key dispatch |
| 集成测试 | 完整 tree → layout → render pipeline |

### 4.3 测试基础设施

```rust
// 辅助函数：创建 buffer + tree → render → snapshot
fn render_tree_to_string(tree: &mut WidgetTree, width: u32, height: u32) -> String {
    let mut buffer = OptimizedBuffer::new(width, height);
    let mut ctx = RenderContext { buffer: &mut buffer, .. };
    tree.layout(width as f32, height as f32);
    tree.render(&mut ctx);
    buffer_to_string(&buffer)
}
```

## Phase 5: 打磨 + 文档（1 session）

### 5.1 API 文档

- 每个公开类型加 doc comments
- module-level 文档加 `//!` 模块说明
- 加 `# Example` 代码块

### 5.2 性能基准

- LayoutEngine: 嵌套深度 10/50/200 的布局计算时间
- WidgetTree: 50/200/1000 节点的 render 时间
- BoxWidget: 100 个 box 的渲染时间
- 与直接使用 opentui_rust 的开销对比

### 5.3 发布准备

- `Cargo.toml` 补全 metadata（description, license, repository, keywords, categories）
- CI 集成：`cargo test -p opentui-core`
- changelog

## 优先级排序

| 优先级 | 任务 | 原因 |
|--------|------|------|
| P0 | Phase 0（修复 + 清理） | 技术债会累积 |
| P0 | Phase 4.2（测试） | 没有测试 = 不能重构 |
| P1 | Phase 1.1-1.2（ScrollView/List Widget） | 核心布局组件 |
| P1 | Phase 2.1-2.2（事件集成） | 不集成就不能实际使用 |
| P1 | Phase 4.1（示例） | 没有示例 = 没人会用 |
| P2 | Phase 1.3（InputWidget） | 常用但非必须 |
| P2 | Phase 1.4（EditorWidget） | 同上 |
| P2 | Phase 1.5（ProgressBar） | 简单且有用 |
| P3 | Phase 2.3（KeyBindings） | 锦上添花 |
| P3 | Phase 3.1-3.3（高级 Widget） | 可以后续迭代 |
| P3 | Phase 5（打磨） | 最后做 |

## 预估规模

| Phase | 新增 LOC | 累计 LOC |
|-------|----------|----------|
| 当前 | - | 2,139 |
| Phase 0 | ~100 | 2,239 |
| Phase 1 | ~1,500 | 3,739 |
| Phase 2 | ~500 | 4,239 |
| Phase 3 | ~600 | 4,839 |
| Phase 4 | ~1,200 | 6,039 |
| Phase 5 | ~200 | 6,239 |

**目标：** ~6,000 LOC 的精炼 UI 框架层，对比原版 TypeScript 层估计 ~3,000-5,000 LOC。
