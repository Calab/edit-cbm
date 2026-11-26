//! Wrapping document lines into screen rows.
//!
//! A line of length `n` wrapped at `w` columns occupies `n / w + 1` rows, so
//! the cursor always has a cell to sit in after the last character -- the same
//! rule the Atari screen editor uses for logical lines.

use crate::document::{Document, Pos};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    /// The machine's native screen width (40 columns, with its left margin).
    Native,
    /// Wrap at the width of the window, like a PC notepad.
    Window,
}

/// Which codes take no room on screen when codes are hidden: control
/// codes, except between quotes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HideRule {
    /// A bit per code.
    codes: [u64; 4],
    quote: Option<u8>,
}

impl HideRule {
    /// Hide `codes`, except between `quote` characters.
    pub fn new(codes: impl IntoIterator<Item = u8>, quote: Option<u8>) -> Self {
        let mut bits = [0u64; 4];
        for c in codes {
            bits[usize::from(c / 64)] |= 1 << (c % 64);
        }
        Self { codes: bits, quote }
    }

    /// Every control code `machine` has, whether or not it acts on the
    /// text (and every code that acts, if any isn't a control code).
    pub fn for_machine(machine: &dyn crate::Machine) -> Self {
        Self::new(
            (0..=255).filter(|&c| machine.is_control(c) || machine.code_effect(c).is_some()),
            machine.quote_code(),
        )
    }

    fn hides(&self, c: u8) -> bool {
        self.codes[usize::from(c / 64)] & (1 << (c % 64)) != 0
    }

    /// Which bytes of `line` are hidden. Quote mode ends with the line.
    pub fn mask(&self, line: &[u8]) -> Vec<bool> {
        let mut quoted = false;
        line.iter()
            .map(|&b| {
                if Some(b) == self.quote {
                    quoted = !quoted;
                    false
                } else {
                    !quoted && self.hides(b)
                }
            })
            .collect()
    }
}

/// Screen geometry for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// Columns available for text on each row.
    pub text_cols: usize,
    /// Blank columns before the text.
    pub left_margin: usize,
    /// Text rows visible on screen.
    pub rows: usize,
    /// Codes that take no room on screen, if any are hidden.
    pub hide: Option<HideRule>,
}

impl Layout {
    /// Compute the layout for a window `win_cols` x `win_rows` cells.
    pub fn new(
        mode: WrapMode,
        win_cols: usize,
        win_rows: usize,
        native_cols: usize,
        native_margin: usize,
    ) -> Self {
        let (text_cols, left_margin) = match mode {
            WrapMode::Native => (native_cols - native_margin, native_margin),
            WrapMode::Window => (win_cols.max(2) - 1, 1),
        };
        Self {
            text_cols: text_cols.max(1),
            left_margin,
            rows: win_rows.max(1),
            hide: None,
        }
    }

    /// The hidden bytes of `line`, if codes are hidden.
    pub fn mask(&self, line: &[u8]) -> Option<Vec<bool>> {
        self.hide.map(|h| h.mask(line))
    }

    /// The indices of the bytes of `line` that take room on screen.
    pub fn visible(&self, line: &[u8]) -> Vec<usize> {
        match self.mask(line) {
            Some(m) => (0..line.len()).filter(|&i| !m[i]).collect(),
            None => (0..line.len()).collect(),
        }
    }

    /// Screen columns `line` takes.
    pub fn visible_len(&self, line: &[u8]) -> usize {
        match self.mask(line) {
            Some(m) => m.iter().filter(|&&h| !h).count(),
            None => line.len(),
        }
    }

    /// Screen rows `line` occupies.
    pub fn line_rows(&self, line: &[u8]) -> usize {
        self.rows_for_len(self.visible_len(line))
    }

    /// The screen column (from the line's start) of byte `col`.
    pub fn visual_col(&self, line: &[u8], col: usize) -> usize {
        match self.mask(line) {
            Some(m) => m[..col.min(line.len())].iter().filter(|&&h| !h).count(),
            None => col,
        }
    }

    /// The byte at screen column `x` of `line` (the line's length past its
    /// end). Hidden codes before that character come before the cursor, so
    /// typing there takes the colour the character shows.
    pub fn byte_col(&self, line: &[u8], x: usize) -> usize {
        match self.mask(line) {
            Some(m) => (0..line.len())
                .filter(|&i| !m[i])
                .nth(x)
                .unwrap_or(line.len()),
            None => x.min(line.len()),
        }
    }

    /// `col` moved past any hidden codes it sits before.
    pub fn snap(&self, line: &[u8], col: usize) -> usize {
        match self.mask(line) {
            Some(m) => {
                let mut c = col.min(line.len());
                while c < line.len() && m[c] {
                    c += 1;
                }
                c
            }
            None => col,
        }
    }

    /// The column of the visible character before `col` (for moving left
    /// and Backspace over hidden codes), or `None` at the line's start.
    pub fn prev_visible(&self, line: &[u8], col: usize) -> Option<usize> {
        let col = col.min(line.len());
        match self.mask(line) {
            Some(m) => (0..col).rev().find(|&i| !m[i]),
            None => col.checked_sub(1),
        }
    }

    /// Screen rows a line of `len` characters occupies.
    pub fn rows_for_len(&self, len: usize) -> usize {
        len / self.text_cols + 1
    }

    /// Total screen rows for the whole document.
    pub fn total_rows(&self, doc: &Document) -> usize {
        doc.lines().iter().map(|l| self.line_rows(l)).sum()
    }

    /// Absolute screen row (from the top of the document) and x of a position.
    pub fn locate(&self, doc: &Document, pos: Pos) -> (usize, usize) {
        let before: usize = doc.lines()[..pos.line]
            .iter()
            .map(|l| self.line_rows(l))
            .sum();
        let x = self.visual_col(doc.line(pos.line), pos.col);
        (before + x / self.text_cols, x % self.text_cols)
    }

    /// Document position shown at absolute screen row `row`, column `x`.
    /// Clamps to the end of the line, or of the document.
    pub fn position_at(&self, doc: &Document, row: usize, x: usize) -> Pos {
        let mut remaining = row;
        for (i, l) in doc.lines().iter().enumerate() {
            let rows = self.line_rows(l);
            if remaining < rows {
                let x = remaining * self.text_cols + x.min(self.text_cols - 1);
                return Pos {
                    line: i,
                    col: self.byte_col(l, x),
                };
            }
            remaining -= rows;
        }
        let last = doc.line_count() - 1;
        Pos {
            line: last,
            col: doc.line(last).len(),
        }
    }

    /// Cursor position `delta` screen rows away, aiming for column `want_x`.
    pub fn move_rows(&self, doc: &Document, pos: Pos, delta: isize, want_x: usize) -> Pos {
        let (row, _) = self.locate(doc, pos);
        let last = self.total_rows(doc) - 1;
        let target = (row as isize + delta).clamp(0, last as isize) as usize;
        self.position_at(doc, target, want_x)
    }

    /// Adjust `top` (first visible row) so that the cursor row is on screen.
    pub fn scroll_to_cursor(&self, doc: &Document, top: usize) -> usize {
        let (row, _) = self.locate(doc, doc.cursor());
        if row < top {
            row
        } else if row >= top + self.rows {
            row + 1 - self.rows
        } else {
            top
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc_with_lens(lens: &[usize]) -> Document {
        Document::from_lines(lens.iter().map(|&n| vec![b'A'; n]).collect(), true)
    }

    fn native() -> Layout {
        Layout::new(WrapMode::Native, 80, 24, 40, 2)
    }

    #[test]
    fn native_mode_has_38_text_columns() {
        let l = native();
        assert_eq!((l.text_cols, l.left_margin), (38, 2));
        assert_eq!(l.rows_for_len(0), 1);
        assert_eq!(l.rows_for_len(37), 1);
        assert_eq!(l.rows_for_len(38), 2); // cursor wraps to the next row
        assert_eq!(l.rows_for_len(114), 4);
    }

    #[test]
    fn locate_and_position_are_inverse() {
        let l = native();
        let d = doc_with_lens(&[10, 50, 0]);
        let p = Pos { line: 1, col: 45 };
        let (row, x) = l.locate(&d, p);
        assert_eq!((row, x), (2, 7));
        assert_eq!(l.position_at(&d, row, x), p);
    }

    #[test]
    fn moving_down_keeps_column_and_clamps() {
        let l = native();
        let d = doc_with_lens(&[30, 5]);
        let p = l.move_rows(&d, Pos { line: 0, col: 20 }, 1, 20);
        assert_eq!(p, Pos { line: 1, col: 5 });
        let p = l.move_rows(&d, p, 5, 20);
        assert_eq!(p, Pos { line: 1, col: 5 });
    }

    #[test]
    fn window_mode_uses_window_width() {
        let l = Layout::new(WrapMode::Window, 100, 30, 40, 2);
        assert_eq!((l.text_cols, l.left_margin), (99, 1));
    }

    #[test]
    fn scrolling_follows_cursor() {
        let l = Layout::new(WrapMode::Native, 40, 3, 40, 2);
        let mut d = doc_with_lens(&[1, 1, 1, 1, 1]);
        d.set_cursor(Pos { line: 4, col: 0 });
        assert_eq!(l.scroll_to_cursor(&d, 0), 2);
        d.set_cursor(Pos { line: 1, col: 0 });
        assert_eq!(l.scroll_to_cursor(&d, 2), 1);
    }

    #[test]
    fn hidden_codes_take_no_room() {
        // Codes 1 and 2 hidden, except between quotes.
        let mut l = Layout::new(WrapMode::Native, 80, 24, 4, 0);
        l.hide = Some(HideRule::new([1, 2], Some(b'"')));
        let line = b"A\x01BC\"\x02\"D\x02".to_vec();
        assert_eq!(l.visible(&line), [0, 2, 3, 4, 5, 6, 7]);
        assert_eq!(l.visible_len(&line), 7);
        assert_eq!(l.line_rows(&line), 2);
        // Screen column 1 is B, after the hidden code.
        assert_eq!(l.byte_col(&line, 1), 2);
        assert_eq!(l.visual_col(&line, 2), 1);
        assert_eq!(l.snap(&line, 1), 2);
        assert_eq!(l.prev_visible(&line, 2), Some(0));
        // Past the end: after the trailing hidden code.
        assert_eq!(l.byte_col(&line, 9), 9);
        assert_eq!(l.snap(&line, 8), 9);
        let d = Document::from_lines(vec![line], true);
        let p = Pos { line: 0, col: 6 };
        assert_eq!(l.locate(&d, p), (1, 1));
        assert_eq!(l.position_at(&d, 1, 1), p);
    }
}
