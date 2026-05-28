# 声明式 View API — 实施计划与进度

> 基于 `docs/core/DECLARATIVE_VIEW_DESIGN.md` 设计文档
> 创建时间: 2026-05-28

## 当前状态

- **core crate**: ~5,230 LOC，包含完整的命令式 WidgetTree、LayoutEngine (Taffy)、12 个 Widget
- **已有 Widget**: BoxWidget, TextWidget, TextLineWidget, StyledTextWidget, InputWidget, ListWidget, ScrollViewWidget, EditorWidget, ProgressBarWidget, TabsWidget, StatusLineWidget, SeparatorWidget, FillWidget
- **已有能力**: focus chain, event dispatch, overlay, scrollbar, theme
- **核心问题**: 全部是命令式 API (`allocate_id()` + `add_child()`)，UI 结构与 ID 管理混在一起

## 设计决策

| 决策 | 选择 | 原因 |
|------|------|------|
| 模块结构 | `view/mod.rs` + 子文件 | 类型较多，分文件更清晰 |
| ViewWidget | Phase A 直接引入 | 一步到位，BoxWidget 保留兼容 |
| Phase A 范围 | 最小核心: view(), panel(), text(), fragment(), when(), empty() | 先验证核心管线 |
| 文本渲染 | Phase A 修复为 grapheme-aware | char-by-char 破坏 emoji 和组合字符 |

## 实施步骤

### Step 1: 修复 Widget::render 为 `&mut self` ✅

**状态**: 已完成

**原因**: 设计文档 §问题 3 — `render(&self)` 限制有状态 widget（ListWidget, InputWidget, ScrollViewWidget）

**改动**:
- `widget.rs:47` — `render(&self, ...)` → `render(&mut self, ...)`
- 所有 14 个 widget 实现文件 — 签名更新
- `widget.rs` execute_render_commands / render_overlays — borrow 重构
- `widget.rs` — 新增 `impl Widget for Box<dyn Widget>` 委托实现
- `layout.rs` — `ComputedLayout` 增加 `Copy` derive

**涉及文件**: `widget.rs`, `widgets/*.rs` (全部 14 个), `layout.rs`

### Step 2: 修复 TextLineWidget/StyledTextWidget 为 grapheme-aware ✅

**状态**: 已完成

**原因**: 设计文档 §问题 7 — 按 char 迭代破坏 multi-codepoint grapheme

**改动**:
- 使用引擎已有的 `ot::unicode::split_graphemes_with_widths()` 替代 `.chars()` 迭代
- `text_line_widget.rs` — 逐 grapheme 渲染
- `styled_text_widget.rs` — 同上
- 无需添加新依赖（通过 `opentui_rust` 复用）

**涉及文件**: `text_line_widget.rs`, `styled_text_widget.rs`

### Step 3: 新增 view 模块 — 类型定义 + builder ✅

**状态**: 已完成

**新建文件**:
```
crates/core/src/view/
├── mod.rs          # 模块声明 + re-exports
├── node.rs         # Node enum (Element / Fragment / Empty)
├── element.rs      # Element struct, ElementKind enum
├── key.rs          # Key enum (Static / Owned / IndexPath)
├── props.rs        # Props enum + ViewProps, TextProps
├── builder.rs      # view(), panel(), text(), fragment(), when(), empty()
```

**核心类型**:
```rust
pub enum Node { Element(Element), Fragment(Vec<Node>), Empty }
pub struct Element { kind, key, layout, props, children }
pub enum ElementKind { View, Text, Custom(&'static str) }
pub enum Key { Static(&'static str), Owned(String), IndexPath(Vec<u32>) }
pub enum Props { View(ViewProps), Text(TextProps), Empty }
```

**Builder API**: `view()`, `panel()`, `text()`, `fragment()`, `when()`, `empty()`

### Step 4: 新增 ViewWidget ✅

**状态**: 已完成

**新建文件**: `crates/core/src/widgets/view_widget.rs`

**设计**: 将 layout + decoration 分离，直接在 `layout` 字段维护布局属性

```rust
pub struct ViewWidget {
    id: WidgetId,
    layout: LayoutStyle,       // 纯布局属性
    bg: Option<Rgba>,
    border: Option<BorderStyle>,
    title: Option<String>,
    // ... 其他视觉/交互状态
}
```

### Step 5: Full Rebuild Builder (Node → WidgetTree) ✅

**状态**: 已完成

**新建文件**: `crates/core/src/view/rebuild.rs`

**职责**: 遍历 Node 树 → 分配 WidgetId → 创建 Widget → 构建 WidgetTree

### Step 6: ViewRuntime ✅

**状态**: 已完成

**新建文件**: `crates/core/src/view/runtime.rs`

**Phase A 最小实现**: `rebuild(&mut self, node)`, `layout()`, `render()`, `dispatch_key()`, `dispatch_mouse()`

### Step 7: 示例 + 集成测试 ✅

**状态**: 已完成

**新建文件**:
- `crates/core/examples/declarative_hello.rs` — 最小声明式示例
- `crates/core/tests/view_builder.rs` — 15 个 builder 单元测试
- `crates/core/tests/view_runtime.rs` — 5 个 rebuild + render 集成测试

### Step 8: lib.rs + prelude 更新 ✅

**状态**: 已完成

**改动**: `lib.rs` 添加 `pub mod view;`, `prelude.rs` 添加 view 类型和 ViewWidget 导出

## 文件变更总览

| 操作 | 文件 | Step |
|------|------|------|
| 修改 | `crates/core/src/widget.rs` | 1 |
| 修改 | `crates/core/src/widgets/*.rs` (全部 14 个) | 1 |
| 修改 | `crates/core/Cargo.toml` | 2 |
| 修改 | `crates/core/src/widgets/text_line_widget.rs` | 2 |
| 修改 | `crates/core/src/widgets/styled_text_widget.rs` | 2 |
| 新建 | `crates/core/src/view/mod.rs` | 3 |
| 新建 | `crates/core/src/view/node.rs` | 3 |
| 新建 | `crates/core/src/view/element.rs` | 3 |
| 新建 | `crates/core/src/view/key.rs` | 3 |
| 新建 | `crates/core/src/view/props.rs` | 3 |
| 新建 | `crates/core/src/view/builder.rs` | 3 |
| 新建 | `crates/core/src/widgets/view_widget.rs` | 4 |
| 新建 | `crates/core/src/view/rebuild.rs` | 5 |
| 新建 | `crates/core/src/view/runtime.rs` | 6 |
| 新建 | `crates/core/examples/declarative_hello.rs` | 7 |
| 新建 | `crates/core/tests/view_builder.rs` | 7 |
| 新建 | `crates/core/tests/view_runtime.rs` | 7 |
| 修改 | `crates/core/src/lib.rs` | 8 |
| 修改 | `crates/core/src/prelude.rs` | 8 |
| 修改 | `crates/core/src/widgets/mod.rs` | 4 |

## 依赖关系

```
Step 1 (render &mut self) ─────────────────────────────────────┐
  └→ Step 2 (grapheme-aware) ──→ Step 4 (ViewWidget) ─────────┤
                                       Step 3 (view 类型) ────┤
                                                              ↓
                                           Step 5 (rebuild) ←─┘
                                              ↓
                                           Step 6 (runtime)
                                              ↓
                                           Step 7 (示例+测试)
                                              ↓
                                           Step 8 (lib 更新)
```

## Phase B（后续，不在本次范围）

- Keyed reconciliation (key_map, props patch, child reorder)
- `input()`, `rich_text()`, `span()`, `list()` builder
- Overlay as Node-level description
- Event action routing (`on_click(Action)`)
- `virtual_list()` for large datasets

## 预估工作量

| Step | 新增 LOC | 修改 LOC |
|------|----------|----------|
| Step 1 | ~30 | ~60 |
| Step 2 | ~40 | ~30 |
| Step 3 | ~350 | 0 |
| Step 4 | ~200 | 0 |
| Step 5 | ~150 | 0 |
| Step 6 | ~80 | 0 |
| Step 7 | ~400 | 0 |
| Step 8 | ~10 | ~5 |
| **合计** | **~1,260** | **~95** |

## 进度日志

### 2026-05-28

- [x] 创建实施计划文档
- [x] Step 1: Widget::render &mut self — 所有 14 个 widget + Box<dyn Widget> 委托实现
- [x] Step 2: grapheme-aware text — 使用 ot::unicode::split_graphemes_with_widths()
- [x] Step 3: view 模块 — Node, Element, Key, Props, Builder (6 个文件)
- [x] Step 4: ViewWidget — layout + decoration 分离的新 widget
- [x] Step 5: rebuild builder — Node → WidgetTree 转换
- [x] Step 6: ViewRuntime — rebuild/layout/render/dispatch 完整生命周期
- [x] Step 7: 示例 + 测试 — declarative_hello.rs + 20 个新测试
- [x] Step 8: lib 更新 — pub mod view + prelude 导出

**测试结果**: 57/57 通过 (4 单元 + 9 集成 + 4 快照 + 15 builder + 5 runtime + 19 widget_tree + 1 doctest)

**验证命令**:
- `cargo check -p opentui-core --all-targets` ✅
- `cargo test -p opentui-core` ✅ (57 passed)
- `cargo fmt -p opentui-core -- --check` ✅
