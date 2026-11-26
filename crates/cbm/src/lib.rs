//! Commodore support: PETSCII, the character sets, the systems (the C64
//! so far), their colours and keyboard.

pub mod c64;
pub mod font;
pub mod keyboard;
pub mod palette;
pub mod petscii;

pub use c64::C64;

use retro_core::FileFormat;

/// File formats, most specific first; plain PETSCII text last.
static FORMATS: &[&dyn FileFormat] = &[&petscii::PetsciiText];
