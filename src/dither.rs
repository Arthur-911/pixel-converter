use crate::lut::Lut3D;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitherMode {
    None,
    Bayer4,
    Bayer8,
    FloydSteinberg,
    Atkinson,
}

impl DitherMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" | "flat" => Some(Self::None),
            "bayer4" | "bayer-4" => Some(Self::Bayer4),
            "bayer8" | "bayer-8" | "bayer" => Some(Self::Bayer8),
            "floyd" | "floyd-steinberg" | "fs" => Some(Self::FloydSteinberg),
            "atkinson" | "atk" => Some(Self::Atkinson),
            _ => None,
        }
    }
}

// 4x4 Bayer matrix normalized to 0..15
const BAYER_4X4: [[f32; 4]; 4] = [
    [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
    [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
    [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
    [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
];

// 8x8 Bayer matrix normalized to 0..63
const BAYER_8X8: [[f32; 8]; 8] = [
    [ 0.0/64.0, 32.0/64.0,  8.0/64.0, 40.0/64.0,  2.0/64.0, 34.0/64.0, 10.0/64.0, 42.0/64.0],
    [48.0/64.0, 16.0/64.0, 56.0/64.0, 24.0/64.0, 50.0/64.0, 18.0/64.0, 58.0/64.0, 26.0/64.0],
    [12.0/64.0, 44.0/64.0,  4.0/64.0, 36.0/64.0, 14.0/64.0, 46.0/64.0,  6.0/64.0, 38.0/64.0],
    [60.0/64.0, 28.0/64.0, 52.0/64.0, 20.0/64.0, 62.0/64.0, 30.0/64.0, 54.0/64.0, 22.0/64.0],
    [ 3.0/64.0, 35.0/64.0, 11.0/64.0, 43.0/64.0,  1.0/64.0, 33.0/64.0,  9.0/64.0, 41.0/64.0],
    [51.0/64.0, 19.0/64.0, 59.0/64.0, 27.0/64.0, 49.0/64.0, 17.0/64.0, 57.0/64.0, 25.0/64.0],
    [15.0/64.0, 47.0/64.0,  7.0/64.0, 39.0/64.0, 13.0/64.0, 45.0/64.0,  5.0/64.0, 37.0/64.0],
    [63.0/64.0, 31.0/64.0, 55.0/64.0, 23.0/64.0, 61.0/64.0, 29.0/64.0, 53.0/64.0, 21.0/64.0],
];

/// Applies dithering and quantization to a flat RGB buffer (width * height * 3 bytes).
pub fn apply_dithering(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    lut: &Lut3D,
    mode: DitherMode,
    strength: f32,
) {
    match mode {
        DitherMode::None => {
            // Embarrassingly parallel direct LUT lookup
            pixels
                .par_chunks_exact_mut(3)
                .for_each(|chunk| {
                    let quantized = lut.lookup(chunk[0], chunk[1], chunk[2]);
                    chunk.copy_from_slice(&quantized);
                });
        }
        DitherMode::Bayer4 => {
            let spread = 48.0 * strength;
            pixels
                .par_chunks_exact_mut(width * 3)
                .enumerate()
                .for_each(|(y, row)| {
                    let by = y % 4;
                    for x in 0..width {
                        let bx = x % 4;
                        let threshold = (BAYER_4X4[by][bx] - 0.5) * spread;

                        let idx = x * 3;
                        let r = (row[idx] as f32 + threshold).clamp(0.0, 255.0) as u8;
                        let g = (row[idx + 1] as f32 + threshold).clamp(0.0, 255.0) as u8;
                        let b = (row[idx + 2] as f32 + threshold).clamp(0.0, 255.0) as u8;

                        let q = lut.lookup(r, g, b);
                        row[idx] = q[0];
                        row[idx + 1] = q[1];
                        row[idx + 2] = q[2];
                    }
                });
        }
        DitherMode::Bayer8 => {
            let spread = 48.0 * strength;
            pixels
                .par_chunks_exact_mut(width * 3)
                .enumerate()
                .for_each(|(y, row)| {
                    let by = y % 8;
                    for x in 0..width {
                        let bx = x % 8;
                        let threshold = (BAYER_8X8[by][bx] - 0.5) * spread;

                        let idx = x * 3;
                        let r = (row[idx] as f32 + threshold).clamp(0.0, 255.0) as u8;
                        let g = (row[idx + 1] as f32 + threshold).clamp(0.0, 255.0) as u8;
                        let b = (row[idx + 2] as f32 + threshold).clamp(0.0, 255.0) as u8;

                        let q = lut.lookup(r, g, b);
                        row[idx] = q[0];
                        row[idx + 1] = q[1];
                        row[idx + 2] = q[2];
                    }
                });
        }
        DitherMode::FloydSteinberg => {
            apply_floyd_steinberg(pixels, width, height, lut, strength);
        }
        DitherMode::Atkinson => {
            apply_atkinson(pixels, width, height, lut, strength);
        }
    }
}

/// Fast scanline-based Floyd-Steinberg error diffusion using integer arithmetic
fn apply_floyd_steinberg(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    lut: &Lut3D,
    strength: f32,
) {
    // We maintain error rows: current row and next row errors in fixed-point (scaled by 16)
    let mut err_curr = vec![[0i32; 3]; width + 2];
    let mut err_next = vec![[0i32; 3]; width + 2];

    let factor = (strength * 16.0).round() as i32;

    for y in 0..height {
        let row_start = y * width * 3;

        for x in 0..width {
            let px_idx = row_start + x * 3;
            let err_x = x + 1; // 1-indexed to avoid boundary checks

            // Add diffused error to current pixel
            let r_in = pixels[px_idx] as i32 + (err_curr[err_x][0] * factor) / 256;
            let g_in = pixels[px_idx + 1] as i32 + (err_curr[err_x][1] * factor) / 256;
            let b_in = pixels[px_idx + 2] as i32 + (err_curr[err_x][2] * factor) / 256;

            let r_clamped = r_in.clamp(0, 255) as u8;
            let g_clamped = g_in.clamp(0, 255) as u8;
            let b_clamped = b_in.clamp(0, 255) as u8;

            let q = lut.lookup(r_clamped, g_clamped, b_clamped);
            pixels[px_idx] = q[0];
            pixels[px_idx + 1] = q[1];
            pixels[px_idx + 2] = q[2];

            // Compute quantization error
            let err_r = r_in - q[0] as i32;
            let err_g = g_in - q[1] as i32;
            let err_b = b_in - q[2] as i32;

            // Distribute error: 7/16 right, 3/16 down-left, 5/16 down, 1/16 down-right
            err_curr[err_x + 1][0] += (err_r * 7) / 16;
            err_curr[err_x + 1][1] += (err_g * 7) / 16;
            err_curr[err_x + 1][2] += (err_b * 7) / 16;

            err_next[err_x - 1][0] += (err_r * 3) / 16;
            err_next[err_x - 1][1] += (err_g * 3) / 16;
            err_next[err_x - 1][2] += (err_b * 3) / 16;

            err_next[err_x][0] += (err_r * 5) / 16;
            err_next[err_x][1] += (err_g * 5) / 16;
            err_next[err_x][2] += (err_b * 5) / 16;

            err_next[err_x + 1][0] += (err_r * 1) / 16;
            err_next[err_x + 1][1] += (err_g * 1) / 16;
            err_next[err_x + 1][2] += (err_b * 1) / 16;
        }

        // Shift error row buffers
        std::mem::swap(&mut err_curr, &mut err_next);
        err_next.fill([0i32; 3]);
    }
}

/// Atkinson error diffusion: diffuses 1/8 to 6 neighbors (retains 25% of error for crisp edges)
fn apply_atkinson(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    lut: &Lut3D,
    strength: f32,
) {
    let mut err_curr = vec![[0i32; 3]; width + 3];
    let mut err_next = vec![[0i32; 3]; width + 3];
    let mut err_next2 = vec![[0i32; 3]; width + 3];

    let factor = (strength * 16.0).round() as i32;

    for y in 0..height {
        let row_start = y * width * 3;

        for x in 0..width {
            let px_idx = row_start + x * 3;
            let err_x = x + 1;

            let r_in = pixels[px_idx] as i32 + (err_curr[err_x][0] * factor) / 128;
            let g_in = pixels[px_idx + 1] as i32 + (err_curr[err_x][1] * factor) / 128;
            let b_in = pixels[px_idx + 2] as i32 + (err_curr[err_x][2] * factor) / 128;

            let r_clamped = r_in.clamp(0, 255) as u8;
            let g_clamped = g_in.clamp(0, 255) as u8;
            let b_clamped = b_in.clamp(0, 255) as u8;

            let q = lut.lookup(r_clamped, g_clamped, b_clamped);
            pixels[px_idx] = q[0];
            pixels[px_idx + 1] = q[1];
            pixels[px_idx + 2] = q[2];

            let err_r = (r_in - q[0] as i32) / 8;
            let err_g = (g_in - q[1] as i32) / 8;
            let err_b = (b_in - q[2] as i32) / 8;

            // Atkinson distributes 1/8 to:
            // (x+1, y), (x+2, y), (x-1, y+1), (x, y+1), (x+1, y+1), (x, y+2)
            err_curr[err_x + 1][0] += err_r;
            err_curr[err_x + 1][1] += err_g;
            err_curr[err_x + 1][2] += err_b;

            err_curr[err_x + 2][0] += err_r;
            err_curr[err_x + 2][1] += err_g;
            err_curr[err_x + 2][2] += err_b;

            err_next[err_x - 1][0] += err_r;
            err_next[err_x - 1][1] += err_g;
            err_next[err_x - 1][2] += err_b;

            err_next[err_x][0] += err_r;
            err_next[err_x][1] += err_g;
            err_next[err_x][2] += err_b;

            err_next[err_x + 1][0] += err_r;
            err_next[err_x + 1][1] += err_g;
            err_next[err_x + 1][2] += err_b;

            err_next2[err_x][0] += err_r;
            err_next2[err_x][1] += err_g;
            err_next2[err_x][2] += err_b;
        }

        std::mem::swap(&mut err_curr, &mut err_next);
        std::mem::swap(&mut err_next, &mut err_next2);
        err_next2.fill([0i32; 3]);
    }
}
