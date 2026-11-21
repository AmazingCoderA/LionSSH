use serde::{Deserialize, Serialize};

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";

type Rgb = [u8; 3];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub primary: Rgb,
    pub secondary: Rgb,
    pub accent: Rgb,
    pub success: Rgb,
    pub warning: Rgb,
    pub error: Rgb,
    pub text: Rgb,
    pub dim: Rgb,
    pub bg: Rgb,
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round() as u8
}

impl Theme {
    pub fn fg(c: Rgb) -> String {
        format!("\x1b[38;2;{};{};{}m", c[0], c[1], c[2])
    }
    pub fn bg(c: Rgb) -> String {
        format!("\x1b[48;2;{};{};{}m", c[0], c[1], c[2])
    }

    pub fn primary(&self) -> String { Self::fg(self.primary) }
    pub fn secondary(&self) -> String { Self::fg(self.secondary) }
    pub fn accent(&self) -> String { Self::fg(self.accent) }
    pub fn success(&self) -> String { Self::fg(self.success) }
    pub fn warning(&self) -> String { Self::fg(self.warning) }
    pub fn error(&self) -> String { Self::fg(self.error) }
    pub fn text(&self) -> String { Self::fg(self.text) }
    pub fn dim(&self) -> String { Self::fg(self.dim) }

    /// Градиентный текст от одного цвета к другому (по символам).
    pub fn gradient(text: &str, from: Rgb, to: Rgb) -> String {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len().max(1);
        let denom = n.saturating_sub(1).max(1) as f32;
        let mut out = String::new();
        for (i, ch) in chars.iter().enumerate() {
            let t = i as f32 / denom;
            out.push_str(&format!(
                "\x1b[38;2;{};{};{}m{}",
                lerp(from[0], to[0], t),
                lerp(from[1], to[1], t),
                lerp(from[2], to[2], t),
                ch
            ));
        }
        out.push_str(RESET);
        out
    }

    pub fn rainbow(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len().max(1);
        let mut out = String::new();
        for (i, ch) in chars.iter().enumerate() {
            let hue = (i as f32 / n as f32) * 360.0;
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            out.push_str(&format!("\x1b[38;2;{};{};{}m{}", r, g, b, ch));
        }
        out.push_str(RESET);
        out
    }

    pub fn swatches(&self) -> String {
        let names = [
            ("primary", self.primary),
            ("secondary", self.secondary),
            ("accent", self.accent),
            ("success", self.success),
            ("warning", self.warning),
            ("error", self.error),
        ];
        let mut out = String::new();
        for (label, c) in names {
            out.push_str(&format!(
                "{} {}█ {} ",
                Self::bg(c),
                RESET,
                self.dim()
            ));
            out.push_str(label);
            out.push(' ');
        }
        out.push_str(RESET);
        out
    }

    pub fn presets() -> Vec<Theme> {
        vec![
            Theme {
                name: "cyberpunk".into(),
                primary: [255, 0, 153], secondary: [0, 255, 255], accent: [180, 80, 255],
                success: [0, 255, 140], warning: [255, 210, 0], error: [255, 60, 80],
                text: [235, 235, 248], dim: [120, 110, 140], bg: [15, 10, 25],
            },
            Theme {
                name: "dracula".into(),
                primary: [189, 147, 249], secondary: [139, 233, 253], accent: [255, 121, 198],
                success: [80, 250, 123], warning: [241, 250, 140], error: [255, 85, 85],
                text: [248, 248, 242], dim: [98, 114, 164], bg: [40, 42, 54],
            },
            Theme {
                name: "matrix".into(),
                primary: [0, 255, 70], secondary: [120, 255, 120], accent: [200, 255, 200],
                success: [0, 255, 70], warning: [200, 255, 0], error: [255, 70, 70],
                text: [190, 255, 190], dim: [0, 120, 40], bg: [0, 10, 0],
            },
            Theme {
                name: "nord".into(),
                primary: [136, 192, 208], secondary: [143, 188, 187], accent: [180, 142, 173],
                success: [163, 190, 140], warning: [235, 203, 139], error: [191, 97, 106],
                text: [236, 239, 244], dim: [76, 86, 106], bg: [46, 52, 64],
            },
            Theme {
                name: "solarized".into(),
                primary: [38, 139, 210], secondary: [42, 161, 152], accent: [211, 54, 130],
                success: [133, 153, 0], warning: [181, 137, 0], error: [220, 50, 47],
                text: [253, 246, 227], dim: [88, 110, 117], bg: [0, 43, 54],
            },
            Theme {
                name: "aurora".into(),
                primary: [0, 230, 180], secondary: [140, 100, 255], accent: [0, 180, 255],
                success: [80, 255, 150], warning: [255, 180, 80], error: [255, 80, 120],
                text: [230, 245, 255], dim: [100, 130, 150], bg: [5, 20, 30],
            },
            Theme {
                name: "blood".into(),
                primary: [255, 60, 60], secondary: [255, 140, 0], accent: [200, 0, 0],
                success: [120, 255, 120], warning: [255, 200, 0], error: [255, 40, 40],
                text: [255, 230, 230], dim: [120, 60, 60], bg: [25, 5, 5],
            },
            Theme {
                name: "vaporwave".into(),
                primary: [255, 113, 167], secondary: [93, 233, 244], accent: [253, 225, 92],
                success: [153, 255, 153], warning: [255, 220, 100], error: [255, 100, 150],
                text: [245, 235, 255], dim: [140, 120, 160], bg: [25, 15, 40],
            },
        ]
    }

    pub fn by_name(name: &str) -> Option<Theme> {
        Self::presets().into_iter().find(|t| t.name.eq_ignore_ascii_case(name))
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 / 60 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
