use opentui_rust::renderer::HitGrid;

use crate::view::node::Node;
use crate::view::rebuild::build_tree;
use crate::widget::{KeyDispatchResult, MouseDispatchResult, RenderContext, WidgetTree};

pub struct ViewRuntime {
    tree: WidgetTree,
}

impl ViewRuntime {
    pub fn new() -> Self {
        Self {
            tree: WidgetTree::new(),
        }
    }

    pub fn rebuild(&mut self, node: &Node) {
        self.tree = build_tree(node);
        self.tree.build_focus_chain();
    }

    pub fn layout(&mut self, width: f32, height: f32) {
        self.tree.layout(width, height);
    }

    pub fn render(&mut self, ctx: &mut RenderContext<'_>) {
        self.tree.render(ctx);
    }

    pub fn dispatch_key(&mut self, key: &opentui_rust::KeyEvent) -> KeyDispatchResult {
        self.tree.dispatch_key(key)
    }

    pub fn dispatch_mouse(
        &mut self,
        mouse: &opentui_rust::MouseEvent,
        hit_grid: Option<&HitGrid>,
    ) -> MouseDispatchResult {
        self.tree.dispatch_mouse(mouse, hit_grid)
    }

    pub fn tree(&self) -> &WidgetTree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut WidgetTree {
        &mut self.tree
    }
}

impl Default for ViewRuntime {
    fn default() -> Self {
        Self::new()
    }
}
