//! Edit CBM: a text and BASIC editor for Commodore computer files.
//!
//! Usage: edit-cbm [FILE]

// Don't open a console window alongside the editor on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cbm::{C64, keyboard::CbmKeyboard, palette};
use retro_sdl::{
    AppSpec,
    render::{ColorModel, Palette},
};

/// The Open and Save dialogs' file types, one per language.
const OPEN_FILTERS: &[(&str, &[&str])] = &[("PETSCII text", &["SEQ"]), ("Text", &["TXT"])];

/// Help > File Types: the extensions for each language.
const FILE_TYPES: &[&str] = &[
    "FILE TYPES",
    "",
    ".SEQ  PETSCII text: a carriage return ($0D)",
    "      ends each line",
    ".TXT  The same",
    "",
    "Opening reads any file as PETSCII text, byte for",
    "byte. Tokenized BASIC programs (.PRG) come later.",
    "",
    "EXPORT AND IMPORT",
    "",
    "File > Export writes PC text; File > Import reads",
    "a PC text file as PETSCII. Characters with no",
    "Commodore equivalent become ?.",
];

const HELP: &[&str] = &[
    "C64 KEYS",
    "",
    "CTRL is right Ctrl and C= is left Alt; either can",
    "be changed in Options > Preferences.",
    "",
    "a-z             Letters ($41-$5A)",
    "Shift+letter    Graphics, or capitals in Mixed",
    "                case ($C1-$DA)",
    "`               \u{2190} (left arrow)",
    "^               \u{2191} (up arrow)",
    "\\               \u{a3} (pound)",
    "~               \u{3c0} (pi)",
    "| and _         Shift+\u{a3} and Shift+- graphics",
    "",
    "CTRL+1 to 8     Colours: BLK WHT RED CYN PUR",
    "                GRN BLU YEL",
    "CTRL+9          RVS ON",
    "CTRL+0          RVS OFF",
    "CTRL+letter     Control codes $01-$1A (CTRL+N",
    "                switches to Mixed case);",
    "                CTRL+M is RETURN",
    "C=+1 to 8       Colours: ORN BRN LRD GR1 GR2",
    "                LGR LBL GR3",
    "C=+letter       The graphic on the key's left",
    "C=+-            The graphic on -",
    "C=+Shift+= 2 8  The graphics on + @ * (the",
    "                PC keys showing them)",
    "",
    "CTRL with an editing key inserts its code",
    "instead of acting:",
    "CTRL+arrows     {UP} {DOWN} {LEFT} {RIGHT}",
    "CTRL+Home       {HOME}; with Shift, {CLR}",
    "CTRL+Insert     {INST}",
    "CTRL+Delete     {DEL}",
    "CTRL+Enter      Shift+RETURN",
    "CTRL+Esc        RUN/STOP; with Shift, Shift+",
    "                RUN/STOP",
    "CTRL+F1 to F8   {F1} to {F8}",
    "",
    "Control codes show as reversed characters, as",
    "in a listing on the C64.",
    "",
    "Keys with no C64 equivalent ({ }) are ignored.",
    "",
    "F7 switches between the Graphics and Mixed case",
    "character sets, as Shift+C= does on the C64.",
];

/// Copyright notice details, for About and Licensing.
const COPYRIGHT_YEAR: &str = "2026";
const COPYRIGHT_HOLDER: &str = "Steve Norton";

/// Where development builds look for character ROMs (see `load_dev_fonts`).
const DEV_FONTS: &str = "fonts-dev";

/// The authors from Cargo.toml, as "A and B" (or "A, B and C").
fn authors() -> String {
    let names: Vec<&str> = env!("CARGO_PKG_AUTHORS").split(':').collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => names.concat(),
    }
}

/// Program details for Help > About.
fn about() -> String {
    format!(
        "Edit CBM\n\
         Version {}\n\
         \n\
         A text and BASIC editor for Commodore computer files.\n\
         \n\
         Author: {}\n\
         Copyright \u{a9} {COPYRIGHT_YEAR} {COPYRIGHT_HOLDER}\n\
         \n\
         Licensed under the GNU GPL, version 3 or later.\n\
         See Help > Licensing for details.\n\
         \n\
         Source code:\n\
         {}",
        env!("CARGO_PKG_VERSION"),
        authors(),
        env!("CARGO_PKG_REPOSITORY"),
    )
}

/// Everything in the program and its licences, for Help > Licensing.
fn licensing() -> String {
    format!(
        "EDIT CBM\n\
         Copyright \u{a9} {COPYRIGHT_YEAR} {COPYRIGHT_HOLDER}\n\
         \n\
         This program is free software: you can redistribute it and/or modify \
         it under the terms of the GNU General Public License as published by \
         the Free Software Foundation, either version 3 of the License, or (at \
         your option) any later version. It is distributed in the hope that it \
         will be useful, but WITHOUT ANY WARRANTY; without even the implied \
         warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. The \
         full licence text is at the end of this page.\n\
         \n\
         The source code is available at:\n\
         {}\n\
         \n\
         \n\
         COMMODORE CHARACTER SETS\n\
         \n\
         No Commodore character set is built into this program yet. \
         Characters are drawn as boxes unless a character ROM file is \
         found in a \"{DEV_FONTS}\" folder beside the program, or is chosen \
         in Options > Fonts.\n\
         \n\
         INTERFACE FONT\n\
         \n\
         The menus and this page use the system's own fonts. When they can't \
         be found, the built-in Noto Sans and Noto Sans Mono are used, under \
         the SIL Open Font License 1.1:\n\
         \n\
         {}\n\
         \n\
         LIBRARIES BUILT INTO THIS PROGRAM\n\
         \n\
         {}\n\
         \n\
         GNU GENERAL PUBLIC LICENSE\n\
         \n\
         {}",
        env!("CARGO_PKG_REPOSITORY"),
        retro_sdl::ui_font::FALLBACK_LICENSE,
        include_str!("../THIRD_PARTY.txt"),
        include_str!("../../../LICENSE"),
    )
}

/// The `fonts-dev` folder: beside the program, or in a folder above it
/// (the workspace, for `target/debug/edit-cbm`).
fn dev_fonts_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    exe.ancestors()
        .skip(1)
        .take(4)
        .map(|d| d.join(DEV_FONTS))
        .find(|d| d.is_dir())
}

/// For development until Edit CBM has fonts of its own: the C64's
/// character ROM, copied from VICE into `fonts-dev/c64-chargen.bin`
/// (never committed, never built in). Without it the placeholder boxes
/// are drawn.
fn load_dev_fonts(c64: &mut C64) {
    let Some(rom) = dev_fonts_dir().and_then(|d| std::fs::read(d.join("c64-chargen.bin")).ok())
    else {
        return;
    };
    if let Err(e) = c64.set_builtin_rom(&rom) {
        eprintln!(
            "edit-cbm: {}: {e}",
            Path::new(DEV_FONTS).join("c64-chargen.bin").display()
        );
    }
}

fn main() -> ExitCode {
    let file = std::env::args_os().nth(1).map(PathBuf::from);
    let mut c64 = C64::default();
    load_dev_fonts(&mut c64);
    let rgb = |c: (u8, u8)| palette::rgb(c.0, c.1);
    let spec = AppSpec {
        title: "Edit CBM",
        app_id: "edit-cbm",
        machine: Box::new(c64),
        disks: None,
        keyboard: Box::new(CbmKeyboard::new()),
        // The C64 at power-on: light blue on blue, light blue border.
        palette: Palette {
            background: rgb(palette::DEFAULT_BACKGROUND),
            foreground: rgb(palette::DEFAULT_TEXT),
            border: rgb(palette::BORDER),
            warning: (0xFF, 0xA8, 0x40),
            selection: (0xF2, 0xC8, 0x4B),
            selection_text: (0x10, 0x1C, 0x40),
        },
        icon: b'C',
        file_types: OPEN_FILTERS,
        colors: Some(ColorModel {
            hues: palette::HUES,
            lums: palette::LUMS,
            lum_step: 1,
            rgb: palette::rgb,
            label: palette::label,
            default_text: palette::DEFAULT_TEXT,
            default_background: palette::DEFAULT_BACKGROUND,
            default_border: palette::BORDER,
        }),
        help: HELP,
        file_types_help: FILE_TYPES,
        about: about(),
        licensing: licensing(),
    };
    match retro_sdl::run(spec, file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("edit-cbm: {e}");
            ExitCode::FAILURE
        }
    }
}
