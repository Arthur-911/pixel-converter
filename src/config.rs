use crate::dither::DitherMode;

#[derive(Debug, Clone)]
pub struct PixelConfig {
    /// Size of each pixel block (0 = auto-calculate optimal retro resolution, e.g. 3, 6, 8)
    pub block_size: usize,
    /// Dithering algorithm
    pub dither_mode: DitherMode,
    /// Dithering strength (0.0 to 1.5, default 1.0)
    pub dither_strength: f32,
    /// Edge sharpness boost before downsampling (0.0 = off, 0.8 = standard sharp)
    pub sharpness: f32,
    /// Whether to generate 1px dark retro outlines
    pub outline: bool,
    /// Sobel outline threshold (lower = more outlines, default ~120)
    pub outline_threshold: i32,
    /// Outline color (default [0, 0, 0])
    pub outline_color: [u8; 3],
    /// Upscale factor for output image (0 = match original resolution)
    pub output_scale: usize,
    /// Brightness boost (-100 to 100, default 0)
    pub brightness: i32,
    /// Contrast boost (-100 to 100, default 0)
    pub contrast: i32,
}

impl Default for PixelConfig {
    fn default() -> Self {
        Self {
            block_size: 0, // Auto-size: intelligently adapts to image resolution
            dither_mode: DitherMode::None, // Crisp, clean retro sprite art by default
            dither_strength: 1.0,
            sharpness: 0.8, // Preserves crisp facial features, eyes, and outlines
            outline: false,
            outline_threshold: 120,
            outline_color: [0, 0, 0],
            output_scale: 0,
            brightness: 0,
            contrast: 0,
        }
    }
}
