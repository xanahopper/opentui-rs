//! EditorWidget — multi-line text editor widget.
//!
//! Wraps `EditBuffer` + `EditorView` for a full multi-line editing experience
//! with cursor display, line numbers, scrolling, and selection.

use std::cell::RefCell;

use crate::text::{EditBuffer, EditorView};
use crate::{Cell, Rgba, Style, WrapMode};

use crate::layout::{ComputedLayout, LayoutStyle};
use crate::renderable::behavior::{Behavior, FrameworkDefaults};
use crate::renderable::context::RenderContext;
use crate::renderable::node::Overflow;

impl std::fmt::Debug for EditorWidget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorWidget")
            .field("focused", &self.focused)
            .field("line_numbers", &self.line_numbers)
            .finish_non_exhaustive()
    }
}

pub struct EditorWidget {
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
    selectable: bool,
    viewport: Option<(u32, u32, u32, u32)>,
}

impl EditorWidget {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            editor: RefCell::new(EditorView::new(EditBuffer::new())),
            line_numbers: false,
            wrap_mode: WrapMode::None,
            placeholder: None,
            placeholder_style: Style::builder().fg(Rgba::new(0.5, 0.5, 0.5, 1.0)).build(),
            visible: true,
            opacity: 1.0,
            focusable: true,
            focused: false,
            selectable: true,
            viewport: None,
        }
    }

    pub fn with_text(style: LayoutStyle, text: &str) -> Self {
        Self {
            editor: RefCell::new(EditorView::new(EditBuffer::with_text(text))),
            ..Self::new(style)
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

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn selection_style(self, style: Style) -> Self {
        self.editor.borrow_mut().set_selection_style(style);
        self
    }

    #[must_use]
    pub fn selected_text(&self) -> Option<String> {
        self.editor.borrow().selected_text()
    }

    fn offset_at_screen_position(&self, position: (i32, i32)) -> Option<usize> {
        let (x, y, width, height) = self.viewport?;
        if width == 0 || height == 0 {
            return None;
        }
        let local_x = position.0 - x as i32;
        let local_y = position.1 - y as i32;
        if local_x < 0 || local_y < 0 {
            return Some(0);
        }
        Some(
            self.editor
                .borrow()
                .offset_at_viewport_position(local_x as u32, local_y as u32),
        )
    }

    fn selection_intersects(&self, anchor: (i32, i32), focus: (i32, i32)) -> bool {
        let Some((x, y, width, height)) = self.viewport else {
            return false;
        };
        if width == 0 || height == 0 {
            return false;
        }
        let (start, end) = if (anchor.1, anchor.0) <= (focus.1, focus.0) {
            (anchor, focus)
        } else {
            (focus, anchor)
        };
        let node_start = (y as i32, x as i32);
        let node_end = (
            y.saturating_add(height.saturating_sub(1)) as i32,
            x.saturating_add(width) as i32,
        );
        (start.1, start.0) <= node_end && (end.1, end.0) >= node_start
    }
}

impl Behavior for EditorWidget {
    fn style(&self) -> &LayoutStyle {
        &self.style
    }

    fn style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }

    fn framework_defaults(&self) -> FrameworkDefaults {
        FrameworkDefaults {
            focusable: self.focusable,
            overflow: Overflow::Hidden,
            ..FrameworkDefaults::default()
        }
    }

    fn set_focus_state(&mut self, focused: bool, _has_focused_descendant: bool) {
        self.focused = focused;
    }

    fn render_self(&mut self, ctx: &mut RenderContext<'_>, layout: &ComputedLayout) {
        let x = layout.x as u32;
        let y = layout.y as u32;
        let w = layout.width as u32;
        let h = layout.height as u32;

        if w == 0 || h == 0 {
            self.viewport = None;
            return;
        }

        self.viewport = Some((x, y, w, h));

        let is_empty = self.editor.borrow().edit_buffer().buffer().is_empty();
        if is_empty && !self.focused {
            if let Some(ref ph) = self.placeholder {
                let display_w = crate::unicode::display_width(ph) as u32;
                let chars: Vec<char> = ph.chars().collect();
                let max = display_w.min(w);
                for i in 0..max {
                    if let Some(ch) = chars.get(i as usize) {
                        ctx.buffer
                            .set_blended(x + i, y, Cell::new(*ch, self.placeholder_style));
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

    fn handle_key(&mut self, key: &crate::KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(crate::KeyModifiers::CTRL);
        let alt = key.modifiers.contains(crate::KeyModifiers::ALT);
        let shift = key.modifiers.contains(crate::KeyModifiers::SHIFT);
        let mut editor = self.editor.borrow_mut();
        let is_navigation = matches!(
            key.code,
            crate::KeyCode::Left
                | crate::KeyCode::Right
                | crate::KeyCode::Up
                | crate::KeyCode::Down
                | crate::KeyCode::Home
                | crate::KeyCode::End
        );
        let selecting = shift && is_navigation;

        if selecting && !editor.has_selection() {
            editor.start_selection();
        } else if is_navigation && !shift {
            editor.clear_selection();
        }

        match key.code {
            crate::KeyCode::Char(ch) if ctrl => match ch {
                'a' => editor.edit_buffer_mut().move_to_line_start(),
                'e' => editor.edit_buffer_mut().move_to_line_end(),
                'u' => {
                    let buf = editor.edit_buffer_mut();
                    let start = buf.cursor();
                    buf.move_to_line_start();
                    let line_start = buf.cursor();
                    buf.delete_range(line_start, start);
                }
                'k' => {
                    let buf = editor.edit_buffer_mut();
                    let start = buf.cursor();
                    buf.move_to_line_end();
                    let line_end = buf.cursor();
                    buf.delete_range(start, line_end);
                }
                _ => return false,
            },
            crate::KeyCode::Char(ch) if !alt => {
                editor.edit_buffer_mut().insert(&ch.to_string());
            }
            crate::KeyCode::Enter => {
                editor.edit_buffer_mut().insert("\n");
            }
            crate::KeyCode::Backspace => {
                editor.edit_buffer_mut().delete_backward();
            }
            crate::KeyCode::Delete => {
                editor.edit_buffer_mut().delete_forward();
            }
            crate::KeyCode::Left if alt => {
                editor.edit_buffer_mut().move_word_left();
            }
            crate::KeyCode::Left => {
                editor.edit_buffer_mut().move_left();
            }
            crate::KeyCode::Right if alt => {
                editor.edit_buffer_mut().move_word_right();
            }
            crate::KeyCode::Right => {
                editor.edit_buffer_mut().move_right();
            }
            crate::KeyCode::Up => {
                editor.edit_buffer_mut().move_up();
            }
            crate::KeyCode::Down => {
                editor.edit_buffer_mut().move_down();
            }
            crate::KeyCode::Home => {
                editor.edit_buffer_mut().move_to_line_start();
            }
            crate::KeyCode::End => {
                editor.edit_buffer_mut().move_to_line_end();
            }
            crate::KeyCode::Tab => {
                editor.edit_buffer_mut().insert("    ");
            }
            _ => return false,
        }

        if selecting {
            editor.extend_selection_to_cursor();
        }

        true
    }

    fn handle_mouse(&mut self, _mouse: &crate::MouseEvent) -> bool {
        false
    }

    fn selectable(&self) -> bool {
        self.selectable
    }

    fn update_selection(&mut self, anchor: (i32, i32), focus: (i32, i32), is_start: bool) -> bool {
        if !self.selectable || !self.selection_intersects(anchor, focus) {
            self.editor.borrow_mut().clear_selection();
            return false;
        }
        let Some(anchor_offset) = self.offset_at_screen_position(anchor) else {
            return false;
        };
        let Some(focus_offset) = self.offset_at_screen_position(focus) else {
            return false;
        };
        let mut end = anchor_offset.max(focus_offset);
        if !is_start && focus_offset < anchor_offset {
            end = end
                .saturating_add(1)
                .min(self.editor.borrow().edit_buffer().buffer().len_chars());
        }
        let mut editor = self.editor.borrow_mut();
        editor.set_selection(anchor_offset.min(focus_offset), end);
        editor.edit_buffer_mut().set_cursor_by_offset(focus_offset);
        anchor_offset != focus_offset
    }

    fn clear_selection(&mut self) {
        self.editor.borrow_mut().clear_selection();
    }

    fn selected_text(&self) -> Option<String> {
        EditorWidget::selected_text(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn shift_navigation_extends_selection() {
        let mut widget = EditorWidget::with_text(LayoutStyle::default(), "hello");
        widget
            .editor
            .borrow_mut()
            .edit_buffer_mut()
            .move_to_line_end();

        assert!(widget.handle_key(&KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT,)));
        assert_eq!(widget.editor.borrow().selected_text().as_deref(), Some("o"));

        assert!(widget.handle_key(&KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT,)));
        assert_eq!(
            widget.editor.borrow().selected_text().as_deref(),
            Some("lo")
        );
    }

    #[test]
    fn navigation_without_shift_clears_selection() {
        let mut widget = EditorWidget::with_text(LayoutStyle::default(), "hello");
        widget
            .editor
            .borrow_mut()
            .edit_buffer_mut()
            .move_to_line_end();
        widget.handle_key(&KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));

        assert!(widget.handle_key(&KeyEvent::new(KeyCode::Left, KeyModifiers::empty(),)));
        assert_eq!(widget.editor.borrow().selected_text(), None);
    }

    #[test]
    fn global_pointer_coordinates_update_editor_selection_and_cursor() {
        let mut widget = EditorWidget::with_text(LayoutStyle::default(), "hello world");
        let mut buffer = crate::OptimizedBuffer::new(12, 1);
        let mut ctx = RenderContext {
            buffer: &mut buffer,
            grapheme_pool: None,
            link_pool: None,
            hit_grid: None,
            theme: None,
        };
        widget.render_self(
            &mut ctx,
            &ComputedLayout {
                x: 0.0,
                y: 0.0,
                width: 12.0,
                height: 1.0,
            },
        );

        assert!(!widget.update_selection((1, 0), (1, 0), true));
        assert!(widget.update_selection((1, 0), (5, 0), false));
        assert_eq!(widget.selected_text().as_deref(), Some("ello"));
        assert_eq!(widget.buffer().cursor().offset, 5);
    }
}
