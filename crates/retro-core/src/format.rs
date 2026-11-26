//! File formats: how a machine's files are read into lines of machine codes
//! and written back.
//!
//! Each machine lists its formats (see [`crate::Machine::formats`]). A file
//! being opened is read with the first format that recognises it, so
//! specific formats (a tokenized BASIC program with a recognisable header)
//! come first and the machine's plain text format, which accepts anything,
//! comes last. Adding a format means writing one more implementation of
//! [`FileFormat`] and adding it to the machine's list.

use crate::document::Document;
use crate::machine::Decoded;

pub trait FileFormat: Sync {
    /// Name for messages and dialogs, e.g. "Atari BASIC".
    fn name(&self) -> &'static str;

    /// Name for the status bar, if shorter than [`FileFormat::name`].
    fn status_name(&self) -> &'static str {
        self.name()
    }

    /// The usual file-name extension, without the dot, e.g. "BAS".
    fn extension(&self) -> &'static str;

    /// File-name extensions (without the dot, any case) that name this
    /// format specifically, e.g. "BXL". A file with one of these is read in
    /// this format if it can be.
    fn extensions(&self) -> &'static [&'static str] {
        &[]
    }

    /// Other extensions this format is commonly saved under but shares
    /// with other formats (`.BAS` for every BASIC dialect). Save As uses
    /// them to offer the format; Open doesn't treat them as a hint.
    fn shared_extensions(&self) -> &'static [&'static str] {
        &[]
    }

    /// Whether `bytes` look like a file in this format. Should be cheap and
    /// strict enough not to claim other formats' files.
    fn detect(&self, bytes: &[u8]) -> bool;

    /// How badly `bytes` fit this format when several formats can read
    /// them (lower is better), or `None` if this format can't. Used to
    /// choose between formats that share a layout, such as BASIC dialects.
    fn misfit(&self, bytes: &[u8]) -> Option<usize> {
        self.detect(bytes).then_some(0)
    }

    /// Whether `bytes` have this format's outward signs (a header, say)
    /// even if [`FileFormat::detect`] rejects them because they're
    /// damaged. Used to explain why such a file opened as plain text.
    fn looks_like(&self, bytes: &[u8]) -> bool {
        self.detect(bytes)
    }

    /// Read a file into lines of machine codes.
    fn decode(&self, bytes: &[u8]) -> Result<Decoded, String>;

    /// The system (its id) a file in this format belongs to, if the file
    /// says (a Commodore program's load address does). `current` is the
    /// id of the system the window shows.
    fn system_for(&self, _bytes: &[u8], _current: &str) -> Option<&'static str> {
        None
    }

    /// The memory set-ups a program in this format can be for, smallest
    /// first (the VIC-20 unexpanded, +3K, +24K). Empty for formats without
    /// a load address. With more than one, a document with no load address
    /// asks which on its first save.
    fn memory(&self) -> &'static [Memory] {
        &[]
    }

    /// The file type it's saved as on a disk with types (`PRG`, `SEQ`),
    /// or empty.
    fn disk_file_type(&self) -> &'static str {
        ""
    }

    /// Whether [`FileFormat::encode`] is available.
    fn can_encode(&self) -> bool {
        false
    }

    /// Write a document in this format.
    fn encode(&self, _doc: &Document) -> Result<Encoded, String> {
        Err(format!("{} files can't be written yet", self.name()))
    }
}

/// A memory set-up a program can be saved for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Memory {
    /// E.g. "VIC-20 +3K".
    pub name: &'static str,
    pub load_address: u16,
    /// BASIC's BYTES FREE at power-on: the most a program can take.
    pub bytes_free: usize,
}

/// A document written in a format.
#[derive(Debug)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    /// Something the user should know before the file is written (lines
    /// that can't be stored as they are, say): the editor shows it and
    /// asks Save Anyway / Cancel.
    pub warning: Option<String>,
}

impl Encoded {
    /// Bytes with nothing to warn about.
    pub fn clean(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            warning: None,
        }
    }
}

/// The format to read a file in:
///
/// 1. one whose [`FileFormat::extensions`] include the file's extension,
///    if it can read the file;
/// 2. otherwise the best fit ([`FileFormat::misfit`]), the earliest in
///    `formats` winning a tie;
/// 3. otherwise the last format (the machine's text format).
pub fn detect(
    formats: &[&'static dyn FileFormat],
    bytes: &[u8],
    extension: Option<&str>,
) -> &'static dyn FileFormat {
    let text = *formats.last().expect("a machine has at least one format");
    let specific = formats.iter().copied().find(|f| {
        extension.is_some_and(|e| f.extensions().iter().any(|x| x.eq_ignore_ascii_case(e)))
            && f.detect(bytes)
    });
    specific
        .or_else(|| {
            formats[..formats.len() - 1]
                .iter()
                .copied()
                .filter_map(|f| f.misfit(bytes).map(|m| (m, f)))
                .min_by_key(|&(m, _)| m) // min_by_key keeps the first of equals
                .map(|(_, f)| f)
        })
        .unwrap_or(text)
}

/// Whether two format references are the same format. (Compared by name:
/// formats are usually zero-sized, so their addresses can coincide.)
pub fn same(a: &dyn FileFormat, b: &dyn FileFormat) -> bool {
    a.name() == b.name()
}
