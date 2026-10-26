//! Static CPU-composited label background and bounded glyph outlines.
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LabelStyle {
    #[default]
    Fill,
    Outline,
    FillAndOutline,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct Appearance {
    pub style: LabelStyle,
    pub show_background: bool,
    pub background_color: [f32; 4],
    pub background_padding: [f32; 2],
    pub outline_color: [f32; 4],
    pub outline_width: f32,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            style: LabelStyle::Fill,
            show_background: false,
            background_color: [0.165, 0.165, 0.165, 0.8],
            background_padding: [7., 5.],
            outline_color: [0., 0., 0., 1.],
            outline_width: 1.,
        }
    }
}
impl Appearance {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .background_color
            .iter()
            .chain(&self.outline_color)
            .any(|x| !x.is_finite() || !(0.0..=1.).contains(x))
            || self
                .background_padding
                .iter()
                .any(|x| !x.is_finite() || !(0.0..=64.).contains(x))
            || !self.outline_width.is_finite()
            || !(0.0..=8.).contains(&self.outline_width)
        {
            return Err("label appearance color/padding/outline limits".into());
        }
        Ok(())
    }
}
pub fn czml(g: &Value) -> Result<Appearance, String> {
    use crate::entities::{known, number};
    let mut a = Appearance::default();
    if let Some(v) = g.get("style") {
        let v = if v.is_object() {
            known(v, &["labelStyle"])?;
            &v["labelStyle"]
        } else {
            v
        };
        a.style = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
    }
    if let Some(v) = g.get("showBackground") {
        a.show_background = crate::entities::boolean(v)?;
    }
    if let Some(v) = g.get("backgroundColor") {
        a.background_color = crate::entity_material::rgba(v)?;
    }
    if let Some(v) = g.get("outlineColor") {
        a.outline_color = crate::entity_material::rgba(v)?;
    }
    if let Some(v) = g.get("outlineWidth") {
        a.outline_width = number(v)? as f32;
    }
    if let Some(v) = g.get("backgroundPadding") {
        known(v, &["cartesian2"])?;
        let p = v["cartesian2"]
            .as_array()
            .ok_or("backgroundPadding cartesian2 required")?;
        if p.len() != 2 {
            return Err("backgroundPadding length".into());
        }
        a.background_padding = [number(&p[0])? as f32, number(&p[1])? as f32];
    }
    a.validate()?;
    Ok(a)
}
/// Compose in linear light, then encode straight-alpha sRGB for the shared renderer.
pub fn raster(
    text: &str,
    px: f32,
    a: &Appearance,
    fill: [f32; 4],
) -> Result<crate::label::Raster, String> {
    a.validate()?;
    if fill
        .iter()
        .any(|x| !x.is_finite() || !(0.0..=1.).contains(x))
    {
        return Err("label fill RGBA".into());
    }
    let base = crate::label::raster(text, px)?;
    let w = base.width as usize;
    let h = base.height as usize;
    let stroke = if a.style != LabelStyle::Fill {
        a.outline_width
    } else {
        0.
    };
    let margin = if stroke > 0. {
        (stroke + 1.).ceil() as usize
    } else {
        0
    };
    let bg = if a.show_background {
        a.background_padding.map(|x| x.ceil() as usize)
    } else {
        [0; 2]
    };
    let pad = [margin.max(bg[0]), margin.max(bg[1])];
    let width = w + 2 * pad[0];
    let height = h + 2 * pad[1];
    if width > 2048 || height > 512 {
        return Err("styled label raster exceeds 2048x512".into());
    }
    let mut mask = vec![0u8; width * height];
    for y in 0..h {
        for x in 0..w {
            mask[(y + pad[1]) * width + x + pad[0]] = base.texture.image.rgba[(y * w + x) * 4 + 3];
        }
    }
    let mut grown = mask.clone();
    if stroke > 0. {
        let offsets = (-((margin) as isize)..=margin as isize)
            .flat_map(|dy| (-((margin) as isize)..=margin as isize).map(move |dx| (dx, dy)))
            .filter_map(|(x, y)| {
                let weight = (stroke + 1. - ((x * x + y * y) as f32).sqrt()).clamp(0., 1.);
                (weight > 0.).then_some((x, y, weight))
            })
            .collect::<Vec<_>>();
        for y in pad[1]..pad[1] + h {
            for x in pad[0]..pad[0] + w {
                let v = mask[y * width + x];
                if v == 0 {
                    continue;
                }
                for &(dx, dy, weight) in &offsets {
                    let qx = x as isize + dx;
                    let qy = y as isize + dy;
                    if qx >= 0 && qy >= 0 && (qx as usize) < width && (qy as usize) < height {
                        let i = qy as usize * width + qx as usize;
                        grown[i] = grown[i].max((v as f32 * weight).round() as u8);
                    }
                }
            }
        }
    }
    let fill = crate::entity_material::linear(fill);
    let outline = crate::entity_material::linear(a.outline_color);
    let background = crate::entity_material::linear(a.background_color);
    let fallback = if stroke > 0. {
        a.outline_color
    } else if a.show_background {
        a.background_color
    } else {
        encode(fill)
    };
    let mut rgba = vec![0u8; width * height * 4];
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            let mut out = [0.; 4];
            if a.show_background
                && x >= pad[0] - bg[0]
                && x < pad[0] + w + bg[0]
                && y >= pad[1] - bg[1]
                && y < pad[1] + h + bg[1]
            {
                over(&mut out, background, 1.);
            }
            if stroke > 0. {
                over(
                    &mut out,
                    outline,
                    grown[i].saturating_sub(mask[i]) as f32 / 255.,
                );
            }
            if a.style != LabelStyle::Outline {
                over(&mut out, fill, mask[i] as f32 / 255.);
            }
            let result = if out[3] > 0. {
                encode([out[0] / out[3], out[1] / out[3], out[2] / out[3], out[3]])
            } else {
                [fallback[0], fallback[1], fallback[2], 0.]
            };
            for c in 0..4 {
                rgba[i * 4 + c] = (result[c].clamp(0., 1.) * 255.).round() as u8;
            }
        }
    }
    Ok(crate::label::Raster {
        texture: cesium_formats::Texture {
            image: std::sync::Arc::new(cesium_formats::TextureImage {
                width: width as u32,
                height: height as u32,
                rgba,
            }),
            sampler: base.texture.sampler,
            tex_coord: 0,
        },
        width: width as f64,
        height: height as f64,
        baseline: base.baseline + pad[1] as f64,
    })
}
fn encode(c: [f32; 4]) -> [f32; 4] {
    let rgb = |x: f32| {
        if x <= 0.0031308 {
            x * 12.92
        } else {
            1.055 * x.powf(1. / 2.4) - 0.055
        }
    };
    [rgb(c[0]), rgb(c[1]), rgb(c[2]), c[3]]
}
fn over(dst: &mut [f32; 4], src: [f32; 4], coverage: f32) {
    let alpha = src[3] * coverage;
    for c in 0..3 {
        dst[c] = src[c] * alpha + dst[c] * (1. - alpha);
    }
    dst[3] = alpha + dst[3] * (1. - alpha);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn background_padding_and_independent_colors() {
        let base = crate::label::raster("地球 O", 32.).unwrap();
        let a = Appearance {
            show_background: true,
            background_color: [0., 0., 1., 1.],
            background_padding: [12., 8.],
            ..Default::default()
        };
        let r = raster("地球 O", 32., &a, [1., 0., 0., 1.]).unwrap();
        assert_eq!(r.width, base.width + 24.);
        assert_eq!(r.height, base.height + 16.);
        assert_eq!(r.baseline, base.baseline + 8.);
        assert_eq!(&r.texture.image.rgba[..4], &[0, 0, 255, 255]);
        assert!(r
            .texture
            .image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] == 255 && p[1] == 0 && p[2] == 0 && p[3] == 255));
    }
    #[test]
    fn outline_only_has_hollow_ink_and_colored_ring() {
        let base = crate::label::raster("O", 48.).unwrap();
        let a = Appearance {
            style: LabelStyle::Outline,
            outline_color: [1., 1., 0., 1.],
            outline_width: 3.,
            ..Default::default()
        };
        let r = raster("O", 48., &a, [1.; 4]).unwrap();
        let margin = 4usize;
        assert_eq!(r.width, base.width + 8.);
        let bw = base.width as usize;
        let w = r.width as usize;
        let solid = base
            .texture
            .image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .position(|p| p[3] == 255)
            .unwrap();
        let i = ((solid / bw + margin) * w + solid % bw + margin) * 4;
        assert_eq!(r.texture.image.rgba[i + 3], 0);
        assert!(r
            .texture
            .image
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p == &[255, 255, 0, 255]));
    }
    #[test]
    fn czml_style_and_resource_bounds() {
        let a=czml(&serde_json::json!({"style":{"labelStyle":"FILL_AND_OUTLINE"},"showBackground":{"boolean":true},"outlineWidth":2.5,"backgroundPadding":{"cartesian2":[12,8]}})).unwrap();
        assert!(a.show_background);
        assert_eq!(a.style, LabelStyle::FillAndOutline);
        assert!(czml(&serde_json::json!({"outlineWidth":9})).is_err());
        assert!(czml(&serde_json::json!({"backgroundPadding":{"cartesian2":[65,0]}})).is_err());
        assert!(
            czml(&serde_json::json!({"style":{"labelStyle":"FILL","interval":"bad"}})).is_err()
        );
    }
}
