
use core::f32;
use std::rc::Rc;

use rand::seq::IndexedRandom;
use reqwest;
use serde::Deserialize;


#[derive(Debug, Default, Clone, Copy)]
pub enum TextColor {
    #[default]
    White,
    Black
}


#[derive(Debug, Deserialize)]
struct JsonPalette {
    pub colors: Vec<RawColor>
}

impl JsonPalette {
    fn load(name: &str) -> Option<Self> {
        let response = reqwest::blocking::get(format!("https://api.color.pizza/v1/?list={name}")).ok()?;

        let palette: Self = response.json().ok()?;

        Some(palette)
    }
}


#[derive(Debug, Default)]
pub struct Palette {
    colors: Vec<Rc<Color>>
}

impl Palette {
    fn load(name: &str) -> Option<Self> {
        let raw = JsonPalette::load(name)?;
        
        let mut colors: Vec<Rc<Color>> = raw
            .colors
            .into_iter()
            .map(|raw| Rc::new(raw.color()))
            .collect();

        colors.sort_by(|a, b| a.search_name.cmp(&b.search_name));

        Some(Self { colors })
    }

    fn load_child(&self, name: &str) -> Option<Self> {
        let raw = JsonPalette::load(name)?;
        
        let mut colors: Vec<Rc<Color>> = raw
            .colors
            .into_iter()
            .filter_map(|col| self.match_name(&col.name)) // discard all non-standard colors
            .collect();

        colors.sort_by(|a, b| a.search_name.cmp(&b.search_name));

        Some(Self { colors })
    }

    pub fn random(&self) -> Rc<Color> {
        Rc::clone(self.colors.choose(&mut rand::rng()).unwrap())
    }

    pub fn match_name(&self, name: &String) -> Option<Rc<Color>> {
        let match_name = &name.to_lowercase().replace(" ", "");

        // match the color using binary search (fast)
        self.colors
            .binary_search_by(|a| a.search_name.cmp(match_name))
            .ok()
            .map(|i| Rc::clone(&self.colors[i]))
    }
}


#[derive(Debug, Default)]
pub struct Palettes {
    pub all: Palette,
    pub basic: Palette,
    pub advanced: Palette,
    pub wikipedia: Palette
}

impl Palettes {
    pub fn new() -> Option<Self> {
        let all = Palette::load("default")?;
        let basic = all.load_child("mlmc_english")?;
        let advanced = all.load_child("bestOf")?;
        let wikipedia = all.load_child("wikipedia")?;

        Some(Self {
            all,
            basic,
            advanced,
            wikipedia
        })
    }
}


#[derive(Debug, Deserialize)]
struct RawColor {
    pub name: String,
    pub hex: String,
    pub rgb: RGB,
    pub bestContrast: String
}

impl RawColor {
    fn color(self) -> Color {
        let search_name = self.name.to_lowercase().replace(" ", "");
        let (l, a, b) = self.rgb.to_oklab();
        let text_color = match self.bestContrast.as_str() {
            "white" => TextColor::White,
            "black" => TextColor::Black,
            other => panic!("Invalid bestContrast: {other}")
        };
        
        Color {
            name: self.name,
            search_name: search_name,
            hex: self.hex,
            l: smoothstep((l * 10_000.0).round() / 10_000.0), // smoothstep cause the extreme lightness ones are too far apart
            a: (a * 10_000.0).round() / 10_000.0,
            b: (b * 10_000.0).round() / 10_000.0,
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
    fn saturation(&self) -> f32 {
        (self.a * self.a + self.b * self.b).cbrt()
    }

    fn dist(&self, other: &Self) -> f32 {
        let d_l = self.l - other.l;
        let d_a = self.a - other.a;
        let d_b = self.b - other.b;

        (d_l * d_l + d_a * d_a + d_b * d_b).sqrt()
    }

    pub fn similarity_old(&self, other: &Self) -> f32 {
        let sim_light = 1.0 - (self.l - other.l).abs();

        let d_a = self.a - other.a;
        let d_b = self.b - other.b;
        let sim_col = 1.0 - (d_a * d_a + d_b * d_b).sqrt();

        let sat1 = self.saturation();
        let sat2 = other.saturation();

        let sat_scale = (sat1 + sat2 + 1.0).log(3.0) * 2.0;

        println!("{}: {}", self.name, sat1);
        println!("{}: {}", other.name, sat2);
        println!("{}, {}, {}", sim_col, sim_light, sat_scale);

        (sim_col * sat_scale + sim_light) / (sat_scale + 1.0)
    }

    pub fn similarity(&self, other: &Self) -> f32 {

        let dist = self.dist(other);
        let sat = 1.0 - (self.saturation() - other.saturation()).abs() * 0.75;

        1.0 - dist * sat
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

    fn to_oklab(&self) -> (f32, f32, f32) {
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


fn smoothstep(x: f32) -> f32 {
    (3.0 - 2.0 * x) * x * x
}
