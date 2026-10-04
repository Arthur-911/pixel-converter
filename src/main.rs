use clap::Parser;
use pixelgenrator::{
    dither::DitherMode,
    palette::Palette,
    processor::{PixelConfig, PixelProcessor},
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "pixelgen")]
#[command(version = "0.2.0")]
#[command(about = "Ultra-fast, multi-threaded pixel art generator in Rust", long_about = None)]
struct Args {
    /// Input image file or directory of images (or leave blank to open file picker)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Positional input image (e.g., when dragged directly onto pixelgenrator.exe)
    #[arg(index = 1)]
    input_pos: Option<PathBuf>,

    /// Output image file or directory (default: [name]_pixelart.png)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Palette preset: vibrant (15-bit retro console, default), pico8, gameboy, c64, nes, cga, cyberpunk, monochrome
    #[arg(short, long, default_value = "vibrant")]
    palette: String,

    /// Custom palette hex list (comma-separated, e.g. "#000000,#ffffff,#ff0055")
    #[arg(long)]
    hex: Option<String>,

    /// Pixel block size (0 = smart auto-size based on image resolution, or e.g. 2, 4, 8)
    #[arg(short, long, default_value_t = 0)]
    scale: usize,

    /// Dither algorithm: none (crisp clean sprites, default), bayer8, bayer4, floyd, atkinson
    #[arg(short, long, default_value = "none")]
    dither: String,

    /// Dithering strength (0.0 = none, 1.0 = standard, up to 2.0)
    #[arg(long, default_value_t = 1.0)]
    dither_strength: f32,

    /// Edge sharpness boost before downsampling (0.0 = off, 0.8 = crisp outlines & facial details)
    #[arg(long, default_value_t = 0.8)]
    sharpness: f32,

    /// Enable 1px retro character/object outlines (Sobel edge filter)
    #[arg(long)]
    outline: bool,

    /// Outline detection threshold (lower = more outlines, default: 120)
    #[arg(long, default_value_t = 120)]
    outline_threshold: i32,

    /// Output upscale factor (0 = auto-scale to match original resolution, 1 = raw pixel art resolution)
    #[arg(long, default_value_t = 0)]
    output_scale: usize,

    /// Brightness adjustment (-100 to 100)
    #[arg(long, default_value_t = 0)]
    brightness: i32,

    /// Contrast adjustment (-100 to 100)
    #[arg(long, default_value_t = 0)]
    contrast: i32,

    /// Do not automatically open the output image
    #[arg(long)]
    no_open: bool,

    /// Run benchmark mode: processes multiple iterations and outputs detailed latency stats
    #[arg(long)]
    benchmark: bool,
}

fn main() {
    let args = Args::parse();

    // 1. Resolve Input File (or open native Windows file dialog if no file passed)
    let is_interactive;
    let input_path = if let Some(p) = args.input.or(args.input_pos) {
        is_interactive = false;
        p
    } else {
        println!("=======================================================");
        println!(" 🎨 PixelGen: Opening image selector...");
        println!("=======================================================");
        match pixelgenrator::io::pick_image_file() {
            Some(p) => {
                is_interactive = true;
                p
            }
            None => {
                println!("No file selected. Exiting.");
                return;
            }
        }
    };

    // 2. Resolve Output File Path
    let output_path = if let Some(out) = args.output {
        out
    } else if input_path.is_file() {
        let parent = input_path.parent().unwrap_or(Path::new("."));
        let stem = input_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        parent.join(format!("{}_pixelart.png", stem))
    } else {
        input_path.join("pixelart_output")
    };

    // 3. Resolve Palette
    let palette = if let Some(hex_list) = &args.hex {
        Palette::from_hex_list("custom", hex_list).unwrap_or_else(|e| {
            eprintln!("Error parsing custom hex palette: {}", e);
            std::process::exit(1);
        })
    } else {
        Palette::from_preset(&args.palette).unwrap_or_else(|| {
            eprintln!(
                "Unknown palette '{}'. Available: vibrant, pico8, gameboy, c64, nes, cga, cyberpunk, monochrome",
                args.palette
            );
            std::process::exit(1);
        })
    };

    // 4. Resolve Dither Mode
    let dither_mode = DitherMode::from_str(&args.dither).unwrap_or_else(|| {
        eprintln!(
            "Unknown dither mode '{}'. Available: none, bayer8, bayer4, floyd, atkinson",
            args.dither
        );
        std::process::exit(1);
    });

    let config = PixelConfig {
        block_size: args.scale,
        dither_mode,
        dither_strength: args.dither_strength,
        sharpness: args.sharpness,
        outline: args.outline,
        outline_threshold: args.outline_threshold,
        outline_color: [0, 0, 0],
        output_scale: args.output_scale,
        brightness: args.brightness,
        contrast: args.contrast,
    };

    println!("=======================================================");
    println!(" 🚀 PixelGen: Blazing Fast Rust Pixel Art Engine");
    println!("=======================================================");
    println!(" Input        : {}", input_path.display());
    println!(" Output       : {}", output_path.display());
    println!(" Palette      : {}", palette.name);
    println!(" Block Size   : {}", if config.block_size == 0 { "Auto (smart retro resolution)".to_string() } else { format!("{} px", config.block_size) });
    println!(" Dither Mode  : {:?}", config.dither_mode);
    println!(" Sharpness    : {:.1}x", config.sharpness);
    println!(" Outlines     : {}", if config.outline { "Enabled (Sobel)" } else { "Disabled" });
    println!("=======================================================");

    let lut_start = Instant::now();
    let processor = PixelProcessor::new(palette);
    println!(" ⚡ LUT Precomputed in: {:.2?}", lut_start.elapsed());

    if input_path.is_dir() {
        process_directory(&processor, &config, &input_path, &output_path);
    } else {
        process_single_image(&processor, &config, &input_path, &output_path, args.benchmark, !args.no_open);
    }

    if is_interactive {
        println!("\nPress Enter to exit...");
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
    }
}

fn process_single_image(
    processor: &PixelProcessor,
    config: &PixelConfig,
    input_path: &Path,
    output_path: &Path,
    benchmark: bool,
    auto_open: bool,
) {
    println!("\nLoading image: {}", input_path.display());
    let decode_start = Instant::now();
    let img = match image::open(input_path) {
        Ok(img) => img.to_rgb8(),
        Err(e) => {
            eprintln!("Failed to read image {}: {}", input_path.display(), e);
            std::process::exit(1);
        }
    };
    let decode_time = decode_start.elapsed();

    let width = img.width();
    let height = img.height();
    let megapixels = (width as f64 * height as f64) / 1_000_000.0;
    println!("Resolution   : {}x{} ({:.2} Megapixels)", width, height, megapixels);
    println!("Decode Time  : {:.2?}", decode_time);

    if benchmark {
        println!("\n--- Running 10-Iteration Benchmark ---");
        let mut total_time = std::time::Duration::ZERO;
        for i in 1..=10 {
            let start = Instant::now();
            let _ = processor.process(&img, config);
            let elapsed = start.elapsed();
            total_time += elapsed;
            println!(" Iteration #{:2}: {:>7.2?} | Throughput: {:>6.2} MP/s", i, elapsed, megapixels / elapsed.as_secs_f64());
        }
        let avg_time = total_time / 10;
        println!("--------------------------------------");
        println!(" Average Transform Time: {:.2?}", avg_time);
        println!(" Average Throughput    : {:.2} Megapixels/sec", megapixels / avg_time.as_secs_f64());
        println!("--------------------------------------\n");
    }

    let proc_start = Instant::now();
    let result = processor.process(&img, config);
    let proc_time = proc_start.elapsed();

    println!("Transform Time: {:.2?} ({:.2} Megapixels/sec)", proc_time, megapixels / proc_time.as_secs_f64());

    let encode_start = Instant::now();
    if let Err(e) = pixelgenrator::io::save_image_fast(output_path, &result) {
        eprintln!("Failed to save output image {}: {}", output_path.display(), e);
        std::process::exit(1);
    }
    let encode_time = encode_start.elapsed();
    println!("Encode Time   : {:.2?}", encode_time);
    println!("\n✨ Successfully generated pixel art at: {}", output_path.display());

    if auto_open {
        println!("🖼️  Opening your pixel art...");
        pixelgenrator::io::open_in_viewer(output_path);
    }
}

fn process_directory(
    processor: &PixelProcessor,
    config: &PixelConfig,
    input_dir: &Path,
    output_dir: &Path,
) {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).expect("Failed to create output directory");
    }

    let entries: Vec<_> = fs::read_dir(input_dir)
        .expect("Failed to read input directory")
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp")
            } else {
                false
            }
        })
        .collect();

    println!("\nFound {} images to process in batch...", entries.len());
    let total_start = Instant::now();

    use rayon::prelude::*;
    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if let Ok(img) = image::open(&path) {
            let rgb = img.to_rgb8();
            let result = processor.process(&rgb, config);
            let filename = path.file_name().unwrap();
            let out_file = output_dir.join(filename);
            let _ = pixelgenrator::io::save_image_fast(&out_file, &result);
        }
    });

    println!(" Batch processing complete in: {:.2?}", total_start.elapsed());
}
