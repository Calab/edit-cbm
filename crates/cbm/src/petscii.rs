//! PETSCII: the Commodore character codes, and PETSCII text files.
//!
//! A PETSCII text file (a SEQ file written by `PRINT#`, `OPEN ...,"S,W"`
//! etc.) is a run of PETSCII codes with a carriage return ($0D) after each
//! line.

use retro_core::Decoded;

/// End of line: carriage return.
pub const CR: u8 = 0x0D;
/// PC line feed. Unused on the Commodore machines, so a file holding one
/// is probably PC text.
pub const LF: u8 = 0x0A;

/// The colour codes, in the order of the C64's colour numbers (black 0
/// to light grey 15): CTRL+1-8, then C=+1-8.
pub const COLOUR_CODES: [u8; 16] = [
    0x90, 0x05, 0x1C, 0x9F, 0x9C, 0x1E, 0x1F, 0x9E, 0x81, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B,
];
pub const RVS_ON: u8 = 0x12;
/// The C128's ESC, TAB and HELP keys' codes.
pub const ESC: u8 = 0x1B;
pub const TAB: u8 = 0x09;
pub const HELP: u8 = 0x84;
/// Switch the screen to the Mixed case set (CTRL+N), or back to Graphics.
pub const TO_MIXED_CASE: u8 = 0x0E;
pub const TO_GRAPHICS: u8 = 0x8E;
pub const RVS_OFF: u8 = 0x92;

/// Whether a code is a control code rather than a character.
pub fn is_control(p: u8) -> bool {
    matches!(p, 0x00..=0x1F | 0x80..=0x9F)
}

/// The screen code drawn for a PETSCII code: its position in the
/// character ROM, with bit 7 set for reverse video. Control codes, which
/// have no character of their own, show reversed, as in a listing on the
/// C64 (`{CLR}` is a reversed heart, `{RVS ON}` a reversed R).
pub fn screen_code(p: u8) -> u8 {
    match p {
        0x00..=0x1F => 0x80 | p,
        0x20..=0x3F => p,
        0x40..=0x5F => p - 0x40,
        0x60..=0x7F => p - 0x20,
        0x80..=0x9F => 0x80 | (p - 0x40),
        0xA0..=0xBF => p - 0x40,
        0xC0..=0xFE => p - 0x80,
        0xFF => 0x5E,
    }
}

/// Decode a PETSCII text file, byte for byte.
pub fn decode(bytes: &[u8]) -> Decoded {
    if bytes.is_empty() {
        // One empty line with no CR, so it saves back as an empty file.
        return Decoded {
            lines: vec![Vec::new()],
            final_eol: false,
            converted_from: None,
            meta: Default::default(),
        };
    }
    let final_eol = bytes.last() == Some(&CR);
    let body = if final_eol {
        &bytes[..bytes.len() - 1]
    } else {
        bytes
    };
    let lines = body.split(|&b| b == CR).map(<[u8]>::to_vec).collect();
    Decoded {
        lines,
        final_eol,
        converted_from: None,
        meta: Default::default(),
    }
}

/// Encode lines as PETSCII with a CR after each line (and after the last
/// line only if `final_eol`).
pub fn encode(lines: &[Vec<u8>], final_eol: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(lines.iter().map(|l| l.len() + 1).sum());
    for (i, l) in lines.iter().enumerate() {
        out.extend_from_slice(l);
        if i + 1 < lines.len() || final_eol {
            out.push(CR);
        }
    }
    out
}

/// Whether a file looks like PC text rather than PETSCII: it holds line
/// feeds, which Commodore files don't use.
pub fn looks_like_pc_text(bytes: &[u8]) -> bool {
    bytes.contains(&LF)
}

/// PETSCII text with CR line ends. Accepts any file, so it goes last in
/// the machine's format list.
pub struct PetsciiText;

impl retro_core::FileFormat for PetsciiText {
    fn name(&self) -> &'static str {
        "PETSCII text"
    }

    fn status_name(&self) -> &'static str {
        "PETSCII"
    }

    fn extension(&self) -> &'static str {
        "SEQ"
    }

    fn shared_extensions(&self) -> &'static [&'static str] {
        &["TXT"]
    }

    fn disk_file_type(&self) -> &'static str {
        "SEQ"
    }

    fn detect(&self, _bytes: &[u8]) -> bool {
        true
    }

    fn decode(&self, bytes: &[u8]) -> Result<Decoded, String> {
        Ok(decode(bytes))
    }

    fn can_encode(&self) -> bool {
        true
    }

    fn encode(&self, doc: &retro_core::Document) -> Result<retro_core::Encoded, String> {
        Ok(retro_core::Encoded::clean(encode(
            doc.lines(),
            doc.final_eol,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_files() {
        for bytes in [
            &b"10 PRINT \"HI\"\r20 GOTO 10\r"[..],
            &b"NO FINAL CR\rLAST"[..],
            &b"\r\r"[..],
            &b"\x00\x93\xff\r"[..],
            &b""[..],
        ] {
            let d = decode(bytes);
            assert_eq!(encode(&d.lines, d.final_eol), bytes, "{bytes:?}");
        }
    }

    #[test]
    fn splits_lines() {
        let d = decode(b"A\rBC\r");
        assert_eq!(d.lines, vec![b"A".to_vec(), b"BC".to_vec()]);
        assert!(d.final_eol);
    }

    #[test]
    fn screen_codes() {
        assert_eq!(screen_code(b'A'), 0x01);
        assert_eq!(screen_code(b'@'), 0x00);
        assert_eq!(screen_code(b' '), 0x20);
        assert_eq!(screen_code(b'1'), 0x31);
        assert_eq!(screen_code(0xC1), 0x41); // Shift+A: spade / capital A
        assert_eq!(screen_code(0x61), 0x41); // the same character
        assert_eq!(screen_code(0xA0), 0x60); // Shift+Space
        assert_eq!(screen_code(0xFF), 0x5E); // pi
        assert_eq!(screen_code(0x93), 0xD3); // CLR: reversed heart
        assert_eq!(screen_code(0x12), 0x92); // RVS ON: reversed R
        assert_eq!(screen_code(0x0D), 0x8D); // CR: reversed M
    }

    #[test]
    fn line_feeds_mean_pc_text() {
        assert!(looks_like_pc_text(b"A\r\nB"));
        assert!(!looks_like_pc_text(b"A\rB\r"));
    }
}
