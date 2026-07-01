//! Render command list for tree-based rendering.
//!
//! During the render phase, the `WidgetTree` traverses the widget tree and
//! builds a flat list of `RenderCommand`s. This list is then executed
//! against the `OptimizedBuffer` in order. This two-phase approach ensures
//! correct scissor and opacity nesting.

use crate::widget::WidgetId;

#[derive(Debug, Clone)]
pub enum RenderCommand {
    Render {
        id: WidgetId,
    },
    PushScissor {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    PopScissor,
    PushOpacity {
        opacity: f32,
    },
    PopOpacity,
}

#[derive(Debug, Default)]
pub struct RenderCommandList {
    commands: Vec<RenderCommand>,
}

impl RenderCommandList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    pub fn commands_mut(&mut self) -> &mut Vec<RenderCommand> {
        &mut self.commands
    }

    pub fn into_commands(self) -> Vec<RenderCommand> {
        self.commands
    }
}
