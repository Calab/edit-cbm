//! The C64's 16 colours (the VIC-II palette, Colodore values).
//!
//! For the Preferences colour picker these are 16 "hues" with a single
//! luminance each: hue = the colour number the C64 uses (`POKE 53281,6`),
//! luminance always 0.

/// Number of colours, and of luminance levels per colour.
pub const HUES: u8 = 16;
pub const LUMS: u8 = 1;

/// The colours in C64 order: name and RGB.
pub const COLOURS: [(&str, (u8, u8, u8)); 16] = [
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

pub const BLUE: u8 = 6;
pub const LIGHT_BLUE: u8 = 14;

/// The screen at power-on: light blue text on blue, light blue border.
pub const DEFAULT_TEXT: (u8, u8) = (LIGHT_BLUE, 0);
pub const DEFAULT_BACKGROUND: (u8, u8) = (BLUE, 0);
pub const BORDER: (u8, u8) = (LIGHT_BLUE, 0);

/// RGB for a colour number (0-15); the luminance is ignored.
pub fn rgb(colour: u8, _lum: u8) -> (u8, u8, u8) {
    COLOURS[usize::from(colour.min(HUES - 1))].1
}

/// The colour's name and number, e.g. "Blue (6)".
pub fn label(colour: u8, _lum: u8) -> String {
    let c = colour.min(HUES - 1);
    format!("{} ({c})", COLOURS[usize::from(c)].0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_on_colours() {
        assert_eq!(rgb(DEFAULT_BACKGROUND.0, 0), (0x2E, 0x2C, 0x9B));
        assert_eq!(rgb(DEFAULT_TEXT.0, 0), (0x70, 0x6D, 0xEB));
        assert_eq!(label(6, 0), "Blue (6)");
    }
}
