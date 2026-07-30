use crate::layout::LayoutStyle;
use crate::terminal::MouseEventKind;
use crate::view::key::Key;
use crate::view::node::Node;
use crate::view::props::Props;

#[derive(Debug, Clone, Default)]
pub struct MouseActions {
    pub down: Option<String>,
    pub up: Option<String>,
    pub move_: Option<String>,
    pub drag: Option<String>,
    pub drag_end: Option<String>,
    pub drop: Option<String>,
    pub over: Option<String>,
    pub out: Option<String>,
    pub scroll: Option<String>,
}

impl MouseActions {
    pub(crate) fn action_for(&self, kind: MouseEventKind) -> Option<&str> {
        match kind {
            MouseEventKind::Press => self.down.as_deref(),
            MouseEventKind::Release => self.up.as_deref(),
            MouseEventKind::Move => self.move_.as_deref(),
            MouseEventKind::Drag => self.drag.as_deref(),
            MouseEventKind::DragEnd => self.drag_end.as_deref(),
            MouseEventKind::Drop => self.drop.as_deref(),
            MouseEventKind::Over => self.over.as_deref(),
            MouseEventKind::Out => self.out.as_deref(),
            MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => self.scroll.as_deref(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.down.is_none()
            && self.up.is_none()
            && self.move_.is_none()
            && self.drag.is_none()
            && self.drag_end.is_none()
            && self.drop.is_none()
            && self.over.is_none()
            && self.out.is_none()
            && self.scroll.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    pub kind: ElementKind,
    pub key: Option<Key>,
    pub layout: LayoutStyle,
    pub props: Props,
    pub children: Vec<Node>,
    pub action: Option<String>,
    pub mouse_actions: MouseActions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    View,
    Text,
    StyledText,
    Input,
    List,
    Fill,
    Separator,
    Checkbox,
    Spinner,
    Badge,
    Slider,
    Select,
    RadioGroup,
    Gauge,
    ScrollBar,
    Custom(&'static str),
}
