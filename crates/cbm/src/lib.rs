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

use basic::{
    BASIC_1, BASIC_2, BASIC_4, BASIC_7, C64_MEMORY, C128_MEMORY, CbmBasic, PET_MEMORY,
    SIMONS_BASIC, SIMONS_MEMORY, VIC20_MEMORY,
};

/// Each model's file formats, most specific first; plain PETSCII text
/// last. BASIC programs are saved at the model's usual load address
/// unless the document came with its own.
/// Simons' BASIC first: it claims only programs using its keywords.
pub static C64_FORMATS: &[&dyn FileFormat] = &[
    &CbmBasic {
        dialect: &SIMONS_BASIC,
        memory: &SIMONS_MEMORY,
    },
    &CbmBasic {
        dialect: &BASIC_2,
        memory: &C64_MEMORY,
    },
    &petscii::PetsciiText,
];
pub static VIC20_FORMATS: &[&dyn FileFormat] = &[
    &CbmBasic {
        dialect: &BASIC_2,
        memory: &VIC20_MEMORY,
    },
    &petscii::PetsciiText,
];
/// The C128: BASIC 7.0 and text.
pub static C128_FORMATS: &[&dyn FileFormat] = &[
    &CbmBasic {
        dialect: &BASIC_7,
        memory: &C128_MEMORY,
    },
    &petscii::PetsciiText,
];

/// BASIC 4 first: a program all three can read opens as BASIC 4
/// (Reopen As picks another).
pub static PET_FORMATS: &[&dyn FileFormat] = &[
    &CbmBasic {
        dialect: &BASIC_4,
        memory: &PET_MEMORY,
    },
    &CbmBasic {
        dialect: &BASIC_2,
        memory: &PET_MEMORY,
    },
    &CbmBasic {
        dialect: &BASIC_1,
        memory: &PET_MEMORY,
    },
    &petscii::PetsciiText,
];
