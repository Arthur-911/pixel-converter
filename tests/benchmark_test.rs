use image::{Rgb, RgbImage};
use pixelgenrator::{
    dither::DitherMode,
    palette::Palette,
    processor::{PixelConfig, PixelProcessor},
};
use std::time::Instant;

#[test]
fn test_throughput_4k() {
    let width = 3840;
    let height = 2160;
    let megapixels = (width as f64 * height as f64) / 1_000_000.0;

    println!("\nGenerating synthetic 4K image ({}x{} = {:.2} MP)...", width, height, megapixels);
    let mut img = RgbImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = ((x * 255) / width) as u8;
        let g = ((y * 255) / height) as u8;
        let b = (((x + y) * 255) / (width + height)) as u8;
        *pixel = Rgb([r, g, b]);
    }

    let processor = PixelProcessor::new(Palette::pico8());

    let configs = [
        ("Bayer8 (Parallel)", DitherMode::Bayer8, false),
        ("Bayer4 (Parallel)", DitherMode::Bayer4, false),
        ("Floyd-Steinberg", DitherMode::FloydSteinberg, false),
        ("Atkinson", DitherMode::Atkinson, false),
        ("Bayer8 + Sobel Outlines", DitherMode::Bayer8, true),
    ];

    println!("\n=== 4K BENCHMARK RESULTS ===");
    for (name, mode, outline) in configs {
        let config = PixelConfig {
            block_size: 8,
            dither_mode: mode,
            dither_strength: 1.0,
            sharpness: 0.0,
            outline,
            outline_threshold: 120,
            outline_color: [0, 0, 0],
            output_scale: 0,
            brightness: 0,
            contrast: 0,
        };

        // Warmup
        let _ = processor.process(&img, &config);

        // Measure
        let start = Instant::now();
        let iters = 5;
        for _ in 0..iters {
            let _ = processor.process(&img, &config);
        }
        let elapsed = start.elapsed() / iters;
        let mp_per_sec = megapixels / elapsed.as_secs_f64();

        println!(
            "{:<25} | Latency: {:>6.2?} | Throughput: {:>7.2} Megapixels/sec",
            name, elapsed, mp_per_sec
        );
    }
    println!("============================\n");
}
