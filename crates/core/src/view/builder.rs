use std::fmt;

use opentui_rust::Rgba;
use opentui_rust::Style;
use opentui_rust::WrapMode;
use opentui_rust::buffer::TitleAlign;

use crate::layout::LayoutStyle;
use crate::view::element::{Element, ElementKind};
use crate::view::event::{EventBinding, EventKind};
use crate::view::key::Key;
use crate::view::node::{Node, OverlayNode};
use crate::view::props::{
    BgFill, FillProps, InputProps, ListProps, Props, SeparatorProps, TextProps, ViewProps,
};
use crate::widget::Overflow;
use crate::widgets::{BorderChars, BorderSides, BorderStyle};

#[derive(Debug, Clone)]
pub struct StyledSegment {
    pub text: String,
    pub fg: Rgba,
    pub bg: Option<Rgba>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl StyledSegment {
    pub fn new(text: impl Into<String>, fg: Rgba) -> Self {
        Self {
            text: text.into(),
            fg,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }

    pub fn bg(mut self, color: Rgba) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

pub fn view() -> ElementBuilder<()> {
    ElementBuilder::new(ElementKind::View)
}

pub fn panel() -> ElementBuilder<()> {
    ElementBuilder::new(ElementKind::View).border_rounded(Rgba::new(0.3, 0.3, 0.35, 1.0))
}

pub fn text(content: impl Into<String>) -> ElementBuilder<()> {
    let mut builder = ElementBuilder::new(ElementKind::Text);
    builder.text_content = Some(content.into());
    builder
}

#[allow(clippy::needless_pass_by_value)]
pub fn rich_text(segments: Vec<StyledSegment>) -> ElementBuilder<()> {
    if segments.is_empty() {
        return ElementBuilder::new(ElementKind::Text);
    }

    let mut content = String::new();
    let mut highlights = Vec::new();

    let default_fg = segments[0].fg;
    let default_bold = segments[0].bold;
    let default_italic = segments[0].italic;
    let default_underline = segments[0].underline;

    for seg in &segments {
        let start = content.len();
        content.push_str(&seg.text);
        let end = content.len();

        let mut builder = Style::builder();
        let mut diff_count = 0usize;

        if seg.fg != default_fg {
            builder = builder.fg(seg.fg);
            diff_count += 1;
        }
        if seg.bold && !default_bold {
            builder = builder.bold();
            diff_count += 1;
        }
        if seg.italic && !default_italic {
            builder = builder.italic();
            diff_count += 1;
        }
        if seg.underline && !default_underline {
            builder = builder.underline();
            diff_count += 1;
        }
        if let Some(bg) = seg.bg {
            builder = builder.bg(bg);
            diff_count += 1;
        }

        if diff_count > 0 {
            highlights.push((start, end, builder.build()));
        }
    }

    let mut builder = ElementBuilder::new(ElementKind::Text);
    builder.text_content = Some(content);
    if let Props::Text(ref mut tp) = builder.props {
        tp.fg = default_fg;
        tp.bold = default_bold;
        tp.italic = default_italic;
        tp.underline = default_underline;
        tp.highlights = highlights;
    }
    builder
}

pub fn span(text: impl Into<String>, fg: Rgba) -> StyledSegment {
    StyledSegment::new(text, fg)
}

pub fn input() -> ElementBuilder<()> {
    let mut builder = ElementBuilder::new(ElementKind::Input);
    builder.props = Props::Input(InputProps::default());
    builder
}

#[doc(hidden)]
pub fn list(item_count: usize) -> ElementBuilder<()> {
    let mut builder = ElementBuilder::new(ElementKind::List);
    builder.props = Props::List(ListProps {
        item_count,
        scrollbar: true,
    });
    builder
}

pub fn fill(color: Rgba) -> ElementBuilder<()> {
    let mut builder = ElementBuilder::new(ElementKind::Fill);
    builder.props = Props::Fill(FillProps { color });
    builder
}

pub fn separator() -> ElementBuilder<()> {
    let mut builder = ElementBuilder::new(ElementKind::Separator);
    builder.props = Props::Separator(SeparatorProps::default());
    builder
}

pub fn fragment<M>(children: Vec<Node<M>>) -> Node<M> {
    Node::Fragment(children)
}

pub fn when<M>(condition: bool, f: impl FnOnce() -> Node<M>) -> Node<M> {
    if condition { f() } else { Node::Empty }
}

pub fn empty<M>() -> Node<M> {
    Node::Empty
}

pub fn overlay<M>(content: Node<M>) -> OverlayBuilder<M> {
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

#[derive(Debug)]
pub struct OverlayBuilder<M> {
    content: Box<Node<M>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    backdrop: bool,
    z_order: u16,
}

impl<M> OverlayBuilder<M> {
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

    pub fn build(self) -> Node<M> {
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

impl<M> From<OverlayBuilder<M>> for Node<M> {
    fn from(builder: OverlayBuilder<M>) -> Self {
        builder.build()
    }
}

pub struct ElementBuilder<M> {
    kind: ElementKind,
    key: Option<Key>,
    layout: LayoutStyle,
    props: Props,
    children: Vec<Node<M>>,
    text_content: Option<String>,
    events: Vec<EventBinding<M>>,
}

impl<M> fmt::Debug for ElementBuilder<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ElementBuilder")
            .field("kind", &self.kind)
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl<M> ElementBuilder<M> {
    pub fn new(kind: ElementKind) -> Self {
        let props = match kind {
            ElementKind::View => Props::View(ViewProps::default()),
            ElementKind::Text => Props::Text(TextProps::default()),
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
            events: Vec::new(),
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

    pub fn wrap(mut self, mode: WrapMode) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.wrap = mode;
        }
        self
    }

    pub fn bg_fill(mut self, mode: BgFill) -> Self {
        if let Props::Text(tp) = &mut self.props {
            tp.bg_fill = mode;
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

    pub fn children(mut self, children: impl IntoChildren<M>) -> Self {
        self.children = children.into_children();
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

    pub fn build(self) -> Node<M> {
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
            Props::Text(tp) => {
                let mut layout = self.layout;
                if !tp.content.is_empty()
                    && layout.inner.size.height.is_auto()
                    && layout.inner.min_size.height.is_auto()
                {
                    layout = layout.min_height(1.0);
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
            events: self.events,
        }))
    }
}

impl<M: Clone> ElementBuilder<M> {
    pub fn add_event(mut self, kind: EventKind, msg: M) -> Self {
        self.events.push(EventBinding { kind, message: msg });
        self
    }

    pub fn interactive(mut self) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.interactive = true;
        }
        self
    }

    pub fn hover_bg(mut self, color: Rgba) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.hover_bg = Some(color);
        }
        self
    }

    pub fn hover_fg(mut self, color: Rgba) -> Self {
        if let Props::View(vp) = &mut self.props {
            vp.hover_fg = Some(color);
        }
        self
    }

    pub fn map_msg<N: Clone>(self, f: impl Fn(M) -> N + Clone + 'static) -> ElementBuilder<N> {
        ElementBuilder {
            kind: self.kind,
            key: self.key,
            layout: self.layout,
            props: self.props,
            children: self
                .children
                .into_iter()
                .map(|c| c.map_msg(f.clone()))
                .collect(),
            text_content: self.text_content,
            events: self
                .events
                .into_iter()
                .map(|e| EventBinding {
                    kind: e.kind,
                    message: (f.clone())(e.message),
                })
                .collect(),
        }
    }
}

impl ElementBuilder<()> {
    fn transition<N: Clone>(self, kind: EventKind, msg: N) -> ElementBuilder<N> {
        ElementBuilder {
            kind: self.kind,
            key: self.key,
            layout: self.layout,
            props: self.props,
            children: self
                .children
                .into_iter()
                .map(|c| c.map_msg(|()| unreachable!("() nodes should not carry events")))
                .collect(),
            text_content: self.text_content,
            events: vec![EventBinding { kind, message: msg }],
        }
    }

    pub fn on_action(self, action: impl Into<String>) -> ElementBuilder<String> {
        self.transition(EventKind::Click, action.into())
    }

    pub fn on_click<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::Click, msg)
    }

    pub fn on_right_click<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::RightClick, msg)
    }

    pub fn on_hover<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::Hover, msg)
    }

    pub fn on_scroll<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::Scroll, msg)
    }

    pub fn typed<N: Clone>(self, msg: N) -> ElementBuilder<N> {
        self.transition(EventKind::Click, msg)
    }
}

impl<M> From<ElementBuilder<M>> for Node<M> {
    fn from(builder: ElementBuilder<M>) -> Self {
        builder.build()
    }
}

pub trait IntoChildren<M> {
    fn into_children(self) -> Vec<Node<M>>;
}

impl<M> IntoChildren<M> for Vec<Node<M>> {
    fn into_children(self) -> Self {
        self
    }
}

impl<M: Clone, const N: usize> IntoChildren<M> for [Node<M>; N] {
    fn into_children(self) -> Vec<Node<M>> {
        self.into_iter().collect()
    }
}

impl<M> IntoChildren<M> for Node<M> {
    fn into_children(self) -> Vec<Self> {
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
