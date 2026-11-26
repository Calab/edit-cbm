//! Commodore support: PETSCII, the character sets, the models (C64,
//! VIC-20), their colours and keyboard.

pub mod basic;
pub mod font;
pub mod keyboard;
pub mod machine;
pub mod model;
pub mod palette;
pub mod petscii;

pub use machine::Cbm;
pub use model::Model;

use retro_core::FileFormat;

use basic::{BASIC_2, BASIC_4, CbmBasic};

/// Each model's file formats, most specific first; plain PETSCII text
/// last. BASIC programs are saved at the model's usual load address
/// unless the document came with its own.
pub static C64_FORMATS: &[&dyn FileFormat] = &[
    &CbmBasic {
        dialect: &BASIC_2,
        load: 0x0801,
    },
    &petscii::PetsciiText,
];
pub static VIC20_FORMATS: &[&dyn FileFormat] = &[
    &CbmBasic {
        dialect: &BASIC_2,
        load: 0x1001,
    },
    &petscii::PetsciiText,
];
/// BASIC 4 first: a program both can read opens as BASIC 4.
pub static PET_FORMATS: &[&dyn FileFormat] = &[
    &CbmBasic {
        dialect: &BASIC_4,
        load: 0x0401,
    },
    &CbmBasic {
        dialect: &BASIC_2,
        load: 0x0401,
    },
    &petscii::PetsciiText,
];
