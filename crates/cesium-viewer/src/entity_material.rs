//! Constant procedural Entity/CZML materials. Colors are sRGB; GLB factors remain linear.
use cesium_formats::{Pattern, Primitive};
use glam::DVec3;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum SurfaceMaterial {
    Image {
        image: String,
        #[serde(default = "white")]
        color: [f32; 4],
        #[serde(default = "repeat")]
        repeat: [f32; 2],
        #[serde(default)]
        transparent: bool,
    },
    RimLighting {
        #[serde(default = "white")]
        color: [f32; 4],
        #[serde(default = "default_rim_power", rename = "rimPower")]
        rim_power: f32,
    },
    Stripe {
        #[serde(default = "white", rename = "evenColor")]
        even: [f32; 4],
        #[serde(default = "black", rename = "oddColor")]
        odd: [f32; 4],
        #[serde(default = "unit_f32")]
        repeat: f32,
        #[serde(default)]
        offset: f32,
        #[serde(default)]
        orientation: Orientation,
    },
    Dot {
        #[serde(rename = "lightColor")]
        light: [f32; 4],
        #[serde(rename = "darkColor")]
        dark: [f32; 4],
        repeat: [f32; 2],
    },
    Checkerboard {
        #[serde(rename = "evenColor")]
        even: [f32; 4],
        #[serde(rename = "oddColor")]
        odd: [f32; 4],
        repeat: [f32; 2],
    },
    Grid {
        color: [f32; 4],
        #[serde(rename = "lineCount")]
        count: [f32; 2],
        #[serde(rename = "lineThickness")]
        thickness: [f32; 2],
        #[serde(rename = "cellAlpha")]
        cell_alpha: f32,
    },
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Orientation {
    #[default]
    Horizontal,
    Vertical,
}
fn white() -> [f32; 4] {
    [1.; 4]
}
fn black() -> [f32; 4] {
    [0., 0., 0., 1.]
}
fn unit_f32() -> f32 {
    1.0
}
fn default_rim_power() -> f32 {
    4.0
}
fn repeat() -> [f32; 2] {
    [1.; 2]
}
fn valid_color(c: [f32; 4]) -> Result<(), String> {
    if c.iter().any(|x| !x.is_finite() || !(0.0..=1.).contains(x)) {
        return Err("material RGBA must be finite in 0..1".into());
    }
    Ok(())
}
pub(crate) fn linear(mut c: [f32; 4]) -> [f32; 4] {
    for v in &mut c[..3] {
        *v = if *v <= 0.04045 {
            *v / 12.92
        } else {
            ((*v + 0.055) / 1.055).powf(2.4)
        };
    }
    c
}
impl SurfaceMaterial {
    pub fn parameters(&self) -> Result<([f32; 4], Pattern), String> {
        let (color, p) = match *self {
            Self::Image {
                ref image,
                color,
                repeat,
                transparent,
            } => {
                valid_color(color)?;
                if image.len() > 6 * 1024 * 1024 || !image.starts_with("data:image/") {
                    return Err(
                        "resolve image URLs before Rust loading; expected PNG/JPEG base64 data URI"
                            .into(),
                    );
                }
                if repeat
                    .iter()
                    .any(|v| !v.is_finite() || !(0.01..=1024.).contains(v))
                {
                    return Err("image repeat limit".into());
                }
                (
                    linear(color),
                    Pattern {
                        kind: 4,
                        secondary: [0.; 4],
                        repeat,
                        offset: 0.,
                        axis: 0,
                        thickness: [0.; 2],
                        cell_alpha: if transparent { 1. } else { 0. },
                    },
                )
            }
            Self::RimLighting { color, rim_power } => {
                valid_color(color)?;
                (
                    linear(color),
                    Pattern {
                        kind: 1,
                        secondary: linear([0.035, 0.16, 0.32, 1.0]),
                        repeat: [rim_power.clamp(0.1, 10.0), 1.],
                        offset: 0.,
                        axis: 0,
                        thickness: [1.; 2],
                        cell_alpha: 1.,
                    },
                )
            }
            Self::Stripe {
                even,
                odd,
                repeat,
                offset,
                orientation,
            } => {
                valid_color(even)?;
                valid_color(odd)?;
                if !repeat.is_finite()
                    || !(0.01..=1024.).contains(&repeat)
                    || !offset.is_finite()
                    || offset.abs() > 1e6
                {
                    return Err("stripe repeat/offset limit".into());
                }
                (
                    linear(even),
                    Pattern {
                        kind: 1,
                        secondary: linear(odd),
                        repeat: [repeat, 1.],
                        offset,
                        axis: u32::from(matches!(orientation, Orientation::Vertical)),
                        thickness: [1.; 2],
                        cell_alpha: 1.,
                    },
                )
            }
            Self::Dot {
                light,
                dark,
                repeat,
            } => {
                valid_color(light)?;
                valid_color(dark)?;
                if repeat
                    .iter()
                    .any(|v| !v.is_finite() || !(0.01..=1024.).contains(v))
                {
                    return Err("dot repeat limit".into());
                }
                (
                    linear(light),
                    Pattern {
                        kind: 6,
                        secondary: linear(dark),
                        repeat,
                        offset: 0.,
                        axis: 0,
                        thickness: [1.; 2],
                        cell_alpha: 1.,
                    },
                )
            }
            Self::Checkerboard { even, odd, repeat } => {
                valid_color(even)?;
                valid_color(odd)?;
                if repeat
                    .iter()
                    .any(|v| !v.is_finite() || !(0.01..=1024.).contains(v))
                {
                    return Err("checkerboard repeat limit".into());
                }
                (
                    linear(even),
                    Pattern {
                        kind: 2,
                        secondary: linear(odd),
                        repeat,
                        offset: 0.,
                        axis: 0,
                        thickness: [1.; 2],
                        cell_alpha: 1.,
                    },
                )
            }
            Self::Grid {
                color,
                count,
                thickness,
                cell_alpha,
            } => {
                valid_color(color)?;
                if count
                    .iter()
                    .any(|v| !v.is_finite() || !(1.0..=1024.).contains(v))
                    || thickness
                        .iter()
                        .any(|v| !v.is_finite() || !(0.0..=32.).contains(v))
                    || !cell_alpha.is_finite()
                    || !(0.0..=1.).contains(&cell_alpha)
                {
                    return Err("grid count/thickness/alpha limit".into());
                }
                (
                    linear(color),
                    Pattern {
                        kind: 3,
                        secondary: [0.; 4],
                        repeat: count,
                        offset: 0.,
                        axis: 0,
                        thickness,
                        cell_alpha,
                    },
                )
            }
        };
        Ok((color, p))
    }
}
pub fn apply(entity: &crate::entities::Entity, p: &mut Primitive) -> Result<(), String> {
    p.material.base_color = linear(entity.color);
    if let Some(spec) = &entity.material {
        let (color, pattern) = spec.parameters()?;
        p.material.base_color = color;
        p.material.alpha_blend = color[3] < 1.
            || matches!(pattern.kind, 1 | 2 | 6) && pattern.secondary[3] < 1.
            || pattern.kind == 3 && pattern.cell_alpha < 1.
            || pattern.kind == 4 && pattern.cell_alpha > 0.5;
        if let SurfaceMaterial::Image { image, .. } = spec {
            let mut texture = decode_image(image)?;
            if matches!(entity.shape, crate::entities::Shape::PolygonSurface { .. }) {
                // Cesium authored ST uses a bottom image origin; WebGPU image rows start at top.
                let data = std::sync::Arc::make_mut(&mut texture.image);
                let stride = data.width as usize * 4;
                let h = data.height as usize;
                for y in 0..h / 2 {
                    let (top, bottom) = data.rgba.split_at_mut((h - 1 - y) * stride);
                    top[y * stride..(y + 1) * stride].swap_with_slice(&mut bottom[..stride]);
                }
            }
            p.material.textures[0] = Some(texture);
        }
        p.material.pattern = Some(pattern);
        p.material.unlit = true;
        if matches!(entity.shape, crate::entities::Shape::PolygonSurface { .. }) {
            return Ok(());
        }
        let inverse = if entity.shape.geographic() {
            let origin = if entity.shape.standalone() {
                crate::geographic::anchor(&entity.shape)?
            } else {
                let c = entity.position_degrees.ok_or("material center")?;
                cesium_math::Cartographic::degrees(c[0], c[1], c[2])
                    .map_err(|e| e.to_string())?
                    .to_ecef()
            };
            cesium_math::Cartographic::from_ecef(origin)
                .map_err(|e| e.to_string())?
                .enu_to_ecef()
                .inverse()
        } else {
            glam::DMat4::IDENTITY
        };
        let local = |v: [f32; 3]| inverse.transform_vector3(DVec3::from_array(v.map(f64::from)));
        let mut min = DVec3::splat(f64::INFINITY);
        let mut max = DVec3::splat(f64::NEG_INFINITY);
        for v in &p.vertices {
            let x = local(v.position);
            min = min.min(x);
            max = max.max(x);
        }
        let extent = (max - min).max(DVec3::splat(1e-6));
        for v in &mut p.vertices {
            let point = (local(v.position) - min) / extent;
            let n = local(v.normal).abs();
            let mut uv = if n.z >= n.x.max(n.y) {
                [point.x, point.y]
            } else if n.y >= n.x {
                [point.x, point.z]
            } else {
                [point.y, point.z]
            };
            if let crate::entities::Shape::Rectangle {
                st_rotation: Some(st),
                ..
            } = entity.shape
            {
                if st.abs() > 1e-8 {
                    let cos_st = st.cos();
                    let sin_st = st.sin();
                    let du = uv[0] - 0.5;
                    let dv = uv[1] - 0.5;
                    uv[0] = 0.5 + du * cos_st - dv * sin_st;
                    uv[1] = 0.5 + du * sin_st + dv * cos_st;
                }
            }
            v.uv = uv.map(|x| x as f32);
        }
    } else {
        p.material.alpha_blend = entity.color[3] < 1.;
    }
    Ok(())
}
pub(crate) fn decode_image(uri: &str) -> Result<cesium_formats::Texture, String> {
    use base64::Engine;
    let (format, payload) = if let Some(s) = uri.strip_prefix("data:image/png;base64,") {
        (image::ImageFormat::Png, s)
    } else if let Some(s) = uri
        .strip_prefix("data:image/jpeg;base64,")
        .or_else(|| uri.strip_prefix("data:image/jpg;base64,"))
    {
        (image::ImageFormat::Jpeg, s)
    } else {
        return Err("PNG/JPEG base64 data URI required".into());
    };
    if payload.len() > 6 * 1024 * 1024 {
        return Err("image encoded size limit".into());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .map_err(|_| "invalid image base64")?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err("image input limit".into());
    }
    let mut reader = image::ImageReader::with_format(std::io::Cursor::new(bytes), format);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    limits.max_alloc = Some(32 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| format!("image decode: {e}"))?
        .to_rgba8();
    Ok(cesium_formats::Texture {
        image: std::sync::Arc::new(cesium_formats::TextureImage {
            width: image.width(),
            height: image.height(),
            rgba: image.into_raw(),
        }),
        sampler: Default::default(),
        tex_coord: 0,
    })
}
fn known(v: &Value, keys: &[&str]) -> Result<(), String> {
    let o = v.as_object().ok_or("material object required")?;
    for k in o.keys() {
        if !keys.contains(&k.as_str()) {
            return Err(format!("unsupported CZML material property: {k}"));
        }
    }
    Ok(())
}
fn number(v: &Value) -> Result<f32, String> {
    if v.is_object() {
        known(v, &["number"])?;
        return number(&v["number"]);
    }
    let n = v.as_f64().ok_or("material number required")? as f32;
    if !n.is_finite() {
        return Err("finite material number required".into());
    }
    Ok(n)
}
fn pair(v: &Value) -> Result<[f32; 2], String> {
    known(v, &["cartesian2"])?;
    let a = v["cartesian2"]
        .as_array()
        .ok_or("material Cartesian2 required")?;
    if a.len() != 2 {
        return Err("material Cartesian2 required".into());
    }
    Ok([number(&a[0])?, number(&a[1])?])
}
pub(crate) fn rgba(v: &Value) -> Result<[f32; 4], String> {
    let v = if let Some(arr) = v.as_array() {
        arr.first().ok_or("color intervals array not empty")?
    } else {
        v
    };
    known(v, &["rgba", "rgbaf", "interval"])?;
    if v.get("rgba").is_some() == v.get("rgbaf").is_some() {
        return Err("exactly one RGBA encoding required".into());
    }
    if let Some(a) = v.get("rgbaf") {
        let a = a
            .as_array()
            .filter(|a| a.len() == 4)
            .ok_or("four RGBA float components required")?;
        let mut c = [0.; 4];
        for (i, n) in a.iter().enumerate() {
            c[i] = n
                .as_f64()
                .filter(|n| n.is_finite() && (0.0..=1.).contains(n))
                .ok_or("RGBA float 0..1 required")? as f32;
        }
        return Ok(c);
    }
    let a = v["rgba"].as_array().ok_or("material RGBA bytes required")?;
    if a.len() != 4 {
        return Err("material RGBA bytes required".into());
    }
    let mut c = [0.; 4];
    for (i, n) in a.iter().enumerate() {
        c[i] = n
            .as_u64()
            .filter(|v| *v <= 255)
            .ok_or("RGBA bytes required")? as f32
            / 255.;
    }
    Ok(c)
}
pub fn czml(g: &Value) -> Result<([f32; 4], Option<SurfaceMaterial>), String> {
    let Some(m) = g.get("material") else {
        return Ok(([1.; 4], None));
    };
    known(
        m,
        &["solidColor", "stripe", "checkerboard", "grid", "image"],
    )?;
    let keys: Vec<_> = ["solidColor", "stripe", "checkerboard", "grid", "image"]
        .into_iter()
        .filter(|k| m.get(*k).is_some())
        .collect();
    if keys.len() != 1 {
        return Err("one CZML material type required".into());
    }
    let key = keys[0];
    let v = &m[key];
    let spec = match key {
        "solidColor" => {
            known(v, &["color"])?;
            return Ok((rgba(&v["color"])?, None));
        }
        "image" => {
            known(v, &["image", "repeat", "color", "transparent"])?;
            let uri = if let Some(uri) = v["image"].as_str() {
                uri.to_owned()
            } else {
                known(&v["image"], &["uri"])?;
                v["image"]["uri"]
                    .as_str()
                    .ok_or("constant image URI required")?
                    .to_owned()
            };
            let transparent = match v.get("transparent") {
                None => false,
                Some(v) => {
                    if let Some(b) = v.as_bool() {
                        b
                    } else {
                        known(v, &["boolean"])?;
                        v["boolean"]
                            .as_bool()
                            .ok_or("constant transparent boolean required")?
                    }
                }
            };
            SurfaceMaterial::Image {
                image: uri,
                color: v.get("color").map(rgba).transpose()?.unwrap_or([1.; 4]),
                repeat: v.get("repeat").map(pair).transpose()?.unwrap_or([1.; 2]),
                transparent,
            }
        }
        "stripe" => {
            known(
                v,
                &["evenColor", "oddColor", "repeat", "offset", "orientation"],
            )?;
            SurfaceMaterial::Stripe {
                even: v.get("evenColor").map(rgba).transpose()?.unwrap_or([1.; 4]),
                odd: v
                    .get("oddColor")
                    .map(rgba)
                    .transpose()?
                    .unwrap_or([0., 0., 0., 1.]),
                repeat: v.get("repeat").map(number).transpose()?.unwrap_or(1.),
                offset: v.get("offset").map(number).transpose()?.unwrap_or(0.),
                orientation: v
                    .get("orientation")
                    .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
                    .transpose()?
                    .unwrap_or_default(),
            }
        }
        "checkerboard" => {
            known(v, &["evenColor", "oddColor", "repeat"])?;
            SurfaceMaterial::Checkerboard {
                even: v.get("evenColor").map(rgba).transpose()?.unwrap_or([1.; 4]),
                odd: v
                    .get("oddColor")
                    .map(rgba)
                    .transpose()?
                    .unwrap_or([0., 0., 0., 1.]),
                repeat: v.get("repeat").map(pair).transpose()?.unwrap_or([2.; 2]),
            }
        }
        _ => {
            known(v, &["color", "cellAlpha", "lineCount", "lineThickness"])?;
            SurfaceMaterial::Grid {
                color: v.get("color").map(rgba).transpose()?.unwrap_or([1.; 4]),
                cell_alpha: v.get("cellAlpha").map(number).transpose()?.unwrap_or(0.1),
                count: v.get("lineCount").map(pair).transpose()?.unwrap_or([8.; 2]),
                thickness: v
                    .get("lineThickness")
                    .map(pair)
                    .transpose()?
                    .unwrap_or([1.; 2]),
            }
        }
    };
    spec.parameters()?;
    Ok(([1.; 4], Some(spec)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_decode_alpha_switch_and_limits() {
        use base64::Engine;
        for (mime, bytes) in [
            (
                "png",
                include_bytes!("../../../tests/entities/image-test.png").as_slice(),
            ),
            (
                "jpeg",
                include_bytes!("../../../tests/entities/image-test.jpg").as_slice(),
            ),
        ] {
            let uri = format!(
                "data:image/{mime};base64,{}",
                base64::engine::general_purpose::STANDARD.encode(bytes)
            );
            let t = decode_image(&uri).unwrap();
            assert_eq!(t.image.width, 64);
            assert_eq!(t.image.height, 64);
        }
        let mut e =
            crate::entities::decode(include_bytes!("../../../tests/entities/image-outline.json"))
                .unwrap()
                .remove(0);
        let p = e.primitive().unwrap();
        assert!(p.material.alpha_blend);
        assert_eq!(p.material.pattern.unwrap().kind, 4);
        if let Some(SurfaceMaterial::Image { transparent, .. }) = &mut e.material {
            *transparent = false;
        }
        let p = e.primitive().unwrap();
        assert!(!p.material.alpha_blend);
        assert_eq!(p.material.pattern.unwrap().cell_alpha, 0.);
    }
    #[test]
    fn corrupt_and_oversized_images_rejected() {
        use base64::Engine;
        assert!(decode_image("data:image/png;base64,AAAA").is_err());
        assert!(decode_image("data:image/svg+xml;base64,AAAA").is_err());
        let image = image::DynamicImage::new_rgba8(4096, 1);
        let mut bytes = std::io::Cursor::new(Vec::new());
        image.write_to(&mut bytes, image::ImageFormat::Png).unwrap();
        let uri = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes.into_inner())
        );
        assert!(decode_image(&uri).is_err());
    }
    #[test]
    fn constant_materials_limits_and_czml() {
        let g = serde_json::json!({"material":{"stripe":{"evenColor":{"rgba":[255,0,0,128]},"oddColor":{"rgba":[0,0,255,255]},"repeat":8,"orientation":"VERTICAL"}}});
        let (_, s) = czml(&g).unwrap();
        let (c, p) = s.unwrap().parameters().unwrap();
        assert!(c[3] < 1.);
        assert_eq!(p.axis, 1);
        assert_eq!(p.kind, 1);
        let invalid = SurfaceMaterial::Grid {
            color: [1.; 4],
            count: [8.; 2],
            thickness: [1.; 2],
            cell_alpha: 2.,
        };
        assert!(invalid.parameters().is_err());
        assert!(czml(&serde_json::json!({"material":{"image":{"image":"foo.png"}}})).is_err());
    }
}
