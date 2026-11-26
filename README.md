# Edit CBM

A text and BASIC editor for Commodore computer files (C64 first; the
VIC-20, PET and C128 to follow), built from
[Edit Atari](https://github.com/Calab/edit_atari).

> [!WARNING]
> **Use at your own risk. There is no warranty.** You are responsible for
> your own files: **keep backups**. The authors are **not responsible for
> lost or damaged files, disk images or data** (see `LICENSE`, sections
> 15 and 16).

**Early development.** So far it opens, edits and saves:

- PETSCII text (`.SEQ`) and tokenized BASIC programs (`.PRG`): BASIC 2.0
  (C64, VIC-20, PET), BASIC 4.0 and 1.0 (PET) and Simons' BASIC (C64),
  stored exactly as typing them in on the machine would;
- files on D64, D71 and D81 disk images (File > Open from Image, Save to
  Image, New Image);

in the screen of a C64, VIC-20, PET or PET 8032 (the System menu
switches): the machine's width, colours, keyboard and its Graphics and
Mixed case character sets (F7 or Shift+C= switches between them).

## Building

Install Rust (https://rustup.rs), then in this folder:

```
cargo run --release -p edit-cbm -- [FILE]
```

SDL2 is built in, so on Linux it needs a C compiler, CMake and the X11
or Wayland development packages (see `.github/workflows/build.yml`).

## Character sets

No Commodore character set is built in yet, so the characters are drawn
as boxes. For development, put the character ROMs (4096 bytes each) in
`fonts-dev`, beside the program or in the folder above
it, for example from VICE:

```
mkdir -p fonts-dev
cp /usr/share/vice/C64/chargen-901225-01.bin fonts-dev/c64-chargen.bin
cp /usr/share/vice/VIC20/chargen-901460-03.bin fonts-dev/vic20-chargen.bin
```

`fonts-dev` is git-ignored: the ROM is Commodore's and must not be
committed or built into the program.

## Licence

GPL-3.0-or-later; see `LICENSE`. It comes with no warranty: see the
warning at the top of this file.
