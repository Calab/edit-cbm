//! The colours of each model, as VICE's Colodore palettes give them.
//!
//! For the Preferences colour picker these are "hues" with a single
//! luminance each: hue = the colour number the machine uses (`POKE
//! 53281,6` on the C64), luminance always 0. Each model has its own `rgb`
//! and `label` functions, as the picker takes plain functions.

/// A colour's name and RGB.
pub type Colour = (&'static str, (u8, u8, u8));

/// The C64's 16 colours (the VIC-II).
pub const C64: [Colour; 16] = [
    ("Black", (0x00, 0x00, 0x00)),
    ("White", (0xFF, 0xFF, 0xFF)),
    ("Red", (0x81, 0x33, 0x38)),
    ("Cyan", (0x75, 0xCE, 0xC8)),
    ("Purple", (0x8E, 0x3C, 0x97)),
    ("Green", (0x56, 0xAC, 0x4D)),
    ("Blue", (0x2E, 0x2C, 0x9B)),
    ("Yellow", (0xED, 0xF1, 0x71)),
    ("Orange", (0x8E, 0x50, 0x29)),
    ("Brown", (0x55, 0x38, 0x00)),
    ("Light red", (0xC4, 0x6C, 0x71)),
    ("Dark grey", (0x4A, 0x4A, 0x4A)),
    ("Grey", (0x7B, 0x7B, 0x7B)),
    ("Light green", (0xA9, 0xFF, 0x9F)),
    ("Light blue", (0x70, 0x6D, 0xEB)),
    ("Light grey", (0xB2, 0xB2, 0xB2)),
];

/// The VIC-20's 16 colours (the VIC chip; VICE's `colodore_vic.vpl`).
/// Characters and the border use the first 8; the screen any of them.
pub const VIC20: [Colour; 16] = [
    ("Black", (0x00, 0x00, 0x00)),
    ("White", (0xFF, 0xFF, 0xFF)),
    ("Red", (0x6D, 0x23, 0x27)),
    ("Cyan", (0xA0, 0xFE, 0xF8)),
    ("Purple", (0x8E, 0x3C, 0x97)),
    ("Green", (0x7E, 0xDA, 0x75)),
    ("Blue", (0x25, 0x23, 0x90)),
    ("Yellow", (0xFF, 0xFF, 0x86)),
    ("Orange", (0xA4, 0x64, 0x3B)),
    ("Light orange", (0xFF, 0xC8, 0xA1)),
    ("Pink", (0xF2, 0xA7, 0xAB)),
    ("Light cyan", (0xDB, 0xFF, 0xFF)),
    ("Light purple", (0xFF, 0xB4, 0xFF)),
    ("Light green", (0xD7, 0xFF, 0xCE)),
    ("Light blue", (0x9D, 0x9A, 0xFF)),
    ("Light yellow", (0xFF, 0xFF, 0xC9)),
];

/// The PET's screen: green phosphor on black (VICE's `green.vpl`).
pub const PET: [Colour; 2] = [("Black", (0x00, 0x00, 0x00)), ("Green", (0x41, 0xFF, 0x00))];

fn rgb(table: &[Colour], colour: u8) -> (u8, u8, u8) {
    table[usize::from(colour).min(table.len() - 1)].1
}

/// The colour's name and number, e.g. "Blue (6)".
fn label(table: &[Colour], colour: u8) -> String {
    let c = usize::from(colour).min(table.len() - 1);
    format!("{} ({c})", table[c].0)
}

pub fn c64_rgb(colour: u8, _lum: u8) -> (u8, u8, u8) {
    rgb(&C64, colour)
}

pub fn c64_label(colour: u8, _lum: u8) -> String {
    label(&C64, colour)
}

pub fn vic20_rgb(colour: u8, _lum: u8) -> (u8, u8, u8) {
    rgb(&VIC20, colour)
}

pub fn vic20_label(colour: u8, _lum: u8) -> String {
    label(&VIC20, colour)
}

pub fn pet_rgb(colour: u8, _lum: u8) -> (u8, u8, u8) {
    rgb(&PET, colour)
}

pub fn pet_label(colour: u8, _lum: u8) -> String {
    label(&PET, colour)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours() {
        assert_eq!(c64_rgb(6, 0), (0x2E, 0x2C, 0x9B));
        assert_eq!(c64_label(6, 0), "Blue (6)");
        assert_eq!(vic20_rgb(3, 0), (0xA0, 0xFE, 0xF8));
        assert_eq!(vic20_label(15, 0), "Light yellow (15)");
    }
}
