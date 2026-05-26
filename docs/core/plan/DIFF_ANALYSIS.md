# opentui-core 与原版 OpenTUI 的差异分析

## 1. 架构定位

原版 OpenTUI 是一个 **TypeScript + Zig** 项目，包含三层：

| 层 | 技术 | 职责 |
|----|------|------|
| Zig Core | ~15,900 LOC | 高性能渲染引擎（buffer/cell/color/text/rope/renderer/terminal） |
| TypeScript API | JS wrapper | 通过 FFI 暴露 Zig 能力 |
| React/SolidJS Reconcilers | 声明式 UI | 组件化 UI 绑定（JSX/VDOM） |

`opentui_rust` 是 Zig Core 的 **1:1 端口**（78/78 功能，100% 对等），仅提供底层渲染引擎。

`opentui-core` 是 **全新设计的一层**，填补 `opentui_rust`（引擎）和应用程序之间的空白：

```
原版 OpenTUI                           Rust 版
─────────────                          ─────────
React/SolidJS reconcilers  ←→  （不需要，Rust 无 JSX）
TypeScript API             ←→  opentui-core（本 crate）
Zig Core                   ←→  opentui_rust（引擎端口）
```

**关键决策：** TypeScript 层提供的组件化 UI（Box、Text、ScrollView、List、Focus 管理）在 Rust 中不需要 JSX/VDOM，改为 trait-based 显式事件分发模型。

## 2. 功能对照表

### 2.1 原版 Zig Core 提供 vs opentui_rust vs opentui-core

| 功能 | Zig Core | opentui_rust | opentui-core |
|------|----------|-------------|-------------|
| RGBA 颜色 + Porter-Duff 混合 | ✅ | ✅ | 使用引擎的 |
| Cell（char/grapheme/empty/continuation） | ✅ | ✅ | 使用引擎的 |
| Style（TextAttributes bitflags） | ✅ | ✅ | 使用引擎的 |
| ANSI 转义序列生成 | ✅ | ✅ | 使用引擎的 |
| Grapheme Pool（24-bit ID） | ✅ | ✅ | 使用引擎的 |
| OptimizedBuffer（scissor/opacity/blend） | ✅ | ✅ | 使用引擎的 |
| Rope 数据结构 | ✅ | ✅ | 使用引擎的 |
| TextBuffer（styled segments） | ✅ | ✅ | TextWidget 内部使用 |
| TextBufferView（wrap/scroll/select） | ✅ | ✅ | TextWidget 内部使用 |
| EditBuffer（cursor/undo/redo） | ✅ | ✅ | 未封装（可用引擎的） |
| EditorView（visual cursor） | ✅ | ✅ | 未封装（可用引擎的） |
| Renderer（double-buffer diff） | ✅ | ✅ | 使用引擎的 |
| Terminal capabilities | ✅ | ✅ | 使用引擎的 |
| Input parser（keyboard/mouse） | ✅ | ✅ | EventDispatcher 使用 |
| **Layout Engine（flexbox/grid）** | ❌ | ❌ | ✅ Taffy |
| **Widget trait + WidgetTree** | ❌ | ❌ | ✅ |
| **RenderCommand 两阶段渲染** | ❌ | ❌ | ✅ |
| **BoxWidget（边框容器）** | ❌ | ❌ | ✅ |
| **TextWidget（文本显示）** | ❌ | ❌ | ✅ |
| **ScrollView + ScrollBar** | ❌ | ❌ | ✅ |
| **VirtualList（虚拟列表）** | ❌ | ❌ | ✅ |
| **FocusManager（焦点管理）** | ❌ | ❌ | ✅ |
| **EventDispatcher（事件分发）** | ❌ | ❌ | ✅ |
| **Theme（UI 颜色 token）** | ❌ 仅有 SyntaxStyle | ✅ 有高亮 Theme | ✅ UI Theme（22 token） |
| ThemeRegistry | ❌ | ✅ | ✅ |

### 2.2 原版 TypeScript/React 层提供 vs opentui-core

原版 TypeScript 层通过 React/SolidJS reconciler 提供以下能力：

| TS/React 能力 | opentui-core 状态 | 说明 |
|---------------|-------------------|------|
| `<Box>` 组件 | ✅ BoxWidget | 边框、背景、标题、overflow |
| `<Text>` 组件 | ✅ TextWidget | wrap/truncate/scroll |
| `<ScrollView>` | ✅ ScrollView + ScrollState | viewport culling + scrollbar |
| `<List>` / `<VirtualList>` | ✅ VirtualList | viewport culling + HitGrid |
| Flexbox layout | ✅ LayoutEngine（Taffy） | column/row/grow/shrink/padding/gap |
| Focus 管理 | ✅ FocusManager | register/unregister/next/prev |
| Event dispatch | ✅ EventDispatcher | mouse hit-test + key dispatch |
| Theme tokens | ✅ Theme（22 colors） | dark/light defaults |
| **`<Input>` / `<TextField>`** | ❌ 未实现 | 单行文本输入 |
| **`<Editor>` 组件** | ❌ 未实现 | 多行编辑器（EditBuffer+EditorView） |
| **`<ProgressBar>`** | ❌ 未实现 | 进度条 |
| **`<Tabs>`** | ❌ 未实现 | 标签页切换 |
| **`<StatusLine>`** | ❌ 未实现 | 状态栏 |
| **Key bindings 系统** | ❌ 未实现 | 快捷键映射/冲突检测 |
| **Animation system** | ❌ 未实现 | 帧动画/tween |
| **Modal / Overlay** | ❌ 未实现 | 模态框、浮层 |
| **Z-order / Layer** | ❌ 未实现 | 层叠顺序 |
| **Accessibility** | ❌ 未实现 | 屏幕阅读器支持 |

## 3. 当前实现状态（2,139 LOC）

| 文件 | 行数 | 状态 | 说明 |
|------|------|------|------|
| `lib.rs` | 64 | ✅ 完成 | 模块声明 + crate 级 lint |
| `layout.rs` | 296 | ✅ 完成 | LayoutEngine + LayoutStyle（27 个 builder 方法） |
| `widget.rs` | 392 | ✅ 完成 | Widget trait + WidgetTree + 两阶段渲染 |
| `render_command.rs` | 57 | ✅ 完成 | RenderCommand enum + RenderCommandList |
| `scroll.rs` | 212 | ✅ 完成 | ScrollState + ScrollBarRenderer + ScrollView |
| `list.rs` | 242 | ✅ 完成 | VirtualList + ItemRenderer trait + VirtualListState |
| `event.rs` | 166 | ✅ 完成 | FocusManager + EventDispatcher |
| `theme.rs` | 151 | ✅ 完成 | Theme（22 colors）+ ThemeRegistry |
| `widgets/mod.rs` | 9 | ✅ 完成 | re-exports |
| `widgets/box_widget.rs` | 362 | ✅ 完成 | BoxWidget + BorderStyle/Chars/Sides |
| `widgets/text_widget.rs` | 188 | ✅ 完成 | TextWidget + TextAlign |

## 4. 设计差异分析

### 4.1 原版：React/SolidJS Reconciler
```tsx
// 声明式 JSX，框架管理 VDOM diff
<Box border title="Files" flexDirection="column">
  <Text>Hello</Text>
  <VirtualList items={files} renderItem={FileItem} />
</Box>
```

### 4.2 opentui-core：Trait + 显式树
```rust
// 命令式，app 管理事件循环
let mut tree = WidgetTree::new();
let root = tree.add(BoxWidget::new(1, LayoutStyle::column().width(80.0).height(24.0))
    .border_rounded(theme.border)
    .title("Files"));
let text = tree.add_child(root, TextWidget::with_text(2, LayoutStyle::default(), "Hello"));
tree.layout(80.0, 24.0);
tree.render(&mut ctx);
```

### 4.3 为什么不用 JSX/VDOM

1. **Rust 没有 JSX** — 宏方案（如 `rsx!`）引入 DSL 复杂度
2. **不需要 VDOM diff** — 终端 UI 节点数量极少（< 100），diff 开销不值
3. **显式控制更 Rust** — trait object + 手动构建树更符合 Rust 的零成本抽象哲学
4. **事件模型不同** — React 冒泡机制 vs Rust 的显式 dispatch（返回结构化结果，app 端路由）

## 5. 已知问题和改进空间

### 5.1 当前问题

| 问题 | 严重度 | 说明 |
|------|--------|------|
| ScrollView 未集成为 Widget | 中 | 当前是静态函数，非 Widget trait 实现 |
| VirtualList 未集成为 Widget | 中 | 同上，需要包装为 ListWidget |
| BoxWidget 不支持 children 管理 | 中 | `children` 字段存在但 WidgetTree 管理 children，BoxWidget 未暴露 add_child |
| Theme 与 opentui_rust::highlight::Theme 冲突 | 低 | 两个 Theme 类型用途不同但名称相同 |
| EventDispatcher 分发结果未被 WidgetTree 使用 | 中 | EventDispatcher 和 WidgetTree 是独立系统 |
| ScrollState 使用 f64 而非 f32 | 低 | 与 opentui_rust 的 f32 RGBA 不一致 |
| 无 examples | 高 | 没有使用示例 |
| 无测试 | 高 | 0 个测试 |

### 5.2 缺失的集成

当前 `scroll.rs` 和 `list.rs` 是独立模块，未与 Widget trait 集成。需要：

1. `ScrollViewWidget` — 实现 Widget trait，内含 ScrollState
2. `ListWidget` — 实现 Widget trait，内含 VirtualListState
3. `InputWidget` — 单行输入，包装 EditBuffer
4. `EditorWidget` — 多行编辑器，包装 EditorView

### 5.3 WidgetTree 的 children 管理

当前 WidgetTree 通过 `add_child()` 管理父子关系。BoxWidget 的 `children: Vec<WidgetId>` 字段未被使用——实际 children 由 WidgetTree 管理。需要决定：

- 方案 A：WidgetTree 全权管理（当前方案）
- 方案 B：每个 Widget 持有自己的 children（Widget.children() 返回真实列表）

当前方案 A 的问题是 `Widget::children()` 的默认实现返回空 slice，但 WidgetTree 在 `add_child()` 时更新 `WidgetNode.children`。BoxWidget 的 `children` 字段是多余的。
