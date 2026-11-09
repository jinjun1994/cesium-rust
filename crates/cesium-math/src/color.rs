//! Color and CSS color representations matching CesiumJS `Core/Color`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{cartesian::Cartesian4, MathError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const PACKED_LENGTH: usize = 4;

    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);
    pub const LIME: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    #[inline]
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    #[inline]
    pub fn byte_to_float(byte: u8) -> f32 {
        byte as f32 / 255.0
    }

    #[inline]
    pub fn float_to_byte(float: f32) -> u8 {
        if float <= 0.0 {
            0
        } else if float >= 1.0 {
            255
        } else {
            (float * 255.0).round() as u8
        }
    }

    #[inline]
    pub fn from_bytes(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self::new(
            Self::byte_to_float(red),
            Self::byte_to_float(green),
            Self::byte_to_float(blue),
            Self::byte_to_float(alpha),
        )
    }

    #[inline]
    pub fn to_bytes(&self) -> [u8; 4] {
        [
            Self::float_to_byte(self.red),
            Self::float_to_byte(self.green),
            Self::float_to_byte(self.blue),
            Self::float_to_byte(self.alpha),
        ]
    }

    #[inline]
    pub fn from_rgba(rgba: u32) -> Self {
        let r = ((rgba >> 24) & 0xff) as u8;
        let g = ((rgba >> 16) & 0xff) as u8;
        let b = ((rgba >> 8) & 0xff) as u8;
        let a = (rgba & 0xff) as u8;
        Self::from_bytes(r, g, b, a)
    }

    #[inline]
    pub fn to_rgba(&self) -> u32 {
        let [r, g, b, a] = self.to_bytes();
        ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
    }

    #[inline]
    pub fn from_cartesian4(c: Cartesian4) -> Self {
        Self::new(c.x as f32, c.y as f32, c.z as f32, c.w as f32)
    }

    #[inline]
    pub fn to_cartesian4(&self) -> Cartesian4 {
        Cartesian4::new(
            self.red as f64,
            self.green as f64,
            self.blue as f64,
            self.alpha as f64,
        )
    }

    #[inline]
    pub fn with_alpha(&self, alpha: f32) -> Self {
        Self::new(self.red, self.green, self.blue, alpha)
    }

    #[inline]
    pub fn brighten(&self, magnitude: f32) -> Self {
        let mag = (1.0 - magnitude).max(0.0);
        Self::new(
            1.0 - (1.0 - self.red) * mag,
            1.0 - (1.0 - self.green) * mag,
            1.0 - (1.0 - self.blue) * mag,
            self.alpha,
        )
    }

    #[inline]
    pub fn darken(&self, magnitude: f32) -> Self {
        let mag = (1.0 - magnitude).max(0.0);
        Self::new(self.red * mag, self.green * mag, self.blue * mag, self.alpha)
    }

    pub fn from_css_color_string(s: &str) -> Result<Self, MathError> {
        let s = s.trim();
        let upper = s.to_ascii_uppercase();
        match upper.as_str() {
            "WHITE" => return Ok(Self::WHITE),
            "BLACK" => return Ok(Self::BLACK),
            "RED" => return Ok(Self::RED),
            "GREEN" => return Ok(Self::GREEN),
            "BLUE" => return Ok(Self::BLUE),
            "YELLOW" => return Ok(Self::YELLOW),
            "CYAN" | "AQUA" => return Ok(Self::CYAN),
            "MAGENTA" | "FUCHSIA" => return Ok(Self::MAGENTA),
            "TRANSPARENT" => return Ok(Self::TRANSPARENT),
            _ => {}
        }

        if let Some(hex) = s.strip_prefix('#') {
            match hex.len() {
                3 => {
                    let r = u8::from_str_radix(&hex[0..1], 16).map_err(|_| MathError)?;
                    let g = u8::from_str_radix(&hex[1..2], 16).map_err(|_| MathError)?;
                    let b = u8::from_str_radix(&hex[2..3], 16).map_err(|_| MathError)?;
                    return Ok(Self::new(
                        r as f32 / 15.0,
                        g as f32 / 15.0,
                        b as f32 / 15.0,
                        1.0,
                    ));
                }
                4 => {
                    let r = u8::from_str_radix(&hex[0..1], 16).map_err(|_| MathError)?;
                    let g = u8::from_str_radix(&hex[1..2], 16).map_err(|_| MathError)?;
                    let b = u8::from_str_radix(&hex[2..3], 16).map_err(|_| MathError)?;
                    let a = u8::from_str_radix(&hex[3..4], 16).map_err(|_| MathError)?;
                    return Ok(Self::new(
                        r as f32 / 15.0,
                        g as f32 / 15.0,
                        b as f32 / 15.0,
                        a as f32 / 15.0,
                    ));
                }
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| MathError)?;
                    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| MathError)?;
                    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| MathError)?;
                    return Ok(Self::from_bytes(r, g, b, 255));
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| MathError)?;
                    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| MathError)?;
                    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| MathError)?;
                    let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| MathError)?;
                    return Ok(Self::from_bytes(r, g, b, a));
                }
                _ => return Err(MathError),
            }
        }

        if let Some(inner) = s.strip_prefix("rgb(").and_then(|t| t.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                let r: f32 = parts[0].parse().map_err(|_| MathError)?;
                let g: f32 = parts[1].parse().map_err(|_| MathError)?;
                let b: f32 = parts[2].parse().map_err(|_| MathError)?;
                return Ok(Self::new(r / 255.0, g / 255.0, b / 255.0, 1.0));
            }
        }

        if let Some(inner) = s.strip_prefix("rgba(").and_then(|t| t.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 4 {
                let r: f32 = parts[0].parse().map_err(|_| MathError)?;
                let g: f32 = parts[1].parse().map_err(|_| MathError)?;
                let b: f32 = parts[2].parse().map_err(|_| MathError)?;
                let a: f32 = parts[3].parse().map_err(|_| MathError)?;
                return Ok(Self::new(r / 255.0, g / 255.0, b / 255.0, a));
            }
        }

        Err(MathError)
    }

    pub fn pack(&self, output: &mut [f64], starting_index: usize) -> Result<(), MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let target = output.get_mut(starting_index..end).ok_or(MathError)?;
        target[0] = self.red as f64;
        target[1] = self.green as f64;
        target[2] = self.blue as f64;
        target[3] = self.alpha as f64;
        Ok(())
    }

    pub fn unpack(array: &[f64], starting_index: usize) -> Result<Self, MathError> {
        let end = starting_index
            .checked_add(Self::PACKED_LENGTH)
            .ok_or(MathError)?;
        let slice = array.get(starting_index..end).ok_or(MathError)?;
        Ok(Self::new(
            slice[0] as f32,
            slice[1] as f32,
            slice[2] as f32,
            slice[3] as f32,
        ))
    }

    #[inline]
    pub fn equals_epsilon(&self, other: &Self, epsilon: f32) -> bool {
        (self.red - other.red).abs() <= epsilon
            && (self.green - other.green).abs() <= epsilon
            && (self.blue - other.blue).abs() <= epsilon
            && (self.alpha - other.alpha).abs() <= epsilon
    }
}
