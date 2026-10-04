# pixel-converter

A fast, lightweight image-to-pixel-art converter written in Rust. It transforms standard images into retro pixel art with sharp edges, authentic console palettes, and zero runtime dependencies.

## Usage

### Without Commands (Windows)

- Double-click `PixelGen.bat` to open a file selector and pick an image.
- Drag and drop an image (or multiple images) directly onto `PixelGen.bat`.

The converted pixel art is generated immediately and opens in your default image viewer.

### Command Line

```bash
# Basic conversion (auto-scales block size and uses vibrant 15-bit colors)
cargo run --release -- input.png

# Classic retro console palettes
cargo run --release -- input.png -p gameboy
cargo run --release -- input.png -p pico8
cargo run --release -- input.png -p nes
cargo run --release -- input.png -p c64

# Manual pixel block size and custom output path
cargo run --release -- -i input.png -o output.png --scale 4

# Optional retro dithering and 1px character outlines
cargo run --release -- input.png -d bayer8 --outline
```

### Options

| Flag | Description | Default |
| --- | --- | --- |
| `-i, --input <PATH>` | Input image file or directory (leave empty for file picker) | Optional |
| `-o, --output <PATH>` | Output image file or destination folder | `[name]_pixelart.png` |
| `-p, --palette <NAME>` | Preset: `vibrant`, `pico8`, `gameboy`, `c64`, `nes`, `cga`, `cyberpunk`, `monochrome` | `vibrant` |
| `--hex <HEX_LIST>` | Comma-separated hex palette (e.g. `"#000000,#ffffff,#ff0055"`) | None |
| `-s, --scale <INT>` | Pixel block size (`0` auto-calculates based on resolution) | `0` |
| `-d, --dither <MODE>` | Dithering: `none`, `bayer8`, `bayer4`, `floyd`, `atkinson` | `none` |
| `--sharpness <FLOAT>` | Pre-downsampling edge sharpness boost | `0.8` |
| `--outline` | Enables 1px dark Sobel edge outlines | Off |
| `--outline-threshold <INT>` | Threshold for edge detection (lower = more outlines) | `120` |
| `--brightness <INT>` | Brightness adjustment (-100 to 100) | `0` |
| `--contrast <INT>` | Contrast adjustment (-100 to 100) | `0` |
| `--no-open` | Disables opening the image viewer after conversion | Off |
| `--benchmark` | Runs a 10-iteration throughput benchmark | Off |

## Architecture & Performance

- 3D Look-Up Table (3D LUT): Precomputes 5-bit RGB quantization tables in under 100 microseconds for O(1) color lookups.
- Integer Sharpening: Applies a Laplacian edge filter using pure integer bit-shifts before downsampling, preserving pupils, mouths, and line art without floating-point overhead.
- Zero-Allocation Upscaling: Nearest-neighbor upscaling writes scanlines directly into contiguous buffer memory using intra-buffer memory copies.
- Multi-Threaded Processing: Box downsampling, palette matching, and scaling run in parallel across CPU cores using Rayon.

## Additional Tools

Located in the `tools/` folder:

- `Auto_Folder_Watcher.bat`: Monitors the `Drop_Images_Here/` directory and converts any dropped images to `Pixel_Outputs/`.
- `Install_Right_Click_Menu.bat`: Adds a "Convert to Pixel Art" option to the Windows Explorer right-click menu for image files.
- `Uninstall_Right_Click_Menu.bat`: Removes the right-click menu entry.

## Requirements

- Rust 1.85+ (2024 edition)
- Windows, macOS, or Linux (interactive file dialog and helper scripts configured for Windows)
