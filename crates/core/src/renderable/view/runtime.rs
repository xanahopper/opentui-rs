use std::collections::HashMap;

use crate::renderable::context::RenderContext;
use crate::renderable::event::RenderableMouseDispatch;
use crate::renderable::node::NodeId;
use crate::renderable::tree::RenderTree;
use crate::renderer::HitGrid;
use crate::view::node::Node;
use crate::view::rebuild::{ElementActions, build_tree_with_actions};

pub struct ViewRuntime {
    tree: RenderTree,
    actions: HashMap<NodeId, ElementActions>,
    hit_grid: HitGrid,
    latest_pointer: Option<(u32, u32)>,
}

pub struct ViewMouseDispatchResult {
    pub target: Option<NodeId>,
    pub consumed: bool,
    pub action: Option<String>,
    pub mouse_actions: Vec<String>,
}

impl ViewRuntime {
    pub fn new() -> Self {
        Self {
            tree: RenderTree::new(),
            actions: HashMap::new(),
            hit_grid: HitGrid::default(),
            latest_pointer: None,
        }
    }

    pub fn rebuild(&mut self, node: &Node) {
        let (tree, actions) = build_tree_with_actions(node);
        self.tree = tree;
        self.actions = actions;
    }

    /// Layout pass only (no render output).
    pub fn layout(&mut self, width: f32, height: f32) {
        self.tree.run_layout(width, height);
    }

    /// Render pass only (layout must have been run first).
    pub fn render(&mut self, ctx: &mut RenderContext) {
        self.tree.run_render(ctx, 0.0);
    }

    /// Run the full frame pipeline: lifecycle → layout → collect → render.
    pub fn render_frame(
        &mut self,
        ctx: &mut RenderContext,
        width: f32,
        height: f32,
        delta_time: f64,
    ) {
        self.tree.render_frame(ctx, width, height, delta_time);
    }

    /// Legacy single-shot: rebuild → layout → register hits → render.
    pub fn render_to_buffer(
        &mut self,
        ctx: &mut RenderContext,
        node: &Node,
        width: f32,
        height: f32,
    ) {
        self.rebuild(node);
        self.tree.run_layout(width, height);
        self.register_hit_areas(width as u32, height as u32);
        self.tree.run_render(ctx, 0.0);
    }

    pub fn register_hit_areas(&mut self, width: u32, height: u32) {
        self.hit_grid.resize(width, height);
        let registrations: Vec<(u32, u32, u32, u32, u32)> = self
            .tree
            .iter_ids()
            .filter_map(|id| {
                let node = self.tree.get(id)?;
                if !node.visible {
                    return None;
                }
                let has_action = self.actions.contains_key(&id);
                if has_action || node.focusable || node.behavior.hit_testable() {
                    Some((
                        node.screen_x as u32,
                        node.screen_y as u32,
                        node.width as u32,
                        node.height as u32,
                        node.num,
                    ))
                } else {
                    None
                }
            })
            .collect();

        for (x, y, w, h, num) in registrations {
            self.hit_grid.register(x, y, w, h, num);
        }
        if let Some((x, y)) = self.latest_pointer {
            let target = self
                .hit_grid
                .test(x, y)
                .and_then(|num| self.tree.resolve_num(num));
            self.tree.recheck_hover(target, x, y);
        }
    }

    pub fn dispatch_key(&mut self, key: &crate::KeyEvent) -> bool {
        self.tree.dispatch_key(key)
    }

    pub fn dispatch_mouse(&mut self, mouse: &crate::MouseEvent) -> ViewMouseDispatchResult {
        self.latest_pointer = Some((mouse.x, mouse.y));
        let target_num = self.hit_grid.test(mouse.x, mouse.y);
        let target = target_num.and_then(|num| self.tree.resolve_num(num));

        let dispatch = self.tree.dispatch_mouse_event(target, mouse);
        let consumed = dispatch.consumed;

        let action = target.and_then(|id| self.action_for_target(id));
        let mouse_actions = self.mouse_actions_for_dispatch(&dispatch);

        ViewMouseDispatchResult {
            target,
            consumed,
            action,
            mouse_actions,
        }
    }

    pub fn dispatch_mouse_with_grid(
        &mut self,
        mouse: &crate::MouseEvent,
        hit_grid: Option<&HitGrid>,
    ) -> ViewMouseDispatchResult {
        self.latest_pointer = Some((mouse.x, mouse.y));
        let target_num = hit_grid.and_then(|grid| grid.test(mouse.x, mouse.y));
        let target = target_num.and_then(|num| self.tree.resolve_num(num));

        let dispatch = self.tree.dispatch_mouse_event(target, mouse);
        let consumed = dispatch.consumed;

        let action = target.and_then(|id| self.action_for_target(id));
        let mouse_actions = self.mouse_actions_for_dispatch(&dispatch);

        ViewMouseDispatchResult {
            target,
            consumed,
            action,
            mouse_actions,
        }
    }

    pub fn action_for_node(&self, id: NodeId) -> Option<&str> {
        self.actions
            .get(&id)
            .and_then(|actions| actions.action.as_deref())
    }

    fn action_for_target(&self, target: NodeId) -> Option<String> {
        let mut current = Some(target);
        while let Some(id) = current {
            if let Some(action) = self
                .actions
                .get(&id)
                .and_then(|actions| actions.action.as_ref())
            {
                return Some(action.clone());
            }
            current = self.tree.get(id).and_then(|node| node.parent);
        }
        None
    }

    fn mouse_actions_for_dispatch(&self, dispatch: &RenderableMouseDispatch) -> Vec<String> {
        dispatch
            .delivered
            .iter()
            .filter_map(|delivery| {
                self.actions
                    .get(&delivery.node)?
                    .mouse
                    .action_for(delivery.kind)
                    .map(str::to_owned)
            })
            .collect()
    }

    pub fn hit_grid(&self) -> &HitGrid {
        &self.hit_grid
    }

    pub fn tree(&self) -> &RenderTree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut RenderTree {
        &mut self.tree
    }

    pub fn needs_render(&self) -> bool {
        self.tree.needs_render()
    }

    pub fn is_live(&self) -> bool {
        self.tree.is_live()
    }

    pub fn request_render(&mut self, id: NodeId) {
        self.tree.request_render(id);
    }
}

impl Default for ViewRuntime {
    fn default() -> Self {
        Self::new()
    }
}
