//! Bounded static screen graphics. Local XY are framebuffer pixels, not metres.
use crate::entities::{Entity, Shape};
use cesium_formats::{Material, Pattern, Primitive, Vertex};
use glam::DMat4;
pub fn validate(e: &Entity) -> Result<(), String> {
    if e.material.is_some() || e.outline || !e.fill {
        return Err(
            "screen graphics require fill=true, no surface material or geometry outline".into(),
        );
    }
    e.screen.validate()?;
    if !matches!(e.shape, Shape::Billboard { .. }) && e.screen.rotation != 0. {
        return Err("rotation currently requires billboard graphics".into());
    }
    if matches!(e.shape, Shape::Point { .. }) && e.screen.pixel_offset_scale_by_distance.is_some() {
        return Err("point has no pixelOffset".into());
    }
    if matches!(e.shape, Shape::Point { .. })
        && (e.screen.horizontal_origin != crate::screen_options::HorizontalOrigin::Center
            || e.screen.vertical_origin != crate::screen_options::VerticalOrigin::Center)
    {
        return Err("point origins must be CENTER".into());
    }
    if !matches!(e.shape, Shape::Label { .. })
        && e.screen.vertical_origin == crate::screen_options::VerticalOrigin::Baseline
    {
        return Err("BASELINE origin requires a label".into());
    }
    let size = |x: f64| x.is_finite() && x > 0. && x <= 2048.;
    match &e.shape {
        Shape::Point {
            pixel_size,
            outline_width,
            outline_color,
        } => {
            if !size(*pixel_size)
                || !outline_width.is_finite()
                || !(0.0..=128.).contains(outline_width)
                || !size(pixel_size + 2. * outline_width)
                || outline_color
                    .iter()
                    .any(|x| !x.is_finite() || !(0.0..=1.).contains(x))
            {
                return Err("point pixelSize/outline limits".into());
            }
        }
        Shape::Billboard {
            width,
            height,
            scale,
            pixel_offset,
            image,
            image_sub_region,
            ..
        } => {
            if let Some(sub) = image_sub_region {
                if sub.iter().any(|v| !v.is_finite() || *v < 0.) || sub[2] <= 0. || sub[3] <= 0. {
                    return Err(
                        "imageSubRegion requires finite non-negative x, y, width, height".into(),
                    );
                }
            }
            if !size(*width)
                || !size(*height)
                || !scale.is_finite()
                || *scale <= 0.
                || *scale > 64.
                || !size(width * scale)
                || !size(height * scale)
                || pixel_offset
                    .iter()
                    .any(|x| !x.is_finite() || x.abs() > 4096.)
            {
                return Err("billboard size/scale/pixelOffset limits".into());
            }
            if image.len() > 6 * 1024 * 1024 || !image.starts_with("data:image/") {
                return Err("resolve billboard image URI before Rust loading".into());
            }
        }
        Shape::Label {
            text,
            font_size,
            scale,
            pixel_offset,
            appearance,
            ..
        } => {
            appearance.validate()?;
            crate::label::validate(text, *font_size)?;
            if !scale.is_finite()
                || *scale <= 0.
                || *scale > 8.
                || pixel_offset
                    .iter()
                    .any(|x| !x.is_finite() || x.abs() > 4096.)
            {
                return Err("label scale/pixelOffset limit".into());
            }
        }
        _ => return Err("screen shape required".into()),
    }
    Ok(())
}
pub fn primitive(e: &Entity) -> Result<Primitive, String> {
    validate(e)?;
    let mut material = Material {
        base_color: crate::entity_material::linear(e.color),
        unlit: true,
        double_sided: true,
        metallic: 0.,
        alpha_blend: true,
        ..Default::default()
    };
    let (width, height, offset, baseline, [u0, u1, v0, v1]) = match &e.shape {
        Shape::Point {
            pixel_size,
            outline_width,
            outline_color,
        } => {
            let diameter = pixel_size + 2. * outline_width;
            material.pattern = Some(Pattern {
                kind: 5,
                secondary: crate::entity_material::linear(*outline_color),
                repeat: [(*pixel_size / diameter) as f32, 1.],
                offset: 0.,
                axis: 0,
                thickness: [1.; 2],
                cell_alpha: 1.,
            });
            (diameter, diameter, [0.; 2], None, [0.0, 1.0, 0.0, 1.0])
        }
        Shape::Billboard {
            image,
            width,
            height,
            scale,
            pixel_offset,
            image_sub_region,
            ..
        } => {
            let tex = crate::entity_material::decode_image(image)?;
            let tex_w = tex.image.width as f64;
            let tex_h = tex.image.height as f64;
            let (uv_box, w, h) = if let Some([sx, sy, sw, sh]) = image_sub_region {
                let u0 = (sx / tex_w) as f32;
                let u1 = ((sx + sw) / tex_w) as f32;
                let v0 = (sy / tex_h) as f32;
                let v1 = ((sy + sh) / tex_h) as f32;
                ([u0, u1, v0, v1], *sw, *sh)
            } else {
                ([0.0, 1.0, 0.0, 1.0], *width, *height)
            };
            material.textures[0] = Some(tex);
            (w * scale, h * scale, *pixel_offset, None, uv_box)
        }
        Shape::Label {
            text,
            font_size,
            scale,
            pixel_offset,
            appearance,
            font,
            show_background,
            background_color,
            outline_color,
            outline_width,
            style,
            fill_color,
        } => {
            let mut app = appearance.clone();
            if let Some(sb) = show_background {
                app.show_background = *sb;
            }
            if let Some(bc) = background_color {
                app.background_color = *bc;
            }
            if let Some(oc) = outline_color {
                app.outline_color = *oc;
            }
            if let Some(ow) = outline_width {
                app.outline_width = *ow;
            }
            if let Some(st) = style {
                app.style = *st;
            }
            let fill = fill_color.unwrap_or(e.color);
            let effective_size = if let Some(f) = font {
                crate::label::css_font(&serde_json::Value::String(f.clone())).unwrap_or(*font_size)
            } else {
                *font_size
            };
            let label = crate::label_appearance::raster(text, effective_size, &app, fill)?;
            material.base_color = [1.; 4];
            let dimensions = (
                label.width * scale,
                label.height * scale,
                *pixel_offset,
                Some(label.baseline * scale),
                [0.0, 1.0, 0.0, 1.0],
            );
            material.textures[0] = Some(label.texture);
            dimensions
        }
        _ => return Err("screen shape required".into()),
    };
    if width > 2048. || height > 2048. {
        return Err("scaled screen dimensions limit".into());
    }
    use crate::screen_options::{HorizontalOrigin as H, VerticalOrigin as V};
    let dx = match e.screen.horizontal_origin {
        H::Center => 0.,
        H::Left => width / 2.,
        H::Right => -width / 2.,
    };
    let dy = match e.screen.vertical_origin {
        V::Center => 0.,
        V::Top => -height / 2.,
        V::Bottom => height / 2.,
        V::Baseline => baseline.ok_or("label baseline")? - height / 2.,
    };
    let vertices = [
        (-1., -1., [u0, v1]),
        (1., -1., [u1, v1]),
        (1., 1., [u1, v0]),
        (-1., 1., [u0, v0]),
    ]
    .into_iter()
    .map(|(x, y, uv)| Vertex {
        position: [
            (x * width / 2. + dx + offset[0]) as f32,
            (y * height / 2. + dy - offset[1]) as f32,
            0.,
        ],
        uv,
        uv1: uv,
        normal: [0., 0., 1.],
        color: [1.; 4],
        tangent: [1., 0., 0., 1.],
        feature_id: 0,
    })
    .collect();
    Ok(Primitive {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        transform: DMat4::IDENTITY,
        material,
    })
}

pub fn pixel_offset(e: &Entity) -> [f32; 2] {
    match &e.shape {
        Shape::Billboard { pixel_offset, .. } | Shape::Label { pixel_offset, .. } => {
            [pixel_offset[0] as f32, -pixel_offset[1] as f32]
        }
        _ => [0.; 2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pixel_geometry_and_bounded_parameters() {
        let mut values: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../../tests/entities/screen.json")).unwrap();
        let specs = crate::entities::decode(&serde_json::to_vec(&values).unwrap()).unwrap();
        let p = specs[0].primitive().unwrap();
        assert_eq!(p.vertices[0].position, [-20., -20., 0.]);
        assert_eq!(p.material.pattern.unwrap().repeat[0], 0.8);
        let p = specs[2].primitive().unwrap();
        assert_eq!(p.vertices[0].position, [16., -44., 0.]);
        for (field, value) in [
            ("pixelSize", serde_json::json!(0)),
            ("outlineWidth", serde_json::json!(129)),
            ("pixelSize", serde_json::json!(2049)),
        ] {
            let old = values[0]["shape"][field].clone();
            values[0]["shape"][field] = value;
            assert!(crate::entities::decode(&serde_json::to_vec(&values).unwrap()).is_err());
            values[0]["shape"][field] = old;
        }
        values[1]["shape"]["scale"] = serde_json::json!(64);
        assert!(crate::entities::decode(&serde_json::to_vec(&values).unwrap()).is_err());
    }
    #[test]
    fn czml_screen_defaults_samples_and_strict_properties() {
        let mut v = serde_json::json!([{"id":"document","version":"1.0"},{"id":"p","position":{"epoch":"2026-01-01T00:00:00Z","cartographicDegrees":[0,110,25,1000,10,111,25,1000]},"point":{"pixelSize":{"number":12},"outlineWidth":3,"outlineColor":{"rgba":[0,255,0,255]}}}]);
        let (specs, _) = crate::entities::czml(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(specs[0].color, [1.; 4]);
        assert!(specs[0].position(specs[0].samples[0].time + 5.).is_some());
        assert!(matches!(
            specs[0].shape,
            Shape::Point {
                pixel_size: 12.,
                outline_width: 3.,
                ..
            }
        ));
        v[1]["point"]["heightReference"] = serde_json::json!("CLAMP_TO_GROUND");
        assert!(crate::entities::czml(&serde_json::to_vec(&v).unwrap()).is_err());
        let bytes = include_bytes!("../../../tests/entities/screen.czml");
        let base = url::Url::parse("file:///samples/screen.czml").unwrap();
        let mut reads = 0;
        let hydrated = crate::entity_resources::hydrate(bytes, &base, |_, _| {
            reads += 1;
            Ok(include_bytes!("../../../tests/entities/image-test.png").to_vec())
        })
        .unwrap();
        assert_eq!(reads, 1);
        assert_eq!(crate::entities::czml(&hydrated).unwrap().0.len(), 3);
    }
}
