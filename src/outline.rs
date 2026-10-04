use rayon::prelude::*;

/// Applies a 3x3 Sobel edge detection filter on the low-res pixel art buffer.
/// Where edges exceed `threshold`, the pixel is colored with `outline_color`.
/// Fully parallelized across CPU cores with Rayon.
pub fn apply_outlines(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    threshold: i32,
    outline_color: [u8; 3],
) {
    if width < 3 || height < 3 {
        return;
    }

    // Convert to luminance buffer for edge detection: Y = (2*R + 5*G + 1*B) >> 3
    let mut luma = vec![0i32; width * height];
    luma.par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, luma_row)| {
            let row_start = y * width * 3;
            for x in 0..width {
                let idx = row_start + x * 3;
                let r = pixels[idx] as i32;
                let g = pixels[idx + 1] as i32;
                let b = pixels[idx + 2] as i32;
                luma_row[x] = (2 * r + 5 * g + b) >> 3;
            }
        });

    // Detect edges in parallel per row
    let mut edge_mask = vec![false; width * height];

    edge_mask
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, mask_row)| {
            if y == 0 || y >= height - 1 {
                return;
            }

            let top_row = (y - 1) * width;
            let mid_row = y * width;
            let bot_row = (y + 1) * width;

            for x in 1..width - 1 {
                // Sobel kernels
                // Gx:
                // -1  0  1
                // -2  0  2
                // -1  0  1
                let gx = -luma[top_row + x - 1] + luma[top_row + x + 1]
                    - 2 * luma[mid_row + x - 1] + 2 * luma[mid_row + x + 1]
                    - luma[bot_row + x - 1] + luma[bot_row + x + 1];

                // Gy:
                // -1 -2 -1
                //  0  0  0
                //  1  2  1
                let gy = -luma[top_row + x - 1] - 2 * luma[top_row + x] - luma[top_row + x + 1]
                    + luma[bot_row + x - 1] + 2 * luma[bot_row + x] + luma[bot_row + x + 1];

                // Fast Manhattan magnitude approximation
                let mag = gx.abs() + gy.abs();
                if mag > threshold {
                    mask_row[x] = true;
                }
            }
        });

    // Apply outline color to marked pixels in parallel
    pixels
        .par_chunks_exact_mut(width * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let mask_row_start = y * width;
            for x in 0..width {
                if edge_mask[mask_row_start + x] {
                    let idx = x * 3;
                    row[idx] = outline_color[0];
                    row[idx + 1] = outline_color[1];
                    row[idx + 2] = outline_color[2];
                }
            }
        });
}
