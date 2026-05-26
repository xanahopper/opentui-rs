# opentui-core API 速查

## 模块总览

```
opentui_core
├── layout         LayoutEngine + LayoutStyle + ComputedLayout
├── widget         Widget trait + WidgetTree + RenderContext + WidgetId
├── render_command RenderCommand + RenderCommandList（内部用）
├── widgets
│   ├── box_widget    BoxWidget + BorderStyle + BorderChars + BorderSides
│   └── text_widget   TextWidget + TextAlign
├── scroll         ScrollState + ScrollBarRenderer + ScrollView + ScrollBarStyle
├── list           VirtualList + VirtualListState + ItemRenderer + FixedHeightItemRenderer
├── event          FocusManager + EventDispatcher + FocusId + DispatchResult
└── theme          Theme（22 colors）+ ThemeRegistry
```

## 核心 API

### LayoutStyle（Builder 模式）

```rust
LayoutStyle::column()                    // flex-direction: column
    .row()                               // flex-direction: row
    .width(80.0)                         // 固定宽度
    .height(24.0)                        // 固定高度
    .width_percent(100.0)                // 百分比宽度
    .height_percent(50.0)                // 百分比高度
    .flex_grow(1.0)                      // flex-grow
    .flex_shrink(0.0)                    // flex-shrink
    .flex_basis(40.0)                    // flex-basis
    .flex_wrap(taffy::style::FlexWrap::Wrap)  // 换行
    .padding(1, 2, 1, 2)                // top right bottom left
    .padding_x(2)                        // horizontal padding
    .padding_y(1)                        // vertical padding
    .margin(0, 1, 0, 1)                  // top right bottom left
    .gap(1.0)                            // gap
    .align_items(taffy::style::AlignItems::Center)
    .justify_content(taffy::style::JustifyContent::SpaceBetween)
    .align_self(taffy::style::AlignSelf::Stretch)
    .overflow(taffy::style::Overflow::Hidden)
    .position_absolute()                 // position: absolute
    .top(10.0).left(5.0)                 // inset
    .min_width(20.0).max_width(100.0)    // 尺寸约束
    .min_height(5.0).max_height(50.0)
    .auto_width().auto_height()          // auto sizing
```

### Widget trait

```rust
trait Widget {
    fn id(&self) -> WidgetId;
    fn style(&self) -> &LayoutStyle;
    fn style_mut(&mut self) -> &mut LayoutStyle;
    fn render(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout);
    fn children(&self) -> &[WidgetId] { &[] }
    fn visible(&self) -> bool { true }
    fn opacity(&self) -> f32 { 1.0 }
    fn overflow(&self) -> Overflow { Overflow::Visible }
    fn focusable(&self) -> bool { false }
    fn focused(&self) -> bool { false }
    fn set_focused(&mut self, focused: bool) {}
    fn handle_key(&mut self, key: &KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, mouse: &MouseEvent) -> bool { false }
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

### WidgetTree

```rust
let mut tree = WidgetTree::new();

// 添加 widget
let root_id = tree.add(BoxWidget::new(1, LayoutStyle::column().width(80.0).height(24.0)));
let child_id = tree.add_child(root_id, TextWidget::with_text(2, LayoutStyle::default(), "Hello"));

// 运行布局
tree.layout(80.0, 24.0);

// 渲染
tree.render(&mut ctx);

// 访问
tree.get(id)            -> Option<&dyn Widget>
tree.get_mut(id)        -> Option<&mut dyn Widget>
tree.computed_layout(id) -> Option<&ComputedLayout>

// 焦点
tree.set_focused(id, true)
tree.has_focused_descendant(id) -> bool
tree.focus_next()       // TODO
tree.focus_prev()       // TODO

// ID 管理
tree.allocate_id()      -> WidgetId
tree.root()             -> Option<WidgetId>
tree.parent(id)         -> Option<WidgetId>
```

### BoxWidget

```rust
BoxWidget::new(id, LayoutStyle::column())
    .background(Rgba::new(0.1, 0.1, 0.15, 1.0))
    .border_rounded(Rgba::new(0.3, 0.3, 0.35, 1.0))
    .border_focused_color(Rgba::new(0.5, 0.7, 1.0, 1.0))
    .title("Files")
    .title_align(TitleAlign::Center)
    .bottom_title("3 items")
    .overflow_hidden()
    .focusable()
    .set_opacity(0.9)
```

### TextWidget

```rust
TextWidget::with_text(id, LayoutStyle::default(), "Hello, world!")
    .wrap(WrapMode::Word)
    .default_style(Style::builder().fg(Rgba::WHITE).build())
    .overflow_visible()
    .focusable()

// 动态修改
widget.set_text("New content");
widget.set_scroll(0, 5);
widget.buffer_mut().insert_char('x');
```

### ScrollState + ScrollView

```rust
let mut state = ScrollState::new();
state.set_content_height(100.0);
state.set_viewport_height(24);
state.scroll_down(3.0);
state.handle_mouse(&mouse_event);
state.handle_key(&key_event);

ScrollView::render(
    buffer, x, y, w, h,
    &mut state,
    true,                           // show scrollbar
    Some(&scrollbar_style),         // optional style
    |buf, cx, cy, cw, ch| {        // render callback
        // draw content at (cx, cy)
    },
);
```

### VirtualList

```rust
let renderer = FixedHeightItemRenderer {
    count: 1000,
    height: 1,
    render_fn: |buf, idx, x, y, w, selected| {
        let style = if selected { selected_style } else { normal_style };
        buf.draw_text(x, y, &format!("Item {idx}"), style);
    },
};

let mut state = VirtualListState::new();
state.select(Some(0), 1000, 1);
state.handle_key(&key, 1000, 1);

VirtualList::render(
    buffer, Some(&mut hit_grid),
    x, y, w, h,
    &mut state,
    &renderer,
    true,                           // show scrollbar
    Some(&scrollbar_style),
);
```

### Theme

```rust
let theme = Theme::dark_default();
// 22 color tokens: primary, secondary, accent,
// background/panel/element/menu,
// text/muted/selected/inverse,
// border/subtle/active,
// success/warning/error/info,
// scrollbar_track/thumb,
// selection_bg/fg

let mut registry = ThemeRegistry::new();  // dark + light defaults
registry.register(custom_theme);
registry.set_active("light-default");
let active = registry.active();
```

## 典型使用流程

```rust
use opentui_core::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建终端
    let _guard = opentui_rust::enable_raw_mode()?;
    let mut terminal = opentui_rust::Terminal::new()?;
    let (w, h) = opentui_rust::terminal_size();
    let mut renderer = opentui_rust::Renderer::new(w, h)?;
    let mut input_parser = opentui_rust::InputParser::new();

    // 2. 构建 widget tree
    let mut tree = WidgetTree::new();
    let root = tree.add(
        BoxWidget::new(1, LayoutStyle::column().width(w as f32).height(h as f32))
            .border_rounded(theme.border)
            .title("My App")
            .background(theme.background)
    );
    let content = tree.add_child(root,
        TextWidget::with_text(2, LayoutStyle::default().flex_grow(1.0), "Hello!")
    );

    // 3. 主循环
    loop {
        tree.layout(w as f32, h as f32);
        let buffer = renderer.buffer();
        let mut ctx = RenderContext {
            buffer,
            grapheme_pool: None,
            link_pool: None,
            hit_grid: None,
            theme: Some(&theme),
        };
        tree.render(&mut ctx);
        renderer.present()?;

        // 4. 输入处理
        // input_parser.advance(bytes, |event| { ... });
    }
}
```
