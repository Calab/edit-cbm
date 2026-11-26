//! Commodore BASIC programs (PRG files), read into LIST text and written
//! back exactly as typing the lines in on the machine would store them.
//!
//! File layout: the load address (2 bytes), then lines: the address of the
//! next line in memory (2 bytes), the line number (2), the text with
//! keywords as tokens, `$00`. A next-line address whose high byte is 0
//! ends the program. Anything after that (machine code started by a SYS
//! line, say) is kept as the file's trailer.
//!
//! Tokens are `$80` + the keyword's place in the dialect's table; `$FF` is
//! π. Tables from the ROMs: BASIC 2 (C64 `$A09E`, the same on the VIC-20
//! and PET BASIC 2) and BASIC 4 (PET `$B0B2`, BASIC 2's plus 15 disk
//! commands). BASIC 1 (the first PET) is BASIC 2 without GO.
//!
//! [`crunch`] is a port of BASIC's CRUNCH routine (C64 `$A579`, PET BASIC
//! 4 `$B4FB`, the same algorithm), so lines are stored as the machine
//! stores them; [`list`] mirrors it, so a program read and written back
//! comes out byte for byte.

use retro_core::{Decoded, Document, Encoded, FileFormat, FileMeta, Memory};

/// A BASIC version: its keywords, from token `$80`.
pub struct Dialect {
    pub name: &'static str,
    pub status: &'static str,
    pub keywords: &'static [&'static str],
    /// Simons' BASIC: its own keywords are made into `$64` tokens first
    /// (see [`simons_pass`]), then BASIC 2's as usual.
    pub simons: bool,
    /// BASIC 7.0: its own CRUNCH ([`crunch7`]), and two-byte tokens: a
    /// prefix and `$02` + the keyword's place in its table, the tables
    /// in the order CRUNCH searches them.
    pub basic7: bool,
    pub prefixed: &'static [(u8, &'static [&'static str])],
}

/// BASIC 2's keywords (C64 ROM `$A09E`).
const BASIC2: [&str; 76] = [
    "END", "FOR", "NEXT", "DATA", "INPUT#", "INPUT", "DIM", "READ", "LET", "GOTO", "RUN", "IF",
    "RESTORE", "GOSUB", "RETURN", "REM", "STOP", "ON", "WAIT", "LOAD", "SAVE", "VERIFY", "DEF",
    "POKE", "PRINT#", "PRINT", "CONT", "LIST", "CLR", "CMD", "SYS", "OPEN", "CLOSE", "GET", "NEW",
    "TAB(", "TO", "FN", "SPC(", "THEN", "NOT", "STEP", "+", "-", "*", "/", "^", "AND", "OR", ">",
    "=", "<", "SGN", "INT", "ABS", "USR", "FRE", "POS", "SQR", "RND", "LOG", "EXP", "COS", "SIN",
    "TAN", "ATN", "PEEK", "LEN", "STR$", "VAL", "ASC", "CHR$", "LEFT$", "RIGHT$", "MID$", "GO",
];

/// BASIC 4's keywords (PET ROM `$B0B2`): BASIC 2's, then the disk commands.
const BASIC4: [&str; 91] = [
    "END",
    "FOR",
    "NEXT",
    "DATA",
    "INPUT#",
    "INPUT",
    "DIM",
    "READ",
    "LET",
    "GOTO",
    "RUN",
    "IF",
    "RESTORE",
    "GOSUB",
    "RETURN",
    "REM",
    "STOP",
    "ON",
    "WAIT",
    "LOAD",
    "SAVE",
    "VERIFY",
    "DEF",
    "POKE",
    "PRINT#",
    "PRINT",
    "CONT",
    "LIST",
    "CLR",
    "CMD",
    "SYS",
    "OPEN",
    "CLOSE",
    "GET",
    "NEW",
    "TAB(",
    "TO",
    "FN",
    "SPC(",
    "THEN",
    "NOT",
    "STEP",
    "+",
    "-",
    "*",
    "/",
    "^",
    "AND",
    "OR",
    ">",
    "=",
    "<",
    "SGN",
    "INT",
    "ABS",
    "USR",
    "FRE",
    "POS",
    "SQR",
    "RND",
    "LOG",
    "EXP",
    "COS",
    "SIN",
    "TAN",
    "ATN",
    "PEEK",
    "LEN",
    "STR$",
    "VAL",
    "ASC",
    "CHR$",
    "LEFT$",
    "RIGHT$",
    "MID$",
    "GO",
    "CONCAT",
    "DOPEN",
    "DCLOSE",
    "RECORD",
    "HEADER",
    "COLLECT",
    "BACKUP",
    "COPY",
    "APPEND",
    "DSAVE",
    "DLOAD",
    "CATALOG",
    "RENAME",
    "SCRATCH",
    "DIRECTORY",
];

/// The first PET's BASIC: BASIC 2's keywords without GO.
pub static BASIC_1: Dialect = Dialect {
    name: "BASIC 1.0",
    simons: false,
    basic7: false,
    prefixed: &[],
    status: "BASIC 1",
    keywords: BASIC2.split_at(75).0,
};

pub static BASIC_2: Dialect = Dialect {
    name: "BASIC 2.0",
    simons: false,
    basic7: false,
    prefixed: &[],
    status: "BASIC 2",
    keywords: &BASIC2,
};

pub static BASIC_4: Dialect = Dialect {
    name: "BASIC 4.0",
    simons: false,
    basic7: false,
    prefixed: &[],
    status: "BASIC 4",
    keywords: &BASIC4,
};

/// BASIC 7.0's keywords (C128 ROM `$4417`): BASIC 2's, then its own one-
/// byte tokens to `$FD`. `$CE` and `$FE` are prefixes, held by a blank
/// entry.
const BASIC7: [&str; 126] = [
    "END",
    "FOR",
    "NEXT",
    "DATA",
    "INPUT#",
    "INPUT",
    "DIM",
    "READ",
    "LET",
    "GOTO",
    "RUN",
    "IF",
    "RESTORE",
    "GOSUB",
    "RETURN",
    "REM",
    "STOP",
    "ON",
    "WAIT",
    "LOAD",
    "SAVE",
    "VERIFY",
    "DEF",
    "POKE",
    "PRINT#",
    "PRINT",
    "CONT",
    "LIST",
    "CLR",
    "CMD",
    "SYS",
    "OPEN",
    "CLOSE",
    "GET",
    "NEW",
    "TAB(",
    "TO",
    "FN",
    "SPC(",
    "THEN",
    "NOT",
    "STEP",
    "+",
    "-",
    "*",
    "/",
    "^",
    "AND",
    "OR",
    ">",
    "=",
    "<",
    "SGN",
    "INT",
    "ABS",
    "USR",
    "FRE",
    "POS",
    "SQR",
    "RND",
    "LOG",
    "EXP",
    "COS",
    "SIN",
    "TAN",
    "ATN",
    "PEEK",
    "LEN",
    "STR$",
    "VAL",
    "ASC",
    "CHR$",
    "LEFT$",
    "RIGHT$",
    "MID$",
    "GO",
    "RGR",
    "RCLR",
    " ",
    "JOY",
    "RDOT",
    "DEC",
    "HEX$",
    "ERR$",
    "INSTR",
    "ELSE",
    "RESUME",
    "TRAP",
    "TRON",
    "TROFF",
    "SOUND",
    "VOL",
    "AUTO",
    "PUDEF",
    "GRAPHIC",
    "PAINT",
    "CHAR",
    "BOX",
    "CIRCLE",
    "GSHAPE",
    "SSHAPE",
    "DRAW",
    "LOCATE",
    "COLOR",
    "SCNCLR",
    "SCALE",
    "HELP",
    "DO",
    "LOOP",
    "EXIT",
    "DIRECTORY",
    "DSAVE",
    "DLOAD",
    "HEADER",
    "SCRATCH",
    "COLLECT",
    "COPY",
    "RENAME",
    "BACKUP",
    "DELETE",
    "RENUMBER",
    "KEY",
    "MONITOR",
    "USING",
    "UNTIL",
    "WHILE",
];

/// BASIC 7.0's `$FE` keywords (ROM `$4609`), from `$FE $02`; blanks hold
/// numbers CRUNCH can't use (a space, a quote).
const BASIC7_FE: [&str; 37] = [
    "BANK",
    "FILTER",
    "PLAY",
    "TEMPO",
    "MOVSPR",
    "SPRITE",
    "SPRCOLOR",
    "RREG",
    "ENVELOPE",
    "SLEEP",
    "CATALOG",
    "DOPEN",
    "APPEND",
    "DCLOSE",
    "BSAVE",
    "BLOAD",
    "RECORD",
    "CONCAT",
    "DVERIFY",
    "DCLEAR",
    "SPRSAV",
    "COLLISION",
    "BEGIN",
    "BEND",
    "WINDOW",
    "BOOT",
    "WIDTH",
    "SPRDEF",
    "QUIT",
    "STASH",
    " ",
    "FETCH",
    " ",
    "SWAP",
    "OFF",
    "FAST",
    "SLOW",
];

/// BASIC 7.0's `$CE` keywords (ROM `$46C9`), from `$CE $02`.
const BASIC7_CE: [&str; 9] = [
    "POT", "BUMP", "PEN", "RSPPOS", "RSPRITE", "RSPCOLOR", "XOR", "RWINDOW", "POINTER",
];

/// The C128's BASIC 7.0.
pub static BASIC_7: Dialect = Dialect {
    name: "BASIC 7.0",
    status: "BASIC 7",
    keywords: &BASIC7,
    simons: false,
    basic7: true,
    prefixed: &[(0xFE, &BASIC7_FE), (0xCE, &BASIC7_CE)],
};

/// Simons' BASIC's keywords, from its cartridge ROM (`$A521`, each
/// ending in `@`): token `$64` then the keyword's place + 1. `>>` marks
/// numbers it leaves unused, those BASIC 2's CRUNCH would turn into
/// something else (`"`, `?`, operators, `T`).
const SIMONS: [&str; 127] = [
    "HIRES", "PLOT", "LINE", "BLOCK", "FCHR", "FCOL", "FILL", "REC", "ROT", "DRAW", "CHAR",
    "HI COL", "INV", "FRAC", "MOVE", "PLACE", "UPB", "UPW", "LEFTW", "LEFTB", "DOWNB", "DOWNW",
    "RIGHTB", "RIGHTW", "MULTI", "COLOUR", "MMOB", "BFLASH", "MOB SET", "MUSIC", "FLASH", "REPEAT",
    "PLAY", ">>", "CENTRE", "ENVELOPE", "CGOTO", "WAVE", "FETCH", "AT(", "UNTIL", ">>", ">>",
    "USE", ">>", "GLOBAL", ">>", "RESET", "PROC", "CALL", "EXEC", "END PROC", "EXIT", "END LOOP",
    "ON KEY", "DISABLE", "RESUME", "LOOP", "DELAY", ">>", ">>", ">>", ">>", "SECURE", "DISAPA",
    "CIRCLE", "ON ERROR", "NO ERROR", "LOCAL", "RCOMP", "ELSE", "RETRACE", "TRACE", "DIR", "PAGE",
    "DUMP", "FIND", "OPTION", "AUTO", "OLD", "JOY", "MOD", "DIV", ">>", "DUP", "INKEY", "INST",
    "TEST", "LIN", "EXOR", "INSERT", "POT", "PENX", ">>", "PENY", "SOUND", "GRAPHICS", "DESIGN",
    "RLOCMOB", "CMOB", "BCKGNDS", "PAUSE", "NRM", "MOB OFF", "OFF", "ANGL", "ARC", "COLD", "SCRSV",
    "SCRLD", "TEXT", "CSET", "VOL", "DISK", "HRDCPY", "KEY", "PAINT", "LOW COL", "COPY", "MERGE",
    "RENUMBER", "MEM", "DETECT", "CHECK", "DISPLAY", "ERR", "OUT",
];

/// Simons' BASIC: BASIC 2 with the cartridge's keywords.
pub static SIMONS_BASIC: Dialect = Dialect {
    name: "Simons' BASIC",
    status: "Simons'",
    keywords: &BASIC2,
    simons: true,
    basic7: false,
    prefixed: &[],
};

/// The first byte of a Simons' BASIC keyword.
pub const SIMONS_PREFIX: u8 = 0x64;
/// DISAPA's number: the cartridge writes four colons after it, so the
/// line keeps its length.
const DISAPA: u8 = 0x41;

/// The Simons' keyword with token number `n`, if it's one.
fn simons_keyword(n: u8) -> Option<&'static str> {
    let k = *SIMONS.get(usize::from(n).checked_sub(1)?)?;
    (k != ">>").then_some(k)
}

/// Simons' BASIC's pass over a typed line before BASIC 2's CRUNCH (its
/// cartridge `$AD2B`, called from its CRUNCH hook at `$8261`): at each
/// capital letter outside quotes, the first of its keywords, in table
/// order, that the text starts with becomes `$64` and its number. It
/// doesn't know REM or DATA, so works there too; an `@` outside quotes
/// ends the pass for the rest of the line.
pub fn simons_pass(text: &[u8]) -> Vec<u8> {
    let mut buf = text.to_vec();
    let mut y = 0;
    while y < buf.len() {
        let a = buf[y];
        if a == b'"' {
            match buf[y + 1..].iter().position(|&b| b == b'"') {
                Some(p) => {
                    y += p + 2;
                    continue;
                }
                None => break,
            }
        }
        if a == b'@' {
            break;
        }
        if !a.is_ascii_uppercase() {
            y += 1;
            continue;
        }
        let found = SIMONS
            .iter()
            .position(|k| buf[y..].starts_with(k.as_bytes()));
        match found {
            Some(i) if i + 1 == usize::from(DISAPA) => {
                buf.splice(y..y + 6, [SIMONS_PREFIX, DISAPA, b':', b':', b':', b':']);
                y += 6;
            }
            Some(i) => {
                buf.splice(y..y + SIMONS[i].len(), [SIMONS_PREFIX, i as u8 + 1]);
                y += 2;
            }
            None => y += 1,
        }
    }
    buf
}

pub const DATA: u8 = 0x83;
pub const REM: u8 = 0x8F;
pub const PRINT: u8 = 0x99;
/// π, stored as `$FF` (the screen editor reads the π typed, `$DE`, as
/// `$FF`).
pub const PI: u8 = 0xFF;
const PI_TYPED: u8 = 0xDE;

/// The first BASIC 4 disk command's token.
const FIRST_DISK_TOKEN: u8 = 0xCC;

/// Highest line number BASIC accepts.
pub const MAX_LINE: u32 = 63999;

/// Where BASIC programs start on each machine.
pub const LOAD_ADDRESSES: [u16; 5] = [0x0801, 0x1001, 0x0401, 0x1201, 0x1C01];

impl Dialect {
    fn keyword(&self, token: u8) -> Option<&'static str> {
        self.keywords
            .get(usize::from(token.wrapping_sub(0x80)))
            .copied()
    }

    /// The keyword table as the ROM holds it: each keyword's last
    /// character with bit 7 set, then a 0.
    fn table(&self) -> Vec<u8> {
        table_of(self.keywords)
    }

    /// The keyword for a two-byte token, if it's one.
    fn prefixed_keyword(&self, prefix: u8, n: u8) -> Option<&'static str> {
        let (_, table) = self.prefixed.iter().find(|(p, _)| *p == prefix)?;
        let k = *table.get(usize::from(n).checked_sub(2)?)?;
        (k != " ").then_some(k)
    }
}

/// `keywords` as a ROM keyword table.
fn table_of(keywords: &[&str]) -> Vec<u8> {
    let mut t = Vec::new();
    for k in keywords {
        let b = k.as_bytes();
        t.extend_from_slice(&b[..b.len() - 1]);
        t.push(b[b.len() - 1] | 0x80);
    }
    t.push(0);
    t
}

/// A program read from a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub load: u16,
    /// Line numbers and tokenized text.
    pub lines: Vec<(u16, Vec<u8>)>,
    /// The program's length (lines and end marker, without the load
    /// address).
    pub len: usize,
    /// Bytes after the end of the program.
    pub trailer: Vec<u8>,
}

fn word(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

/// Read a PRG file as a BASIC program: a known load address, then lines
/// whose links point to the next line and whose numbers go up.
pub fn parse(bytes: &[u8]) -> Result<Program, String> {
    if bytes.len() < 4 {
        return Err("too short for a BASIC program".into());
    }
    let load = word(bytes, 0);
    if !LOAD_ADDRESSES.contains(&load) {
        return Err(format!("loads at ${load:04X}, not where BASIC starts"));
    }
    let mut lines: Vec<(u16, Vec<u8>)> = Vec::new();
    let mut pos = 2;
    let mut addr = load as usize;
    loop {
        if pos + 2 > bytes.len() {
            return Err("ends before the end-of-program marker".into());
        }
        let link = word(bytes, pos);
        if link >> 8 == 0 {
            pos += 2;
            break;
        }
        if pos + 4 > bytes.len() {
            return Err("ends part way through a line".into());
        }
        let number = word(bytes, pos + 2);
        if u32::from(number) > MAX_LINE {
            return Err(format!("line number {number} is over {MAX_LINE}"));
        }
        if let Some(&(prev, _)) = lines.last()
            && number <= prev
        {
            return Err(format!("line {number} comes after line {prev}"));
        }
        let Some(end) = bytes[pos + 4..].iter().position(|&b| b == 0) else {
            return Err(format!("line {number} has no end"));
        };
        let text = bytes[pos + 4..pos + 4 + end].to_vec();
        let next = addr + 4 + end + 1;
        if usize::from(link) != next {
            return Err(format!(
                "line {number} links to ${link:04X}, not the next line at ${next:04X}"
            ));
        }
        lines.push((number, text));
        pos += 4 + end + 1;
        addr = next;
    }
    Ok(Program {
        load,
        lines,
        len: pos - 2,
        trailer: bytes[pos..].to_vec(),
    })
}

/// What CRUNCH is doing at a point in a line: after REM nothing more is
/// tokenized; inside quotes nothing; after DATA nothing until a `:`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Quote,
    Data,
    Rem,
}

/// LIST's state in a line (BASIC 2's view): which [`Mode`], and whether
/// a string is inside DATA.
struct Lister {
    mode: Mode,
    data_quote: bool,
}

impl Lister {
    fn new() -> Self {
        Self {
            mode: Mode::Normal,
            data_quote: false,
        }
    }

    /// Show byte `b`, keywords spelled out.
    fn byte(&mut self, d: &Dialect, out: &mut Vec<u8>, b: u8) {
        match self.mode {
            Mode::Rem => out.push(b),
            Mode::Quote => {
                out.push(b);
                if b == b'"' {
                    self.mode = if self.data_quote {
                        Mode::Data
                    } else {
                        Mode::Normal
                    };
                }
            }
            Mode::Data => {
                out.push(b);
                match b {
                    b'"' => {
                        self.data_quote = true;
                        self.mode = Mode::Quote;
                    }
                    b':' => self.mode = Mode::Normal,
                    _ => {}
                }
            }
            Mode::Normal => {
                if b == b'"' {
                    self.data_quote = false;
                    self.mode = Mode::Quote;
                    out.push(b);
                } else if b >= 0x80 && b != PI {
                    match d.keyword(b) {
                        Some(k) => out.extend_from_slice(k.as_bytes()),
                        None => out.push(b),
                    }
                    match b {
                        REM => self.mode = Mode::Rem,
                        DATA => self.mode = Mode::Data,
                        _ => {}
                    }
                } else {
                    out.push(b);
                }
            }
        }
    }
}

/// A line's text as LIST shows it: keywords spelled out. After REM and
/// DATA, bytes show as they are (so the program saves back unchanged).
/// Simons' keywords show wherever its pass would have made them (outside
/// quotes, before any `@`), REM and DATA included, so typing the listing
/// in again gives the same bytes.
pub fn list(d: &Dialect, text: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len() * 2);
    let mut lister = Lister::new();
    // Simons' pass's view: in quotes, or past an @.
    let mut quote = false;
    let mut stopped = false;
    let mut i = 0;
    while i < text.len() {
        let b = text[i];
        if d.simons
            && !quote
            && !stopped
            && b == SIMONS_PREFIX
            && let Some(k) = text.get(i + 1).and_then(|&n| simons_keyword(n))
        {
            out.extend_from_slice(k.as_bytes());
            let n = text[i + 1];
            i += 2;
            // The second byte went through BASIC 2's CRUNCH too (LOOP's
            // is a colon, which ends DATA).
            lister.byte(d, &mut Vec::new(), n);
            if n == DISAPA && text[i..].starts_with(b"::::") {
                for _ in 0..4 {
                    lister.byte(d, &mut Vec::new(), b':');
                }
                i += 4;
            }
            continue;
        }
        // BASIC 7.0's two-byte tokens.
        if lister.mode == Mode::Normal
            && let Some(k) = text.get(i + 1).and_then(|&n| d.prefixed_keyword(b, n))
        {
            out.extend_from_slice(k.as_bytes());
            i += 2;
            continue;
        }
        if b == b'"' {
            quote = !quote;
        } else if b == b'@' && !quote {
            stopped = true;
        }
        lister.byte(d, &mut out, b);
        i += 1;
    }
    out
}

/// BASIC 7.0's CRUNCH (C128 ROM `$430D`), which works on the line in
/// place: from each character that isn't a digit, space or colon, it
/// looks up the `$FE` keywords, then the `$CE` ones, then the one-byte
/// ones, the first match in each table winning. Outside quotes, shifted
/// characters are dropped (but π kept); `?` is PRINT; after REM nothing
/// more is tokenized, after DATA nothing until a colon outside quotes.
/// Returns the bytes and how many characters were dropped.
pub fn crunch7(d: &Dialect, text: &[u8]) -> (Vec<u8>, usize) {
    // CHRGOT: the character at `p`, past any spaces. CHRGET: the next.
    fn got(buf: &[u8], p: &mut usize) -> u8 {
        while buf.get(*p) == Some(&b' ') {
            *p += 1;
        }
        buf.get(*p).copied().unwrap_or(0)
    }
    fn get(buf: &[u8], p: &mut usize) -> u8 {
        *p += 1;
        got(buf, p)
    }
    let main = d.table();
    let prefixed: Vec<(u8, Vec<u8>)> = d.prefixed.iter().map(|&(p, k)| (p, table_of(k))).collect();
    let mut buf = text.to_vec();
    buf.push(0);
    let mut dropped = 0;
    let mut p = 0;
    let mut c = got(&buf, &mut p);
    'line: loop {
        if c.is_ascii_digit() {
            c = get(&buf, &mut p);
            continue;
        }
        match c {
            0 => break,
            b':' | PI => {
                c = get(&buf, &mut p);
                continue;
            }
            0x80..=0xFE => {
                buf.remove(p);
                dropped += 1;
                c = got(&buf, &mut p);
                continue;
            }
            b'"' => loop {
                c = get(&buf, &mut p);
                if c == 0 {
                    break 'line;
                }
                if c == b'"' {
                    c = get(&buf, &mut p);
                    continue 'line;
                }
            },
            b'?' => {
                buf[p] = PRINT;
                c = get(&buf, &mut p);
                continue;
            }
            _ => {}
        }
        for (prefix, table) in &prefixed {
            if let Some((token, len)) = find_keyword(table, &buf[p..]) {
                buf.splice(p..p + len, [*prefix, (token & 0x7F) + 2]);
                get(&buf, &mut p);
                c = get(&buf, &mut p);
                continue 'line;
            }
        }
        match find_keyword(&main, &buf[p..]) {
            Some((REM, len)) => {
                buf.splice(p..p + len, [REM]);
                break;
            }
            Some((DATA, len)) => {
                buf.splice(p..p + len, [DATA]);
                get(&buf, &mut p);
                // Past DATA's items: to a colon outside quotes, or the end.
                let mut quoted = false;
                while let Some(&ch) = buf.get(p).filter(|&&ch| ch != 0) {
                    if ch == b'"' {
                        quoted = !quoted;
                    } else if ch == b':' && !quoted {
                        break;
                    }
                    p += 1;
                }
                c = got(&buf, &mut p);
            }
            Some((token, len)) => {
                buf.splice(p..p + len, [token]);
                c = get(&buf, &mut p);
            }
            None => c = get(&buf, &mut p),
        }
    }
    buf.truncate(buf.iter().position(|&b| b == 0).unwrap_or(buf.len()));
    (buf, dropped)
}

/// The keyword at the start of `input`, as CRUNCH finds it: the token and
/// how many bytes it takes. Compares against the ROM's table byte by byte:
/// a byte differing by exactly $80 ends a match, whether it's the
/// keyword's last character (bit 7 set in the table) or a shifted letter
/// typed as an abbreviation (`pR` is PRINT).
fn find_keyword(table: &[u8], input: &[u8]) -> Option<(u8, usize)> {
    let at = |x: usize| input.get(x).copied().unwrap_or(0);
    let mut token = 0u8;
    let mut y = 0;
    let mut x = 0;
    loop {
        if table[y] == 0 {
            return None;
        }
        let diff = at(x).wrapping_sub(table[y]);
        if diff == 0 {
            x += 1;
            y += 1;
            continue;
        }
        if diff == 0x80 {
            return Some((0x80 | token, x + 1));
        }
        // Not this keyword: on to the next.
        x = 0;
        token += 1;
        while table[y] & 0x80 == 0 {
            y += 1;
        }
        y += 1;
    }
}

/// Tokenize a line's text (after its line number) as CRUNCH does. Returns
/// the bytes and how many characters were dropped: outside quotes, REM and
/// DATA's text, BASIC drops shifted characters and graphics it can't use
/// (`$80` and up, except π).
pub fn crunch(d: &Dialect, text: &[u8]) -> (Vec<u8>, usize) {
    let table = d.table();
    let mut out = Vec::with_capacity(text.len());
    let mut dropped = 0;
    let mut data = false;
    let mut i = 0;
    while i < text.len() {
        let c = text[i];
        let stored;
        if c >= 0x80 && c != PI {
            dropped += 1;
            i += 1;
            continue;
        } else if c == b'"' {
            // Copy the string, and its closing quote, as it is.
            out.push(c);
            i += 1;
            while i < text.len() {
                out.push(text[i]);
                i += 1;
                if text[i - 1] == b'"' {
                    break;
                }
            }
            continue;
        } else if c == b' ' || data || c == PI || (b'0'..b'<').contains(&c) {
            stored = c;
            i += 1;
        } else if c == b'?' {
            stored = PRINT;
            i += 1;
        } else if let Some((token, len)) = find_keyword(&table, &text[i..]) {
            stored = token;
            i += len;
        } else {
            stored = c;
            i += 1;
        }
        out.push(stored);
        match stored {
            b':' => data = false,
            DATA => data = true,
            REM => {
                out.extend_from_slice(&text[i..]);
                break;
            }
            _ => {}
        }
    }
    (out, dropped)
}

/// The line number at the start of a typed line, as BASIC reads it
/// (spaces between digits are skipped), and where its text starts.
/// `None` if the line has no number.
fn line_number(line: &[u8]) -> Option<(u32, usize)> {
    let mut n: u32 = 0;
    let mut digits = 0;
    let mut i = 0;
    while i < line.len() {
        match line[i] {
            b' ' => {}
            c @ b'0'..=b'9' => {
                n = (n * 10 + u32::from(c - b'0')).min(u32::from(u16::MAX) + 1);
                digits += 1;
            }
            _ => break,
        }
        i += 1;
    }
    (digits > 0).then_some((n, i))
}

/// Whether `program` has Simons' BASIC keywords (`$64` tokens where its
/// pass would have made them).
pub fn uses_simons(program: &Program) -> bool {
    program.lines.iter().any(|(_, text)| {
        let mut quote = false;
        for (i, &b) in text.iter().enumerate() {
            match b {
                b'"' => quote = !quote,
                b'@' if !quote => return false,
                SIMONS_PREFIX
                    if !quote && text.get(i + 1).and_then(|&n| simons_keyword(n)).is_some() =>
                {
                    return true;
                }
                _ => {}
            }
        }
        false
    })
}

/// Whether `program` uses BASIC 4's disk commands (tokens `$CC-$DA`
/// outside quotes, REM and DATA).
pub fn uses_basic4(program: &Program) -> bool {
    program
        .lines
        .iter()
        .any(|(_, text)| tokens(text).any(|t| (FIRST_DISK_TOKEN..=0xDA).contains(&t)))
}

/// The tokens in a line: bytes of `$80` and up outside quotes, REM's text
/// and DATA's.
fn tokens(text: &[u8]) -> impl Iterator<Item = u8> + '_ {
    let mut mode = Mode::Normal;
    text.iter().filter_map(move |&b| match mode {
        Mode::Rem => None,
        Mode::Quote => {
            if b == b'"' {
                mode = Mode::Normal;
            }
            None
        }
        Mode::Data => {
            match b {
                b'"' => mode = Mode::Quote,
                b':' => mode = Mode::Normal,
                _ => {}
            }
            None
        }
        Mode::Normal => match b {
            b'"' => {
                mode = Mode::Quote;
                None
            }
            REM => {
                mode = Mode::Rem;
                Some(b)
            }
            DATA => {
                mode = Mode::Data;
                Some(b)
            }
            PI => None,
            0x80.. => Some(b),
            _ => None,
        },
    })
}

/// A program as text for the editor: each line as LIST shows it.
pub fn decode(d: &Dialect, bytes: &[u8]) -> Result<Decoded, String> {
    let p = parse(bytes)?;
    let mut lines: Vec<Vec<u8>> = p
        .lines
        .iter()
        .map(|(n, text)| {
            let mut l = format!("{n} ").into_bytes();
            l.extend(list(d, text));
            l
        })
        .collect();
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    Ok(Decoded {
        lines,
        final_eol: true,
        converted_from: None,
        meta: FileMeta {
            load_address: Some(p.load),
            trailer: p.trailer,
            program_len: p.len,
        },
    })
}

/// Lines typed in, as a program: each line tokenized as BASIC would when
/// it's typed, sorted by number, a later line replacing an earlier one
/// with the same number and a line number alone deleting the line.
/// Returns the file and anything to warn about first.
pub fn encode(d: &Dialect, lines: &[Vec<u8>], meta: &FileMeta, load: u16) -> Encoded {
    let mut program: std::collections::BTreeMap<u32, Vec<u8>> = Default::default();
    let mut no_number = Vec::new();
    let mut too_big = Vec::new();
    let mut dropped = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.iter().all(|&b| b == b' ') {
            continue;
        }
        let Some((number, start)) = line_number(line) else {
            no_number.push(index + 1);
            continue;
        };
        if number > MAX_LINE {
            too_big.push(index + 1);
            continue;
        }
        // The screen editor reads π as $FF.
        let text: Vec<u8> = line[start..]
            .iter()
            .map(|&b| if b == PI_TYPED { PI } else { b })
            .collect();
        if text.is_empty() {
            program.remove(&number);
            continue;
        }
        let (bytes, lost) = if d.basic7 {
            crunch7(d, &text)
        } else if d.simons {
            crunch(d, &simons_pass(&text))
        } else {
            crunch(d, &text)
        };
        if lost > 0 {
            dropped.push((number, index + 1, lost));
        }
        program.insert(number, bytes);
    }
    let mut out = load.to_le_bytes().to_vec();
    let mut addr = usize::from(load);
    for (number, text) in &program {
        addr += 4 + text.len() + 1;
        out.extend_from_slice(&(addr as u16).to_le_bytes());
        out.extend_from_slice(&(*number as u16).to_le_bytes());
        out.extend_from_slice(text);
        out.push(0);
    }
    out.extend_from_slice(&[0, 0]);
    let len = out.len() - 2;
    out.extend_from_slice(&meta.trailer);

    let mut warnings = Vec::new();
    let list_lines = |v: &[usize]| {
        let shown: Vec<String> = v.iter().take(8).map(|n| n.to_string()).collect();
        let more = if v.len() > 8 { ", ..." } else { "" };
        format!("{}{more}", shown.join(", "))
    };
    if !no_number.is_empty() {
        warnings.push(format!(
            "Editor lines {} have no line number, so BASIC can't store them; \
             they're left out.",
            list_lines(&no_number)
        ));
    }
    if !too_big.is_empty() {
        warnings.push(format!(
            "Editor lines {} have line numbers over {MAX_LINE}; they're left out.",
            list_lines(&too_big)
        ));
    }
    if !dropped.is_empty() {
        let which: Vec<String> = dropped
            .iter()
            .take(8)
            .map(|(n, i, k)| format!("line {n} (editor line {i}): {k}"))
            .collect();
        warnings.push(format!(
            "Some characters outside quotes are ones BASIC drops when a line is \
             typed in (shifted letters and graphics); they're left out, as \
             typing them would:\n    {}",
            which.join("\n    ")
        ));
    }
    if !meta.trailer.is_empty() && meta.program_len != 0 && len != meta.program_len {
        let by = len as i64 - meta.program_len as i64;
        warnings.push(format!(
            "The {} bytes after the program (machine code?) move {} by {} bytes, \
             since the program is a different length now. A SYS to them may \
             need changing.",
            meta.trailer.len(),
            if by > 0 { "up" } else { "down" },
            by.abs()
        ));
    }
    Encoded {
        bytes: out,
        warning: (!warnings.is_empty()).then(|| warnings.join("\n\n")),
    }
}

/// The memory set-ups: where BASIC starts and BYTES FREE at power-on,
/// the largest standard memory for each start address (S2).
pub static C64_MEMORY: [Memory; 1] = [Memory {
    name: "C64",
    load_address: 0x0801,
    bytes_free: 38911,
}];
/// The C128's bank 0, which holds the program (FRE(0) in VICE).
pub static C128_MEMORY: [Memory; 1] = [Memory {
    name: "C128",
    load_address: 0x1C01,
    bytes_free: 58109,
}];
/// With the Simons' BASIC cartridge in (checked in VICE).
pub static SIMONS_MEMORY: [Memory; 1] = [Memory {
    name: "C64 with Simons' BASIC",
    load_address: 0x0801,
    bytes_free: 30719,
}];
pub static VIC20_MEMORY: [Memory; 3] = [
    Memory {
        name: "VIC-20 unexpanded",
        load_address: 0x1001,
        bytes_free: 3583,
    },
    Memory {
        name: "VIC-20 +3K",
        load_address: 0x0401,
        bytes_free: 6655,
    },
    Memory {
        name: "VIC-20 +24K",
        load_address: 0x1201,
        bytes_free: 28159,
    },
];
pub static PET_MEMORY: [Memory; 1] = [Memory {
    name: "PET 32K",
    load_address: 0x0401,
    bytes_free: 31743,
}];

/// A tokenized BASIC program in dialect `dialect`, as one model saves it:
/// for one of `memory`, the first unless the document says.
pub struct CbmBasic {
    pub dialect: &'static Dialect,
    pub memory: &'static [Memory],
}

impl FileFormat for CbmBasic {
    fn name(&self) -> &'static str {
        self.dialect.name
    }

    fn status_name(&self) -> &'static str {
        self.dialect.status
    }

    fn extension(&self) -> &'static str {
        "PRG"
    }

    fn shared_extensions(&self) -> &'static [&'static str] {
        &["PRG"]
    }

    /// A program; for Simons' BASIC, one using its keywords.
    fn detect(&self, bytes: &[u8]) -> bool {
        parse(bytes).is_ok_and(|p| !self.dialect.simons || uses_simons(&p))
    }

    /// Tokens the dialect doesn't know.
    fn misfit(&self, bytes: &[u8]) -> Option<usize> {
        let p = parse(bytes).ok()?;
        if self.dialect.simons && !uses_simons(&p) {
            return None;
        }
        Some(
            p.lines
                .iter()
                .flat_map(|(_, t)| tokens(t).collect::<Vec<_>>())
                .filter(|&t| self.dialect.keyword(t).is_none())
                .count(),
        )
    }

    fn looks_like(&self, bytes: &[u8]) -> bool {
        bytes.len() >= 4 && LOAD_ADDRESSES.contains(&word(bytes, 0))
    }

    fn decode(&self, bytes: &[u8]) -> Result<Decoded, String> {
        decode(self.dialect, bytes)
    }

    /// The load address says: $0801 the C64, $1001 and $1201 the VIC-20,
    /// $0401 the PET or a VIC-20 with 3K more memory (BASIC 4's disk
    /// commands mean a PET; otherwise the window's system if it's one of
    /// those, else a PET).
    fn system_for(&self, bytes: &[u8], current: &str) -> Option<&'static str> {
        let p = parse(bytes).ok()?;
        Some(match p.load {
            0x0801 => "c64",
            0x1001 | 0x1201 => "vic20",
            0x0401 => {
                let pets = ["pet", "pet8032"];
                let keep = |ids: &[&'static str]| ids.iter().copied().find(|&id| id == current);
                if uses_basic4(&p) {
                    keep(&pets).unwrap_or("pet")
                } else {
                    keep(&["vic20", "pet", "pet8032"]).unwrap_or("pet")
                }
            }
            0x1C01 => ["c128", "c128_80"]
                .into_iter()
                .find(|&id| id == current)
                .unwrap_or("c128"),
            _ => return None,
        })
    }

    fn memory(&self) -> &'static [Memory] {
        self.memory
    }

    fn disk_file_type(&self) -> &'static str {
        "PRG"
    }

    fn can_encode(&self) -> bool {
        true
    }

    fn encode(&self, doc: &Document) -> Result<Encoded, String> {
        let load = doc.meta.load_address.unwrap_or(self.memory[0].load_address);
        Ok(encode(self.dialect, doc.lines(), &doc.meta, load))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crunched(s: &[u8]) -> Vec<u8> {
        crunch(&BASIC_2, s).0
    }

    #[test]
    fn keywords_become_tokens() {
        assert_eq!(crunched(b"PRINT\"HI\""), b"\x99\"HI\"");
        assert_eq!(crunched(b"?A"), b"\x99A");
        assert_eq!(crunched(b"FORI=1TO10"), b"\x81I\xb21\xa410");
        // Spaces are kept; digits and : ; are never looked up.
        assert_eq!(crunched(b"GOTO 10:END"), b"\x89 10:\x80");
        // Keywords inside names, as the machine does it.
        assert_eq!(crunched(b"SCORE"), b"SC\xb0E");
        // PRINT# before PRINT, GOTO before GO.
        assert_eq!(crunched(b"PRINT#1"), b"\x981");
        assert_eq!(crunched(b"GO TO"), b"\xcb \xa4");
    }

    #[test]
    fn abbreviations() {
        // p + shifted R is PRINT#, not PRINT: PRINT# comes first in the
        // table, as on the C64 (PRINT's abbreviation is ?). g + shifted O
        // is GOTO.
        assert_eq!(crunched(b"P\xd2"), b"\x98");
        assert_eq!(crunched(b"G\xcf10"), b"\x8910");
    }

    #[test]
    fn strings_rem_and_data_stay_as_typed() {
        assert_eq!(crunched(b"PRINT\"TO\xc1\""), b"\x99\"TO\xc1\"");
        assert_eq!(crunched(b"REM TO \xc1"), b"\x8f TO \xc1");
        // DATA: no tokens until a colon outside quotes; then tokens again.
        assert_eq!(crunched(b"DATA TO,\"A:B\":TO"), b"\x83 TO,\"A:B\":\xa4");
        // In DATA, ? stays ?.
        assert_eq!(crunched(b"DATA ?"), b"\x83 ?");
    }

    #[test]
    fn shifted_characters_outside_quotes_are_dropped() {
        assert_eq!(crunch(&BASIC_2, b"A\xc1B"), (b"AB".to_vec(), 1));
        // π is kept.
        assert_eq!(crunch(&BASIC_2, b"A=\xff"), (b"A\xb2\xff".to_vec(), 0));
    }

    #[test]
    fn listing_spells_out_keywords() {
        assert_eq!(
            list(&BASIC_2, b"\x99\"\x99\":\x8f\x99"),
            b"PRINT\"\x99\":REM\x99"
        );
        assert_eq!(
            list(&BASIC_2, b"\x83\x99,\"A:\x99\":\x99"),
            b"DATA\x99,\"A:\x99\":PRINT"
        );
        assert_eq!(list(&BASIC_4, b"\xda"), b"DIRECTORY");
        assert_eq!(list(&BASIC_2, b"\xda"), b"\xda");
    }

    fn program(lines: &[&[u8]]) -> Vec<u8> {
        let lines: Vec<Vec<u8>> = lines.iter().map(|l| l.to_vec()).collect();
        encode(&BASIC_2, &lines, &FileMeta::default(), 0x0801).bytes
    }

    #[test]
    fn saves_as_the_c64_does() {
        // 10 PRINT"HELLO" / 20 GOTO 10, as SAVEd by a C64.
        let want = b"\x01\x08\x0e\x08\x0a\x00\x99\"HELLO\"\x00\x17\x08\x14\x00\x89 10\x00\x00\x00";
        assert_eq!(program(&[b"20 GOTO 10", b"10 PRINT\"HELLO\""]), want);
        // Read back: the same listing, and the same bytes again.
        let d = decode(&BASIC_2, want).unwrap();
        assert_eq!(
            d.lines,
            vec![b"10 PRINT\"HELLO\"".to_vec(), b"20 GOTO 10".to_vec()]
        );
        assert_eq!(d.meta.load_address, Some(0x0801));
        let e = encode(&BASIC_2, &d.lines, &d.meta, 0x0801);
        assert_eq!(e.bytes, want);
        assert_eq!(e.warning, None);
    }

    #[test]
    fn typing_rules_for_lines() {
        // A later line replaces an earlier one; a number alone deletes;
        // spaces after the number go, spaces between digits are skipped.
        let p = program(&[b"10 A", b"20 B", b"10    C", b"20", b"3 0 D"]);
        let d = decode(&BASIC_2, &p).unwrap();
        assert_eq!(d.lines, vec![b"10 C".to_vec(), b"30 D".to_vec()]);
        // π typed ($DE) is stored as $FF.
        let p = program(&[b"10 A=\xde"]);
        assert_eq!(&p[6..9], b"A\xb2\xff");
    }

    #[test]
    fn basic_1_has_no_go() {
        assert_eq!(crunch(&BASIC_1, b"GO TO").0, b"GO \xa4");
        assert_eq!(crunch(&BASIC_1, b"GOTO").0, b"\x89");
        assert_eq!(list(&BASIC_1, b"\xcb"), b"\xcb");
    }

    #[test]
    fn warnings() {
        let lines = vec![b"PRINT".to_vec(), b"10 A\xc1".to_vec(), b"70000 X".to_vec()];
        let e = encode(&BASIC_2, &lines, &FileMeta::default(), 0x0801);
        let w = e.warning.unwrap();
        assert!(w.contains("Editor lines 1 have no line number"), "{w}");
        assert!(w.contains("over 63999"), "{w}");
        assert!(w.contains("line 10 (editor line 2): 1"), "{w}");
    }

    #[test]
    fn machine_code_after_the_program_is_kept() {
        let mut file = program(&[b"10 SYS 2061"]);
        file.extend_from_slice(&[0xA9, 0x00, 0x60]);
        let d = decode(&BASIC_2, &file).unwrap();
        assert_eq!(d.meta.trailer, [0xA9, 0x00, 0x60]);
        let e = encode(&BASIC_2, &d.lines, &d.meta, 0x0801);
        assert_eq!(e.bytes, file);
        assert_eq!(e.warning, None);
        // A longer line moves it: a warning.
        let e = encode(&BASIC_2, &[b"10 SYS 20610".to_vec()], &d.meta, 0x0801);
        assert!(e.warning.unwrap().contains("move up by 1 bytes"));
    }

    #[test]
    fn detection_and_systems() {
        let f = CbmBasic {
            dialect: &BASIC_2,
            memory: &C64_MEMORY,
        };
        let c64 = program(&[b"10 PRINT"]);
        assert!(f.detect(&c64));
        assert_eq!(f.system_for(&c64, "pet"), Some("c64"));
        // Plain text isn't a program.
        assert!(!f.detect(b"10 PRINT\r"));
        // $0401: the current system if VIC-20 or PET, else PET; BASIC 4
        // tokens mean PET.
        let lines = vec![b"10 PRINT".to_vec()];
        let pet = encode(&BASIC_2, &lines, &FileMeta::default(), 0x0401).bytes;
        assert_eq!(f.system_for(&pet, "vic20"), Some("vic20"));
        assert_eq!(f.system_for(&pet, "c64"), Some("pet"));
        let lines = vec![b"10 DIRECTORY".to_vec()];
        let p4 = encode(&BASIC_4, &lines, &FileMeta::default(), 0x0401).bytes;
        assert_eq!(f.system_for(&p4, "vic20"), Some("pet"));
        assert_eq!(f.system_for(&p4, "pet8032"), Some("pet8032"));
        assert_eq!(f.misfit(&p4), Some(1));
        let f4 = CbmBasic {
            dialect: &BASIC_4,
            memory: &PET_MEMORY,
        };
        assert_eq!(f4.misfit(&p4), Some(0));
    }
}

/// Programs typed into VICE (x64sc) and SAVEd there: the C64's own bytes.
#[cfg(test)]
mod vice {
    use super::*;

    /// Keys as VICE's keyboard buffer typed them (lower case unshifted,
    /// capitals shifted) to the bytes the editor holds for them.
    fn typed(text: &str) -> Vec<Vec<u8>> {
        text.lines()
            .map(|l| {
                l.bytes()
                    .map(|b| match b {
                        b'a'..=b'z' => b - 0x20,
                        b'A'..=b'Z' => b | 0x80,
                        _ => b,
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn c64_crunch_matches_the_rom() {
        let file = include_bytes!("../tests/data/c64-crunch.prg");
        let text = include_str!("../tests/data/c64-crunch.txt");
        let e = encode(&BASIC_2, &typed(text), &FileMeta::default(), 0x0801);
        assert_eq!(e.bytes, file);
        // Opened and saved again: the same bytes.
        let d = decode(&BASIC_2, file).unwrap();
        assert_eq!(encode(&BASIC_2, &d.lines, &d.meta, 0x0801).bytes, file);
    }

    #[test]
    fn vic20_unexpanded() {
        let file = include_bytes!("../tests/data/vic20-crunch.prg");
        let text = include_str!("../tests/data/vic20-crunch.txt");
        let e = encode(&BASIC_2, &typed(text), &FileMeta::default(), 0x1001);
        assert_eq!(e.bytes, file);
        let f = CbmBasic {
            dialect: &BASIC_2,
            memory: &C64_MEMORY,
        };
        assert_eq!(f.system_for(file, "c64"), Some("vic20"));
    }

    #[test]
    fn simons_basic_matches_the_cartridge() {
        let file = include_bytes!("../tests/data/c64-simons.prg");
        let text = include_str!("../tests/data/c64-simons.txt");
        let e = encode(&SIMONS_BASIC, &typed(text), &FileMeta::default(), 0x0801);
        assert_eq!(e.bytes, file);
        // Listed as typed (DISAPA's colons and all), and saved again: the
        // same bytes.
        let d = decode(&SIMONS_BASIC, file).unwrap();
        assert_eq!(d.lines[6], b"70 DISAPA:PRINT");
        assert_eq!(d.lines[4], b"50 PLOTX=1:A@HIRES");
        assert_eq!(encode(&SIMONS_BASIC, &d.lines, &d.meta, 0x0801).bytes, file);
        // Recognised as Simons' BASIC; a plain program isn't.
        let simons = CbmBasic {
            dialect: &SIMONS_BASIC,
            memory: &SIMONS_MEMORY,
        };
        assert!(simons.detect(file));
        assert!(!simons.detect(include_bytes!("../tests/data/c64-crunch.prg")));
    }

    #[test]
    fn c128_basic7_matches_the_rom() {
        let file = include_bytes!("../tests/data/c128-basic7.prg");
        let text = include_str!("../tests/data/c128-basic7.txt");
        let e = encode(&BASIC_7, &typed(text), &FileMeta::default(), 0x1C01);
        assert_eq!(e.bytes, file);
        let d = decode(&BASIC_7, file).unwrap();
        assert_eq!(d.lines[1], b"20 BANK 15:PLAY \"CDEG\":SPRITE 1,1");
        assert_eq!(d.lines[2], b"30 A=POT(1) XOR 3:DO WHILE A<5:A=A+1:LOOP");
        assert_eq!(encode(&BASIC_7, &d.lines, &d.meta, 0x1C01).bytes, file);
        let f = CbmBasic {
            dialect: &BASIC_2,
            memory: &C64_MEMORY,
        };
        assert_eq!(f.system_for(file, "c64"), Some("c128"));
    }

    #[test]
    fn pet_basic4() {
        let file = include_bytes!("../tests/data/pet-basic4.prg");
        let text = include_str!("../tests/data/pet-basic4.txt");
        let e = encode(&BASIC_4, &typed(text), &FileMeta::default(), 0x0401);
        assert_eq!(e.bytes, file);
        let d = decode(&BASIC_4, file).unwrap();
        assert_eq!(d.lines[0], b"10 DIRECTORY:DLOAD\"X\"");
        assert_eq!(encode(&BASIC_4, &d.lines, &d.meta, 0x0401).bytes, file);
        // Its disk commands make it a PET program, from any system.
        let f = CbmBasic {
            dialect: &BASIC_2,
            memory: &C64_MEMORY,
        };
        assert_eq!(f.system_for(file, "vic20"), Some("pet"));
    }
}
