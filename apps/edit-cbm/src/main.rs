//! Edit CBM: a text and BASIC editor for Commodore computer files.
//!
//! Usage: edit-cbm [FILE]

// Don't open a console window alongside the editor on Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use cbm::{Cbm, Model, keyboard::CbmKeyboard, model};
use retro_sdl::{
    AppSpec, System,
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

const C64_HELP: &[&str] = &[
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
    "Shift+C= (or F7) switches between the Graphics",
    "and Mixed case character sets. In quotes, or with",
    "CTRL held too, it also types the code that",
    "switches: {$0E} to Mixed case (CTRL+N types it",
    "too), {$8E} to Graphics.",
];

/// The VIC-20's keys: as the C64's, except CTRL and C=+digits.
const VIC20_HELP: &[&str] = &[
    "VIC-20 KEYS",
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
    "                GRN BLU YEL (the VIC-20's eight",
    "                character colours)",
    "CTRL+9          RVS ON",
    "CTRL+0          RVS OFF",
    "CTRL+`          $06 (CTRL+\u{2190})",
    "                Other CTRL keys type nothing,",
    "                as on the VIC-20.",
    "C=+letter       The graphic on the key's left",
    "C=+1 to 8       The shifted digit (! \" # ...)",
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
    "Shift+C= (or F7) switches between the Graphics",
    "and Mixed case character sets. In quotes, or with",
    "CTRL held too, it also types the code that",
    "switches: {$0E} to Mixed case, {$8E} to Graphics.",
    "",
    "Text and border colours are 0-7, as on the",
    "VIC-20; the screen can be any of the 16.",
];

/// The PET's graphics keyboard.
const PET_HELP: &[&str] = &[
    "PET KEYS (THE GRAPHICS KEYBOARD)",
    "",
    "The PET has no CTRL or C= key: Shift with any key",
    "types a graphic. Here typing follows the PC's",
    "keys, and the C= key (left Alt) gives the PET's",
    "shifted graphics.",
    "",
    "a-z             Letters ($41-$5A)",
    "Shift+letter    Graphics, or capitals in Mixed",
    "                case ($C1-$DA)",
    "Alt+key         What Shift does on the PET key",
    "                with that character: Alt+1 the",
    "                graphic on 1, Alt+Shift+1 the one",
    "                on !, Alt+A the same as Shift+A",
    "Alt+Space       Shift+Space ($A0)",
    "`               \u{2190} (left arrow)",
    "^               \u{2191} (up arrow)",
    "\\               \\ (backslash: the PET's $5C)",
    "~               \u{3c0} (pi)",
    "| _ { }         The graphics on \\ - [ ]",
    "",
    "Tab             RVS ON",
    "Shift+Tab       RVS OFF",
    "CTRL+Tab        Next tab stop",
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
    "",
    "Shift+C= (or F7) switches between the Graphics",
    "and Mixed case (business) character sets. In",
    "quotes, or with CTRL held too, it also types the",
    "code that switches: {$0E} to Mixed case, {$8E}",
    "to Graphics (BASIC 4).",
    "",
    "The PET has no colours: green on black. The PET",
    "8032 shows 80 columns of narrower characters.",
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

/// For development until Edit CBM has fonts of its own: the model's
/// character ROM, copied from VICE into `fonts-dev` (never committed,
/// never built in). Without it the placeholder boxes are drawn.
fn load_dev_fonts(m: &mut Cbm) {
    let file = m.model().rom_file;
    let Some(rom) = dev_fonts_dir().and_then(|d| std::fs::read(d.join(file)).ok()) else {
        return;
    };
    if let Err(e) = m.set_builtin_rom(&rom) {
        eprintln!(
            "edit-cbm: {}: {e}",
            Path::new(DEV_FONTS).join(file).display()
        );
    }
}

/// The editor's system for a Commodore model.
fn system(model: &'static Model, help: &'static [&'static str]) -> System {
    let mut machine = Cbm::new(model);
    load_dev_fonts(&mut machine);
    let rgb = model.rgb;
    let c = |n: u8| (n, 0);
    System {
        id: model.id,
        machine: Box::new(machine),
        keyboard: Box::new(CbmKeyboard::new(model)),
        // The model's screen at power-on.
        palette: Palette {
            background: rgb(model.background, 0),
            foreground: rgb(model.text, 0),
            border: rgb(model.border, 0),
            warning: (0xFF, 0xA8, 0x40),
            selection: (0xF2, 0xC8, 0x4B),
            selection_text: (0x10, 0x1C, 0x40),
            // Magenta: in neither the C64's palette nor the VIC-20's.
            codes: (0xFF, 0x00, 0xFF),
        },
        // The PET's green on black can't be changed.
        colors: (!model.fixed_colours).then_some(ColorModel {
            hues: model.colours.len() as u8,
            text_hues: model.character_colours,
            border_hues: model.border_colours,
            lums: 1,
            lum_step: 1,
            rgb,
            label: model.label,
            default_text: c(model.text),
            default_background: c(model.background),
            default_border: c(model.border),
        }),
        help,
        line_warning: model.logical_line,
    }
}

fn main() -> ExitCode {
    let file = std::env::args_os().nth(1).map(PathBuf::from);
    let spec = AppSpec {
        title: "Edit CBM",
        app_id: "edit-cbm",
        systems: vec![
            system(&model::C64, C64_HELP),
            system(&model::VIC20, VIC20_HELP),
            system(&model::PET, PET_HELP),
            system(&model::PET8032, PET_HELP),
        ],
        disks: None,
        icon: b'C',
        file_types: OPEN_FILTERS,
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
