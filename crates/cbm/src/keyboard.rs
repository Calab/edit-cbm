//! The Commodore keyboard on a PC keyboard.
//!
//! The codes come from the C64 KERNAL's own keyboard tables (normal,
//! Shift, C= and CTRL, at $EB81-$ECB8), so each key types what the C64's
//! key does.
//!
//! * Typing follows the PC layout. Unshifted letters type $41-$5A and
//!   shifted ones $C1-$DA, as on the C64: capitals and graphics in the
//!   Graphics set, lower case and capitals in Mixed case.
//! * Keys the C64 has but a PC lacks: `` ` `` is ←, `^` is ↑, `\` is £,
//!   `~` is π (Shift+↑). `|` is Shift+£ and `_` is Shift+-, the graphics
//!   on those C64 keys.
//! * The CTRL key (right Ctrl by default) types control codes:
//!   CTRL+1-8 the colours, CTRL+9/0 RVS ON/OFF, CTRL+letter $01-$1A
//!   (CTRL+M is RETURN, as on the C64). With the cursor keys, Home,
//!   Insert, Delete, Enter, Esc and F1-F8 it inserts their codes instead
//!   of acting: CTRL+Down is {DOWN}, CTRL+Shift+Home {CLR}, CTRL+F1 {F1}.
//! * The C= key (left Alt by default) types the graphics on the left of
//!   the C64's keys, and C=+1-8 the other eight colours.

use retro_core::{EditAction, Key, KeyInput, KeyboardRules};

use crate::petscii::CR;

#[derive(Debug, Default, Clone)]
pub struct CbmKeyboard;

/// C= plus A-Z.
const COMMODORE_LETTERS: [u8; 26] = [
    0xB0, 0xBF, 0xBC, 0xAC, 0xB1, 0xBB, 0xA5, 0xB4, 0xA2, 0xB5, 0xA1, 0xB6, 0xA7, 0xAA, 0xB9, 0xAF,
    0xAB, 0xB2, 0xAE, 0xA3, 0xB8, 0xBE, 0xB3, 0xBD, 0xB7, 0xAD,
];

/// C= plus 1-8: orange, brown, light red, dark grey, grey, light green,
/// light blue, light grey.
const COMMODORE_DIGITS: [u8; 8] = [0x81, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B];

/// CTRL plus 1-9 and 0: black, white, red, cyan, purple, green, blue,
/// yellow, RVS ON, RVS OFF.
const CONTROL_DIGITS: [u8; 10] = [0x90, 0x05, 0x1C, 0x9F, 0x9C, 0x1E, 0x1F, 0x9E, 0x12, 0x92];

/// The PETSCII code a printable PC character types, if the C64 has the
/// key.
pub fn printable(c: char) -> Option<u8> {
    Some(match c {
        'a'..='z' => c as u8 - b'a' + 0x41,
        'A'..='Z' => c as u8 - b'A' + 0xC1,
        '`' => 0x5F,
        '^' => 0x5E,
        '\\' => 0x5C,
        '~' => 0xDE,
        '|' => 0xA9,
        '_' => 0xDD,
        ' '..='@' | '[' | ']' => c as u8,
        _ => return None,
    })
}

/// CTRL plus a key (its unshifted character).
pub fn control_code(c: char) -> Option<u8> {
    Some(match c {
        'a'..='z' => c as u8 - b'a' + 1,
        '1'..='9' => CONTROL_DIGITS[usize::from(c as u8 - b'1')],
        '0' => CONTROL_DIGITS[9],
        '`' => 0x06,  // CTRL+←
        '\\' => 0x1C, // CTRL+£
        '[' => 0x1B,  // CTRL+: on the C64, ESC in ASCII
        ']' | ';' => 0x1D,
        '=' => 0x1F,
        _ => return None,
    })
}

/// C= plus a key: its unshifted character, or with Shift the C64 key
/// the PC key shows (Shift+2 is @, Shift+8 is *, Shift+= is +).
pub fn commodore_code(c: char, shift: bool) -> Option<u8> {
    let c = match (c, shift) {
        ('2', true) => '@',
        ('8', true) => '*',
        ('=', true) => '+',
        (c, true) if !c.is_ascii_alphabetic() => return None,
        (c, _) => c,
    };
    Some(match c {
        'a'..='z' => COMMODORE_LETTERS[usize::from(c as u8 - b'a')],
        '1'..='8' => COMMODORE_DIGITS[usize::from(c as u8 - b'1')],
        '-' => 0xDC,
        '+' => 0xA6,
        '@' => 0xA4,
        '*' => 0xDF,
        '\\' => 0xA8, // C=+£
        _ => return None,
    })
}

/// The code CTRL plus an editing key inserts: the cursor keys, CLR/HOME,
/// INST/DEL, Shift+RETURN, RUN/STOP and the function keys.
pub fn control_key_code(key: Key, shift: bool) -> Option<u8> {
    Some(match key {
        Key::Up => 0x91,
        Key::Down => 0x11,
        Key::Left => 0x9D,
        Key::Right => 0x1D,
        Key::Home if shift => 0x93, // CLR
        Key::Home => 0x13,
        Key::Insert => 0x94,
        Key::Delete | Key::Backspace => 0x14,
        Key::Enter => 0x8D,
        Key::Escape if shift => 0x83,
        Key::Escape => 0x03,
        // F1, F3, F5, F7 are $85-$88; F2, F4, F6, F8 $89-$8C.
        Key::Function(n @ 1..=8) => {
            let n = n - 1;
            0x85 + n / 2 + (n % 2) * 4
        }
        _ => return None,
    })
}

impl CbmKeyboard {
    pub fn new() -> Self {
        Self
    }

    /// Translate one key press into an editing action.
    pub fn handle(&mut self, input: KeyInput) -> EditAction {
        let KeyInput { key, mods } = input;
        let insert = |code: Option<u8>| match code {
            Some(CR) => EditAction::NewLine,
            Some(code) => EditAction::Insert(code),
            None => EditAction::None,
        };
        if mods.control && !matches!(key, Key::Control(_)) {
            return insert(control_key_code(key, mods.shift));
        }
        match key {
            Key::Char(c) => insert(printable(c)),
            Key::Control(c) => insert(control_code(c)),
            Key::Commodore(c) => insert(commodore_code(c, mods.shift)),
            Key::Function(_) | Key::Escape => EditAction::None,
            Key::Up => EditAction::MoveUp,
            Key::Down => EditAction::MoveDown,
            Key::Left => EditAction::MoveLeft,
            Key::Right => EditAction::MoveRight,
            Key::Home => EditAction::LineStart,
            Key::End => EditAction::LineEnd,
            Key::PageUp => EditAction::PageUp,
            Key::PageDown => EditAction::PageDown,
            Key::Enter => EditAction::NewLine,
            Key::Backspace if mods.shift => EditAction::DeleteLine,
            Key::Backspace => EditAction::Backspace,
            Key::Delete if mods.shift => EditAction::DeleteLine,
            Key::Delete => EditAction::DeleteForward,
            Key::Insert if mods.shift => EditAction::InsertLine,
            Key::Insert => EditAction::ToggleOverwrite,
            Key::Tab if mods.shift => EditAction::SetTabStop,
            Key::Tab => EditAction::NextTabStop,
        }
    }
}

impl KeyboardRules for CbmKeyboard {
    fn handle(&mut self, input: KeyInput) -> EditAction {
        CbmKeyboard::handle(self, input)
    }

    fn indicators(&self) -> Vec<&'static str> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use retro_core::Key::*;
    use retro_core::Mods;

    fn press(k: Key) -> EditAction {
        CbmKeyboard::new().handle(KeyInput::plain(k))
    }

    fn with(k: Key, control: bool, commodore: bool, shift: bool) -> EditAction {
        CbmKeyboard::new().handle(KeyInput {
            key: k,
            mods: Mods {
                shift,
                control,
                commodore,
            },
        })
    }

    #[test]
    fn letters_follow_the_c64() {
        assert_eq!(press(Char('a')), EditAction::Insert(0x41));
        assert_eq!(press(Char('z')), EditAction::Insert(0x5A));
        assert_eq!(press(Char('A')), EditAction::Insert(0xC1));
        assert_eq!(press(Char('Z')), EditAction::Insert(0xDA));
    }

    #[test]
    fn substitutes_and_punctuation() {
        assert_eq!(press(Char('`')), EditAction::Insert(0x5F));
        assert_eq!(press(Char('^')), EditAction::Insert(0x5E));
        assert_eq!(press(Char('\\')), EditAction::Insert(0x5C));
        assert_eq!(press(Char('~')), EditAction::Insert(0xDE));
        assert_eq!(press(Char('|')), EditAction::Insert(0xA9));
        assert_eq!(press(Char('_')), EditAction::Insert(0xDD));
        assert_eq!(press(Char('"')), EditAction::Insert(b'"'));
        assert_eq!(press(Char('@')), EditAction::Insert(b'@'));
        assert_eq!(press(Char('[')), EditAction::Insert(b'['));
        assert_eq!(press(Char('{')), EditAction::None);
        assert_eq!(press(Char('é')), EditAction::None);
    }

    #[test]
    fn control_codes() {
        let ctrl = |c| with(Control(c), true, false, false);
        assert_eq!(ctrl('1'), EditAction::Insert(0x90)); // BLK
        assert_eq!(ctrl('2'), EditAction::Insert(0x05)); // WHT
        assert_eq!(ctrl('8'), EditAction::Insert(0x9E)); // YEL
        assert_eq!(ctrl('9'), EditAction::Insert(0x12)); // RVS ON
        assert_eq!(ctrl('0'), EditAction::Insert(0x92)); // RVS OFF
        // CTRL ignores Shift, as on the C64.
        assert_eq!(
            with(Control('9'), true, false, true),
            EditAction::Insert(0x12)
        );
        assert_eq!(ctrl('a'), EditAction::Insert(0x01));
        assert_eq!(ctrl('n'), EditAction::Insert(0x0E)); // to Mixed case
        assert_eq!(ctrl('m'), EditAction::NewLine); // RETURN, never a CR in a line
        assert_eq!(ctrl('\\'), EditAction::Insert(0x1C));
        assert_eq!(ctrl(','), EditAction::None);
    }

    #[test]
    fn control_editing_keys_insert_their_codes() {
        let ctrl = |k, shift| with(k, true, false, shift);
        assert_eq!(ctrl(Up, false), EditAction::Insert(0x91));
        assert_eq!(ctrl(Down, false), EditAction::Insert(0x11));
        assert_eq!(ctrl(Left, false), EditAction::Insert(0x9D));
        assert_eq!(ctrl(Right, false), EditAction::Insert(0x1D));
        assert_eq!(ctrl(Home, false), EditAction::Insert(0x13));
        assert_eq!(ctrl(Home, true), EditAction::Insert(0x93));
        assert_eq!(ctrl(Insert, false), EditAction::Insert(0x94));
        assert_eq!(ctrl(Delete, false), EditAction::Insert(0x14));
        assert_eq!(ctrl(Escape, false), EditAction::Insert(0x03));
        let f: Vec<_> = (1..=8).map(|n| ctrl(Function(n), false)).collect();
        let want: Vec<_> = [0x85, 0x89, 0x86, 0x8A, 0x87, 0x8B, 0x88, 0x8C]
            .map(EditAction::Insert)
            .into();
        assert_eq!(f, want);
        // Without CTRL they edit.
        assert_eq!(press(Down), EditAction::MoveDown);
        assert_eq!(press(Function(1)), EditAction::None);
    }

    #[test]
    fn commodore_graphics_and_colours() {
        let cbm = |c, shift| with(Commodore(c), false, true, shift);
        assert_eq!(cbm('a', false), EditAction::Insert(0xB0));
        assert_eq!(cbm('z', false), EditAction::Insert(0xAD));
        assert_eq!(cbm('1', false), EditAction::Insert(0x81)); // orange
        assert_eq!(cbm('8', false), EditAction::Insert(0x9B)); // light grey
        assert_eq!(cbm('-', false), EditAction::Insert(0xDC));
        assert_eq!(cbm('=', true), EditAction::Insert(0xA6)); // C=+ +
        assert_eq!(cbm('2', true), EditAction::Insert(0xA4)); // C=+ @
        assert_eq!(cbm('8', true), EditAction::Insert(0xDF)); // C=+ *
        assert_eq!(cbm('9', false), EditAction::None);
    }

    #[test]
    fn editing_keys() {
        assert_eq!(press(Up), EditAction::MoveUp);
        assert_eq!(press(Enter), EditAction::NewLine);
        assert_eq!(press(Backspace), EditAction::Backspace);
        assert_eq!(press(Insert), EditAction::ToggleOverwrite);
    }
}
