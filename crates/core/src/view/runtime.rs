use std::collections::HashMap;

use opentui_rust::renderer::HitGrid;

use crate::view::node::Node;
use crate::view::rebuild::build_tree_with_actions;
use crate::widget::{KeyDispatchResult, MouseDispatchResult, RenderContext, WidgetId, WidgetTree};

pub struct ViewRuntime {
    tree: WidgetTree,
    actions: HashMap<WidgetId, String>,
    hit_grid: HitGrid,
}

pub struct ViewMouseDispatchResult {
    pub inner: MouseDispatchResult,
    pub action: Option<String>,
}

impl ViewRuntime {
    pub fn new() -> Self {
        Self {
            tree: WidgetTree::new(),
            actions: HashMap::new(),
            hit_grid: HitGrid::default(),
        }
    }

    pub fn rebuild(&mut self, node: &Node) {
        let (tree, actions) = build_tree_with_actions(node);
        self.tree = tree;
        self.actions = actions;
        self.tree.build_focus_chain();
    }

    pub fn layout(&mut self, width: f32, height: f32) {
        self.tree.layout(width, height);
    }

    pub fn render(&mut self, ctx: &mut RenderContext<'_>) {
        self.tree.render(ctx);
    }

    pub fn render_to_buffer(
        &mut self,
        ctx: &mut RenderContext<'_>,
        node: &Node,
        width: f32,
        height: f32,
    ) {
        self.rebuild(node);
        self.layout(width, height);
        self.register_hit_areas(width as u32, height as u32);
        self.render(ctx);
    }

    pub fn register_hit_areas(&mut self, width: u32, height: u32) {
        self.hit_grid.resize(width, height);
        let actions = &self.actions;
        for (id, layout, focusable) in self.tree.hit_registration_order() {
            let has_action = actions.contains_key(&id);
            if has_action || focusable {
                let x = layout.x as u32;
                let y = layout.y as u32;
                let w = layout.width as u32;
                let h = layout.height as u32;
                let hit_id = id as u32;
                self.hit_grid.register(x, y, w, h, hit_id);
            }
        }
    }

    pub fn dispatch_key(&mut self, key: &opentui_rust::KeyEvent) -> KeyDispatchResult {
        self.tree.dispatch_key(key)
    }

    pub fn dispatch_mouse(&mut self, mouse: &opentui_rust::MouseEvent) -> ViewMouseDispatchResult {
        let inner = self.tree.dispatch_mouse(mouse, Some(&self.hit_grid));
        let action = inner.target.and_then(|id| self.actions.get(&id).cloned());
        ViewMouseDispatchResult { inner, action }
    }

    pub fn dispatch_mouse_with_grid(
        &mut self,
        mouse: &opentui_rust::MouseEvent,
        hit_grid: Option<&HitGrid>,
    ) -> ViewMouseDispatchResult {
        let inner = self.tree.dispatch_mouse(mouse, hit_grid);
        let action = inner.target.and_then(|id| self.actions.get(&id).cloned());
        ViewMouseDispatchResult { inner, action }
    }

    pub fn action_for_widget(&self, id: WidgetId) -> Option<&str> {
        self.actions.get(&id).map(String::as_str)
    }

    pub fn hit_grid(&self) -> &HitGrid {
        &self.hit_grid
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
