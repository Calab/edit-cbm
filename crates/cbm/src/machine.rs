//! A Commodore computer as the editor sees it: PETSCII, its character
//! sets, its screen width and colour codes. The differences between the
//! models are in [`crate::model`].

use retro_core::{CodeEffect, Decoded, FileFormat, Machine};

use crate::model::Model;
use crate::{font, petscii};

/// Index of the Mixed case set in [`Machine::charsets`].
pub const MIXED_CASE: usize = 1;

#[derive(Debug, Clone)]
pub struct Cbm {
    model: &'static Model,
    /// 0 = Graphics (upper case and graphics), 1 = Mixed case.
    charset: usize,
    /// The built-in character sets (placeholders, or a development ROM
    /// given by [`Cbm::set_builtin_rom`]).
    builtin: [Vec<u8>; 2],
    /// Fonts loaded from files, in place of the built-in sets.
    fonts: [Option<Vec<u8>>; 2],
}

impl Cbm {
    pub fn new(model: &'static Model) -> Self {
        Self {
            model,
            charset: 0,
            builtin: [font::placeholder(), font::placeholder()],
            fonts: [None, None],
        }
    }

    pub fn model(&self) -> &'static Model {
        self.model
    }

    /// Use a character ROM (Graphics then Mixed case: 4096 bytes, or the
    /// PET's 2048) as the built-in sets. For development only: the ROM is
    /// Commodore's, so it is read from a file when the program starts,
    /// never built in.
    pub fn set_builtin_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        let size = if self.model.rom_reversed { 4096 } else { 2048 };
        if rom.len() != size {
            return Err(format!(
                "{} bytes; a {} character ROM is {size}",
                rom.len(),
                self.model.name
            ));
        }
        for (i, set) in self.builtin.iter_mut().enumerate() {
            *set = font::set_from_rom(rom, i, self.model.rom_reversed)?;
        }
        Ok(())
    }

    fn mixed(&self) -> bool {
        self.charset == MIXED_CASE
    }

    /// The colour codes this model has, in colour-number order.
    fn colour_codes(&self) -> &'static [u8] {
        &petscii::COLOUR_CODES[..self.model.colour_codes]
    }
}

/// The clipboard character for a code: ASCII where PETSCII has the same
/// character, £ ↑ ← for $5C $5E $5F, and U+E000 + code for the rest. In
/// Mixed case, $41-$5A are lower case and $C1-$DA capitals.
pub fn char_for_code(code: u8, mixed: bool, pound: bool) -> char {
    match code {
        0x20..=0x40 | 0x5B | 0x5D => code as char,
        0x41..=0x5A if mixed => (code + 0x20) as char,
        0x41..=0x5A => code as char,
        0xC1..=0xDA if mixed => (code - 0x80) as char,
        0x5C if pound => '£',
        0x5C => '\\',
        0x5E => '↑',
        0x5F => '←',
        _ => char::from_u32(0xE000 + u32::from(code)).unwrap(),
    }
}

/// The code for a clipboard character, if the C64 has it. Letters of
/// either case type the set's own letters: in the Graphics set, which has
/// no lower case, `a` is read as `A`.
pub fn code_for_char(c: char, mixed: bool, pound: bool) -> Option<u8> {
    Some(match c {
        ' '..='@' | '[' | ']' => c as u8,
        'a'..='z' => c as u8 - 0x20,
        'A'..='Z' if mixed => c as u8 + 0x80,
        'A'..='Z' => c as u8,
        '£' if pound => 0x5C,
        '\\' if !pound => 0x5C,
        '↑' => 0x5E,
        '←' => 0x5F,
        _ => match c as u32 {
            n @ 0xE000..=0xE0FF => (n - 0xE000) as u8,
            _ => return None,
        },
    })
}

/// The C128's 80-column codes: CHR$(2) and CHR$(130) underline on and
/// off, CHR$(15) and CHR$(143) flash on and off, CHR$(14) and CHR$(142)
/// the Mixed case and Graphics sets for what follows.
const UNDERLINE_ON: u8 = 0x02;
const UNDERLINE_OFF: u8 = 0x82;
const FLASH_ON: u8 = 0x0F;
const FLASH_OFF: u8 = 0x8F;

fn vdc_effect(code: u8) -> Option<CodeEffect> {
    Some(match code {
        UNDERLINE_ON => CodeEffect::UnderlineOn,
        UNDERLINE_OFF => CodeEffect::UnderlineOff,
        FLASH_ON => CodeEffect::FlashOn,
        FLASH_OFF => CodeEffect::FlashOff,
        petscii::TO_MIXED_CASE => CodeEffect::Charset(MIXED_CASE),
        petscii::TO_GRAPHICS => CodeEffect::Charset(0),
        _ => return None,
    })
}

impl Machine for Cbm {
    fn name(&self) -> &'static str {
        self.model.name
    }

    fn glyph(&self, code: u8) -> [u8; 8] {
        self.charset_glyph(self.charset, code)
    }

    fn charset_glyph(&self, charset: usize, code: u8) -> [u8; 8] {
        let i = charset.min(MIXED_CASE);
        let set = self.fonts[i].as_ref().unwrap_or(&self.builtin[i]);
        font::glyph(set, petscii::screen_code(code))
    }

    fn load_font(&mut self, charset: usize, file: Option<&[u8]>) -> Result<(), String> {
        let slot = self.fonts.get_mut(charset).ok_or("no such character set")?;
        let reversed = self.model.rom_reversed;
        *slot = file
            .map(|f| font::set_from_rom(f, charset, reversed))
            .transpose()?;
        Ok(())
    }

    fn charsets(&self) -> &'static [&'static str] {
        &["Graphics", "Mixed case"]
    }

    fn charset(&self) -> usize {
        self.charset
    }

    fn set_charset(&mut self, index: usize) {
        self.charset = index.min(MIXED_CASE);
    }

    fn space(&self) -> u8 {
        b' '
    }

    fn narrow_pixels(&self) -> bool {
        self.model.narrow
    }

    fn screen_columns(&self) -> usize {
        self.model.columns
    }

    /// The colour codes the model has (16 on the C64, 8 on the VIC-20)
    /// and RVS ON/OFF. Other control codes ({CLR}, cursor codes...) are
    /// only shown.
    fn code_effect(&self, code: u8) -> Option<CodeEffect> {
        if self.model.vdc
            && let Some(e) = vdc_effect(code)
        {
            return Some(e);
        }
        match code {
            petscii::RVS_ON => Some(CodeEffect::ReverseOn),
            petscii::RVS_OFF => Some(CodeEffect::ReverseOff),
            _ => self
                .colour_codes()
                .iter()
                .position(|&c| c == code)
                .map(|n| CodeEffect::Colour(n as u8)),
        }
    }

    fn effect_code(&self, effect: CodeEffect) -> Option<u8> {
        match effect {
            CodeEffect::Colour(n) => self.colour_codes().get(usize::from(n)).copied(),
            CodeEffect::ReverseOn => Some(petscii::RVS_ON),
            CodeEffect::ReverseOff => Some(petscii::RVS_OFF),
            _ if !self.model.vdc => None,
            CodeEffect::UnderlineOn => Some(UNDERLINE_ON),
            CodeEffect::UnderlineOff => Some(UNDERLINE_OFF),
            CodeEffect::FlashOn => Some(FLASH_ON),
            CodeEffect::FlashOff => Some(FLASH_OFF),
            CodeEffect::Charset(c) => self.charset_code(c),
        }
    }

    fn is_control(&self, code: u8) -> bool {
        petscii::is_control(code)
    }

    /// $8E switches to Graphics (upper case and graphics), $0E to Mixed
    /// case: CHR$(142) and CHR$(14), CTRL+N on the keyboard.
    fn charset_code(&self, index: usize) -> Option<u8> {
        Some(if index == MIXED_CASE {
            petscii::TO_MIXED_CASE
        } else {
            petscii::TO_GRAPHICS
        })
    }

    fn quote_code(&self) -> Option<u8> {
        Some(b'"')
    }

    fn left_margin(&self) -> usize {
        0
    }

    /// Letters as unshifted ones ($41-$5A), so a name typed in either case
    /// reads as it would on the C64's screen at power-on.
    fn ascii_code(&self, c: char) -> u8 {
        match c {
            'a'..='z' => c as u8 - 0x20,
            ' '..='@' | 'A'..='Z' | '[' | ']' => c as u8,
            _ => b'?',
        }
    }

    fn char_for_code(&self, code: u8) -> char {
        char_for_code(code, self.mixed(), self.model.pound)
    }

    fn code_for_char(&self, c: char) -> Option<u8> {
        code_for_char(c, self.mixed(), self.model.pound)
    }

    /// As on the clipboard, with `?` (counted) for characters outside
    /// standard Unicode.
    fn export_standard(&self, line: &[u8]) -> (String, usize) {
        let mut lost = 0;
        let s = line
            .iter()
            .map(|&code| match self.char_for_code(code) {
                '\u{E000}'..='\u{E0FF}' => {
                    lost += 1;
                    '?'
                }
                c => c,
            })
            .collect();
        (s, lost)
    }

    /// As on the clipboard, with `{$xx}` for the rest.
    fn export_ascii(&self, line: &[u8]) -> String {
        line.iter()
            .map(|&code| match self.char_for_code(code) {
                c @ ' '..='~' => c.to_string(),
                _ => format!("{{${code:02X}}}"),
            })
            .collect()
    }

    fn looks_like_pc_text(&self, bytes: &[u8]) -> bool {
        petscii::looks_like_pc_text(bytes)
    }

    fn import_text(&self, bytes: &[u8]) -> Decoded {
        let text = String::from_utf8_lossy(bytes);
        let lines = text
            .lines()
            .map(|l| {
                l.chars()
                    .map(|c| self.code_for_char(c).unwrap_or(b'?'))
                    .collect()
            })
            .collect();
        Decoded {
            lines,
            final_eol: true,
            converted_from: Some("PC text".into()),
            meta: Default::default(),
        }
    }

    fn formats(&self) -> &'static [&'static dyn FileFormat] {
        self.model.formats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;
    use retro_core::clipboard::{from_text, to_text};

    #[test]
    fn clipboard_round_trips_every_code_in_both_sets() {
        let mut m = Cbm::new(&model::C64);
        let all: Vec<u8> = (0..=255).filter(|&c| c != petscii::CR).collect();
        for set in [0, MIXED_CASE] {
            m.set_charset(set);
            let text = to_text(std::slice::from_ref(&all), &m);
            assert_eq!(from_text(&text, &m), (vec![all.clone()], 0), "set {set}");
        }
    }

    #[test]
    fn clipboard_letters() {
        let mut m = Cbm::new(&model::C64);
        let line = vec![0x48, 0x49, 0xC1, 0x5C];
        assert_eq!(to_text(std::slice::from_ref(&line), &m), "HI\u{E0C1}£");
        assert_eq!(from_text("hi", &m).0, vec![vec![0x48, 0x49]]);
        m.set_charset(MIXED_CASE);
        assert_eq!(to_text(std::slice::from_ref(&line), &m), "hiA£");
        assert_eq!(from_text("Hi", &m).0, vec![vec![0xC8, 0x49]]);
    }

    #[test]
    fn glyphs_come_from_the_rom() {
        let mut rom = vec![0u8; 4096];
        rom[8..16].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]); // Graphics A
        rom[2048 + 8] = 0x99; // Mixed case a
        let mut m = Cbm::new(&model::C64);
        m.set_builtin_rom(&rom).unwrap();
        assert_eq!(m.glyph(b'A'), [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(m.glyph(0x93), [0xFF; 8]); // CLR: reversed blank heart
        m.set_charset(MIXED_CASE);
        assert_eq!(m.glyph(b'A')[0], 0x99);
        // A loaded font replaces the built-in set, until it's taken away.
        m.load_font(MIXED_CASE, Some(&[0x11; 1024])).unwrap();
        assert_eq!(m.glyph(b'A')[0], 0x11);
        m.load_font(MIXED_CASE, None).unwrap();
        assert_eq!(m.glyph(b'A')[0], 0x99);
        assert!(m.set_builtin_rom(&rom[..2048]).is_err());
    }

    #[test]
    fn colour_codes() {
        let m = Cbm::new(&model::C64);
        assert_eq!(m.code_effect(0x90), Some(CodeEffect::Colour(0)));
        assert_eq!(m.code_effect(0x05), Some(CodeEffect::Colour(1)));
        assert_eq!(m.code_effect(0x9B), Some(CodeEffect::Colour(15)));
        assert_eq!(m.code_effect(0x12), Some(CodeEffect::ReverseOn));
        assert_eq!(m.code_effect(0x93), None); // CLR is only shown
        // The colour codes are the ones the keyboard types.
        use crate::keyboard::{commodore_code, control_code};
        let typed: Vec<u8> = ('1'..='8')
            .filter_map(|c| control_code(&model::C64, c))
            .chain(('1'..='8').filter_map(|c| commodore_code(&model::C64, c, false)))
            .collect();
        assert_eq!(typed, petscii::COLOUR_CODES);
    }

    #[test]
    fn editing_with_codes_hidden() {
        use retro_core::{Document, EditAction::*, Editor, HideRule, Pos, WrapMode};
        let m = Cbm::new(&model::C64);
        // A{RED}B"{RED}"
        let doc = Document::from_lines(vec![b"A\x1cB\"\x1c\"".to_vec()], true);
        let mut ed = Editor::new(doc, WrapMode::Native, m.default_tab_stops());
        let mut layout = ed.layout(&m, 40, 24);
        layout.hide = Some(HideRule::for_machine(&m));
        // Only the code outside quotes is hidden.
        assert_eq!(layout.visible_len(ed.doc.line(0)), 5);
        // The cursor right after A sits on B, past the hidden {RED}, so
        // typing there is red.
        ed.doc.set_cursor(Pos { line: 0, col: 1 });
        ed.apply(Insert(b'X'), &layout, &m, false);
        assert_eq!(ed.doc.line(0), b"A\x1cXB\"\x1c\"");
        assert_eq!(ed.doc.cursor().col, 3);
        // Left steps over the hidden code to A.
        ed.apply(MoveLeft, &layout, &m, false);
        ed.apply(MoveLeft, &layout, &m, false);
        assert_eq!(ed.doc.cursor().col, 0);
        // Backspace after X deletes X, not the code.
        ed.doc.set_cursor(Pos { line: 0, col: 3 });
        ed.apply(Backspace, &layout, &m, false);
        assert_eq!(ed.doc.line(0), b"A\x1cB\"\x1c\"");
        // Backspace after A, with the hidden code between: deletes A.
        ed.doc.set_cursor(Pos { line: 0, col: 2 });
        ed.apply(Backspace, &layout, &m, false);
        assert_eq!(ed.doc.line(0), b"\x1cB\"\x1c\"");
        assert_eq!(ed.doc.cursor().col, 1);
    }

    #[test]
    fn tab_stops_ignore_codes_outside_quotes() {
        use retro_core::{Document, EditAction::*, Editor, Pos, WrapMode};
        let m = Cbm::new(&model::C64);
        // {RED}AB: the code doesn't count, so the first stop (column 8)
        // is 8 characters in, one byte further on.
        let doc = Document::from_lines(vec![b"\x1cAB".to_vec(), b"\"\x1c\"".to_vec()], true);
        let mut ed = Editor::new(doc, WrapMode::Native, m.default_tab_stops());
        let layout = ed.layout(&m, 40, 24);
        ed.doc.set_cursor(Pos { line: 0, col: 1 });
        ed.apply(NextTabStop, &layout, &m, false);
        assert_eq!(ed.doc.cursor().col, 9);
        assert_eq!(ed.doc.line(0), b"\x1cAB      ");
        // In quotes the code counts.
        ed.doc.set_cursor(Pos { line: 1, col: 0 });
        ed.apply(NextTabStop, &layout, &m, false);
        assert_eq!(ed.doc.cursor().col, 8);
    }

    #[test]
    fn the_vic20_has_22_columns_and_8_colour_codes() {
        let m = Cbm::new(&model::VIC20);
        assert_eq!((m.name(), m.screen_columns()), ("VIC-20", 22));
        assert_eq!(m.code_effect(0x9E), Some(CodeEffect::Colour(7))); // yellow
        assert_eq!(m.code_effect(0x81), None); // no orange code
        assert_eq!(m.effect_code(CodeEffect::Colour(8)), None);
    }

    #[test]
    fn case_switch_codes_and_quotes() {
        use retro_core::attrs::in_quotes;
        let m = Cbm::new(&model::C64);
        assert_eq!(m.charset_code(MIXED_CASE), Some(0x0E));
        assert_eq!(m.charset_code(0), Some(0x8E));
        let line = b"10 PRINT\"AB\"C";
        assert!(!in_quotes(&m, line, 8));
        assert!(in_quotes(&m, line, 9));
        assert!(in_quotes(&m, line, 11));
        assert!(!in_quotes(&m, line, 12));
    }

    #[test]
    fn the_pet() {
        use retro_core::clipboard::{from_text, to_text};
        let mut m = Cbm::new(&model::PET);
        // $5C is a backslash, both ways, in either set.
        for set in [0, MIXED_CASE] {
            m.set_charset(set);
            assert_eq!(to_text(&[vec![0x5C]], &m), "\\");
            assert_eq!(from_text("\\", &m).0, vec![vec![0x5C]]);
        }
        // No colours: only RVS acts.
        assert_eq!(m.code_effect(0x1C), None);
        assert_eq!(m.code_effect(0x12), Some(CodeEffect::ReverseOn));
        // Its ROM: two plain sets of 1024.
        let mut rom = vec![0u8; 2048];
        rom[1024 + 8] = 0x77;
        m.set_builtin_rom(&rom).unwrap();
        assert_eq!(m.charset_glyph(MIXED_CASE, b'A')[0], 0x77);
        assert!(m.set_builtin_rom(&[0; 4096]).is_err());
        assert!(!m.narrow_pixels());
        assert!(Cbm::new(&model::PET8032).narrow_pixels());
        assert_eq!(Cbm::new(&model::PET8032).screen_columns(), 80);
    }

    #[test]
    fn the_c128_80_column_codes() {
        let m = Cbm::new(&model::C128_80);
        assert_eq!(m.code_effect(0x02), Some(CodeEffect::UnderlineOn));
        assert_eq!(m.code_effect(0x8F), Some(CodeEffect::FlashOff));
        assert_eq!(m.code_effect(0x0E), Some(CodeEffect::Charset(MIXED_CASE)));
        assert_eq!(m.code_effect(0x1C), Some(CodeEffect::Colour(2)));
        assert_eq!(m.effect_code(CodeEffect::Charset(0)), Some(0x8E));
        assert!(m.narrow_pixels());
        // Not on the 40-column screen.
        let m40 = Cbm::new(&model::C128_40);
        assert_eq!(m40.code_effect(0x02), None);
        assert_eq!(m40.code_effect(0x0E), None);
        assert_eq!(m40.effect_code(CodeEffect::FlashOn), None);
    }

    #[test]
    fn export_forms() {
        let m = Cbm::new(&model::C64);
        assert_eq!(m.export_standard(&[0x41, 0x93, 0x5F]), ("A?←".into(), 1));
        assert_eq!(m.export_ascii(&[0x41, 0x93, 0x5F]), "A{$93}{$5F}");
    }
}
