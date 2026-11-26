# Edit CBM

A text and BASIC editor for Commodore computer files (C64 first; the
VIC-20, PET and C128 to follow), built from
[Edit Atari](https://github.com/Calab/edit_atari).

**Early development.** So far it opens, edits and saves PETSCII text
(`.SEQ`) in a C64 screen: 40 columns, the C64's colours, and its
Graphics and Mixed case character sets (F7 switches between them).

## Building

Install Rust (https://rustup.rs), then in this folder:

```
cargo run --release -p edit-cbm -- [FILE]
```

SDL2 is built in, so on Linux it needs a C compiler, CMake and the X11
or Wayland development packages (see `.github/workflows/build.yml`).

## Character sets

No Commodore character set is built in yet, so the characters are drawn
as boxes. For development, put a C64 character ROM (4096 bytes) at
`fonts-dev/c64-chargen.bin`, beside the program or in the folder above
it, for example from VICE:

```
mkdir -p fonts-dev
cp /usr/share/vice/C64/chargen-901225-01.bin fonts-dev/c64-chargen.bin
```

`fonts-dev` is git-ignored: the ROM is Commodore's and must not be
committed or built into the program.

## Licence

GPL-3.0-or-later; see `LICENSE`.
