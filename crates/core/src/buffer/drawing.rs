//! Text and box drawing operations.

use crate::buffer::OptimizedBuffer;
use crate::cell::{Cell, CellContent};
use crate::color::Rgba;
use crate::grapheme_pool::GraphemePool;
use crate::style::Style;
use unicode_segmentation::UnicodeSegmentation;

/// Box drawing style with corner and edge characters.
#[derive(Clone, Debug)]
pub struct BoxStyle {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub style: Style,
}

/// Box side visibility.
#[derive(Clone, Copy, Debug)]
pub struct BoxSides {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl Default for BoxSides {
    fn default() -> Self {
        Self {
            top: true,
            right: true,
            bottom: true,
            left: true,
        }
    }
}

/// Title alignment for boxed titles.
#[derive(Clone, Copy, Debug, Default)]
pub enum TitleAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Extended box drawing options.
#[derive(Clone, Debug)]
pub struct BoxOptions {
    pub style: BoxStyle,
    pub sides: BoxSides,
    pub fill: Option<Rgba>,
    pub title: Option<String>,
    pub title_align: TitleAlign,
}

impl BoxOptions {
    #[must_use]
    pub fn new(style: BoxStyle) -> Self {
        Self {
            style,
            sides: BoxSides::default(),
            fill: None,
            title: None,
            title_align: TitleAlign::Left,
        }
    }
}

impl BoxStyle {
    /// Single-line box drawing characters.
    #[must_use]
    pub fn single(style: Style) -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
            style,
        }
    }

    /// Double-line box drawing characters.
    #[must_use]
    pub fn double(style: Style) -> Self {
        Self {
            top_left: '╔',
            top_right: '╗',
            bottom_left: '╚',
            bottom_right: '╝',
            horizontal: '═',
            vertical: '║',
            style,
        }
    }

    /// Rounded corner box drawing characters.
    #[must_use]
    pub fn rounded(style: Style) -> Self {
        Self {
            top_left: '╭',
            top_right: '╮',
            bottom_left: '╰',
            bottom_right: '╯',
            horizontal: '─',
            vertical: '│',
            style,
        }
    }

    /// Heavy (bold) box drawing characters.
    #[must_use]
    pub fn heavy(style: Style) -> Self {
        Self {
            top_left: '┏',
            top_right: '┓',
            bottom_left: '┗',
            bottom_right: '┛',
            horizontal: '━',
            vertical: '┃',
            style,
        }
    }

    /// ASCII box drawing characters (works in all terminals).
    #[must_use]
    pub fn ascii(style: Style) -> Self {
        Self {
            top_left: '+',
            top_right: '+',
            bottom_left: '+',
            bottom_right: '+',
            horizontal: '-',
            vertical: '|',
            style,
        }
    }
}

impl Default for BoxStyle {
    fn default() -> Self {
        Self::single(Style::NONE)
    }
}

/// Compute the next tab stop for the given column.
///
/// Tab stops are at multiples of `tab_width`. A `tab_width` of 0 is treated
/// as 1 to avoid division by zero.
#[inline]
fn next_tab_stop(col: u32, tab_width: u32) -> u32 {
    let tw = tab_width.max(1);
    ((col / tw) + 1) * tw
}

/// Expand a tab at column `col` to the next tab stop, filling with `space_cell`
/// via `set_blended`. Returns the new column position.
#[inline]
fn expand_tab_blended(
    buffer: &mut OptimizedBuffer,
    col: u32,
    row: u32,
    tab_width: u32,
    space_cell: Cell,
) -> u32 {
    let stop = next_tab_stop(col, tab_width);
    for c in col..stop {
        buffer.set_blended(c, row, space_cell);
    }
    stop
}

/// Draw text at position, handling grapheme clusters and wide characters.
///
/// Newlines (`\n`) advance to the next row, resetting to the starting X column.
/// Carriage returns (`\r`) reset to the starting X column without advancing rows.
/// Tabs (`\t`) advance to the next tab stop (see [`OptimizedBuffer::tab_width`]),
/// filling the gap with spaces.
///
/// **Note:** Multi-codepoint graphemes are stored with placeholder IDs.
/// For proper grapheme pool integration, use [`draw_text_with_pool`].
pub fn draw_text(buffer: &mut OptimizedBuffer, x: u32, y: u32, text: &str, style: Style) {
    let mut col = x;
    let mut row = y;
    let fg = style.fg.unwrap_or(Rgba::WHITE);
    let bg = style.bg.unwrap_or(Rgba::TRANSPARENT);
    let attrs = style.attributes;
    let tab_width = buffer.tab_width();
    let space_cell = Cell {
        content: CellContent::Char(' '),
        fg,
        bg,
        attributes: attrs,
    };

    // Fast path: pure ASCII text (very common case)
    if text.is_ascii() {
        for &byte in text.as_bytes() {
            if byte == b'\n' {
                row += 1;
                col = x;
                continue;
            }
            if byte == b'\r' {
                col = x;
                continue;
            }
            if byte == b'\t' {
                col = expand_tab_blended(buffer, col, row, tab_width, space_cell);
                continue;
            }
            let ch = byte as char;
            let width = u32::from((' '..='~').contains(&ch));
            let cell = Cell {
                content: CellContent::Char(ch),
                fg,
                bg,
                attributes: attrs,
            };
            buffer.set_blended(col, row, cell);
            col += width;
        }
        return;
    }

    // Slow path: Unicode text with grapheme segmentation
    for grapheme in text.graphemes(true) {
        if grapheme == "\n" {
            row += 1;
            col = x;
            continue;
        }
        if grapheme == "\r" {
            col = x;
            continue;
        }
        if grapheme == "\t" {
            col = expand_tab_blended(buffer, col, row, tab_width, space_cell);
            continue;
        }

        let cell = Cell::from_grapheme(grapheme, style);
        let width = cell.display_width();

        buffer.set_blended(col, row, cell);

        // Add continuation cells for wide characters
        for i in 1..width {
            buffer.set_blended(col + i as u32, row, Cell::continuation(bg));
        }

        col += width as u32;
    }
}

/// Draw text at position, allocating grapheme IDs from the pool.
///
/// This version properly allocates multi-codepoint graphemes (emoji, ZWJ sequences)
/// in the pool, allowing them to be resolved during rendering.
///
/// Newlines (`\n`) advance to the next row, resetting to the starting X column.
/// Carriage returns (`\r`) reset to the starting X column without advancing rows.
///
/// # Arguments
///
/// * `buffer` - The buffer to draw to
/// * `pool` - The grapheme pool for allocating multi-codepoint graphemes
/// * `x` - Starting X position
/// * `y` - Y position
/// * `text` - The text to draw
/// * `style` - Style to apply to the text
pub fn draw_text_with_pool(
    buffer: &mut OptimizedBuffer,
    pool: &mut GraphemePool,
    x: u32,
    y: u32,
    text: &str,
    style: Style,
) {
    let mut col = x;
    let mut row = y;
    let fg = style.fg.unwrap_or(Rgba::WHITE);
    let bg = style.bg.unwrap_or(Rgba::TRANSPARENT);
    let attrs = style.attributes;
    let tab_width = buffer.tab_width();
    let space_cell = Cell {
        content: CellContent::Char(' '),
        fg,
        bg,
        attributes: attrs,
    };

    for grapheme in text.graphemes(true) {
        if grapheme == "\n" {
            row += 1;
            col = x;
            continue;
        }
        if grapheme == "\r" {
            col = x;
            continue;
        }
        if grapheme == "\t" {
            let stop = next_tab_stop(col, tab_width);
            for c in col..stop {
                buffer.set_blended_with_pool(pool, c, row, space_cell);
            }
            col = stop;
            continue;
        }

        // Determine cell content and width
        // Fast path: ASCII single-byte characters are always single codepoint
        let (content, width) = if grapheme.len() == 1 {
            // SAFETY: len() == 1 means exactly one ASCII byte
            let ch = grapheme.as_bytes()[0] as char;
            // ASCII printable characters have width 1
            let w = usize::from((' '..='~').contains(&ch));
            (CellContent::Char(ch), w)
        } else if grapheme.chars().count() == 1 {
            // Single non-ASCII codepoint - store directly as Char
            // SAFETY: chars().count() == 1 guarantees next() returns Some
            let ch = grapheme
                .chars()
                .next()
                .expect("chars().count() == 1 but next() returned None");
            let w = crate::unicode::display_width_char(ch);
            (CellContent::Char(ch), w)
        } else {
            // Multi-codepoint grapheme - allocate from pool
            let id = pool.intern(grapheme);
            (CellContent::Grapheme(id), id.width())
        };

        let cell = Cell {
            content,
            fg,
            bg,
            attributes: attrs,
        };

        buffer.set_blended_with_pool(pool, col, row, cell);

        // Add continuation cells for wide characters
        for i in 1..width {
            buffer.set_blended_with_pool(pool, col + i as u32, row, Cell::continuation(bg));
        }

        col += width as u32;
    }
}

/// Draw a single character at position, allocating from pool if needed.
///
/// For single codepoints, stores directly. For multi-codepoint graphemes,
/// allocates from the pool.
pub fn draw_char_with_pool(
    buffer: &mut OptimizedBuffer,
    pool: &mut GraphemePool,
    x: u32,
    y: u32,
    grapheme: &str,
    style: Style,
) {
    let fg = style.fg.unwrap_or(Rgba::WHITE);
    let bg = style.bg.unwrap_or(Rgba::TRANSPARENT);
    let attrs = style.attributes;

    // Fast path: ASCII single-byte characters are always single codepoint
    let (content, width) = if grapheme.len() == 1 {
        // SAFETY: len() == 1 means exactly one ASCII byte
        let ch = grapheme.as_bytes()[0] as char;
        let w = usize::from((' '..='~').contains(&ch));
        (CellContent::Char(ch), w)
    } else if grapheme.chars().count() == 1 {
        // SAFETY: chars().count() == 1 guarantees next() returns Some
        let ch = grapheme
            .chars()
            .next()
            .expect("chars().count() == 1 but next() returned None");
        let w = crate::unicode::display_width_char(ch);
        (CellContent::Char(ch), w)
    } else {
        let id = pool.intern(grapheme);
        (CellContent::Grapheme(id), id.width())
    };

    let cell = Cell {
        content,
        fg,
        bg,
        attributes: attrs,
    };

    buffer.set_blended_with_pool(pool, x, y, cell);

    // Add continuation cells for wide characters
    for i in 1..width {
        buffer.set_blended_with_pool(pool, x + i as u32, y, Cell::continuation(bg));
    }
}

/// Draw a box border.
pub fn draw_box(buffer: &mut OptimizedBuffer, x: u32, y: u32, w: u32, h: u32, box_style: BoxStyle) {
    if w < 2 || h < 2 {
        return;
    }

    let style = box_style.style;

    // Corners
    buffer.set_blended(x, y, Cell::new(box_style.top_left, style));
    buffer.set_blended(x + w - 1, y, Cell::new(box_style.top_right, style));
    buffer.set_blended(x, y + h - 1, Cell::new(box_style.bottom_left, style));
    buffer.set_blended(
        x + w - 1,
        y + h - 1,
        Cell::new(box_style.bottom_right, style),
    );

    // Horizontal edges
    for col in (x + 1)..(x + w - 1) {
        buffer.set_blended(col, y, Cell::new(box_style.horizontal, style));
        buffer.set_blended(col, y + h - 1, Cell::new(box_style.horizontal, style));
    }

    // Vertical edges
    for row in (y + 1)..(y + h - 1) {
        buffer.set_blended(x, row, Cell::new(box_style.vertical, style));
        buffer.set_blended(x + w - 1, row, Cell::new(box_style.vertical, style));
    }
}

/// Draw a box border with extended options.
pub fn draw_box_with_options(
    buffer: &mut OptimizedBuffer,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    options: BoxOptions,
) {
    if w == 0 || h == 0 {
        return;
    }

    let style = options.style.style;

    let has_top = options.sides.top && options.style.horizontal != '\0';
    let has_bottom = options.sides.bottom && options.style.horizontal != '\0';
    let has_left = options.sides.left && options.style.vertical != '\0';
    let has_right = options.sides.right && options.style.vertical != '\0';

    let inner_x = if has_left { x + 1 } else { x };
    let inner_y = if has_top { y + 1 } else { y };
    let inner_right = x + w;
    let inner_bottom = y + h;
    let inner_w = inner_right.saturating_sub(inner_x);
    let inner_h = inner_bottom.saturating_sub(inner_y);

    if let Some(bg) = options.fill {
        if inner_w > 0 && inner_h > 0 {
            buffer.fill_rect(inner_x, inner_y, inner_w, inner_h, bg);
        }
    }

    // Corners — skip if char is '\0'
    if has_top && has_left && options.style.top_left != '\0' {
        buffer.set_blended(x, y, Cell::new(options.style.top_left, style));
    }
    if has_top && has_right && options.style.top_right != '\0' {
        buffer.set_blended(x + w - 1, y, Cell::new(options.style.top_right, style));
    }
    if has_bottom && has_left && options.style.bottom_left != '\0' {
        buffer.set_blended(x, y + h - 1, Cell::new(options.style.bottom_left, style));
    }
    if has_bottom && has_right && options.style.bottom_right != '\0' {
        buffer.set_blended(
            x + w - 1,
            y + h - 1,
            Cell::new(options.style.bottom_right, style),
        );
    }

    // Horizontal edges
    if has_top {
        let start = if has_left { x + 1 } else { x };
        let end = if has_right { x + w - 1 } else { x + w };
        for col in start..end {
            buffer.set_blended(col, y, Cell::new(options.style.horizontal, style));
        }
    }
    if has_bottom {
        let start = if has_left { x + 1 } else { x };
        let end = if has_right { x + w - 1 } else { x + w };
        for col in start..end {
            buffer.set_blended(col, y + h - 1, Cell::new(options.style.horizontal, style));
        }
    }

    // Vertical edges
    if has_left {
        let start = if has_top { y + 1 } else { y };
        let end = if has_bottom { y + h - 1 } else { y + h };
        for row in start..end {
            buffer.set_blended(x, row, Cell::new(options.style.vertical, style));
        }
    }
    if has_right {
        let start = if has_top { y + 1 } else { y };
        let end = if has_bottom { y + h - 1 } else { y + h };
        for row in start..end {
            buffer.set_blended(x + w - 1, row, Cell::new(options.style.vertical, style));
        }
    }

    // Title
    if let Some(title) = options.title {
        if has_top && w > 2 {
            let title_width = crate::unicode::display_width(&title) as i32;
            let box_width = w as i32;
            let min_title_space = 4;

            if title_width > 0 && box_width >= title_width + min_title_space {
                let padding = 2;
                let start_x = x as i32;
                let end_x = start_x + box_width - 1;

                let mut title_x = match options.title_align {
                    TitleAlign::Left => start_x + padding,
                    TitleAlign::Center => {
                        let centered = (box_width - title_width) / 2;
                        start_x + padding.max(centered)
                    }
                    TitleAlign::Right => start_x + box_width - padding - title_width,
                };

                // Clamp title position to respect padding on both sides.
                // min_x ensures left padding, max_x ensures right padding.
                let min_x = start_x + padding;
                let max_x = end_x - padding - title_width + 1;
                title_x = title_x.clamp(min_x, max_x);

                buffer.draw_text(title_x as u32, y, &title, style);
            }
        }
    }
}

/// Draw a horizontal line.
pub fn draw_hline(buffer: &mut OptimizedBuffer, x: u32, y: u32, len: u32, ch: char, style: Style) {
    for col in x..x.saturating_add(len) {
        buffer.set_blended(col, y, Cell::new(ch, style));
    }
}

/// Draw a vertical line.
pub fn draw_vline(buffer: &mut OptimizedBuffer, x: u32, y: u32, len: u32, ch: char, style: Style) {
    for row in y..y.saturating_add(len) {
        buffer.set_blended(x, row, Cell::new(ch, style));
    }
}

/// Draw a table grid.
///
/// `column_offsets` are the x-positions of vertical borders and `row_offsets`
/// are the y-positions of horizontal borders, both relative to the `(x, y)`
/// origin. Line segments between intersections are drawn with `border_char`;
/// crossings of a row and column border are drawn with `intersection_char`.
/// All cells are alpha-blended and respect the buffer's scissor stack.
pub fn draw_grid(
    buffer: &mut OptimizedBuffer,
    x: u32,
    y: u32,
    column_offsets: &[u32],
    row_offsets: &[u32],
    border_char: char,
    intersection_char: char,
    fg: Rgba,
    bg: Rgba,
) {
    if column_offsets.is_empty() || row_offsets.is_empty() {
        return;
    }

    let style = Style {
        fg: Some(fg),
        bg: Some(bg),
        attributes: crate::style::TextAttributes::empty(),
    };

    let Some(&min_col_rel) = column_offsets.iter().min() else {
        return;
    };
    let Some(&max_col_rel) = column_offsets.iter().max() else {
        return;
    };
    let Some(&min_row_rel) = row_offsets.iter().min() else {
        return;
    };
    let Some(&max_row_rel) = row_offsets.iter().max() else {
        return;
    };

    if min_col_rel > max_col_rel || min_row_rel > max_row_rel {
        return;
    }

    // Horizontal border lines: at each row offset, walk the full column span.
    // Column border positions become intersections.
    for &row_off in row_offsets {
        let ry = y.saturating_add(row_off);
        for cx_rel in min_col_rel..=max_col_rel {
            let cx = x.saturating_add(cx_rel);
            let ch = if column_offsets.contains(&cx_rel) {
                intersection_char
            } else {
                border_char
            };
            buffer.set_blended(cx, ry, Cell::new(ch, style));
        }
    }

    // Vertical border lines: at each column offset, walk the full row span,
    // skipping row-offset positions (already drawn as intersections above).
    for &col_off in column_offsets {
        let cx = x.saturating_add(col_off);
        for cy_rel in min_row_rel..=max_row_rel {
            if row_offsets.contains(&cy_rel) {
                continue;
            }
            let ry = y.saturating_add(cy_rel);
            buffer.set_blended(cx, ry, Cell::new(border_char, style));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_text() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        draw_text(&mut buffer, 0, 0, "Hello", Style::fg(Rgba::RED));

        assert_eq!(
            buffer.get(0, 0).unwrap().content,
            crate::cell::CellContent::Char('H')
        );
        assert_eq!(
            buffer.get(4, 0).unwrap().content,
            crate::cell::CellContent::Char('o')
        );
    }

    // =========================================================================
    // Newline handling tests (bd-1k1s)
    // =========================================================================

    #[test]
    fn test_draw_text_with_newline() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        draw_text(&mut buffer, 0, 0, "Hello\nWorld", Style::NONE);

        // "Hello" on row 0
        assert_eq!(
            buffer.get(0, 0).unwrap().content,
            crate::cell::CellContent::Char('H')
        );
        assert_eq!(
            buffer.get(4, 0).unwrap().content,
            crate::cell::CellContent::Char('o')
        );

        // "World" on row 1
        assert_eq!(
            buffer.get(0, 1).unwrap().content,
            crate::cell::CellContent::Char('W')
        );
        assert_eq!(
            buffer.get(4, 1).unwrap().content,
            crate::cell::CellContent::Char('d')
        );
    }

    #[test]
    fn test_draw_text_with_carriage_return() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        draw_text(&mut buffer, 0, 0, "XXXXX\rHello", Style::NONE);

        // \r resets to start column, "Hello" overwrites "XXXXX"
        assert_eq!(
            buffer.get(0, 0).unwrap().content,
            crate::cell::CellContent::Char('H')
        );
        assert_eq!(
            buffer.get(4, 0).unwrap().content,
            crate::cell::CellContent::Char('o')
        );
    }

    #[test]
    fn test_draw_text_with_crlf() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        draw_text(&mut buffer, 0, 0, "Line1\r\nLine2", Style::NONE);

        // "Line1" on row 0
        assert_eq!(
            buffer.get(0, 0).unwrap().content,
            crate::cell::CellContent::Char('L')
        );

        // "Line2" on row 1 (CRLF advances one row total)
        assert_eq!(
            buffer.get(0, 1).unwrap().content,
            crate::cell::CellContent::Char('L')
        );
        assert_eq!(
            buffer.get(4, 1).unwrap().content,
            crate::cell::CellContent::Char('2')
        );
    }

    #[test]
    fn test_draw_text_multiline_with_offset() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        // Start at column 5, row 2
        draw_text(&mut buffer, 5, 2, "A\nB\nC", Style::NONE);

        // Each line should start at column 5 (the starting x)
        assert_eq!(
            buffer.get(5, 2).unwrap().content,
            crate::cell::CellContent::Char('A')
        );
        assert_eq!(
            buffer.get(5, 3).unwrap().content,
            crate::cell::CellContent::Char('B')
        );
        assert_eq!(
            buffer.get(5, 4).unwrap().content,
            crate::cell::CellContent::Char('C')
        );
    }

    #[test]
    fn test_draw_text_with_pool_multiline() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        let mut pool = GraphemePool::new();
        draw_text_with_pool(&mut buffer, &mut pool, 0, 0, "Line1\nLine2", Style::NONE);

        // "Line1" on row 0
        assert_eq!(
            buffer.get(0, 0).unwrap().content,
            crate::cell::CellContent::Char('L')
        );

        // "Line2" on row 1
        assert_eq!(
            buffer.get(0, 1).unwrap().content,
            crate::cell::CellContent::Char('L')
        );
        assert_eq!(
            buffer.get(4, 1).unwrap().content,
            crate::cell::CellContent::Char('2')
        );
    }

    #[test]
    fn test_draw_wide_char() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        draw_text(&mut buffer, 0, 0, "漢字", Style::NONE);

        // First character at 0, continuation at 1
        // Second character at 2, continuation at 3
        assert!(!buffer.get(0, 0).unwrap().is_continuation());
        assert!(buffer.get(1, 0).unwrap().is_continuation());
        assert!(!buffer.get(2, 0).unwrap().is_continuation());
        assert!(buffer.get(3, 0).unwrap().is_continuation());
    }

    #[test]
    fn test_draw_box() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        draw_box(&mut buffer, 0, 0, 10, 5, BoxStyle::single(Style::NONE));

        // Check corners
        assert_eq!(
            buffer.get(0, 0).unwrap().content,
            crate::cell::CellContent::Char('┌')
        );
        assert_eq!(
            buffer.get(9, 0).unwrap().content,
            crate::cell::CellContent::Char('┐')
        );
        assert_eq!(
            buffer.get(0, 4).unwrap().content,
            crate::cell::CellContent::Char('└')
        );
        assert_eq!(
            buffer.get(9, 4).unwrap().content,
            crate::cell::CellContent::Char('┘')
        );
    }

    #[test]
    fn test_draw_box_with_options_title() {
        let mut buffer = OptimizedBuffer::new(20, 5);
        let options = BoxOptions {
            style: BoxStyle::single(Style::NONE),
            sides: BoxSides::default(),
            fill: None,
            title: Some("Title".to_string()),
            title_align: TitleAlign::Left,
        };
        draw_box_with_options(&mut buffer, 0, 0, 10, 4, options);
        assert_eq!(
            buffer.get(1, 0).unwrap().content,
            crate::cell::CellContent::Char('─')
        );
        assert_eq!(
            buffer.get(2, 0).unwrap().content,
            crate::cell::CellContent::Char('T')
        );
    }

    #[test]
    fn test_draw_text_with_pool_ascii() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        let mut pool = GraphemePool::new();

        draw_text_with_pool(&mut buffer, &mut pool, 0, 0, "Hello", Style::fg(Rgba::RED));

        assert_eq!(buffer.get(0, 0).unwrap().content, CellContent::Char('H'));
        assert_eq!(buffer.get(4, 0).unwrap().content, CellContent::Char('o'));

        // No graphemes should be allocated for ASCII
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_draw_text_with_pool_emoji() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        let mut pool = GraphemePool::new();

        // Use a ZWJ family emoji which is multi-codepoint
        draw_text_with_pool(&mut buffer, &mut pool, 0, 0, "Hi 👨‍👩‍👧!", Style::NONE);

        // H, i, space should be Char
        assert!(matches!(
            buffer.get(0, 0).unwrap().content,
            CellContent::Char('H')
        ));
        assert!(matches!(
            buffer.get(1, 0).unwrap().content,
            CellContent::Char('i')
        ));
        assert!(matches!(
            buffer.get(2, 0).unwrap().content,
            CellContent::Char(' ')
        ));

        // 👨‍👩‍👧 should be Grapheme with width 2 (multi-codepoint ZWJ sequence)
        let emoji_cell = buffer.get(3, 0).unwrap();
        assert!(matches!(emoji_cell.content, CellContent::Grapheme(_)));
        assert_eq!(emoji_cell.display_width(), 2);

        // Cell 4 should be continuation
        assert!(buffer.get(4, 0).unwrap().is_continuation());

        // ! at position 5
        assert!(matches!(
            buffer.get(5, 0).unwrap().content,
            CellContent::Char('!')
        ));

        // One grapheme should be allocated
        assert_eq!(pool.active_count(), 1);

        // Can resolve the grapheme from the pool
        if let CellContent::Grapheme(id) = emoji_cell.content {
            assert_eq!(pool.get(id), Some("👨‍👩‍👧"));
        }
    }

    #[test]
    fn test_draw_text_with_pool_single_codepoint_emoji() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        let mut pool = GraphemePool::new();

        // Single codepoint emoji (👍) should be stored as Char, not Grapheme
        draw_text_with_pool(&mut buffer, &mut pool, 0, 0, "👍", Style::NONE);

        let cell = buffer.get(0, 0).unwrap();
        // Single codepoint emoji stored as Char (the codepoint fits in char)
        assert!(matches!(cell.content, CellContent::Char('👍')));
        assert_eq!(cell.display_width(), 2);
        assert!(buffer.get(1, 0).unwrap().is_continuation());

        // No graphemes allocated for single codepoint
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn test_draw_text_with_pool_deduplication() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        let mut pool = GraphemePool::new();

        // Draw the same multi-codepoint grapheme twice (family emoji)
        draw_text_with_pool(&mut buffer, &mut pool, 0, 0, "👨‍👩‍👧👨‍👩‍👧", Style::NONE);

        // Only one grapheme should be allocated (intern deduplicates)
        assert_eq!(pool.active_count(), 1);

        // Both cells should reference the same grapheme with refcount 2
        // First family at 0, continuation at 1; second family at 2, continuation at 3
        let cell1 = buffer.get(0, 0).unwrap();
        let cell2 = buffer.get(2, 0).unwrap();

        match (cell1.content, cell2.content) {
            (CellContent::Grapheme(id1), CellContent::Grapheme(id2)) => {
                assert_eq!(id1, id2);
                assert_eq!(pool.refcount(id1), 2);
            }
            other => {
                assert!(
                    matches!(other, (CellContent::Grapheme(_), CellContent::Grapheme(_))),
                    "Expected grapheme content for pooled multi-codepoint cells"
                );
            }
        }
    }

    #[test]
    fn test_draw_char_with_pool() {
        let mut buffer = OptimizedBuffer::new(80, 24);
        let mut pool = GraphemePool::new();

        // Single codepoint
        draw_char_with_pool(&mut buffer, &mut pool, 0, 0, "A", Style::NONE);
        assert!(matches!(
            buffer.get(0, 0).unwrap().content,
            CellContent::Char('A')
        ));

        // Multi-codepoint grapheme
        draw_char_with_pool(&mut buffer, &mut pool, 5, 0, "👨‍👩‍👧", Style::NONE);
        let cell = buffer.get(5, 0).unwrap();
        assert!(cell.content.is_grapheme());
        assert_eq!(cell.display_width(), 2);
        assert!(buffer.get(6, 0).unwrap().is_continuation());

        // Can resolve from pool
        if let CellContent::Grapheme(id) = cell.content {
            assert_eq!(pool.get(id), Some("👨‍👩‍👧"));
        }
    }

    #[test]
    fn test_draw_text_consistency() {
        // Case 1: draw_text (Fast path for ASCII)
        let mut buf1 = OptimizedBuffer::new(16, 1);
        draw_text(&mut buf1, 0, 0, "A\tB", Style::NONE);

        // Case 2: draw_text_with_pool (Standard path, even for ASCII if pool is used)
        let mut buf2 = OptimizedBuffer::new(16, 1);
        let mut pool = GraphemePool::new();
        draw_text_with_pool(&mut buf2, &mut pool, 0, 0, "A\tB", Style::NONE);

        // Verify consistency:
        // 'A' at 0
        assert_eq!(buf1.get(0, 0).unwrap().content, CellContent::Char('A'));
        assert_eq!(buf2.get(0, 0).unwrap().content, CellContent::Char('A'));

        // Tab expanded to next tab stop (col 8), filling 1..8 with spaces
        assert_eq!(buf1.get(1, 0).unwrap().content, CellContent::Char(' '));
        assert_eq!(buf2.get(1, 0).unwrap().content, CellContent::Char(' '));
        assert_eq!(buf1.get(7, 0).unwrap().content, CellContent::Char(' '));
        assert_eq!(buf2.get(7, 0).unwrap().content, CellContent::Char(' '));

        // 'B' at 8 (the next tab stop with default tab_width=8)
        assert_eq!(buf1.get(8, 0).unwrap().content, CellContent::Char('B'));
        assert_eq!(buf2.get(8, 0).unwrap().content, CellContent::Char('B'));
    }

    // =========================================================================
    // Tab expansion tests
    // =========================================================================

    #[test]
    fn test_draw_text_tab_default_width() {
        let mut buffer = OptimizedBuffer::new(20, 1);
        draw_text(&mut buffer, 0, 0, "\t", Style::NONE);
        // From col 0, next stop is 8 -> cols 0..8 are spaces.
        for c in 0..8 {
            assert_eq!(buffer.get(c, 0).unwrap().content, CellContent::Char(' '));
        }
        assert!(buffer.get(8, 0).unwrap().content.is_empty());
    }

    #[test]
    fn test_draw_text_tab_mid_line() {
        let mut buffer = OptimizedBuffer::new(20, 1);
        // 'A' at col 0, tab from col 1 -> next stop 8, 'B' at col 8.
        draw_text(&mut buffer, 0, 0, "A\tB", Style::NONE);
        assert_eq!(buffer.get(0, 0).unwrap().content, CellContent::Char('A'));
        for c in 1..8 {
            assert_eq!(buffer.get(c, 0).unwrap().content, CellContent::Char(' '));
        }
        assert_eq!(buffer.get(8, 0).unwrap().content, CellContent::Char('B'));
    }

    #[test]
    fn test_draw_text_tab_custom_width() {
        let mut buffer = OptimizedBuffer::new(20, 1);
        buffer.set_tab_width(4);
        // 'A' at 0, tab from 1 -> next stop 4, 'B' at 4.
        draw_text(&mut buffer, 0, 0, "A\tB", Style::NONE);
        assert_eq!(buffer.get(0, 0).unwrap().content, CellContent::Char('A'));
        for c in 1..4 {
            assert_eq!(buffer.get(c, 0).unwrap().content, CellContent::Char(' '));
        }
        assert_eq!(buffer.get(4, 0).unwrap().content, CellContent::Char('B'));
    }

    #[test]
    fn test_draw_text_tab_aligned_to_stop() {
        // Drawing at x=8 (already on a tab stop boundary): tab should advance a
        // full tab_width (to col 16), not zero.
        let mut buffer = OptimizedBuffer::new(24, 1);
        draw_text(&mut buffer, 8, 0, "\tX", Style::NONE);
        for c in 8..16 {
            assert_eq!(buffer.get(c, 0).unwrap().content, CellContent::Char(' '));
        }
        assert_eq!(buffer.get(16, 0).unwrap().content, CellContent::Char('X'));
    }

    #[test]
    fn test_draw_text_tab_zero_width_no_panic() {
        let mut buffer = OptimizedBuffer::new(20, 1);
        buffer.set_tab_width(0);
        draw_text(&mut buffer, 0, 0, "A\tB", Style::NONE);
        // tab_width=0 treated as 1: from col 1 next stop is 2.
        assert_eq!(buffer.get(0, 0).unwrap().content, CellContent::Char('A'));
        assert_eq!(buffer.get(1, 0).unwrap().content, CellContent::Char(' '));
        assert_eq!(buffer.get(2, 0).unwrap().content, CellContent::Char('B'));
    }

    #[test]
    fn test_draw_text_tab_unicode_path() {
        // Non-ASCII text forces the grapheme (slow) path; tab must still expand.
        let mut buffer = OptimizedBuffer::new(20, 1);
        draw_text(&mut buffer, 0, 0, "é\tB", Style::NONE);
        assert_eq!(buffer.get(0, 0).unwrap().content, CellContent::Char('é'));
        for c in 1..8 {
            assert_eq!(buffer.get(c, 0).unwrap().content, CellContent::Char(' '));
        }
        assert_eq!(buffer.get(8, 0).unwrap().content, CellContent::Char('B'));
    }

    #[test]
    fn test_draw_text_tab_multiline_resets_col() {
        // After a newline the column resets to x; tab stops recompute from x.
        let mut buffer = OptimizedBuffer::new(20, 2);
        draw_text(&mut buffer, 0, 0, "A\n\tB", Style::NONE);
        assert_eq!(buffer.get(0, 0).unwrap().content, CellContent::Char('A'));
        // Row 1: tab from col 0 -> fills 0..8, B at 8.
        for c in 0..8 {
            assert_eq!(buffer.get(c, 1).unwrap().content, CellContent::Char(' '));
        }
        assert_eq!(buffer.get(8, 1).unwrap().content, CellContent::Char('B'));
    }

    #[test]
    fn test_tab_width_getter_setter() {
        let mut buffer = OptimizedBuffer::new(10, 1);
        assert_eq!(buffer.tab_width(), 8);
        buffer.set_tab_width(4);
        assert_eq!(buffer.tab_width(), 4);
    }

    #[test]
    fn test_draw_text_with_pool_tab() {
        let mut buffer = OptimizedBuffer::new(20, 1);
        let mut pool = GraphemePool::new();
        draw_text_with_pool(&mut buffer, &mut pool, 0, 0, "A\tB", Style::NONE);
        assert_eq!(buffer.get(0, 0).unwrap().content, CellContent::Char('A'));
        for c in 1..8 {
            assert_eq!(buffer.get(c, 0).unwrap().content, CellContent::Char(' '));
        }
        assert_eq!(buffer.get(8, 0).unwrap().content, CellContent::Char('B'));
        // No graphemes allocated for ASCII + tab.
        assert_eq!(pool.active_count(), 0);
    }

    // =========================================================================
    // draw_grid tests
    // =========================================================================

    #[test]
    fn test_draw_grid_basic() {
        let mut buffer = OptimizedBuffer::new(12, 6);
        // Grid origin (1,1), vertical borders at cols 0/4/8 (rel), horizontal at rows 0/2/4 (rel).
        buffer.draw_grid(
            1,
            1,
            &[0, 4, 8],
            &[0, 2, 4],
            'H',
            'X',
            Rgba::WHITE,
            Rgba::TRANSPARENT,
        );

        // Intersections (column border x row border): absolute (1,1),(5,1),(9,1),(1,3),(5,3),(9,3),(1,5),(5,5),(9,5)
        for &(gx, gy) in &[
            (1, 1),
            (5, 1),
            (9, 1),
            (1, 3),
            (5, 3),
            (9, 3),
            (1, 5),
            (5, 5),
            (9, 5),
        ] {
            assert_eq!(
                buffer.get(gx, gy).unwrap().content,
                CellContent::Char('X'),
                "intersection at ({gx}, {gy})"
            );
        }

        // Horizontal segment on row 1 between col 1 and 5 -> cols 2,3,4 are 'H'
        for gx in 2..5 {
            assert_eq!(buffer.get(gx, 1).unwrap().content, CellContent::Char('H'));
        }
        // Vertical segment on col 1 between row 1 and 3 -> row 2 is 'H'
        assert_eq!(buffer.get(1, 2).unwrap().content, CellContent::Char('H'));
    }

    #[test]
    fn test_draw_grid_respects_scissor() {
        let mut buffer = OptimizedBuffer::new(12, 6);
        // Visible region: x in [1,5), y in [1,3).
        buffer.push_scissor(crate::buffer::ClipRect::new(1, 1, 4, 2));

        buffer.draw_grid(
            0,
            0,
            &[0, 4, 8],
            &[0, 2, 4],
            'H',
            'X',
            Rgba::WHITE,
            Rgba::TRANSPARENT,
        );

        // Outside scissor: nothing drawn.
        assert!(buffer.get(0, 0).unwrap().content.is_empty());
        assert!(buffer.get(5, 1).unwrap().content.is_empty());
        assert!(buffer.get(9, 5).unwrap().content.is_empty());

        // Vertical border at col 4, row 1 (within scissor) -> 'H'.
        assert_eq!(buffer.get(4, 1).unwrap().content, CellContent::Char('H'));
        // Horizontal border at row 2: cols 1..4 are 'H', col 4 is intersection 'X'.
        assert_eq!(buffer.get(1, 2).unwrap().content, CellContent::Char('H'));
        assert_eq!(buffer.get(3, 2).unwrap().content, CellContent::Char('H'));
        assert_eq!(buffer.get(4, 2).unwrap().content, CellContent::Char('X'));
    }

    #[test]
    fn test_draw_grid_empty_offsets() {
        let mut buffer = OptimizedBuffer::new(10, 10);
        // Empty offset slices are a no-op.
        buffer.draw_grid(0, 0, &[], &[0, 2], 'H', 'X', Rgba::WHITE, Rgba::TRANSPARENT);
        buffer.draw_grid(0, 0, &[0, 2], &[], 'H', 'X', Rgba::WHITE, Rgba::TRANSPARENT);
        for cell in buffer.cells() {
            assert!(cell.content.is_empty());
        }
    }
}
