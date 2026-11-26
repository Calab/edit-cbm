//! Character sets: 8x8 glyphs in screen-code order, as in the character
//! ROM.
//!
//! A character ROM holds each set as 128 characters and then the same 128
//! reversed. Only the first 128 are kept here; reversed glyphs are drawn
//! by inverting them, as the ROM's own are.

/// One character set: 128 glyphs of 8 bytes.
pub const SET_SIZE: usize = 128 * 8;

/// Placeholder glyphs, used until real fonts are built in: a hollow box
/// for every character but the blank ones (space and Shift+Space).
pub fn placeholder() -> Vec<u8> {
    const BOX: [u8; 8] = [0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7E, 0x00];
    (0..128u8)
        .flat_map(|s| if s == 0x20 || s == 0x60 { [0; 8] } else { BOX })
        .collect()
}

/// The glyphs of character set `charset` (0 = Graphics, 1 = Mixed case)
/// from a font file: a whole C64 character ROM (4096 bytes, both sets),
/// one set from it (2048 bytes, normal then reversed), or just its normal
/// characters (1024 bytes).
pub fn set_from_rom(file: &[u8], charset: usize) -> Result<Vec<u8>, String> {
    let start = match file.len() {
        4096 => charset.min(1) * 2048,
        2048 | 1024 => 0,
        n => {
            return Err(format!(
                "not a character ROM ({n} bytes; expected 1024, 2048 or 4096)"
            ));
        }
    };
    Ok(file[start..start + SET_SIZE].to_vec())
}

/// The glyph for screen code `s` (bit 7 = reversed) in `set`.
pub fn glyph(set: &[u8], s: u8) -> [u8; 8] {
    let i = usize::from(s & 0x7F) * 8;
    let mut g: [u8; 8] = set[i..i + 8].try_into().unwrap();
    if s & 0x80 != 0 {
        for row in &mut g {
            *row = !*row;
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rom_sizes() {
        let mut rom = vec![0u8; 4096];
        rom[2048] = 0xAA;
        assert_eq!(set_from_rom(&rom, 0).unwrap()[0], 0);
        assert_eq!(set_from_rom(&rom, 1).unwrap()[0], 0xAA);
        assert_eq!(set_from_rom(&rom[2048..], 0).unwrap()[0], 0xAA);
        assert_eq!(set_from_rom(&rom[..1024], 1).unwrap().len(), SET_SIZE);
        assert!(set_from_rom(&rom[..1000], 0).is_err());
    }

    #[test]
    fn reversed_is_inverted() {
        let set = placeholder();
        assert_eq!(glyph(&set, 0x20), [0; 8]);
        assert_eq!(glyph(&set, 0xA0), [0xFF; 8]);
        let a = glyph(&set, 0x01);
        let ra = glyph(&set, 0x81);
        assert!(a.iter().zip(ra).all(|(n, r)| *n == !r));
    }
}
