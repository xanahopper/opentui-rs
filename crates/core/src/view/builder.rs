use opentui_rust::Rgba;
use opentui_rust::buffer::TitleAlign;

use crate::layout::LayoutStyle;
use crate::view::element::{Element, ElementKind};
use crate::view::key::Key;
use crate::view::node::{Node, OverlayNode};
use crate::view::props::{
    FillProps, InputProps, ListProps, Props, SeparatorProps, StyledTextProps, TextProps, ViewProps,
};
use crate::widget::Overflow;
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
        Node::Overlay(OverlayNode {
            content: self.content,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            backdrop: self.backdrop,
            z_order: self.z_order,
        })
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
        match &mut self.props {
            Props::View(vp) => vp.overflow = Overflow::Hidden,
            _ => {}
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
            _ => {}
        }
        self
    }

    pub fn fg(mut self, color: Rgba) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.fg = color;
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

    pub fn value(mut self, text: impl Into<String>) -> Self {
        if let Props::Input(ip) = &mut self.props {
            ip.initial_value = Some(text.into());
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
        Node::Element(Element {
            kind: self.kind,
            key: self.key,
            layout,
            props,
            children: self.children,
            action: self.action,
        })
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
    fn from(s: &'static str) -> Key {
        Key::Static(s)
    }
}

impl From<String> for Key {
    fn from(s: String) -> Key {
        Key::Owned(s)
    }
}
