//! EditorWidget — multi-line text editor widget.
//!
//! Wraps `EditBuffer` + `EditorView` for a full multi-line editing experience
//! with cursor display, line numbers, scrolling, and selection.

use std::cell::RefCell;

use opentui_rust as ot;
use opentui_rust::{Cell, Rgba, Style, WrapMode};
use opentui_rust::text::{EditBuffer, EditorView};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::widget::{Overflow, RenderContext, Widget, WidgetId};

impl std::fmt::Debug for EditorWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorWidget")
            .field("id", &self.id)
            .field("focused", &self.focused)
            .field("line_numbers", &self.line_numbers)
            .finish_non_exhaustive()
    }
}

pub struct EditorWidget {
    id: WidgetId,
    style: LayoutStyle,
    editor: RefCell<EditorView>,
    line_numbers: bool,
    wrap_mode: WrapMode,
    placeholder: Option<String>,
    placeholder_style: Style,
    visible: bool,
    opacity: f32,
    focusable: bool,
    focused: bool,
}

impl EditorWidget {
    pub fn new(id: WidgetId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            editor: RefCell::new(EditorView::new(EditBuffer::new())),
            line_numbers: false,
            wrap_mode: WrapMode::None,
            placeholder: None,
            placeholder_style: Style::builder()
                .fg(Rgba::new(0.5, 0.5, 0.5, 1.0))
                .build(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
        }
    }

    pub fn with_text(id: WidgetId, style: LayoutStyle, text: &str) -> Self {
        Self {
            editor: RefCell::new(EditorView::new(EditBuffer::with_text(text))),
            ..Self::new(id, style)
        }
    }

    pub fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    pub fn wrap_mode(mut self, mode: WrapMode) -> Self {
        self.wrap_mode = mode;
        self
    }

    pub fn buffer(&self) -> std::cell::Ref<'_, EditBuffer> {
        std::cell::Ref::map(self.editor.borrow(), |e| e.edit_buffer())
    }

    pub fn buffer_mut(&self) -> std::cell::RefMut<'_, EditBuffer> {
        std::cell::RefMut::map(self.editor.borrow_mut(), |e| e.edit_buffer_mut())
    }

    pub fn set_text(&self, text: &str) {
        self.editor.borrow_mut().edit_buffer_mut().set_text(text);
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn placeholder_style(mut self, style: Style) -> Self {
        self.placeholder_style = style;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.editor.borrow().edit_buffer().buffer().is_empty()
    }
}

impl Widget for EditorWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn render(&self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            return;
        }

        let is_empty = self.editor.borrow().edit_buffer().buffer().is_empty();
        if is_empty && !self.focused {
            if let Some(ref ph) = self.placeholder {
                let display_w = ot::unicode::display_width(ph) as u32;
                let chars: Vec<char> = ph.chars().collect();
                let max = display_w.min(w);
                for i in 0..max {
                    if let Some(ch) = chars.get(i as usize) {
                        ctx.buffer.set_blended(
                            x + i,
                            y,
                            Cell::new(*ch, self.placeholder_style),
                        );
                    }
                }
                return;
            }
        }

        let mut editor = self.editor.borrow_mut();
        editor.set_wrap_mode(self.wrap_mode);
        editor.set_line_numbers(self.line_numbers);
        editor.set_viewport(x, y, w, h);
        editor.render_to(ctx.buffer, x, y, w, h);
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn opacity(&self) -> f32 {
        self.opacity
    }

    fn overflow(&self) -> Overflow {
        Overflow::Hidden
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

    fn handle_key(&mut self, key: &ot::KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(ot::KeyModifiers::CTRL);
        let alt = key.modifiers.contains(ot::KeyModifiers::ALT);
        let mut editor = self.editor.borrow_mut();
        let buf = editor.edit_buffer_mut();

        match key.code {
            ot::KeyCode::Char(ch) if ctrl => match ch {
                'a' => buf.move_to_line_start(),
                'e' => buf.move_to_line_end(),
                'u' => {
                    let start = buf.cursor();
                    buf.move_to_line_start();
                    let line_start = buf.cursor();
                    buf.delete_range(line_start, start);
                }
                'k' => {
                    let start = buf.cursor();
                    buf.move_to_line_end();
                    let line_end = buf.cursor();
                    buf.delete_range(start, line_end);
                }
                _ => return false,
            },
            ot::KeyCode::Char(ch) if !alt => {
                buf.insert(&ch.to_string());
            }
            ot::KeyCode::Enter => {
                buf.insert("\n");
            }
            ot::KeyCode::Backspace => {
                buf.delete_backward();
            }
            ot::KeyCode::Delete => {
                buf.delete_forward();
            }
            ot::KeyCode::Left if alt => {
                buf.move_word_left();
            }
            ot::KeyCode::Left => {
                buf.move_left();
            }
            ot::KeyCode::Right if alt => {
                buf.move_word_right();
            }
            ot::KeyCode::Right => {
                buf.move_right();
            }
            ot::KeyCode::Up => {
                buf.move_up();
            }
            ot::KeyCode::Down => {
                buf.move_down();
            }
            ot::KeyCode::Home => {
                buf.move_to_line_start();
            }
            ot::KeyCode::End => {
                buf.move_to_line_end();
            }
            ot::KeyCode::Tab => {
                buf.insert("    ");
            }
            _ => return false,
        }

        true
    }

    fn handle_mouse(&mut self, _mouse: &ot::MouseEvent) -> bool {
        false
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
