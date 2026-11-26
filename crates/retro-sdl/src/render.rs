//! Drawing the character grid with a glyph-atlas texture.

use retro_core::Machine;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

/// Colours for one machine's look.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: (u8, u8, u8),
    pub foreground: (u8, u8, u8),
    pub border: (u8, u8, u8),
    /// Text past the line-length warning column.
    pub warning: (u8, u8, u8),
    /// Selected text: background and character colours. Distinct from
    /// inverse video so selections can't be mistaken for inverse text.
    pub selection: (u8, u8, u8),
    pub selection_text: (u8, u8, u8),
    /// Control codes when Highlight Codes is on: a colour the machine
    /// doesn't have, so they stand out.
    pub codes: (u8, u8, u8),
}

/// A machine's selectable colours as a grid of hues x luminances (the
/// Atari's SETCOLOR model; a machine with a plain list of colours can use
/// one luminance).
#[derive(Clone, Copy)]
pub struct ColorModel {
    pub hues: u8,
    /// How many of the hues text and the border can use (the VIC-20's
    /// characters and border have 8 of its 16 colours).
    pub text_hues: u8,
    pub border_hues: u8,
    pub lums: u8,
    /// How luminance is numbered for display (2 on the Atari: 0, 2 ... 14).
    pub lum_step: u8,
    pub rgb: fn(u8, u8) -> (u8, u8, u8),
    /// The colour's name in the machine's terms, e.g. "$94".
    pub label: fn(u8, u8) -> String,
    pub default_text: (u8, u8),
    pub default_background: (u8, u8),
    pub default_border: (u8, u8),
}

pub fn rgb((r, g, b): (u8, u8, u8)) -> Color {
    Color::RGB(r, g, b)
}

/// Two 16x16 grids of 8x8 glyphs, white on transparent, tinted when
/// drawn: the character set in use, then below it the machine's other set
/// (for the C128's 80-column screen, where each character has its own).
pub fn build_atlas<'a>(
    creator: &'a TextureCreator<WindowContext>,
    machine: &dyn Machine,
) -> Result<Texture<'a>, String> {
    let mut tex = creator
        .create_texture_static(PixelFormatEnum::RGBA32, 128, 256)
        .map_err(|e| e.to_string())?;
    let mut px = vec![0u8; 128 * 256 * 4];
    let other = (machine.charset() + 1) % machine.charsets().len().max(1);
    for half in 0..2 {
        for code in 0..=255u8 {
            let g = if half == 0 {
                machine.glyph(code)
            } else {
                machine.charset_glyph(other, code)
            };
            let (gx, gy) = (
                (code as usize % 16) * 8,
                half * 128 + (code as usize / 16) * 8,
            );
            for (row, bits) in g.iter().enumerate() {
                for col in 0..8 {
                    if bits & (0x80 >> col) != 0 {
                        let i = ((gy + row) * 128 + gx + col) * 4;
                        px[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }
        }
    }
    tex.update(None, &px, 128 * 4).map_err(|e| e.to_string())?;
    tex.set_blend_mode(BlendMode::BLEND);
    Ok(tex)
}

/// Draws cells on a grid of `cell_w` x `cell` pixels, offset by
/// `origin_x`. (Cells are square except for 80-column screens, whose
/// characters are half as wide.)
pub struct Grid<'c, 't> {
    pub canvas: &'c mut Canvas<Window>,
    pub atlas: &'c mut Texture<'t>,
    /// Cell height.
    pub cell: u32,
    pub cell_w: u32,
    pub origin_x: i32,
    /// Y pixel of row 0 (below the menu bar).
    pub origin_y: i32,
}

impl Grid<'_, '_> {
    fn rect(&self, col: usize, row: usize, cols: usize) -> Rect {
        Rect::new(
            self.origin_x + (col as u32 * self.cell_w) as i32,
            self.origin_y + (row as u32 * self.cell) as i32,
            self.cell_w * cols as u32,
            self.cell,
        )
    }

    /// Underline the cell at (col, row): its bottom pixel row.
    pub fn underline(&mut self, col: usize, row: usize, color: Color) {
        let r = self.rect(col, row, 1);
        let h = (self.cell / 8).max(1);
        self.canvas.set_draw_color(color);
        let _ = self.canvas.fill_rect(Rect::new(
            r.x(),
            r.y() + (self.cell - h) as i32,
            r.width(),
            h,
        ));
    }

    pub fn fill(&mut self, col: usize, row: usize, cols: usize, color: Color) {
        let r = self.rect(col, row, cols);
        self.canvas.set_draw_color(color);
        let _ = self.canvas.fill_rect(r);
    }

    /// Draw codes starting at (col, row) in `fg` (background untouched).
    pub fn codes(&mut self, col: usize, row: usize, codes: &[u8], fg: Color) {
        self.codes_in(col, row, codes, fg, false);
    }

    /// The same, from the machine's other character set if `other`.
    pub fn codes_in(&mut self, col: usize, row: usize, codes: &[u8], fg: Color, other: bool) {
        self.atlas.set_color_mod(fg.r, fg.g, fg.b);
        let top = if other { 128 } else { 0 };
        for (i, &code) in codes.iter().enumerate() {
            let src = Rect::new((code as i32 % 16) * 8, top + (code as i32 / 16) * 8, 8, 8);
            let dst = self.rect(col + i, row, 1);
            let _ = self.canvas.copy(self.atlas, src, dst);
        }
    }
}
