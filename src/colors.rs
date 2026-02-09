
use reqwest;
use serde::Deserialize;


#[derive(Debug, Default, Clone, Copy)]
pub enum TextColor {
    #[default]
    White,
    Black
}


#[derive(Debug, Deserialize)]
pub struct Palette {
    pub colors: Vec<RawColor>
}

impl Palette {
    pub fn load() -> Option<Self> {
        let response = reqwest::blocking::get("https://api.color.pizza/v1/").ok()?;

        let palette: Self = response.json().ok()?;

        Some(palette)
    }
}


#[derive(Debug, Deserialize)]
pub struct RawColor {
    pub name: String,
    pub hex: String,
    pub rgb: RGB,
    pub bestContrast: String
}

impl RawColor {
    pub fn color(self) -> Color {
        let search_name = self.name.to_lowercase().replace(" ", "");
        let (l, a, b) = self.rgb.to_oklab();
        let text_color = match self.bestContrast.as_str() {
            "white" => TextColor::White,
            "black" => TextColor::Black,
            other => panic!("Invalid contrast: {other}")
        };
        
        Color {
            name: self.name,
            search_name: search_name,
            hex: self.hex,
            l,
            a,
            b,
            text_color
        }
    }
}


#[derive(Debug, Default, Clone)]
pub struct Color {
    pub name: String,
    pub search_name: String,
    pub hex: String,
    pub l: f32,
    pub a: f32,
    pub b: f32,
    pub text_color: TextColor
}

impl Color {
    pub fn similarity(&self, other: &Self) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        
        let dist = (dl * dl + da * da + db * db).sqrt();

        1.0 - dist
    }
}

#[derive(Debug, Deserialize)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl RGB {
    fn linearize(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }

    pub fn to_oklab(&self) -> (f32, f32, f32) {
        let (r, g, b) = (
            Self::linearize(self.r / 255.0),
            Self::linearize(self.g / 255.0),
            Self::linearize(self.b / 255.0)
        );

        let (l, m, s) = (
            r * 0.4122214708 + g * 0.5363325363 + b * 0.0514459929,
            r * 0.2119034982 + g * 0.6806995451 + b * 0.1073969566,
            r * 0.0883024619 + g * 0.2817188376 + b * 0.6299787005
        );

        let (l_, m_, s_) = (
            l.cbrt(),
            m.cbrt(),
            s.cbrt()
        );

        let (l, a, b) = (
            0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
            1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
            0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
        );

        (l, a, b)
    }
}
