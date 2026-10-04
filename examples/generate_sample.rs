use image::{Rgb, RgbImage};

fn main() {
    let width = 640;
    let height = 480;
    let mut img = RgbImage::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let cx = x as f32 - (width as f32 / 2.0);
        let cy = y as f32 - (height as f32 / 2.0);
        let dist = (cx * cx + cy * cy).sqrt();

        // Rings & gradient pattern
        let r = ((dist * 2.0).sin() * 127.0 + 128.0) as u8;
        let g = ((x as f32 / width as f32) * 255.0) as u8;
        let b = ((y as f32 / height as f32) * 255.0) as u8;

        *pixel = Rgb([r, g, b]);
    }

    img.save("sample.png").expect("Failed to save sample.png");
    println!("Saved sample.png (640x480)");
}
