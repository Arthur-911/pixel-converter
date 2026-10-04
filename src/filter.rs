use image::RgbImage;
use rayon::prelude::*;

/// Ultra-fast parallel integer unsharp mask (3x3 Laplacian edge-enhancement).
/// Uses pure bit-shifts and integer SIMD math with zero floating-point overhead.
pub fn apply_sharpening_int(input: &RgbImage, in_w: usize, in_h: usize, strength: f32) -> RgbImage {
    let raw = input.as_raw();
    let mut sharp_buf = vec![0u8; in_w * in_h * 3];

    // Precalculate fixed-point weight: strength * 256
    let weight = (strength * 256.0).clamp(0.0, 512.0) as i32;

    sharp_buf
        .par_chunks_exact_mut(in_w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let y_prev = if y > 0 { y - 1 } else { 0 };
            let y_next = if y + 1 < in_h { y + 1 } else { in_h - 1 };

            let cur_offset = y * in_w * 3;
            let top_offset = y_prev * in_w * 3;
            let bot_offset = y_next * in_w * 3;

            for x in 0..in_w {
                let x_prev = if x > 0 { x - 1 } else { 0 };
                let x_next = if x + 1 < in_w { x + 1 } else { in_w - 1 };

                let c_idx = cur_offset + x * 3;
                let t_idx = top_offset + x * 3;
                let b_idx = bot_offset + x * 3;
                let l_idx = cur_offset + x_prev * 3;
                let r_idx = cur_offset + x_next * 3;

                let out_idx = x * 3;
                for ch in 0..3 {
                    let c = raw[c_idx + ch] as i32;
                    let blur = (raw[t_idx + ch] as i32
                        + raw[b_idx + ch] as i32
                        + raw[l_idx + ch] as i32
                        + raw[r_idx + ch] as i32)
                        >> 2;
                    let diff = c - blur;
                    let sharp = c + ((diff * weight) >> 8);
                    row[out_idx + ch] = sharp.clamp(0, 255) as u8;
                }
            }
        });

    RgbImage::from_raw(in_w as u32, in_h as u32, sharp_buf)
        .expect("Failed to create sharpened image")
}

/// Fast contrast and brightness adjustments using integer arithmetic
pub fn adjust_brightness_contrast(pixels: &mut [u8], brightness: i32, contrast: i32) {
    let c = contrast.clamp(-100, 100);
    let factor_num = 259 * (c + 255);
    let factor_den = 255 * (259 - c);

    pixels.par_chunks_exact_mut(3).for_each(|chunk| {
        for ch in 0..3 {
            let mut val = chunk[ch] as i32 + brightness;
            if contrast != 0 {
                val = ((factor_num * (val - 128)) / factor_den) + 128;
            }
            chunk[ch] = val.clamp(0, 255) as u8;
        }
    });
}
