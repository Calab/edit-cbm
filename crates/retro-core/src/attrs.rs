//! The colour and reverse of each character, worked out from the codes in
//! the text (see [`crate::machine::CodeEffect`]).
//!
//! Nothing is stored: the codes are the only record, so deleting a colour
//! code gives the text after it the colour in force before the code.
//!
//! * A colour code colours the text after it, over later lines too, until
//!   the next colour code. Before any, the screen's text colour.
//! * Reverse on lasts until reverse off, over later lines too.
//! * Between quotes, codes act only up to the closing quote: after it the
//!   colour and reverse are what they were before the opening quote.
//!   Quote mode ends with the line, as on the C64.
//! * A colour code is drawn in its own colour (RVS ON/OFF in the colour
//!   in force), and control codes are never drawn reversed, so they
//!   always look like codes.

use crate::document::Pos;
use crate::machine::{CodeEffect, Machine};

/// How the text is drawn at a point.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Attr {
    /// A colour number, or `None` for the screen's text colour.
    pub colour: Option<u8>,
    pub reverse: bool,
}

impl Attr {
    fn apply(&mut self, effect: CodeEffect) {
        match effect {
            CodeEffect::Colour(c) => self.colour = Some(c),
            CodeEffect::ReverseOn => self.reverse = true,
            CodeEffect::ReverseOff => self.reverse = false,
        }
    }
}

/// Each character's look in `line`, which starts in state `start`, and the
/// state the next line starts in.
pub fn line_attrs(machine: &dyn Machine, line: &[u8], start: Attr) -> (Vec<Attr>, Attr) {
    let (looks, outer, _) = scan(machine, line, start);
    (looks, outer)
}

/// [`line_attrs`], also giving the state inside quotes if the line ends
/// in a string.
fn scan(machine: &dyn Machine, line: &[u8], start: Attr) -> (Vec<Attr>, Attr, Option<Attr>) {
    let quote = machine.quote_code();
    let mut outer = start;
    let mut inner: Option<Attr> = None;
    let mut out = Vec::with_capacity(line.len());
    for &b in line {
        if Some(b) == quote {
            inner = match inner {
                Some(_) => None,
                None => Some(outer),
            };
            out.push(outer);
            continue;
        }
        let state = inner.as_mut().unwrap_or(&mut outer);
        // A code shows in the colour it sets.
        if let Some(e) = machine.code_effect(b) {
            state.apply(e);
        }
        let mut look = *state;
        if machine.is_control(b) {
            look.reverse = false;
        }
        out.push(look);
    }
    (out, outer, inner)
}

/// The colour and reverse a character typed at `pos` would get.
pub fn state_at(machine: &dyn Machine, lines: &[Vec<u8>], pos: Pos) -> Attr {
    let line = &lines[pos.line];
    let start = start_of(machine, lines, pos.line);
    let (_, outer, inner) = scan(machine, &line[..pos.col.min(line.len())], start);
    inner.unwrap_or(outer)
}

/// The state after `text` (lines, as on the clipboard), starting in
/// `start`.
pub fn state_after(machine: &dyn Machine, text: &[Vec<u8>], start: Attr) -> Attr {
    let mut state = start;
    for (i, line) in text.iter().enumerate() {
        let (_, outer, inner) = scan(machine, line, state);
        state = if i + 1 == text.len() {
            inner.unwrap_or(outer)
        } else {
            outer
        };
    }
    state
}

/// The codes that change state `from` into `to`: a colour code, then RVS
/// ON or OFF, as needed. `default_colour` is the colour number of the
/// screen's text colour, for a state with no colour code in force.
pub fn codes_between(
    machine: &dyn Machine,
    from: Attr,
    to: Attr,
    default_colour: Option<u8>,
) -> Vec<u8> {
    let mut out = Vec::new();
    let colour = |a: Attr| a.colour.or(default_colour);
    if colour(from) != colour(to)
        && let Some(c) = colour(to).and_then(|c| machine.effect_code(CodeEffect::Colour(c)))
    {
        out.push(c);
    }
    if from.reverse != to.reverse {
        let e = if to.reverse {
            CodeEffect::ReverseOn
        } else {
            CodeEffect::ReverseOff
        };
        out.extend(machine.effect_code(e));
    }
    out
}

/// The state at the start of line `index` of `lines`.
pub fn start_of(machine: &dyn Machine, lines: &[Vec<u8>], index: usize) -> Attr {
    lines[..index].iter().fold(Attr::default(), |state, line| {
        line_attrs(machine, line, state).1
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FileFormat;

    /// Codes 1-3 are colours, 4 reverse on, 5 reverse off; `"` quotes.
    struct M;
    impl Machine for M {
        fn name(&self) -> &'static str {
            "test"
        }
        fn glyph(&self, _: u8) -> [u8; 8] {
            [0; 8]
        }
        fn space(&self) -> u8 {
            b' '
        }
        fn screen_columns(&self) -> usize {
            40
        }
        fn left_margin(&self) -> usize {
            0
        }
        fn formats(&self) -> &'static [&'static dyn FileFormat] {
            &[]
        }
        fn code_effect(&self, code: u8) -> Option<CodeEffect> {
            match code {
                1..=3 => Some(CodeEffect::Colour(code)),
                4 => Some(CodeEffect::ReverseOn),
                5 => Some(CodeEffect::ReverseOff),
                _ => None,
            }
        }
        fn is_control(&self, code: u8) -> bool {
            code < 0x20
        }
        fn quote_code(&self) -> Option<u8> {
            Some(b'"')
        }
        fn effect_code(&self, effect: CodeEffect) -> Option<u8> {
            Some(match effect {
                CodeEffect::Colour(c) => c,
                CodeEffect::ReverseOn => 4,
                CodeEffect::ReverseOff => 5,
            })
        }
    }

    fn colours(line: &[u8], start: Attr) -> Vec<Option<u8>> {
        line_attrs(&M, line, start)
            .0
            .iter()
            .map(|a| a.colour)
            .collect()
    }

    #[test]
    fn colour_codes_colour_what_follows() {
        let d = Attr::default();
        assert_eq!(
            colours(b"A\x01B\x02C", d),
            [None, Some(1), Some(1), Some(2), Some(2)]
        );
        // Colour carries over to the next line.
        let (_, end) = line_attrs(&M, b"A\x01", d);
        assert_eq!(colours(b"X", end), [Some(1)]);
        let lines = vec![b"\x03".to_vec(), b"".to_vec(), b"X".to_vec()];
        assert_eq!(start_of(&M, &lines, 2).colour, Some(3));
    }

    #[test]
    fn reverse_lasts_until_reverse_off_even_over_lines() {
        let (a, end) = line_attrs(&M, b"A\x04B", Attr::default());
        assert_eq!(
            a.iter().map(|a| a.reverse).collect::<Vec<_>>(),
            [false, false, true]
        );
        assert!(end.reverse);
        let (a, end) = line_attrs(&M, b"C\x05D", end);
        // The code itself is never drawn reversed.
        assert_eq!(
            a.iter().map(|a| a.reverse).collect::<Vec<_>>(),
            [true, false, false]
        );
        assert!(!end.reverse);
    }

    #[test]
    fn quotes_keep_codes_inside() {
        let d = Attr::default();
        // PRINT"{1}HI"X: HI in colour 1, X back to the colour before.
        assert_eq!(
            colours(b"P\"\x01HI\"X", d),
            [None, None, Some(1), Some(1), Some(1), None, None]
        );
        // An unclosed quote ends with the line; nothing carries over.
        let (_, end) = line_attrs(&M, b"\"\x01", d);
        assert_eq!(end, d);
    }

    #[test]
    fn state_at_and_after() {
        let lines = vec![b"\x01A\x04B".to_vec(), b"C\"\x02D".to_vec()];
        let at = |line, col| state_at(&M, &lines, Pos { line, col });
        assert_eq!(at(0, 0), Attr::default());
        assert_eq!(
            at(0, 2),
            Attr {
                colour: Some(1),
                reverse: false
            }
        );
        // Inside the string: its colour.
        assert_eq!(at(1, 4).colour, Some(2));
        let text = vec![b"\x03X".to_vec(), b"\x05".to_vec()];
        assert_eq!(
            state_after(&M, &text, at(0, 4)),
            Attr {
                colour: Some(3),
                reverse: false
            }
        );
    }

    #[test]
    fn codes_between_states() {
        let red = Attr {
            colour: Some(2),
            reverse: false,
        };
        let rvs = Attr {
            colour: None,
            reverse: true,
        };
        assert_eq!(codes_between(&M, red, red, None), b"");
        assert_eq!(codes_between(&M, Attr::default(), red, None), [2]);
        // Back to the screen colour (1) with reverse on.
        assert_eq!(codes_between(&M, red, rvs, Some(1)), [1, 4]);
        // No screen colour known: only the reverse changes.
        assert_eq!(codes_between(&M, red, rvs, None), [4]);
    }
}
