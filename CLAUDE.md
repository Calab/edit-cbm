# Edit CBM: notes for Claude

A text and BASIC editor for Commodore computers (PET, VIC-20, C64, C128;
maybe C16/Plus4 later), built from Edit Atari (`~/Coding/atari_editor`,
github.com/Calab/edit_atari). Started 2026-11-25. GPL-3.0-or-later, as
Edit Atari. Repository: github.com/Calab/edit-cbm (Steve creates it).

**Start by reading `tmp/HANDOFF.md`** (where the work stands), then
`tmp/PLAN.md` and `tmp/SYSTEMS.md` (the agreed plan and every answer
Steve has given).

## Working with Steve
- Steve Norton, the author: expert in the 8-bit machines, new to Rust;
  explain cargo steps plainly. VS Code on Ubuntu.
- **Agree a plan before building anything new.** Write it in `tmp/`, ask,
  wait for answers. (Starting too soon once was a problem.)
- Commit only when asked; end messages with the Co-Authored-By line given
  in the session. **Never push**: Steve pushes himself. `gh` isn't
  installed.
- Files for Steve to review go in this project's `tmp/` (git-ignored),
  not the session scratchpad.
- **Names:** "Edit CBM" wherever people read it; dashes, not
  underscores, in every internal name (packages `retro-core`,
  `retro-sdl`, `edit-cbm`; executable and config folder `edit-cbm`).
  Rust code still says `retro_core` etc.; Cargo maps that itself.
- **Mimic the real Commodore machines** as closely as is practical, and
  check against VICE (x64sc, xvic, xpet, x128, petcat, c1541 are
  installed; ROMs in `/usr/local/share/vice`).
- **Follow Edit Atari's approach** unless there's a reason not to. If it
  shows a shortcoming, note it for Steve (he updates Edit Atari himself).
- CI should pass `cargo fmt --check`, `cargo clippy --workspace
  --all-targets -- -D warnings` and `cargo test --workspace`.

## Related projects
- **cbm-disk** (`~/Coding/cbm-disk`, github.com/Calab/cbm-disk): Steve's
  crate for D64/D71/D81 images, built for this editor. Link it exactly as
  Edit Atari links atari-disk: a git dependency (Cargo.lock pins a
  commit; `cargo update -p cbm-disk` after Steve pushes a change; pin to
  a tag at release). Its `src/volume.rs` documents the API.
- **Edit Atari** keeps its own `retro_core`/`retro_sdl`; this project has
  its own copies, maintained separately (Steve's choice). Don't change
  Edit Atari from here.
