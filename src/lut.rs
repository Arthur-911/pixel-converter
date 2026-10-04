use crate::palette::Palette;

/// A 5-bit 3D Look-Up Table (32x32x32 = 32,768 entries).
/// Memory footprint: 32,768 * 3 bytes = 98,304 bytes (~96 KB).
/// Fits entirely inside CPU L2/L3 cache, enabling O(1) instantaneous color quantization.
pub struct Lut3D {
    table: Vec<[u8; 3]>,
}

impl Lut3D {
    /// Precomputes the 3D LUT from a given palette.
    /// Takes < 1 millisecond to construct.
    pub fn new(palette: &Palette) -> Self {
        if palette.name == "vibrant" || palette.name == "auto" || palette.name == "truecolor" || palette.name == "15bit" {
            return Self::vibrant();
        }

        let mut table = vec![[0, 0, 0]; 32 * 32 * 32];

        for r in 0..32 {
            // Expand 5-bit (0..31) to 8-bit (0..255)
            let r8 = (r * 255 / 31) as u8;
            for g in 0..32 {
                let g8 = (g * 255 / 31) as u8;
                for b in 0..32 {
                    let b8 = (b * 255 / 31) as u8;

                    let nearest = palette.find_nearest([r8, g8, b8]);
                    let index = (r << 10) | (g << 5) | b;
                    table[index] = nearest;
                }
            }
        }

        Self { table }
    }

    /// Fast 15-bit RGB quantization (5 bits per channel, 32,768 retro console colors).
    /// Used by SNES, GBA, and classic arcade hardware.
    pub fn vibrant() -> Self {
        let mut table = vec![[0, 0, 0]; 32 * 32 * 32];
        for r in 0..32 {
            let r8 = (r * 255 / 31) as u8;
            for g in 0..32 {
                let g8 = (g * 255 / 31) as u8;
                for b in 0..32 {
                    let b8 = (b * 255 / 31) as u8;
                    let index = (r << 10) | (g << 5) | b;
                    table[index] = [r8, g8, b8];
                }
            }
        }
        Self { table }
    }

    /// Constant-time O(1) color lookup.
    /// Compiles down to simple bit-shifts and single array indexing.
    #[inline(always)]
    pub fn lookup(&self, r: u8, g: u8, b: u8) -> [u8; 3] {
        let r_idx = (r >> 3) as usize;
        let g_idx = (g >> 3) as usize;
        let b_idx = (b >> 3) as usize;
        let index = (r_idx << 10) | (g_idx << 5) | b_idx;
        // Unchecked or direct slice indexing is optimized away by compiler
        self.table[index]
    }
}
