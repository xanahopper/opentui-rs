use crate::Rgba;
use crate::buffer::TitleAlign;

use crate::layout::LayoutStyle;
use crate::renderable::node::Overflow;
use crate::view::element::{Element, ElementKind};
use crate::view::key::Key;
use crate::view::node::{Node, OverlayNode};
use crate::view::props::{
    BadgeProps, CheckboxProps, FillProps, GaugeProps, InputProps, ListProps, Props,
    RadioGroupProps, ScrollBarProps, SelectProps, SeparatorProps, SliderProps, SpinnerProps,
    StyledTextProps, TextProps, ViewProps,
};
use crate::widgets::{BorderChars, BorderSides, BorderStyle, StyledSegment};

pub fn view() -> ElementBuilder {
    ElementBuilder::new(ElementKind::View)
}

pub fn panel() -> ElementBuilder {
    ElementBuilder::new(ElementKind::View).border_rounded(Rgba::new(0.3, 0.3, 0.35, 1.0))
}

pub fn text(content: impl Into<String>) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Text);
    builder.text_content = Some(content.into());
    builder
}

pub fn rich_text(segments: Vec<StyledSegment>) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::StyledText);
    builder.props = Props::StyledText(StyledTextProps { segments });
    builder
}

pub fn span(text: impl Into<String>, fg: Rgba) -> StyledSegment {
    StyledSegment::new(text, fg)
}

pub fn input() -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Input);
    builder.props = Props::Input(InputProps::default());
    builder
}

/// **Experimental:** Creates a list element wrapping `ListWidget`.
///
/// Item rendering is not yet wired — `ListWidget::render()` is a no-op
/// without `render_with_renderer()`. Use `view().children(items.map(...))`
/// for static lists until `virtual_list()` is implemented.
#[doc(hidden)]
pub fn list(item_count: usize) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::List);
    builder.props = Props::List(ListProps {
        item_count,
        scrollbar: true,
    });
    builder
}

pub fn fill(color: Rgba) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Fill);
    builder.props = Props::Fill(FillProps { color });
    builder
}

pub fn separator() -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Separator);
    builder.props = Props::Separator(SeparatorProps::default());
    builder
}

pub fn checkbox(label: impl Into<String>) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Checkbox);
    builder.props = Props::Checkbox(CheckboxProps {
        checked: false,
        label: Some(label.into()),
    });
    builder
}

pub fn spinner() -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Spinner);
    builder.props = Props::Spinner(SpinnerProps::default());
    builder
}

pub fn badge(text: impl Into<String>) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Badge);
    builder.props = Props::Badge(BadgeProps {
        text: text.into(),
        ..Default::default()
    });
    builder
}

pub fn slider() -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Slider);
    builder.props = Props::Slider(SliderProps::default());
    builder
}

pub fn select(items: Vec<String>) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Select);
    builder.props = Props::Select(SelectProps {
        items,
        ..Default::default()
    });
    builder
}

pub fn radio_group(options: Vec<String>) -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::RadioGroup);
    builder.props = Props::RadioGroup(RadioGroupProps {
        options,
        ..Default::default()
    });
    builder
}

pub fn gauge() -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::Gauge);
    builder.props = Props::Gauge(GaugeProps::default());
    builder
}

pub fn scrollbar() -> ElementBuilder {
    let mut builder = ElementBuilder::new(ElementKind::ScrollBar);
    builder.props = Props::ScrollBar(ScrollBarProps::default());
    builder
}

pub fn fragment(children: Vec<Node>) -> Node {
    Node::Fragment(children)
}

pub fn when(condition: bool, f: impl FnOnce() -> Node) -> Node {
    if condition { f() } else { Node::Empty }
}

pub fn empty() -> Node {
    Node::Empty
}

pub fn overlay(content: Node) -> OverlayBuilder {
    OverlayBuilder {
        content: Box::new(content),
        x: 0,
        y: 0,
        width: 0,
        height: 0,
        backdrop: false,
        z_order: 0,
    }
}

#[derive(Debug, Clone)]
pub struct OverlayBuilder {
    content: Box<Node>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    backdrop: bool,
    z_order: u16,
}

impl OverlayBuilder {
    pub fn position(mut self, x: u32, y: u32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn backdrop(mut self) -> Self {
        self.backdrop = true;
        self
    }

    pub fn z_order(mut self, z: u16) -> Self {
        self.z_order = z;
        self
    }

    pub fn build(self) -> Node {
        Node::Overlay(Box::new(OverlayNode {
            content: self.content,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            backdrop: self.backdrop,
            z_order: self.z_order,
        }))
    }
}

impl From<OverlayBuilder> for Node {
    fn from(builder: OverlayBuilder) -> Self {
        builder.build()
    }
}

#[derive(Debug, Clone)]
pub struct ElementBuilder {
    kind: ElementKind,
    key: Option<Key>,
    layout: LayoutStyle,
    props: Props,
    children: Vec<Node>,
    text_content: Option<String>,
    action: Option<String>,
}

impl ElementBuilder {
    pub fn new(kind: ElementKind) -> Self {
        let props = match kind {
            ElementKind::View => Props::View(ViewProps::default()),
            ElementKind::Text => Props::Text(TextProps::default()),
            ElementKind::StyledText => Props::StyledText(StyledTextProps::default()),
            ElementKind::Input => Props::Input(InputProps::default()),
            ElementKind::List => Props::List(ListProps::default()),
            ElementKind::Fill => Props::Fill(FillProps::default()),
            ElementKind::Separator => Props::Separator(SeparatorProps::default()),
            ElementKind::Checkbox => Props::Checkbox(CheckboxProps::default()),
            ElementKind::Spinner => Props::Spinner(SpinnerProps::default()),
            ElementKind::Badge => Props::Badge(BadgeProps::default()),
            ElementKind::Slider => Props::Slider(SliderProps::default()),
            ElementKind::Select => Props::Select(SelectProps::default()),
            ElementKind::RadioGroup => Props::RadioGroup(RadioGroupProps::default()),
            ElementKind::Gauge => Props::Gauge(GaugeProps::default()),
            ElementKind::ScrollBar => Props::ScrollBar(ScrollBarProps::default()),
            ElementKind::Custom(_) => Props::Empty,
        };
        Self {
            kind,
            key: None,
            layout: LayoutStyle::default(),
            props,
            children: Vec::new(),
            text_content: None,
            action: None,
        }
    }

    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn row(mut self) -> Self {
        self.layout = LayoutStyle::row();
        self
    }

    pub fn column(mut self) -> Self {
        self.layout = LayoutStyle::column();
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.layout = self.layout.width(w);
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.layout = self.layout.height(h);
        self
    }

    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.layout = self.layout.width(w).height(h);
        self
    }

    pub fn width_pct(mut self, pct: f32) -> Self {
        self.layout = self.layout.width_percent(pct);
        self
    }

    pub fn height_pct(mut self, pct: f32) -> Self {
        self.layout = self.layout.height_percent(pct);
        self
    }

    pub fn size_pct(mut self, wp: f32, hp: f32) -> Self {
        self.layout = self.layout.width_percent(wp).height_percent(hp);
        self
    }

    pub fn grow(mut self, val: f32) -> Self {
        self.layout = self.layout.flex_grow(val);
        self
    }

    pub fn shrink(mut self, val: f32) -> Self {
        self.layout = self.layout.flex_shrink(val);
        self
    }

    pub fn padding_all(mut self, v: f32) -> Self {
        self.layout = self.layout.padding_all(v);
        self
    }

    pub fn padding_x(mut self, v: f32) -> Self {
        self.layout = self.layout.padding_x(v);
        self
    }

    pub fn padding_y(mut self, v: f32) -> Self {
        self.layout = self.layout.padding_y(v);
        self
    }

    pub fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.layout = self.layout.padding(top, right, bottom, left);
        self
    }

    pub fn margin(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.layout = self.layout.margin(top, right, bottom, left);
        self
    }

    pub fn gap(mut self, v: f32) -> Self {
        self.layout = self.layout.gap(v);
        self
    }

    pub fn overflow_hidden(mut self) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.overflow = Overflow::Hidden;
        }
        self.layout = self.layout.overflow(taffy::style::Overflow::Hidden);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.opacity = opacity.clamp(0.0, 1.0);
        }
        self
    }

    pub fn focusable(mut self) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.focusable = true;
        }
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.visible = visible;
        }
        self
    }

    pub fn bg(mut self, color: Rgba) -> Self {
        match &mut self.props {
            Props::View(vp) => vp.bg = Some(color),
            Props::Text(tp) => tp.bg = Some(color),
            Props::Badge(bp) => bp.bg = color,
            _ => {}
        }
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        match &mut self.props {
            Props::Checkbox(cp) => cp.label = Some(label),
            Props::Spinner(sp) => sp.label = Some(label),
            _ => {}
        }
        self
    }

    pub fn fg(mut self, color: Rgba) -> Self {
        match &mut self.props {
            Props::Text(tp) => tp.fg = color,
            Props::Badge(bp) => bp.fg = color,
            _ => {}
        }
        self
    }

    pub fn bold(mut self) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.bold = true;
        }
        self
    }

    pub fn italic(mut self) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.italic = true;
        }
        self
    }

    pub fn underline(mut self) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.underline = true;
        }
        self
    }

    pub fn border_rounded(mut self, color: Rgba) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.border = Some(BorderStyle {
                chars: BorderChars::rounded(),
                color,
                focused_color: None,
                sides: BorderSides::all(),
            });
        }
        self
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.border = Some(border);
        }
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.title = Some(title.into());
        }
        self
    }

    pub fn title_align(mut self, align: TitleAlign) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.title_align = align;
        }
        self
    }

    pub fn title_color(mut self, color: Rgba) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.title_color = Some(color);
        }
        self
    }

    pub fn align_left(mut self) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.align = crate::widgets::TextLineAlign::Left;
        }
        self
    }

    pub fn align_center(mut self) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.align = crate::widgets::TextLineAlign::Center;
        }
        self
    }

    pub fn align_right(mut self) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.align = crate::widgets::TextLineAlign::Right;
        }
        self
    }

    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_children();
        self
    }

    pub fn on_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn char_(mut self, ch: char) -> Self {
        if let Props::Separator(sp) = &mut self.props {
            sp.char = ch;
        }
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        if let Props::Input(ip) = &mut self.props {
            ip.placeholder = Some(text.into());
        }
        self
    }

    pub fn password(mut self) -> Self {
        if let Props::Input(ip) = &mut self.props {
            ip.password = true;
        }
        self
    }

    pub fn default_value(mut self, text: impl Into<String>) -> Self {
        if let Props::Input(ip) = &mut self.props {
            ip.default_value = Some(text.into());
        }
        self
    }

    // ── Checkbox ──────────────────────────────────────────────

    pub fn checked(mut self, checked: bool) -> Self {
        if let Props::Checkbox(cp) = &mut self.props {
            cp.checked = checked;
        }
        self
    }

    // ── Spinner ───────────────────────────────────────────────

    pub fn running(mut self, running: bool) -> Self {
        if let Props::Spinner(sp) = &mut self.props {
            sp.running = running;
        }
        self
    }

    // ── Slider / Gauge shared ─────────────────────────────────

    pub fn value(mut self, value: f32) -> Self {
        match &mut self.props {
            Props::Slider(sp) => sp.value = value,
            Props::Gauge(gp) => gp.value = value,
            _ => {}
        }
        self
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        match &mut self.props {
            Props::Slider(sp) => {
                sp.min = min;
                sp.max = max;
            }
            Props::Gauge(gp) => {
                gp.min = min;
                gp.max = max;
            }
            _ => {}
        }
        self
    }

    // ── Orientation (Slider, Select, RadioGroup, Gauge, ScrollBar) ──

    pub fn horizontal(mut self) -> Self {
        match &mut self.props {
            Props::Slider(sp) => sp.horizontal = true,
            Props::RadioGroup(rp) => rp.horizontal = true,
            Props::Gauge(gp) => gp.horizontal = true,
            Props::ScrollBar(sb) => sb.horizontal = true,
            _ => {}
        }
        self
    }

    pub fn vertical(mut self) -> Self {
        match &mut self.props {
            Props::Slider(sp) => sp.horizontal = false,
            Props::RadioGroup(rp) => rp.horizontal = false,
            Props::Gauge(gp) => gp.horizontal = false,
            Props::ScrollBar(sb) => sb.horizontal = false,
            _ => {}
        }
        self
    }

    // ── Select ────────────────────────────────────────────────

    pub fn selected(mut self, index: usize) -> Self {
        match &mut self.props {
            Props::Select(sp) => sp.selected = index,
            Props::RadioGroup(rp) => rp.selected = index,
            _ => {}
        }
        self
    }

    pub fn wrap(mut self) -> Self {
        if let Props::Select(sp) = &mut self.props {
            sp.wrap = true;
        }
        self
    }

    pub fn show_description(mut self) -> Self {
        if let Props::Select(sp) = &mut self.props {
            sp.show_description = true;
        }
        self
    }

    // ── Gauge ─────────────────────────────────────────────────

    pub fn segments(mut self, count: u32) -> Self {
        if let Props::Gauge(gp) = &mut self.props {
            gp.segments = count;
        }
        self
    }

    pub fn show_label(mut self) -> Self {
        if let Props::Gauge(gp) = &mut self.props {
            gp.show_label = true;
        }
        self
    }

    // ── ScrollBar ─────────────────────────────────────────────

    pub fn scroll_size(mut self, size: f32) -> Self {
        if let Props::ScrollBar(sb) = &mut self.props {
            sb.scroll_size = size;
        }
        self
    }

    pub fn viewport_size(mut self, size: f32) -> Self {
        if let Props::ScrollBar(sb) = &mut self.props {
            sb.viewport_size = size;
        }
        self
    }

    pub fn scroll_position(mut self, pos: f32) -> Self {
        if let Props::ScrollBar(sb) = &mut self.props {
            sb.scroll_position = pos;
        }
        self
    }

    pub fn show_arrows(mut self) -> Self {
        if let Props::ScrollBar(sb) = &mut self.props {
            sb.show_arrows = true;
        }
        self
    }

    pub fn build(self) -> Node {
        let mut props = self.props;
        if let (Props::Text(tp), Some(content)) = (&mut props, self.text_content) {
            tp.content = content;
        }
        let layout = match &props {
            Props::View(vp) => {
                let mut layout = self.layout;
                if let Some(ref border) = vp.border {
                    let mut top = 0.0_f32;
                    let mut right = 0.0_f32;
                    let mut bottom = 0.0_f32;
                    let mut left = 0.0_f32;
                    if border.sides.top && border.chars.horizontal != '\0' {
                        top = 1.0;
                    }
                    if border.sides.bottom && border.chars.horizontal != '\0' {
                        bottom = 1.0;
                    }
                    if border.sides.left && border.chars.vertical != '\0' {
                        left = 1.0;
                    }
                    if border.sides.right && border.chars.vertical != '\0' {
                        right = 1.0;
                    }
                    if top > 0.0 || right > 0.0 || bottom > 0.0 || left > 0.0 {
                        layout = layout.add_padding(top, right, bottom, left);
                    }
                }
                layout
            }
            _ => self.layout,
        };
        Node::Element(Box::new(Element {
            kind: self.kind,
            key: self.key,
            layout,
            props,
            children: self.children,
            action: self.action,
        }))
    }
}

impl From<ElementBuilder> for Node {
    fn from(builder: ElementBuilder) -> Self {
        builder.build()
    }
}

pub trait IntoChildren {
    fn into_children(self) -> Vec<Node>;
}

impl IntoChildren for Vec<Node> {
    fn into_children(self) -> Vec<Node> {
        self
    }
}

impl<const N: usize> IntoChildren for [Node; N] {
    fn into_children(self) -> Vec<Node> {
        self.to_vec()
    }
}

impl IntoChildren for Node {
    fn into_children(self) -> Vec<Node> {
        vec![self]
    }
}

impl From<&'static str> for Key {
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}
