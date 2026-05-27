//! TUI snapshot test for declarative layout.
//!
//! Creates the same widget tree as `opencode_declarative` example and verifies
//! that layout + rendering produces correct output.

#![allow(clippy::float_cmp)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::cell::RefCell;
use std::rc::Rc;

use opentui_rust::{Cell, OptimizedBuffer, Rgba, Style};

use opentui_core::layout::{ComputedLayout, LayoutStyle};
use opentui_core::widget::{Overflow, RenderContext, Widget, WidgetId, WidgetTree};
use opentui_core::widgets::BoxWidget;

const BG: Rgba = Rgba::new(0.059, 0.059, 0.086, 1.0);
const BG_PANEL: Rgba = Rgba::new(0.078, 0.078, 0.118, 1.0);
const BG_ELEMENT: Rgba = Rgba::new(0.098, 0.098, 0.137, 1.0);
const BORDER_ACTIVE: Rgba = Rgba::new(0.294, 0.549, 0.902, 1.0);
const TEXT: Rgba = Rgba::new(0.878, 0.878, 0.922, 1.0);
const TEXT_MUTED: Rgba = Rgba::new(0.498, 0.498, 0.549, 1.0);

struct FakeApp {
    messages: Vec<(&'static str, &'static str)>,
    input_text: String,
}

struct MessageAreaWidget {
    id: WidgetId,
    style: LayoutStyle,
    app: Rc<RefCell<FakeApp>>,
}

struct PromptWidget {
    id: WidgetId,
    style: LayoutStyle,
    app: Rc<RefCell<FakeApp>>,
}

struct HintBarWidget {
    id: WidgetId,
    style: LayoutStyle,
}

struct SidebarWidget {
    id: WidgetId,
    style: LayoutStyle,
}

impl std::fmt::Debug for MessageAreaWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageAreaWidget").finish()
    }
}
impl std::fmt::Debug for PromptWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptWidget").finish()
    }
}
impl std::fmt::Debug for HintBarWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HintBarWidget").finish()
    }
}
impl std::fmt::Debug for SidebarWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SidebarWidget").finish()
    }
}

macro_rules! impl_widget_boilerplate {
    ($t:ty) => {
        impl Widget for $t {
            fn id(&self) -> WidgetId { self.id }
            fn style(&self) -> &LayoutStyle { &self.style }
            fn style_mut(&mut self) -> &mut LayoutStyle { &mut self.style }
            fn render(&self, _ctx: &mut RenderContext<'_>, _layout: &ComputedLayout) {}
            fn visible(&self) -> bool { true }
            fn opacity(&self) -> f32 { 1.0 }
            fn overflow(&self) -> Overflow { Overflow::Hidden }
            fn focusable(&self) -> bool { false }
            fn focused(&self) -> bool { false }
            fn set_focused(&mut self, _focused: bool) {}
            fn handle_key(&mut self, _key: &opentui_rust::KeyEvent) -> bool { false }
            fn handle_mouse(&mut self, _mouse: &opentui_rust::MouseEvent) -> bool { false }
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        }
    };
}

impl_widget_boilerplate!(HintBarWidget);
impl_widget_boilerplate!(SidebarWidget);

impl MessageAreaWidget {
    fn new(id: WidgetId, app: Rc<RefCell<FakeApp>>) -> Self {
        Self { id, style: LayoutStyle::column().flex_grow(1.0), app }
    }
}

impl PromptWidget {
    fn new(id: WidgetId, app: Rc<RefCell<FakeApp>>) -> Self {
        Self { id, style: LayoutStyle::column().height(5.0).flex_shrink(0.0), app }
    }
}

impl HintBarWidget {
    fn new(id: WidgetId) -> Self {
        Self { id, style: LayoutStyle::column().height(1.0).flex_shrink(0.0) }
    }
}

impl SidebarWidget {
    fn new(id: WidgetId) -> Self {
        Self { id, style: LayoutStyle::column().width(42.0).flex_shrink(0.0) }
    }
}

impl Widget for MessageAreaWidget {
    fn id(&self) -> WidgetId { self.id }
    fn style(&self) -> &LayoutStyle { &self.style }
    fn style_mut(&mut self) -> &mut LayoutStyle { &mut self.style }

    fn render(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;
        let bg_style = Style::builder().bg(BG).build();
        for row in 0..h {
            for col in 0..w {
                ctx.buffer.set(x + col, y + row, Cell::new(' ', bg_style));
            }
        }
        let app = self.app.borrow();
        let text_style = Style::builder().fg(TEXT).bg(BG).build();
        let mut row = 0u32;
        for (_, text) in &app.messages {
            for line in text.split('\n') {
                if row < h {
                    ctx.buffer.draw_text(x + 2, y + row, line, text_style);
                    row += 1;
                }
            }
            row += 1;
        }
    }

    fn visible(&self) -> bool { true }
    fn opacity(&self) -> f32 { 1.0 }
    fn overflow(&self) -> Overflow { Overflow::Hidden }
    fn focusable(&self) -> bool { false }
    fn focused(&self) -> bool { false }
    fn set_focused(&mut self, _focused: bool) {}
    fn handle_key(&mut self, _key: &opentui_rust::KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _mouse: &opentui_rust::MouseEvent) -> bool { false }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl Widget for PromptWidget {
    fn id(&self) -> WidgetId { self.id }
    fn style(&self) -> &LayoutStyle { &self.style }
    fn style_mut(&mut self) -> &mut LayoutStyle { &mut self.style }

    fn render(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;
        let el_style = Style::builder().bg(BG_ELEMENT).build();
        for row in 0..h {
            for col in 0..w {
                ctx.buffer.set(x + col, y + row, Cell::new(' ', el_style));
            }
        }
        let app = self.app.borrow();
        if app.input_text.is_empty() {
            let ph = Style::builder().fg(TEXT_MUTED).bg(BG_ELEMENT).build();
            ctx.buffer.draw_text(x + 3, y + 1, "Ask anything...", ph);
        } else {
            let ts = Style::builder().fg(TEXT).bg(BG_ELEMENT).build();
            ctx.buffer.draw_text(x + 3, y + 1, &app.input_text, ts);
        }
    }

    fn visible(&self) -> bool { true }
    fn opacity(&self) -> f32 { 1.0 }
    fn overflow(&self) -> Overflow { Overflow::Hidden }
    fn focusable(&self) -> bool { false }
    fn focused(&self) -> bool { false }
    fn set_focused(&mut self, _focused: bool) {}
    fn handle_key(&mut self, _key: &opentui_rust::KeyEvent) -> bool { false }
    fn handle_mouse(&mut self, _mouse: &opentui_rust::MouseEvent) -> bool { false }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

fn cell_char(buf: &OptimizedBuffer, x: u32, y: u32) -> Option<char> {
    buf.get(x, y).and_then(|c| c.content.as_char())
}

fn build_tree(
    w: f32,
    h: f32,
    app: Rc<RefCell<FakeApp>>,
    sidebar: bool,
) -> (WidgetTree, [WidgetId; 5]) {
    let mut tree = WidgetTree::new();

    let root_id = tree.allocate_id();
    let main_id = tree.allocate_id();
    let msg_id = tree.allocate_id();
    let prompt_id = tree.allocate_id();
    let hint_id = tree.allocate_id();

    tree.add(
        BoxWidget::new(root_id, LayoutStyle::row().width(w).height(h))
            .background(BG)
            .overflow_hidden(),
    );

    tree.add_child(
        root_id,
        BoxWidget::new(main_id, LayoutStyle::column().flex_grow(1.0))
            .background(BG)
            .overflow_hidden(),
    );

    tree.add_child(main_id, MessageAreaWidget::new(msg_id, app.clone()));
    tree.add_child(main_id, PromptWidget::new(prompt_id, app.clone()));
    tree.add_child(main_id, HintBarWidget::new(hint_id));

    if sidebar {
        let sb_id = tree.allocate_id();
        tree.add_child(root_id, SidebarWidget::new(sb_id));
    }

    tree.layout(w, h);

    (tree, [root_id, main_id, msg_id, prompt_id, hint_id])
}

fn dump_layouts(tree: &WidgetTree, ids: &[WidgetId], names: &[&str]) -> String {
    let mut out = String::new();
    for (id, name) in ids.iter().zip(names) {
        if let Some(l) = tree.computed_layout(*id) {
            out.push_str(&format!(
                "{}: x={} y={} w={} h={}\n",
                name, l.x, l.y, l.width, l.height
            ));
        } else {
            out.push_str(&format!("{name}: NO LAYOUT\n"));
        }
    }
    out
}

fn render_to_string(buf: &OptimizedBuffer, w: u32, h: u32) -> String {
    let mut out = String::new();
    for row in 0..h {
        let mut line = String::new();
        for col in 0..w {
            let ch = cell_char(buf, col, row).unwrap_or(' ');
            line.push(ch);
        }
        out.push_str(&format!("{row:3}: |{line}|\n"));
    }
    out
}

#[test]
fn test_layout_no_sidebar() {
    let app = Rc::new(RefCell::new(FakeApp {
        messages: vec![("user", "Hello"), ("assistant", "World")],
        input_text: String::new(),
    }));

    let (tree, ids) = build_tree(100.0, 30.0, app, false);

    let names = ["root", "main", "messages", "prompt", "hint"];
    let dump = dump_layouts(&tree, &ids, &names);

    let (root_id, main_id, msg_id, prompt_id, hint_id) = (ids[0], ids[1], ids[2], ids[3], ids[4]);

    let root_l = tree.computed_layout(root_id).unwrap();
    assert_eq!(root_l.x, 0.0);
    assert_eq!(root_l.y, 0.0);
    assert_eq!(root_l.width, 100.0);
    assert_eq!(root_l.height, 30.0);

    let main_l = tree.computed_layout(main_id).unwrap();
    assert_eq!(main_l.x, 0.0);
    assert_eq!(main_l.y, 0.0);
    assert_eq!(main_l.width, 100.0);
    assert_eq!(main_l.height, 30.0);

    let msg_l = tree.computed_layout(msg_id).unwrap();
    let prompt_l = tree.computed_layout(prompt_id).unwrap();
    let hint_l = tree.computed_layout(hint_id).unwrap();

    // Messages should fill remaining space
    assert_eq!(msg_l.x, 0.0);
    assert_eq!(msg_l.y, 0.0);
    assert_eq!(msg_l.width, 100.0);
    // messages height = 30 - 5 (prompt) - 1 (hint) = 24
    assert_eq!(msg_l.height, 24.0, "messages height should be 24 (30-5-1), got {}\n{dump}", msg_l.height);

    // Prompt at row 24, height 5
    assert_eq!(prompt_l.x, 0.0);
    assert_eq!(prompt_l.y, 24.0);
    assert_eq!(prompt_l.width, 100.0);
    assert_eq!(prompt_l.height, 5.0);

    // Hint bar at row 29, height 1
    assert_eq!(hint_l.x, 0.0);
    assert_eq!(hint_l.y, 29.0);
    assert_eq!(hint_l.width, 100.0);
    assert_eq!(hint_l.height, 1.0);
}

#[test]
fn test_layout_with_sidebar() {
    let app = Rc::new(RefCell::new(FakeApp {
        messages: vec![],
        input_text: String::new(),
    }));

    let (tree, ids) = build_tree(140.0, 30.0, app, true);

    let root_l = tree.computed_layout(ids[0]).unwrap();
    let main_l = tree.computed_layout(ids[1]).unwrap();

    assert_eq!(root_l.width, 140.0);
    assert_eq!(main_l.width, 98.0, "main width should be 98 (140-42), got {}", main_l.width);
}

#[test]
fn test_render_snapshot_no_sidebar() {
    let app = Rc::new(RefCell::new(FakeApp {
        messages: vec![("user", "Hello")],
        input_text: String::new(),
    }));

    let w = 40u32;
    let h = 12u32;

    let (mut tree, ids) = build_tree(w as f32, h as f32, app, false);

    let dump = dump_layouts(&tree, &ids, &["root", "main", "messages", "prompt", "hint"]);

    let mut buf = OptimizedBuffer::new(w, h);
    buf.clear(Rgba::TRANSPARENT);

    {
        let mut ctx = RenderContext {
            buffer: &mut buf,
            grapheme_pool: None,
            link_pool: None,
            hit_grid: None,
            theme: None,
        };
        tree.render(&mut ctx);
    }

    let snapshot = render_to_string(&buf, w, h);

    // Messages area: first 6 rows (h=12, prompt=5, hint=1, messages=6)
    let msg_l = tree.computed_layout(ids[2]).unwrap();
    assert_eq!(msg_l.height, 6.0, "messages should be 6 rows tall\n{dump}\n{snapshot}");

    // "Hello" should appear in messages area at row 0, col 2
    assert_eq!(cell_char(&buf, 2, 0), Some('H'), "H at (2,0)\n{snapshot}");
    assert_eq!(cell_char(&buf, 3, 0), Some('e'), "e at (3,0)\n{snapshot}");

    // Prompt area starts at row 6
    let prompt_l = tree.computed_layout(ids[3]).unwrap();
    assert_eq!(prompt_l.y, 6.0, "prompt should start at y=6\n{dump}");

    // Placeholder text in prompt at row 7 (y=6 + 1 padding row)
    assert_eq!(cell_char(&buf, 3, 7), Some('A'), "A (from 'Ask anything...') at (3,7)\n{snapshot}");

    // Hint bar at row 11
    let hint_l = tree.computed_layout(ids[4]).unwrap();
    assert_eq!(hint_l.y, 11.0, "hint should start at y=11\n{dump}");
}

#[test]
fn test_render_snapshot_with_sidebar() {
    let app = Rc::new(RefCell::new(FakeApp {
        messages: vec![("user", "Hi")],
        input_text: String::new(),
    }));

    let w = 80u32;
    let h = 12u32;

    let (mut tree, ids) = build_tree(w as f32, h as f32, app, true);

    let dump = dump_layouts(&tree, &ids, &["root", "main", "messages", "prompt", "hint"]);

    let main_l = tree.computed_layout(ids[1]).unwrap();
    assert_eq!(main_l.width, 38.0, "main width = 80-42 = 38\n{dump}");

    let mut buf = OptimizedBuffer::new(w, h);
    buf.clear(Rgba::TRANSPARENT);

    {
        let mut ctx = RenderContext {
            buffer: &mut buf,
            grapheme_pool: None,
            link_pool: None,
            hit_grid: None,
            theme: None,
        };
        tree.render(&mut ctx);
    }

    let snapshot = render_to_string(&buf, w, h);

    // "Hi" in messages area at (2, 0)
    assert_eq!(cell_char(&buf, 2, 0), Some('H'), "H at (2,0)\n{snapshot}");
    assert_eq!(cell_char(&buf, 3, 0), Some('i'), "i at (3,0)\n{snapshot}");

    // Sidebar starts at col 38
    // SidebarWidget renders but we gave it empty render, so check bg
    // Actually SidebarWidget uses impl_widget_boilerplate! which has empty render
    // The BoxWidget wrapping main has bg=BG, sidebar's BoxWidget is not in this tree
    // sidebar is a direct child of root but not wrapped in BoxWidget
}
