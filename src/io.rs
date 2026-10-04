use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    ExtendedColorType, ImageEncoder, RgbImage,
};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

/// High-speed image saver using buffered I/O and fast PNG compression.
/// Replaces slow default level-6 DEFLATE compression with sub-millisecond fast compression.
pub fn save_image_fast(path: &Path, img: &RgbImage) -> Result<(), Box<dyn std::error::Error>> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    if ext == "png" {
        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(256 * 1024, file);
        let encoder =
            PngEncoder::new_with_quality(&mut writer, CompressionType::Fast, FilterType::NoFilter);
        encoder.write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::Rgb8,
        )?;
        Ok(())
    } else {
        img.save(path)?;
        Ok(())
    }
}

/// Opens native Windows file selection dialog.
pub fn pick_image_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Select an image to convert to Pixel Art")
        .add_filter("Image Files", &["png", "jpg", "jpeg", "webp", "bmp"])
        .pick_file()
}

/// Automatically opens the resulting image in the system's default viewer.
pub fn open_in_viewer(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", &path.to_string_lossy()])
            .spawn();
    }
}
