//! What differs between the Commodore models the editor shows.

use crate::palette;

pub struct Model {
    /// Settings key, e.g. "c64".
    pub id: &'static str,
    /// Display name, e.g. "C64".
    pub name: &'static str,
    /// Native screen width in characters.
    pub columns: usize,
    /// The screen editor's logical line, for the line-length warning.
    pub logical_line: usize,
    /// How many colour codes the keyboard has (the first `n` of
    /// [`crate::petscii::COLOUR_CODES`]).
    pub colour_codes: usize,
    /// The development character ROM's name in `fonts-dev`.
    pub rom_file: &'static str,
    /// The colours, and functions for the colour picker.
    pub colours: &'static [palette::Colour],
    pub rgb: fn(u8, u8) -> (u8, u8, u8),
    pub label: fn(u8, u8) -> String,
    /// Colour numbers at power-on.
    pub text: u8,
    pub background: u8,
    pub border: u8,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

/// The C64: 40 columns, 16 colours; light blue on blue at power-on.
pub static C64: Model = Model {
    id: "c64",
    name: "C64",
    columns: 40,
    logical_line: 80,
    colour_codes: 16,
    rom_file: "c64-chargen.bin",
    colours: &palette::C64,
    rgb: palette::c64_rgb,
    label: palette::c64_label,
    text: 14,
    background: 6,
    border: 14,
};

/// The VIC-20: 22 columns, 8 character colours; blue on white with a
/// cyan border at power-on. Its logical line is 4 rows of 22.
pub static VIC20: Model = Model {
    id: "vic20",
    name: "VIC-20",
    columns: 22,
    logical_line: 88,
    colour_codes: 8,
    rom_file: "vic20-chargen.bin",
    colours: &palette::VIC20,
    rgb: palette::vic20_rgb,
    label: palette::vic20_label,
    text: 6,
    background: 1,
    border: 3,
};

/// Every model, in menu order.
pub static MODELS: [&Model; 2] = [&C64, &VIC20];
