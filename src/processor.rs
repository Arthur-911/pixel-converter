pub use crate::config::PixelConfig;
use crate::dither::apply_dithering;
use crate::filter::{adjust_brightness_contrast, apply_sharpening_int};
use crate::lut::Lut3D;
use crate::outline::apply_outlines;
use crate::palette::Palette;
use image::RgbImage;
use rayon::prelude::*;

pub struct PixelProcessor {
    lut: Lut3D,
    pub palette: Palette,
}

impl PixelProcessor {
    pub fn new(palette: Palette) -> Self {
        let lut = Lut3D::new(&palette);
        Self { lut, palette }
    }

    /// Primary high-speed transformation pipeline.
    /// Takes an input RgbImage and returns the transformed pixel art RgbImage.
    pub fn process(&self, input: &RgbImage, config: &PixelConfig) -> RgbImage {
        let in_w = input.width() as usize;
        let in_h = input.height() as usize;

        // Auto-determine block size if 0: targets ~180 to 240px retro console resolution
        let block = if config.block_size == 0 {
            let max_dim = in_w.max(in_h);
            (max_dim / 200).clamp(2, 16)
        } else {
            config.block_size.max(1)
        };

        let low_w = (in_w + block - 1) / block;
        let low_h = (in_h + block - 1) / block;

        // Step 1: Pre-sharpen input if enabled to preserve fine details like eyes and outlines
        let sharpened_img;
        let source_img = if config.sharpness > 0.0 {
            sharpened_img = apply_sharpening_int(input, in_w, in_h, config.sharpness);
            &sharpened_img
        } else {
            input
        };

        // Step 2: High-speed parallel box-downsampling
        let mut low_res = downsample_box(source_img, in_w, in_h, low_w, low_h, block);

        // Step 3: Brightness & Contrast adjustment (if any)
        if config.brightness != 0 || config.contrast != 0 {
            adjust_brightness_contrast(&mut low_res, config.brightness, config.contrast);
        }

        // Step 4: Outline detection on downsampled buffer
        if config.outline {
            apply_outlines(
                &mut low_res,
                low_w,
                low_h,
                config.outline_threshold,
                config.outline_color,
            );
        }

        // Step 5: Dithering & Palette Quantization via 3D LUT
        apply_dithering(
            &mut low_res,
            low_w,
            low_h,
            &self.lut,
            config.dither_mode,
            config.dither_strength,
        );

        // Step 6: High-speed nearest-neighbor upscaling for razor-sharp retro output
        let scale = if config.output_scale > 0 {
            config.output_scale
        } else {
            block
        };

        if scale == 1 {
            // Return raw low-res buffer
            RgbImage::from_raw(low_w as u32, low_h as u32, low_res)
                .expect("Failed to create low-res image")
        } else {
            upscale_nearest(&low_res, low_w, low_h, scale)
        }
    }
}

/// Downsamples the image using parallel box-averaging.
/// This preserves shapes and colors far better than naive nearest-neighbor point sampling.
fn downsample_box(
    input: &RgbImage,
    in_w: usize,
    in_h: usize,
    out_w: usize,
    out_h: usize,
    block: usize,
) -> Vec<u8> {
    let mut out_buffer = vec![0u8; out_w * out_h * 3];
    let in_raw = input.as_raw();

    out_buffer
        .par_chunks_exact_mut(out_w * 3)
        .enumerate()
        .for_each(|(by, row)| {
            let start_y = by * block;
            let end_y = (start_y + block).min(in_h);
            let h_count = end_y - start_y;

            for bx in 0..out_w {
                let start_x = bx * block;
                let end_x = (start_x + block).min(in_w);
                let w_count = end_x - start_x;
                let total_px = (w_count * h_count) as u32;

                let mut sum_r = 0u32;
                let mut sum_g = 0u32;
                let mut sum_b = 0u32;

                for y in start_y..end_y {
                    let y_offset = y * in_w * 3;
                    for x in start_x..end_x {
                        let idx = y_offset + x * 3;
                        sum_r += in_raw[idx] as u32;
                        sum_g += in_raw[idx + 1] as u32;
                        sum_b += in_raw[idx + 2] as u32;
                    }
                }

                let out_idx = bx * 3;
                row[out_idx] = (sum_r / total_px) as u8;
                row[out_idx + 1] = (sum_g / total_px) as u8;
                row[out_idx + 2] = (sum_b / total_px) as u8;
            }
        });

    out_buffer
}

/// Upscales low-res pixels to a crisp full-resolution image using zero-allocation parallel memcpy replication.
fn upscale_nearest(
    low_res: &[u8],
    low_w: usize,
    low_h: usize,
    scale: usize,
) -> RgbImage {
    let out_w = low_w * scale;
    let out_h = low_h * scale;
    let row_bytes = out_w * 3;
    let mut out_buffer = vec![0u8; out_w * out_h * 3];

    out_buffer
        .par_chunks_exact_mut(row_bytes * scale)
        .enumerate()
        .for_each(|(low_y, block_rows)| {
            let low_row_start = low_y * low_w * 3;

            // Generate first upscaled scanline directly in block_rows
            for low_x in 0..low_w {
                let px_idx = low_row_start + low_x * 3;
                let r = low_res[px_idx];
                let g = low_res[px_idx + 1];
                let b = low_res[px_idx + 2];

                let target_start = low_x * scale * 3;
                for s in 0..scale {
                    let out_idx = target_start + s * 3;
                    block_rows[out_idx] = r;
                    block_rows[out_idx + 1] = g;
                    block_rows[out_idx + 2] = b;
                }
            }

            // High-speed intra-buffer copy for remaining vertical lines (zero heap allocations)
            for s in 1..scale {
                let dest_start = s * row_bytes;
                block_rows.copy_within(0..row_bytes, dest_start);
            }
        });

    RgbImage::from_raw(out_w as u32, out_h as u32, out_buffer)
        .expect("Failed to create upscaled image")
}
