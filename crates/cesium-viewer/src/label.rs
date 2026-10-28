//! Deterministic static text, using one bundled OFL font on native and WASM.
use cesium_formats::{Texture, TextureImage, TextureSampler};
use fontdue::{
    layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};
use std::sync::{Arc, OnceLock};
static FONT: OnceLock<Result<Font, String>> = OnceLock::new();
fn font() -> Result<&'static Font, String> {
    FONT.get_or_init(|| {
        Font::from_bytes(
            include_bytes!("../../../assets/fonts/GlobeSans-Regular.ttf") as &[u8],
            FontSettings::default(),
        )
        .map_err(|e| e.to_string())
    })
    .as_ref()
    .map_err(Clone::clone)
}
pub fn validate(text: &str, px: f32) -> Result<(), String> {
    if text.len() > 4096
        || text.chars().count() > 256
        || text.split('\n').count() > 8
        || text.trim().is_empty()
        || !px.is_finite()
        || !(8.0..=96.).contains(&px)
    {
        return Err("label requires 1..256 characters, <=8 lines and fontSize 8..96".into());
    }
    let font = font()?;
    for ch in text.chars().filter(|&c| c != '\n') {
        let is_rtl = ('\u{0590}'..='\u{08FF}').contains(&ch);
        if ch.is_control()
            || ('\u{0300}'..='\u{036f}').contains(&ch)
            || (font.lookup_glyph_index(ch) == 0 && !is_rtl)
        {
            return Err(format!(
                "unsupported label character U+{:04X}; font subset/complex shaping limit",
                ch as u32
            ));
        }
    }
    Ok(())
}
pub struct Raster {
    pub texture: Texture,
    pub width: f64,
    pub height: f64,
    pub baseline: f64,
}
pub fn raster(text: &str, px: f32) -> Result<Raster, String> {
    validate(text, px)?;
    let font = font()?;
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings::default());
    let sanitized: String = text
        .chars()
        .map(|ch| {
            if font.lookup_glyph_index(ch) == 0 {
                '?'
            } else {
                ch
            }
        })
        .collect();
    layout.append(&[font], &TextStyle::new(&sanitized, px, 0));
    let min_x = layout
        .glyphs()
        .iter()
        .map(|g| g.x)
        .fold(0., f32::min)
        .floor();
    let min_y = layout
        .glyphs()
        .iter()
        .map(|g| g.y)
        .fold(0., f32::min)
        .floor();
    let right = layout
        .glyphs()
        .iter()
        .map(|g| g.x + g.width as f32)
        .fold(1., f32::max)
        .ceil();
    let bottom = layout
        .glyphs()
        .iter()
        .map(|g| g.y + g.height as f32)
        .fold(layout.height(), f32::max)
        .ceil();
    let w = (right - min_x + 2.) as usize;
    let h = (bottom - min_y + 2.) as usize;
    if w > 2048 || h > 512 {
        return Err("label raster exceeds 2048x512 pixel limit".into());
    }
    let mut rgba = vec![0u8; w * h * 4];
    for pixel in rgba.as_chunks_mut::<4>().0 {
        pixel[..3].fill(255);
    }
    for g in layout.glyphs() {
        let (_, bitmap) = font.rasterize_config(g.key);
        let x = (g.x - min_x + 1.) as usize;
        let y = (g.y - min_y + 1.) as usize;
        for row in 0..g.height {
            for col in 0..g.width {
                let i = ((y + row) * w + x + col) * 4;
                if x + col >= w || y + row >= h {
                    return Err("glyph raster bounds".into());
                }
                rgba[i..i + 3].fill(255);
                let a = bitmap[row * g.width + col] as u32;
                let old = rgba[i + 3] as u32;
                rgba[i + 3] = (a + old * (255 - a) / 255) as u8;
            }
        }
    }
    let baseline = layout
        .lines()
        .and_then(|l| l.first())
        .map_or(0., |l| l.baseline_y)
        - min_y
        + 1.;
    Ok(Raster {
        texture: Texture {
            image: Arc::new(TextureImage {
                width: w as u32,
                height: h as u32,
                rgba,
            }),
            sampler: TextureSampler {
                wrap_s: 33071,
                wrap_t: 33071,
                ..Default::default()
            },
            tex_coord: 0,
        },
        width: w as f64,
        height: h as f64,
        baseline: baseline as f64,
    })
}
/// A deliberately bounded CSS font subset; all families use the bundled font.
pub fn css_font(v: &serde_json::Value) -> Result<f32, String> {
    let raw = if v.is_object() {
        super::entities::known(v, &["string"])?;
        v["string"].as_str()
    } else {
        v.as_str()
    }
    .ok_or("constant label font string required")?;
    let (size, family) = if let Some(p) = raw.split_once("px ") {
        (p.0.parse::<f32>().map_err(|_| "label font size")?, p.1)
    } else if let Some(p) = raw.split_once("pt ") {
        (
            p.0.parse::<f32>().map_err(|_| "label font size")? * (4.0 / 3.0),
            p.1,
        )
    } else {
        return Err("label font must specify size in px or pt followed by family".into());
    };
    if !matches!(family, "sans-serif" | "Globe Sans" | "Lucida Console") {
        return Err("label font family is unsupported".into());
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chinese_multiline_coverage_and_baseline() {
        let line = raster("全球地球 Globe 123", 24.).unwrap();
        let multi = raster("全球地球\nGlobe 123", 24.).unwrap();
        assert!(line.width > 150.);
        assert!(multi.height > line.height);
        assert!(multi.baseline > 0. && multi.baseline < multi.height);
        let data = &multi.texture.image.rgba;
        assert!(data.as_chunks::<4>().0.iter().any(|p| p[3] > 0));
        assert!(data.as_chunks::<4>().0.iter().any(|p| p[3] == 0));
        assert!(data.as_chunks::<4>().0.iter().all(|p| p[..3] == [255; 3]));
        assert!(raster("", 24.).is_err());
        assert!(raster("😀", 24.).is_err());
        assert!(raster(&"中".repeat(257), 24.).is_err());
        assert!(raster("font", 97.).is_err());
        assert!(raster(&"中".repeat(100), 96.).is_err());
    }
    #[test]
    fn bounded_css_fonts_and_text_placement() {
        assert_eq!(
            css_font(&serde_json::json!("24px sans-serif")).unwrap(),
            24.
        );
        assert!(css_font(&serde_json::json!("bold 24px serif")).is_err());
        let mut value = serde_json::json!([{"id":"label","positionDegrees":[110,25,1000],"color":[1,1,1,1],"shape":{"type":"label","text":"Label 地球","fontSize":24},"screen":{"horizontalOrigin":"LEFT","verticalOrigin":"BASELINE"}}]);
        let e = crate::entities::decode(&serde_json::to_vec(&value).unwrap())
            .unwrap()
            .remove(0);
        let p = e.primitive().unwrap();
        assert_eq!(p.vertices[0].position[0], 0.);
        assert!(p.vertices[2].position[1] > 0.);
        value[0]["shape"]["text"] = serde_json::json!("😀");
        assert!(crate::entities::decode(&serde_json::to_vec(&value).unwrap()).is_err());
    }
}
