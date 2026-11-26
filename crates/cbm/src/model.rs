//! What differs between the Commodore models the editor shows.

use crate::palette;

/// Which keyboard a model has (see [`crate::keyboard`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keys {
    C64,
    /// As the C64's, but CTRL types only colours, RVS and CTRL+←, and C=
    /// with 1-8 the shifted digit.
    Vic20,
    /// The PET's full-size graphics keyboard: no CTRL or C= key; Shift
    /// with any key types a graphic; RVS is a key of its own.
    PetGraphics,
}

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
    pub keys: Keys,
    /// $5C is £ (C64, VIC-20); on the PET it's a backslash.
    pub pound: bool,
    /// An 80-column screen: characters half as wide (all 8 columns shown).
    pub narrow: bool,
    /// The character ROM holds each set followed by its reversed copy
    /// (C64, VIC-20: 4096 bytes); the PET's holds two plain sets (2048).
    pub rom_reversed: bool,
    /// No colour choice: the screen is always `text` on `background`.
    pub fixed_colours: bool,
    /// The development character ROM's name in `fonts-dev`.
    pub rom_file: &'static str,
    /// The colours, and functions for the colour picker.
    pub colours: &'static [palette::Colour],
    pub rgb: fn(u8, u8) -> (u8, u8, u8),
    pub label: fn(u8, u8) -> String,
    /// How many colours characters and the border can be (the first n).
    pub character_colours: u8,
    pub border_colours: u8,
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
    keys: Keys::C64,
    pound: true,
    narrow: false,
    rom_reversed: true,
    fixed_colours: false,
    rom_file: "c64-chargen.bin",
    colours: &palette::C64,
    rgb: palette::c64_rgb,
    label: palette::c64_label,
    character_colours: 16,
    border_colours: 16,
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
    keys: Keys::Vic20,
    pound: true,
    narrow: false,
    rom_reversed: true,
    fixed_colours: false,
    rom_file: "vic20-chargen.bin",
    colours: &palette::VIC20,
    rgb: palette::vic20_rgb,
    label: palette::vic20_label,
    character_colours: 8,
    border_colours: 8,
    text: 6,
    background: 1,
    border: 3,
};

/// The PET (2001, 3000, 4000): 40 columns, green on black, the graphics
/// keyboard. Its logical line is 2 rows of 40.
pub static PET: Model = PET_40;

const PET_40: Model = Model {
    id: "pet",
    name: "PET",
    columns: 40,
    logical_line: 80,
    colour_codes: 0,
    keys: Keys::PetGraphics,
    pound: false,
    narrow: false,
    rom_reversed: false,
    fixed_colours: true,
    rom_file: "pet-chargen.bin",
    colours: &palette::PET,
    rgb: palette::pet_rgb,
    label: palette::pet_label,
    character_colours: 2,
    border_colours: 2,
    text: 1,
    background: 0,
    border: 0,
};

/// The PET 8032: as the PET, with 80 columns (narrow characters); its
/// logical line is one row of 80.
pub static PET8032: Model = Model {
    id: "pet8032",
    name: "PET 8032",
    columns: 80,
    narrow: true,
    ..PET_40
};

/// Every model, in menu order.
pub static MODELS: [&Model; 4] = [&C64, &VIC20, &PET, &PET8032];
