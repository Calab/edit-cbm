//! Commodore support: PETSCII, the character sets, the models (C64,
//! VIC-20), their colours and keyboard.

pub mod font;
pub mod keyboard;
pub mod machine;
pub mod model;
pub mod palette;
pub mod petscii;

pub use machine::Cbm;
pub use model::Model;

use retro_core::FileFormat;

/// File formats, most specific first; plain PETSCII text last.
static FORMATS: &[&dyn FileFormat] = &[&petscii::PetsciiText];
