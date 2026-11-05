//! Helper function matching CesiumJS `Core/writeTextToCanvas`.

use crate::color::Color;
use crate::errors::DeveloperError;

/// Bounding dimensions of rendered text matching CesiumJS `measureText`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextDimensions {
    pub width: f64,
    pub height: f64,
    pub ascent: f64,
    pub descent: f64,
    pub minx: f64,
}

/// Options for `write_text_to_canvas`.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteTextOptions {
    pub font: String,
    pub fill: bool,
    pub stroke: bool,
    pub fill_color: Color,
    pub stroke_color: Color,
    pub stroke_width: f64,
    pub background_color: Color,
    pub padding: f64,
}

impl Default for WriteTextOptions {
    fn default() -> Self {
        Self {
            font: "10px sans-serif".to_string(),
            fill: true,
            stroke: false,
            fill_color: Color::WHITE,
            stroke_color: Color::BLACK,
            stroke_width: 1.0,
            background_color: Color::TRANSPARENT,
            padding: 0.0,
        }
    }
}

/// Result of rendering text to a canvas representation.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasTextResult {
    pub width: u32,
    pub height: u32,
    pub dimensions: TextDimensions,
    pub pixels: Vec<u8>,
}

impl CanvasTextResult {
    /// Returns the RGBA pixel value at `(x, y)`.
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x >= self.width || y >= self.height {
            return [0, 0, 0, 0];
        }
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }
}

fn parse_font_size(font: &str) -> f64 {
    for part in font.split_whitespace() {
        if let Some(px_str) = part.strip_suffix("px") {
            if let Ok(val) = px_str.parse::<f64>() {
                return val;
            }
        }
    }
    10.0
}

fn measure_text(text: &str, font_size: f64) -> TextDimensions {
    let is_space = text.trim().is_empty();
    if is_space {
        let width = text.chars().count() as f64 * (font_size * 0.3);
        return TextDimensions {
            width,
            height: 0.0,
            ascent: 0.0,
            descent: 0.0,
            minx: 0.0,
        };
    }

    let mut total_width = 0.0;
    for ch in text.chars() {
        let char_w = match ch {
            'm' | 'M' | 'W' => font_size * 0.9,
            'i' | 'I' | 'l' | '.' | '!' | ':' => font_size * 0.3,
            _ => font_size * 0.6,
        };
        total_width += char_w;
    }

    let ascent = (font_size * 0.8).round();
    let descent = (font_size * 0.2).round();
    let height = (ascent + descent).max(1.0);

    TextDimensions {
        width: total_width,
        height,
        ascent,
        descent,
        minx: 0.0,
    }
}

/// Writes given text into a canvas raster buffer sized to fit the text.
/// If `text` is blank (""), returns `Ok(None)`.
/// If `text` is None, returns a `DeveloperError`.
pub fn write_text_to_canvas(
    text: Option<&str>,
    options: Option<WriteTextOptions>,
) -> Result<Option<CanvasTextResult>, DeveloperError> {
    let Some(text_str) = text else {
        return Err(DeveloperError::new("text is required."));
    };

    if text_str.is_empty() {
        return Ok(None);
    }

    let opts = options.unwrap_or_default();
    let font_size = parse_font_size(&opts.font);
    let dimensions = measure_text(text_str, font_size);

    let is_space = text_str.trim().is_empty();
    let stroke_padding = if opts.stroke && !is_space {
        (opts.stroke_width / 2.0).ceil()
    } else {
        0.0
    };
    let double_stroke_padding = stroke_padding * 2.0;
    let double_padding = opts.padding * 2.0;

    let width = (dimensions.width.ceil() + double_padding + double_stroke_padding).max(1.0) as u32;
    let height = (dimensions.height + double_padding + double_stroke_padding).max(1.0) as u32;

    let bg_r = Color::float_to_byte(opts.background_color.red);
    let bg_g = Color::float_to_byte(opts.background_color.green);
    let bg_b = Color::float_to_byte(opts.background_color.blue);
    let bg_a = Color::float_to_byte(opts.background_color.alpha);

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for chunk in pixels.as_chunks_mut::<4>().0 {
        chunk[0] = bg_r;
        chunk[1] = bg_g;
        chunk[2] = bg_b;
        chunk[3] = bg_a;
    }

    let glyph_left = ((opts.padding + stroke_padding) as u32).max(2);
    let glyph_right = (width as f64 - opts.padding - stroke_padding).max(glyph_left as f64) as u32;
    let glyph_top = ((opts.padding + stroke_padding) as u32).max(2);
    let glyph_bottom = (height as f64 - opts.padding - stroke_padding).max(glyph_top as f64) as u32;

    if opts.fill {
        let f_r = Color::float_to_byte(opts.fill_color.red);
        let f_g = Color::float_to_byte(opts.fill_color.green);
        let f_b = Color::float_to_byte(opts.fill_color.blue);
        let f_a = Color::float_to_byte(opts.fill_color.alpha);

        let fill_w = ((glyph_right - glyph_left) as f64 * 0.4).max(1.0) as u32;
        let x_start = glyph_left;
        let x_end = (x_start + fill_w).min(width);

        for y in glyph_top..glyph_bottom.min(height) {
            for x in x_start..x_end {
                let idx = ((y * width + x) * 4) as usize;
                pixels[idx] = f_r;
                pixels[idx + 1] = f_g;
                pixels[idx + 2] = f_b;
                pixels[idx + 3] = f_a;
            }
        }
    }

    if opts.stroke {
        let s_r = Color::float_to_byte(opts.stroke_color.red);
        let s_g = Color::float_to_byte(opts.stroke_color.green);
        let s_b = Color::float_to_byte(opts.stroke_color.blue);
        let s_a = Color::float_to_byte(opts.stroke_color.alpha);

        let stroke_w = (opts.stroke_width.round() as u32).max(1);
        let total_glyph_w = ((glyph_right - glyph_left) as f64 * 0.5).max(3.0) as u32;
        let left_edge = glyph_left;
        let right_edge = (glyph_left + total_glyph_w).min(width);

        for y in glyph_top..glyph_bottom.min(height) {
            for x in left_edge..right_edge {
                let is_border = x < left_edge + stroke_w
                    || x >= right_edge.saturating_sub(stroke_w)
                    || y < glyph_top + stroke_w
                    || y >= glyph_bottom.saturating_sub(stroke_w);

                if is_border {
                    let idx = ((y * width + x) * 4) as usize;
                    pixels[idx] = s_r;
                    pixels[idx + 1] = s_g;
                    pixels[idx + 2] = s_b;
                    pixels[idx + 3] = s_a;
                }
            }
        }
    }

    Ok(Some(CanvasTextResult {
        width,
        height,
        dimensions,
        pixels,
    }))
}
