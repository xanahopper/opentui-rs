#[cfg(test)]
mod tests {
    use opentui_core::layout::{ComputedLayout, LayoutStyle};
    use opentui_core::render_command::RenderCommand;
    use opentui_core::widget::{RenderContext, Widget, WidgetId, WidgetTree};

    struct StubWidget {
        id: WidgetId,
        style: LayoutStyle,
        visible: bool,
        focusable: bool,
        focused: bool,
    }

    impl StubWidget {
        fn new(id: WidgetId) -> Self {
            Self {
                id,
                style: LayoutStyle::default(),
                visible: true,
                focusable: false,
                focused: false,
            }
        }

        fn with_focusable(mut self) -> Self {
            self.focusable = true;
            self
        }

        fn with_invisible(mut self) -> Self {
            self.visible = false;
            self
        }
    }

    impl Widget for StubWidget {
        fn id(&self) -> WidgetId {
            self.id
        }
        fn style(&self) -> &LayoutStyle {
            &self.style
        }
        fn style_mut(&mut self) -> &mut LayoutStyle {
            &mut self.style
        }
        fn render(&self, _ctx: &mut RenderContext<'_>, _layout: &ComputedLayout) {}
        fn visible(&self) -> bool {
            self.visible
        }
        fn focusable(&self) -> bool {
            self.focusable
        }
        fn focused(&self) -> bool {
            self.focused
        }
        fn set_focused(&mut self, focused: bool) {
            self.focused = focused;
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_add_root() {
        let mut tree = WidgetTree::new();
        let id = tree.add(StubWidget::new(1));
        assert_eq!(id, 1);
        assert_eq!(tree.root(), Some(1));
    }

    #[test]
    fn test_add_child() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let child = tree.add_child(root, StubWidget::new(2));
        assert_eq!(child, 2);
        assert_eq!(tree.parent(child), Some(root));
    }

    #[test]
    fn test_remove() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let child = tree.add_child(root, StubWidget::new(2));
        tree.remove(child);
        assert!(tree.get(child).is_none());
    }

    #[test]
    fn test_layout() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let _child = tree.add_child(root, StubWidget::new(2));

        // Sync taffy style with explicit dimensions
        tree.set_widget_style(root, LayoutStyle::default().width(80.0).height(24.0));
        tree.layout(80.0, 24.0);

        let root_layout = tree.computed_layout(root).unwrap();
        assert_eq!(root_layout.width, 80.0);
        assert_eq!(root_layout.height, 24.0);
    }

    #[test]
    fn test_focus_chain() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let a = tree.add_child(root, StubWidget::new(2).with_focusable());
        let b = tree.add_child(root, StubWidget::new(3).with_focusable());
        let c = tree.add_child(root, StubWidget::new(4).with_focusable());

        tree.build_focus_chain();

        // Focus next cycles through
        let next = tree.focus_next();
        assert_eq!(next, Some(a));
        assert_eq!(tree.focused_id(), Some(a));

        let next = tree.focus_next();
        assert_eq!(next, Some(b));
        assert_eq!(tree.focused_id(), Some(b));

        let next = tree.focus_next();
        assert_eq!(next, Some(c));

        // Wraps around
        let next = tree.focus_next();
        assert_eq!(next, Some(a));
    }

    #[test]
    fn test_focus_prev() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let a = tree.add_child(root, StubWidget::new(2).with_focusable());
        let _b = tree.add_child(root, StubWidget::new(3).with_focusable());

        tree.build_focus_chain();

        let prev = tree.focus_prev();
        assert_eq!(prev, Some(_b));

        let prev = tree.focus_prev();
        assert_eq!(prev, Some(a));
    }

    #[test]
    fn test_set_focused() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let a = tree.add_child(root, StubWidget::new(2).with_focusable());
        let b = tree.add_child(root, StubWidget::new(3).with_focusable());

        tree.build_focus_chain();

        tree.set_focused_widget(Some(a));
        assert_eq!(tree.focused_id(), Some(a));

        tree.set_focused_widget(Some(b));
        assert_eq!(tree.focused_id(), Some(b));
        // a should no longer be focused
        let widget = tree.get_mut(a).unwrap();
        assert!(!widget.focused());
    }

    #[test]
    fn test_invisible_widget_skipped() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let _hidden = tree.add_child(root, StubWidget::new(2).with_invisible());

        tree.layout(80.0, 24.0);
        tree.build_render_commands();
        let cmds = tree.render_commands();
        let render_ids: Vec<WidgetId> = cmds
            .iter()
            .filter_map(|c| match c {
                RenderCommand::Render { id } => Some(*id),
                _ => None,
            })
            .collect();
        assert!(render_ids.contains(&1));
        assert!(!render_ids.contains(&2));
    }

    #[test]
    fn test_allocate_id() {
        let mut tree = WidgetTree::new();
        let id1 = tree.allocate_id();
        let id2 = tree.allocate_id();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_parent() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let child = tree.add_child(root, StubWidget::new(2));
        assert_eq!(tree.parent(root), None);
        assert_eq!(tree.parent(child), Some(root));
    }

    #[test]
    fn test_has_focused_descendant() {
        let mut tree = WidgetTree::new();
        let root = tree.add(StubWidget::new(1));
        let child = tree.add_child(root, StubWidget::new(2).with_focusable());

        tree.set_focused_widget(Some(child));
        assert!(tree.has_focused_descendant(root));
    }
}
