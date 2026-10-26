//! Pixel-width WGS84 polylines. Previous/next relative positions are projected on GPU.
//! Material equations follow the pinned Cesium shaders (Apache-2.0).
use crate::entities::Entity;
use cesium_formats::{Material, Pattern, Primitive, Vertex};
use cesium_math::Cartographic;
use glam::{DMat4, DVec3};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub enum Arc {
    #[default]
    GEODESIC,
    RHUMB,
    NONE,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Polyline {
    pub positions: Vec<[f64; 3]>,
    pub width: f64,
    #[serde(default)]
    pub arc_type: Arc,
    #[serde(default)]
    pub clamp_to_ground: bool,
    #[serde(default)]
    pub colors: Vec<[f32; 4]>,
    #[serde(default)]
    pub colors_per_vertex: bool,
    #[serde(default)]
    pub material: LineMaterial,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum LineMaterial {
    #[default]
    Solid,
    Glow {
        #[serde(rename = "glowPower")]
        glow_power: f32,
        #[serde(rename = "taperPower")]
        taper_power: f32,
    },
    Arrow,
    Outline {
        #[serde(rename = "outlineColor")]
        color: [f32; 4],
        #[serde(rename = "outlineWidth")]
        width: f32,
    },
    Dash {
        #[serde(rename = "gapColor")]
        gap: [f32; 4],
        #[serde(rename = "dashLength")]
        length: f32,
        #[serde(rename = "dashPattern")]
        pattern: u16,
    },
}
fn color(c: [f32; 4]) -> bool {
    c.iter().all(|n| n.is_finite() && (0.0..=1.).contains(n))
}
impl Polyline {
    pub fn validate(&self) -> Result<(), String> {
        if !(2..=1024).contains(&self.positions.len())
            || !self.width.is_finite()
            || !(1.0..=256.).contains(&self.width)
        {
            return Err("polyline positions/width limit".into());
        }
        for c in &self.positions {
            if c.iter().any(|n| !n.is_finite())
                || c[0].abs() > 180.
                || c[1].abs() > 85.
                || c[2].abs() > 2e6
            {
                return Err("polyline coordinate limit".into());
            }
        }
        if self.clamp_to_ground && self.arc_type == Arc::NONE {
            return Err("clamped straight ECEF line unsupported".into());
        }
        if !self.colors.is_empty()
            && (self.colors.len() < self.positions.len() - usize::from(!self.colors_per_vertex)
                || self.colors.len() > self.positions.len()
                || self.colors.iter().any(|c| !color(*c)))
        {
            return Err("polyline color count/range".into());
        }
        match self.material {
            LineMaterial::Glow {
                glow_power: g,
                taper_power: t,
            } if !g.is_finite()
                || !t.is_finite()
                || !(0.0..=1.).contains(&g)
                || !(0.0..=1.).contains(&t) =>
            {
                return Err("glow power limit".into())
            }
            LineMaterial::Outline { color: c, width: w }
                if !color(c) || !w.is_finite() || w < 0. || w > self.width as f32 =>
            {
                return Err("line outline limit".into())
            }
            LineMaterial::Dash {
                gap: g, length: l, ..
            } if !color(g) || !l.is_finite() || !(1.0..=1024.).contains(&l) => {
                return Err("dash limit".into())
            }
            _ => {}
        }
        Ok(())
    }
}
fn world(p: [f64; 3], clamp: bool) -> Result<DVec3, String> {
    Ok(
        Cartographic::degrees(p[0], p[1], if clamp { 10.5 } else { p[2] + 10.5 })
            .map_err(|e| e.to_string())?
            .to_ecef(),
    )
}
fn rhumb(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let e = (1. - (1. - 1. / 298.257223563_f64).powi(2)).sqrt();
    let iso = |p: f64| {
        (std::f64::consts::FRAC_PI_4 + p / 2.).tan().ln()
            - e * ((1. + e * p.sin()) / (1. - e * p.sin())).ln() / 2.
    };
    let target = iso(a[1].to_radians()) * (1. - t) + iso(b[1].to_radians()) * t;
    let mut lat = 2. * target.exp().atan() - std::f64::consts::FRAC_PI_2;
    for _ in 0..10 {
        lat = 2.
            * (target + e * ((1. + e * lat.sin()) / (1. - e * lat.sin())).ln() / 2.)
                .exp()
                .atan()
            - std::f64::consts::FRAC_PI_2;
    }
    [
        (a[0] + ((b[0] - a[0] + 180.).rem_euclid(360.) - 180.) * t + 180.).rem_euclid(360.) - 180.,
        lat.to_degrees(),
        a[2] + (b[2] - a[2]) * t,
    ]
}
pub fn primitive(e: &Entity, d: &Polyline) -> Result<Primitive, String> {
    d.validate()?;
    if !e.fill || e.outline || e.material.is_some() || e.surface_appearance.is_some() {
        return Err("wide line requires fill, line material only".into());
    }
    let origin = crate::geographic::anchor(&e.shape)?;
    let mut p = Primitive {
        vertices: vec![],
        indices: vec![],
        transform: DMat4::IDENTITY,
        material: Material {
            base_color: crate::entity_material::linear(e.color),
            unlit: true,
            double_sided: true,
            alpha_blend: true,
            ..Default::default()
        },
    };
    let (kind, secondary, repeat, thickness) = match d.material {
        LineMaterial::Solid => (0, [0.; 4], [0.; 2], [0.; 2]),
        LineMaterial::Glow {
            glow_power: g,
            taper_power: t,
        } => (7, [0.; 4], [g, t], [0.; 2]),
        LineMaterial::Arrow => (8, [0.; 4], [0.; 2], [0.; 2]),
        LineMaterial::Outline { color, width } => (
            9,
            crate::entity_material::linear(color),
            [d.width as f32, width],
            [0.; 2],
        ),
        LineMaterial::Dash {
            gap,
            length,
            pattern,
        } => (
            10,
            crate::entity_material::linear(gap),
            [length, pattern as f32],
            [0.; 2],
        ),
    };
    p.material.pattern = Some(Pattern {
        kind,
        secondary,
        repeat,
        offset: 0.,
        axis: 0,
        thickness,
        cell_alpha: 1.,
    });
    // Sample each authored segment independently, preserving per-segment discontinuities.
    let mut samples: Vec<Vec<(DVec3, f64)>> = vec![];
    let mut total = 0.;
    let mut sampled_count = 0;
    for pair in d.positions.windows(2) {
        let a = world(pair[0], d.clamp_to_ground)?;
        let b = world(pair[1], d.clamp_to_ground)?;
        if a.distance(b) < 1e-3 {
            return Err("distinct line positions required".into());
        }
        let arc = crate::geodesic::Line::new([pair[0][0], pair[0][1]], [pair[1][0], pair[1][1]])?;
        let n = if d.arc_type == Arc::NONE {
            1
        } else {
            (arc.distance / 11000.).ceil().max(1.) as usize
        };
        sampled_count += n + 1;
        if n > 3600 || sampled_count > 50000 {
            return Err("line segment budget".into());
        }
        let mut s = vec![];
        for k in 0..=n {
            let t = k as f64 / n as f64;
            let w = match d.arc_type {
                Arc::NONE => a.lerp(b, t),
                Arc::RHUMB => world(rhumb(pair[0], pair[1], t), d.clamp_to_ground)?,
                Arc::GEODESIC => {
                    let [lon, lat] = arc.at(t);
                    world(
                        [lon, lat, pair[0][2] + (pair[1][2] - pair[0][2]) * t],
                        d.clamp_to_ground,
                    )?
                }
            };
            if let Some((last, _)) = s.last() {
                total += w.distance(*last);
            }
            s.push((w, total));
        }
        samples.push(s);
    }
    let estimate: usize = samples.iter().map(|s| s.len() * 2).sum();
    if estimate > 100000 {
        return Err("line vertex budget".into());
    }
    for (seg, s) in samples.iter().enumerate() {
        for (i, (w, distance)) in s.iter().enumerate() {
            let prev = if i > 0 {
                s[i - 1].0
            } else if seg > 0 {
                let before = &samples[seg - 1];
                before[before.len() - 2].0
            } else {
                *w * 2. - s[1].0
            };
            let next = if i + 1 < s.len() {
                s[i + 1].0
            } else if seg + 1 < samples.len() {
                samples[seg + 1][1].0
            } else {
                *w * 2. - s[i - 1].0
            };
            let c = if d.colors.is_empty() {
                [1.; 4]
            } else if d.colors_per_vertex {
                let t = i as f32 / (s.len() - 1) as f32;
                std::array::from_fn(|j| d.colors[seg][j] * (1. - t) + d.colors[seg + 1][j] * t)
            } else {
                d.colors[seg]
            };
            for side in [0., 1.] {
                p.vertices.push(Vertex {
                    position: (*w - origin).as_vec3().to_array(),
                    normal: (prev - origin).as_vec3().to_array(),
                    tangent: (next - origin).as_vec3().extend(0.).to_array(),
                    uv: [(*distance / total) as f32, side],
                    uv1: [d.width as f32, side * 2. - 1.],
                    color: crate::entity_material::linear(c),
                    feature_id: 0,
                });
            }
            if i + 1 < s.len() {
                let b = p.vertices.len() as u32 - 2;
                p.indices.extend([b, b + 1, b + 2, b + 2, b + 1, b + 3]);
            }
        }
    }
    Ok(p)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rhumb_latitude_and_dateline() {
        let p = rhumb([170., 35., 0.], [-170., 35., 10.], 0.5);
        assert!((p[0].abs() - 180.).abs() < 1e-8);
        assert!((p[1] - 35.).abs() < 1e-8);
        assert_eq!(p[2], 5.);
    }
    #[test]
    fn rejects_invalid_budgets_and_colors() {
        let mut p: Polyline =
            serde_json::from_value(serde_json::json!({"positions":[[0,0,0],[1,0,0]],"width":5}))
                .unwrap();
        assert!(p.validate().is_ok());
        p.colors = vec![[1.; 4]];
        p.colors_per_vertex = true;
        assert!(p.validate().is_err());
        p.colors.clear();
        p.width = 257.;
        assert!(p.validate().is_err());
    }
}
fn keys(v: &serde_json::Value, allowed: &[&str]) -> Result<(), String> {
    if !v
        .as_object()
        .is_some_and(|o| o.keys().all(|k| allowed.contains(&k.as_str())))
    {
        return Err("unknown polyline source field".into());
    }
    Ok(())
}
pub fn source_material(t: &str, v: &serde_json::Value) -> Result<(LineMaterial, [f32; 4]), String> {
    use serde_json::json;
    keys(
        v,
        &[
            "color",
            "glowPower",
            "taperPower",
            "outlineColor",
            "outlineWidth",
            "gapColor",
            "dashLength",
            "dashPattern",
        ],
    )?;
    let c = |k: &str, default: serde_json::Value| -> Result<[f32; 4], String> {
        serde_json::from_value(v.get(k).cloned().unwrap_or(default)).map_err(|e| e.to_string())
    };
    let n = |k: &str, d: f32| -> Result<f32, String> {
        v.get(k).map_or(Ok(d), |v| {
            v.as_f64()
                .map(|v| v as f32)
                .ok_or("line material number".into())
        })
    };
    Ok((
        match t {
            "Color" | "Solid" => LineMaterial::Solid,
            "PolylineGlow" => LineMaterial::Glow {
                glow_power: n("glowPower", 0.25)?,
                taper_power: n("taperPower", 1.)?,
            },
            "PolylineArrow" => LineMaterial::Arrow,
            "PolylineOutline" => LineMaterial::Outline {
                color: c("outlineColor", json!([0, 0, 0, 1]))?,
                width: n("outlineWidth", 1.)?,
            },
            "PolylineDash" => LineMaterial::Dash {
                gap: c("gapColor", json!([0, 0, 0, 0]))?,
                length: n("dashLength", 16.)?,
                pattern: v.get("dashPattern").map_or(Ok(255), |v| {
                    v.as_u64()
                        .filter(|n| *n <= 65535)
                        .map(|n| n as u16)
                        .ok_or("dashPattern u16")
                })?,
            },
            _ => return Err("unknown line material".into()),
        },
        c("color", json!([1, 1, 1, 1]))?,
    ))
}
pub fn primitive_entity(
    g: &serde_json::Value,
    a: &serde_json::Value,
    attrs: &serde_json::Value,
    id: String,
    pi: usize,
    ii: usize,
) -> Result<serde_json::Value, String> {
    use serde_json::json;
    keys(
        g,
        &[
            "type",
            "positions",
            "width",
            "vertexFormat",
            "colors",
            "colorsPerVertex",
            "arcType",
        ],
    )?;
    keys(a, &["type", "material"])?;
    keys(attrs, &["color"])?;
    let is_color = a["type"] == "PolylineColorAppearance";
    if (!is_color && a["type"] != "PolylineMaterialAppearance")
        || g["vertexFormat"]
            != if is_color {
                "POLYLINE_COLOR"
            } else {
                "POLYLINE_MATERIAL"
            }
    {
        return Err("polyline appearance/vertexFormat mismatch".into());
    }
    let (m, c) = if is_color {
        (
            LineMaterial::Solid,
            serde_json::from_value(attrs.get("color").cloned().unwrap_or(json!([1, 1, 1, 1])))
                .map_err(|e| e.to_string())?,
        )
    } else {
        keys(&a["material"], &["type", "uniforms"])?;
        let t = a["material"]["type"].as_str().ok_or("line material type")?;
        let mut u = a["material"].get("uniforms").cloned().unwrap_or(json!({}));
        if !u.is_object() {
            return Err("material uniforms object".into());
        }
        if u.get("color").is_none() {
            u["color"] = match t {
                "PolylineGlow" => json!([0, 0.5, 1, 1]),
                "PolylineDash" => json!([1, 0, 1, 1]),
                _ => json!([1, 1, 1, 1]),
            };
        }
        if t == "PolylineOutline" && u.get("outlineColor").is_none() {
            u["outlineColor"] = json!([1, 0, 0, 1]);
        }
        source_material(t, &u)?
    };
    let d = json!({"positions":g["positions"],"width":g.get("width").cloned().unwrap_or(json!(1)),"arcType":g.get("arcType").cloned().unwrap_or(json!("GEODESIC")),"clampToGround":false,"colors":g.get("colors").cloned().unwrap_or(json!([])),"colorsPerVertex":g.get("colorsPerVertex").cloned().unwrap_or(json!(false)),"material":m});
    Ok(
        json!({"id":id,"positionDegrees":null,"shape":{"type":"widePolyline","data":d},"color":c,"fill":true,"outline":false,"outlineColor":[0,0,0,1],"outlineWidth":1,"properties":{"primitiveIndex":pi,"instanceIndex":ii}}),
    )
}
pub fn czml(packet: &serde_json::Value) -> Result<Entity, String> {
    use serde_json::{json, Value};
    let g = &packet["polyline"];
    keys(
        g,
        &[
            "positions",
            "width",
            "arcType",
            "clampToGround",
            "material",
            "show",
        ],
    )?;
    keys(
        &g["material"],
        &[
            "solidColor",
            "polylineGlow",
            "polylineArrow",
            "polylineOutline",
            "polylineDash",
        ],
    )?;
    let entries = g["material"].as_object().ok_or("line material object")?;
    if entries.len() != 1 {
        return Err("one line material required".into());
    }
    let (kind, v) = entries.iter().next().unwrap();
    let mut v = v.clone();
    for k in ["color", "outlineColor", "gapColor"] {
        if let Some(c) = v.get(k) {
            keys(c, &["rgba", "rgbaf"])?;
            let c: Vec<f32> = if let Some(a) = c.get("rgba") {
                serde_json::from_value::<Vec<f32>>(a.clone())
                    .map_err(|e| e.to_string())?
                    .into_iter()
                    .map(|n| n / 255.)
                    .collect()
            } else {
                serde_json::from_value(c["rgbaf"].clone()).map_err(|e| e.to_string())?
            };
            v[k] = json!(c);
        }
    }
    let t = match kind.as_str() {
        "solidColor" => "Color",
        "polylineGlow" => "PolylineGlow",
        "polylineArrow" => "PolylineArrow",
        "polylineOutline" => "PolylineOutline",
        _ => "PolylineDash",
    };
    let (m, c) = source_material(t, &v)?;
    let d = json!({"positions":crate::entities::position_list(&g["positions"])?,"width":g.get("width").cloned().unwrap_or(json!(1)),"arcType":g.get("arcType").cloned().unwrap_or(json!("GEODESIC")),"clampToGround":g.get("clampToGround").cloned().unwrap_or(json!(false)),"material":m});
    let e:Entity=serde_json::from_value(json!({"id":packet["id"],"name":packet.get("name").cloned().unwrap_or(json!("")),"positionDegrees":null,"shape":{"type":"widePolyline","data":d},"color":c,"fill":true,"outline":false,"show":g.get("show").cloned().unwrap_or(json!(true)),"properties":packet.get("properties").cloned().unwrap_or(Value::Null),"availability":packet.get("availability").map(|_| Value::Null).unwrap_or(Value::Null)})).map_err(|e|e.to_string())?;
    if packet.get("position").is_some() || packet.get("availability").is_some() {
        return Err("static standalone line required".into());
    }
    e.primitive()?;
    Ok(e)
}
#[cfg(test)]
mod source_tests {
    use super::*;
    use crate::entities::Shape;
    use serde_json::{json, Value};
    fn near(a: &Value, b: &Value) {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => assert!(
                (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-6,
                "{a} != {b}"
            ),
            (Value::Array(a), Value::Array(b)) => {
                assert_eq!(a.len(), b.len());
                for (a, b) in a.iter().zip(b) {
                    near(a, b)
                }
            }
            (Value::Object(a), Value::Object(b)) => {
                for (k, v) in b {
                    near(a.get(k).unwrap(), v)
                }
            }
            _ => assert_eq!(a, b),
        }
    }
    #[test]
    fn all_six_complete_source_contracts() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");
        for id in [
            "polyline",
            "polyline-dev",
            "polyline-color-dev",
            "polyline-material-dev",
            "polyline-dash",
            "czml-polyline",
        ] {
            let read = |ext: &str| std::fs::read(base.join(format!("{id}.{ext}"))).unwrap();
            let actual = if id.starts_with("czml-") {
                crate::entities::czml(&read("czml")).unwrap().0
            } else if id.ends_with("-dev") {
                crate::primitive_instances::decode(&read("primitive.json")).unwrap()
            } else {
                crate::entities::decode(&read("json")).unwrap()
            };
            let expected: Vec<Value> = serde_json::from_slice(&read("json")).unwrap();
            assert_eq!(actual.len(), expected.len());
            for (e, x) in actual.iter().zip(expected) {
                near(&serde_json::to_value(e).unwrap(), &x);
                let p = e.primitive().unwrap();
                assert!(p.indices.len() >= 6);
                assert!(p.material.unlit);
                assert!(p.vertices.iter().all(|v| v
                    .position
                    .iter()
                    .chain(v.normal.iter())
                    .chain(v.tangent.iter())
                    .all(|x| x.is_finite())));
            }
        }
    }
    #[test]
    fn interpolation_and_segment_discontinuities() {
        let mut e:Entity=serde_json::from_value(json!({"id":"test","shape":{"type":"widePolyline","data":{"positions":[[-100,35,0],[-90,35,10000],[-80,35,0]],"width":5,"colors":[[1,0,0,1],[0,1,0,1],[0,0,1,1]]}},"color":[1,1,1,1]})).unwrap();
        let p = e.primitive().unwrap();
        let Shape::WidePolyline { data } = &mut e.shape else {
            panic!()
        };
        data.colors_per_vertex = true;
        let q = e.primitive().unwrap();
        assert_eq!(p.vertices.len(), q.vertices.len());
        assert!(p
            .vertices
            .iter()
            .all(|v| v.color == [1., 0., 0., 1.] || v.color == [0., 1., 0., 1.]));
        assert!(q
            .vertices
            .iter()
            .any(|v| v.color[0] > 0.1 && v.color[1] > 0.1));
    }
    #[test]
    fn strict_material_and_source_validation() {
        assert!(source_material("PolylineDash", &json!({"unknown":0})).is_err());
        assert!(source_material("PolylineDash", &json!({"dashPattern":65536})).is_err());
        let mut p: Polyline =
            serde_json::from_value(json!({"positions":[[0,0,0],[0,0,0]],"width":5})).unwrap();
        let e = |p: &Polyline| {
            serde_json::from_value::<Entity>(
                json!({"id":"x","shape":{"type":"widePolyline","data":p}}),
            )
            .unwrap()
        };
        assert!(e(&p).primitive().is_err());
        p.positions[1] = [1., 1., 0.];
        p.material = LineMaterial::Dash {
            gap: [0.; 4],
            length: 0.,
            pattern: 255,
        };
        assert!(e(&p).primitive().is_err());
    }
}
#[cfg(test)]
mod material_defaults_test {
    use super::*;
    use crate::entities::Shape;
    use serde_json::json;
    #[test]
    fn pinned_material_defaults_are_distinct_from_property_defaults() {
        let reference: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/sandcastle/reference-shaders/material-defaults.json"
        ))
        .unwrap();
        assert!(reference["extracts"]["PolylineGlow"]
            .as_str()
            .unwrap()
            .contains("new Color(0.0, 0.5, 1.0, 1.0)"));
        assert!(reference["extracts"]["PolylineOutline"]
            .as_str()
            .unwrap()
            .contains("outlineColor: new Color(1.0, 0.0, 0.0, 1.0)"));
        let g = json!({"type":"PolylineGeometry","positions":[[-100,35,0],[-90,35,0]],"width":10,"vertexFormat":"POLYLINE_MATERIAL"});
        let build = |t: &str| {
            serde_json::from_value::<Entity>(primitive_entity(&g,&json!({"type":"PolylineMaterialAppearance","material":{"type":t,"uniforms":{}}}),&json!({}),"x".into(),0,0).unwrap()).unwrap()
        };
        let glow = build("PolylineGlow");
        assert_eq!(glow.color, [0., 0.5, 1., 1.]);
        let outline = build("PolylineOutline");
        let Shape::WidePolyline { data } = outline.shape else {
            panic!()
        };
        let LineMaterial::Outline { color, .. } = data.material else {
            panic!()
        };
        assert_eq!(color, [1., 0., 0., 1.]);
        // Property defaults use the explicitly normalized values from Entity/CZML inputs.
        let (m, _) = source_material(
            "PolylineOutline",
            &json!({"color":[1,0.5,0,1],"outlineColor":[0,0,0,1]}),
        )
        .unwrap();
        let LineMaterial::Outline { color, .. } = m else {
            panic!()
        };
        assert_eq!(color, [0., 0., 0., 1.]);
    }
}
