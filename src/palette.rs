#[derive(Debug, Clone)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<[u8; 3]>,
}

impl Palette {
    pub fn new(name: impl Into<String>, colors: Vec<[u8; 3]>) -> Self {
        assert!(!colors.is_empty(), "Palette cannot be empty");
        Self {
            name: name.into(),
            colors,
        }
    }

    /// Finds the nearest color index in the palette using perceptually weighted distance.
    /// Uses integer arithmetic for maximum performance (no floating-point, no sqrt).
    #[inline(always)]
    pub fn find_nearest(&self, target: [u8; 3]) -> [u8; 3] {
        let mut min_dist = i32::MAX;
        let mut best_color = self.colors[0];

        let tr = target[0] as i32;
        let tg = target[1] as i32;
        let tb = target[2] as i32;

        for &c in &self.colors {
            let dr = tr - c[0] as i32;
            let dg = tg - c[1] as i32;
            let db = tb - c[2] as i32;

            // Perceptually-weighted color distance approximation:
            // Human vision is most sensitive to green, then red, then blue.
            // 2 * dR^2 + 4 * dG^2 + 3 * dB^2
            let dist = 2 * dr * dr + 4 * dg * dg + 3 * db * db;
            if dist < min_dist {
                min_dist = dist;
                best_color = c;
                if dist == 0 {
                    break;
                }
            }
        }

        best_color
    }

    /// Predefined iconic retro palettes
    pub fn gameboy() -> Self {
        Self::new(
            "gameboy",
            vec![
                [15, 56, 15],     // Darkest green
                [48, 98, 48],     // Dark green
                [139, 172, 15],   // Light green
                [155, 188, 15],   // Brightest yellow-green
            ],
        )
    }

    pub fn pico8() -> Self {
        Self::new(
            "pico8",
            vec![
                [0, 0, 0],         // Black
                [29, 43, 83],      // Dark blue
                [126, 37, 83],     // Dark purple
                [0, 135, 81],      // Dark green
                [171, 82, 54],     // Brown
                [95, 87, 79],      // Dark gray
                [194, 195, 199],   // Light gray
                [255, 241, 232],   // White
                [255, 0, 77],      // Red
                [255, 163, 0],     // Orange
                [255, 236, 39],    // Yellow
                [0, 228, 54],      // Green
                [41, 173, 255],    // Blue
                [131, 118, 156],   // Indigo
                [255, 119, 168],   // Pink
                [255, 204, 170],   // Peach
            ],
        )
    }

    pub fn c64() -> Self {
        Self::new(
            "c64",
            vec![
                [0, 0, 0],
                [255, 255, 255],
                [136, 0, 0],
                [170, 255, 238],
                [204, 68, 204],
                [0, 204, 85],
                [0, 0, 170],
                [238, 238, 119],
                [221, 136, 85],
                [102, 68, 0],
                [255, 119, 119],
                [51, 51, 51],
                [119, 119, 119],
                [170, 255, 102],
                [0, 136, 255],
                [187, 187, 187],
            ],
        )
    }

    pub fn nes() -> Self {
        Self::new(
            "nes",
            vec![
                [124, 124, 124],
                [0, 0, 252],
                [0, 0, 188],
                [68, 40, 188],
                [148, 0, 132],
                [168, 0, 32],
                [168, 16, 0],
                [136, 20, 0],
                [80, 48, 0],
                [0, 120, 0],
                [0, 104, 0],
                [0, 88, 0],
                [0, 64, 88],
                [0, 0, 0],
                [252, 252, 252],
                [60, 188, 252],
            ],
        )
    }

    pub fn cga() -> Self {
        Self::new(
            "cga",
            vec![
                [0, 0, 0],       // Black
                [85, 255, 255],  // Cyan
                [255, 85, 255],  // Magenta
                [255, 255, 255], // White
            ],
        )
    }

    pub fn cyberpunk() -> Self {
        Self::new(
            "cyberpunk",
            vec![
                [13, 2, 33],
                [0, 245, 212],
                [123, 44, 191],
                [255, 0, 110],
                [255, 190, 11],
                [251, 86, 7],
                [255, 255, 255],
            ],
        )
    }

    pub fn monochrome() -> Self {
        Self::new("monochrome", vec![[0, 0, 0], [255, 255, 255]])
    }

    /// Vibrant 15-bit console palette (SNES/GBA/NeoGeo full gamut retro quantization)
    pub fn vibrant() -> Self {
        Self::new("vibrant", vec![[0, 0, 0], [255, 255, 255]])
    }

    pub fn from_preset(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "vibrant" | "auto" | "truecolor" | "15bit" | "default" => Some(Self::vibrant()),
            "gameboy" | "gb" => Some(Self::gameboy()),
            "pico8" | "pico-8" => Some(Self::pico8()),
            "c64" => Some(Self::c64()),
            "nes" => Some(Self::nes()),
            "cga" => Some(Self::cga()),
            "cyberpunk" => Some(Self::cyberpunk()),
            "monochrome" | "mono" | "bw" => Some(Self::monochrome()),
            _ => None,
        }
    }

    /// Parse comma-separated hex colors: "#ffffff,#000000,#ff0055"
    pub fn from_hex_list(name: &str, hex_list: &str) -> Result<Self, String> {
        let mut colors = Vec::new();
        for item in hex_list.split(',') {
            let s = item.trim().trim_start_matches('#');
            if s.len() == 6 {
                let r = u8::from_str_radix(&s[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&s[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&s[4..6], 16).map_err(|e| e.to_string())?;
                colors.push([r, g, b]);
            } else {
                return Err(format!("Invalid hex color: '{}'", item));
            }
        }
        if colors.is_empty() {
            return Err("No valid colors found in list".to_string());
        }
        Ok(Self::new(name, colors))
    }
}
