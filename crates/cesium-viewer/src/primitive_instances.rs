//! Bounded static outline GeometryInstance compiler. Shares the resident draw/selection store.
use crate::{entities, RenderError, Scene};
use cesium_renderer::Renderer;
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    primitives: Vec<Primitive>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Primitive {
    geometry_instances: Instances,
    appearance: Appearance,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum Instances {
    One(Box<Instance>),
    Many(Vec<Instance>),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Instance {
    #[serde(default)]
    id: Option<String>,
    geometry: Geometry,
    #[serde(default)]
    model_matrix: Option<[f64; 16]>,
    attributes: Attributes,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Attributes {
    color: [f32; 4],
}
fn yes() -> bool {
    true
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Appearance {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    flat: bool,
    #[serde(default)]
    closed: bool,
    #[serde(default = "yes")]
    translucent: bool,
    #[serde(default)]
    render_state: Option<RenderState>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct RenderState {
    line_width: f64,
}
#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum Geometry {
    #[serde(rename = "BoxGeometry")]
    FilledBox {
        dimensions: [f64; 3],
        #[serde(rename = "vertexFormat")]
        format: String,
    },
    #[serde(rename = "CylinderGeometry")]
    FilledCylinder {
        length: f64,
        #[serde(rename = "topRadius")]
        top: f64,
        #[serde(rename = "bottomRadius")]
        bottom: f64,
        #[serde(rename = "vertexFormat")]
        format: String,
    },
    #[serde(rename = "SphereGeometry")]
    FilledSphere {
        radius: f64,
        #[serde(rename = "vertexFormat")]
        format: String,
    },
    #[serde(rename = "EllipsoidGeometry")]
    FilledEllipsoid {
        radii: [f64; 3],
        #[serde(rename = "vertexFormat")]
        format: String,
    },
    #[serde(rename = "WallGeometry")]
    FilledWall {
        positions: Vec<[f64; 3]>,
        #[serde(default, rename = "minimumHeights")]
        minimums: Vec<f64>,
        #[serde(default, rename = "maximumHeights")]
        maximums: Vec<f64>,
        #[serde(default, rename = "minimumHeight")]
        minimum: Option<f64>,
        #[serde(default, rename = "maximumHeight")]
        maximum: Option<f64>,
        #[serde(rename = "vertexFormat")]
        format: String,
    },
    #[serde(rename = "EllipseGeometry")]
    FilledEllipse {
        center: [f64; 3],
        #[serde(rename = "semiMinorAxis")]
        minor: f64,
        #[serde(rename = "semiMajorAxis")]
        major: f64,
        #[serde(default)]
        height: f64,
        #[serde(default)]
        rotation: f64,
        #[serde(default, rename = "extrudedHeight")]
        extruded: Option<f64>,
        #[serde(rename = "vertexFormat")]
        format: String,
    },
    #[serde(rename = "BoxOutlineGeometry")]
    Box { dimensions: [f64; 3] },
    #[serde(rename = "CylinderOutlineGeometry")]
    Cylinder {
        length: f64,
        #[serde(rename = "topRadius")]
        top: f64,
        #[serde(rename = "bottomRadius")]
        bottom: f64,
        #[serde(rename = "numberOfVerticalLines")]
        vertical: u32,
    },
    #[serde(rename = "SphereOutlineGeometry")]
    Sphere {
        radius: f64,
        #[serde(rename = "stackPartitions")]
        stacks: u32,
        #[serde(rename = "slicePartitions")]
        slices: u32,
    },
    #[serde(rename = "EllipsoidOutlineGeometry")]
    Ellipsoid {
        radii: [f64; 3],
        #[serde(rename = "stackPartitions")]
        stacks: u32,
        #[serde(rename = "slicePartitions")]
        slices: u32,
    },
    #[serde(rename = "RectangleOutlineGeometry")]
    Rectangle { rectangle: [f64; 4] },
    #[serde(rename = "WallOutlineGeometry")]
    Wall {
        positions: Vec<[f64; 3]>,
        #[serde(rename = "maximumHeight")]
        maximum: f64,
        #[serde(default, rename = "minimumHeight")]
        minimum: f64,
    },
}
impl Geometry {
    fn outline(&self) -> bool {
        matches!(
            self,
            Self::Box { .. }
                | Self::Cylinder { .. }
                | Self::Sphere { .. }
                | Self::Ellipsoid { .. }
                | Self::Rectangle { .. }
                | Self::Wall { .. }
        )
    }
}
fn vertex_format(format: &str) -> Result<(), String> {
    if format != "POSITION_NORMAL" {
        return Err("filled instances require POSITION_NORMAL vertexFormat".into());
    }
    Ok(())
}

pub fn decode(bytes: &[u8]) -> Result<Vec<entities::Entity>, String> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("primitive input byte limit".into());
    }
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if crate::extended_instances::handles(&value) {
        return crate::extended_instances::decode(value);
    }
    let input: Input = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if input.primitives.is_empty() || input.primitives.len() > 256 {
        return Err("1..256 static primitives required".into());
    }
    let mut output = Vec::new();
    for (p, primitive) in input.primitives.into_iter().enumerate() {
        if primitive.appearance.kind != "PerInstanceColorAppearance" {
            return Err("PerInstanceColorAppearance required".into());
        }
        let instances = match primitive.geometry_instances {
            Instances::One(i) => vec![*i],
            Instances::Many(i) => i,
        };
        if instances.is_empty() || instances.len() + output.len() > 256 {
            return Err("1..256 geometry instances required".into());
        }
        for (i, instance) in instances.into_iter().enumerate() {
            let outline = instance.geometry.outline();
            if outline
                && (!primitive.appearance.flat
                    || primitive
                        .appearance
                        .render_state
                        .as_ref()
                        .is_none_or(|r| r.line_width != 1.))
            {
                return Err("flat lineWidth=1 required for outline geometry".into());
            }
            if !outline && primitive.appearance.render_state.is_some() {
                return Err("custom renderState unsupported for filled geometry".into());
            }
            let mut position = None;
            let shape = match instance.geometry {
                Geometry::FilledBox { dimensions, format } => {
                    vertex_format(&format)?;
                    json!({"type":"box","dimensions":dimensions})
                }
                Geometry::FilledCylinder {
                    length,
                    top,
                    bottom,
                    format,
                } => {
                    vertex_format(&format)?;
                    json!({"type":"cylinder","length":length,"topRadius":top,"bottomRadius":bottom})
                }
                Geometry::FilledSphere { radius, format } => {
                    vertex_format(&format)?;
                    json!({"type":"ellipsoid","radii":[radius,radius,radius]})
                }
                Geometry::FilledEllipsoid { radii, format } => {
                    vertex_format(&format)?;
                    json!({"type":"ellipsoid","radii":radii})
                }
                Geometry::FilledWall {
                    positions,
                    minimums,
                    maximums,
                    minimum,
                    maximum,
                    format,
                } => {
                    vertex_format(&format)?;
                    let count = positions.len();
                    if (maximum.is_some() || minimum.is_some())
                        && (!minimums.is_empty() || !maximums.is_empty())
                    {
                        return Err(
                            "constant and per-position wall heights are mutually exclusive".into(),
                        );
                    }
                    let minimums = minimum.map_or(minimums, |h| vec![h; count]);
                    let minimums = if minimums.is_empty() {
                        vec![0.; count]
                    } else {
                        minimums
                    };
                    let maximums = maximum.map_or(maximums, |h| vec![h; count]);
                    json!({"type":"wall","positionsDegrees":positions,"minimumHeights":minimums,"maximumHeights":maximums})
                }
                Geometry::FilledEllipse {
                    center,
                    minor,
                    major,
                    height,
                    rotation,
                    extruded,
                    format,
                } => {
                    vertex_format(&format)?;
                    position = Some(center);
                    json!({"type":"ellipse","semiMajorAxis":major,"semiMinorAxis":minor,"height":height,"rotation":rotation,"extrudedHeight":extruded})
                }
                Geometry::Box { dimensions } => {
                    json!({"type":"box","dimensions":dimensions})
                }
                Geometry::Cylinder {
                    length,
                    top,
                    bottom,
                    vertical,
                } => {
                    if vertical != 16 {
                        return Err("static cylinder numberOfVerticalLines=16 required".into());
                    }
                    json!({"type":"cylinder","length":length,"topRadius":top,"bottomRadius":bottom})
                }
                Geometry::Sphere {
                    radius,
                    stacks,
                    slices,
                } => {
                    json!({"type":"ellipsoid","radii":[radius,radius,radius],"stackPartitions":stacks,"slicePartitions":slices})
                }
                Geometry::Ellipsoid {
                    radii,
                    stacks,
                    slices,
                } => {
                    json!({"type":"ellipsoid","radii":radii,"stackPartitions":stacks,"slicePartitions":slices})
                }
                Geometry::Rectangle { rectangle } => {
                    json!({"type":"rectangle","coordinatesDegrees":rectangle,"height":0,"extrudedHeight":null})
                }
                Geometry::Wall {
                    positions,
                    minimum,
                    maximum,
                } => {
                    let count = positions.len();
                    json!({"type":"wall","positionsDegrees":positions,"minimumHeights":vec![minimum;count],"maximumHeights":vec![maximum;count]})
                }
            };
            let world = matches!(
                shape["type"].as_str(),
                Some("rectangle" | "wall" | "ellipse")
            );
            if world
                && instance
                    .model_matrix
                    .is_some_and(|m| m != glam::DMat4::IDENTITY.to_cols_array())
            {
                return Err(
                    "world outline geometry currently requires identity modelMatrix".into(),
                );
            }
            if !world && instance.model_matrix.is_none() {
                return Err("local outline geometry requires modelMatrix".into());
            }
            let id = instance
                .id
                .unwrap_or_else(|| format!("primitive-{p}-instance-{i}"));
            let mut value = json!({"id":id,"name":"","positionDegrees":null,"shape":shape,"fill":!outline,"outline":outline,"outlineColor":if outline {instance.attributes.color}else{[0.,0.,0.,1.]},"outlineWidth":1,"color":if outline {[1.;4]}else{instance.attributes.color},"properties":{"primitiveIndex":p,"instanceIndex":i}});
            if !outline {
                value["surfaceAppearance"] = json!({"flat":primitive.appearance.flat,"closed":primitive.appearance.closed,"translucent":primitive.appearance.translucent});
            }
            value["positionDegrees"] = json!(position);
            if !world {
                value["modelMatrix"] = json!(instance.model_matrix);
            }
            output.push(value);
        }
    }
    entities::decode(&serde_json::to_vec(&output).map_err(|e| e.to_string())?)
}
impl Scene {
    /// Atomically replace the shared resident geometry collection with compiled static instances.
    pub fn set_primitive_instances(
        &mut self,
        renderer: &mut Renderer,
        bytes: &[u8],
    ) -> Result<(), RenderError> {
        let specs = decode(bytes).map_err(RenderError)?;
        self.set_entities(renderer, specs)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use cesium_math::Cartographic;
    use glam::{DMat4, DVec3};
    fn input(id: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.primitive.json")),
        )
        .unwrap()
    }
    #[test]
    fn source_instances_keep_all_parameters() {
        for id in [
            "box-outline-dev",
            "cylinder-outline-dev",
            "sphere-outline-dev",
            "ellipsoid-outline-dev",
            "rectangle-outline-dev",
            "wall-outline-dev",
        ] {
            let compiled = decode(&input(id)).unwrap();
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.json"));
            let expected: Vec<entities::Entity> =
                serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(&compiled).unwrap(),
                serde_json::to_value(&expected).unwrap(),
                "{id}"
            );
            assert!(!compiled[0].fill && compiled[0].outline);
        }
    }
    #[test]
    fn enu_model_matrix_translation_is_exact() {
        let e = decode(&input("box-outline-dev")).unwrap().remove(0);
        let m = e.model(0.).unwrap();
        let ground = Cartographic::degrees(-106., 45., 0.).unwrap();
        let offset = ground
            .enu_to_ecef()
            .inverse()
            .transform_point3(m.w_axis.truncate());
        assert!((offset - DVec3::new(0., 0., 200000.)).length() < 1e-6);
        assert_eq!(m, DMat4::from_cols_array(&e.model_matrix.unwrap()));
        assert_eq!(
            e.model_matrix.unwrap()[12].to_bits(),
            (-1284197.8131982181_f64).to_bits(),
            "preserve original JSON ECEF f64 bits"
        );
        assert!(e.position(0.).unwrap().distance(m.w_axis.truncate()) < 1e-9);
    }
    #[test]
    fn affine_model_rejections_and_nonuniform_transform() {
        let mut v: serde_json::Value = serde_json::from_slice(&input("box-outline-dev")).unwrap();
        let original = v.clone();
        for (index, value) in [(3, 0.5), (15, 0.), (0, 0.), (12, 1e12)] {
            v = original.clone();
            v["primitives"][0]["geometryInstances"]["modelMatrix"][index] = json!(value);
            if index == 0 {
                for i in 0..3 {
                    v["primitives"][0]["geometryInstances"]["modelMatrix"][i] = json!(0.);
                }
            }
            assert!(decode(&serde_json::to_vec(&v).unwrap()).is_err());
        }
        v = original;
        let a = v["primitives"][0]["geometryInstances"]["modelMatrix"]
            .as_array_mut()
            .unwrap();
        for value in a.iter_mut().take(3) {
            *value = json!(value.as_f64().unwrap() * 2.);
        }
        let e = decode(&serde_json::to_vec(&v).unwrap()).unwrap().remove(0);
        assert!((e.model(0.).unwrap().x_axis.truncate().length() - 2.).abs() < 1e-9);
    }
    #[test]
    fn unsupported_fields_budgets_and_colors_fail() {
        let original: serde_json::Value =
            serde_json::from_slice(&input("cylinder-outline-dev")).unwrap();
        let mut cases = vec![];
        let mut v = original.clone();
        v["primitives"][0]["appearance"]["renderState"]["lineWidth"] = json!(2);
        cases.push(v);
        let mut v = original.clone();
        v["primitives"][0]["geometryInstances"]["geometry"]["numberOfVerticalLines"] = json!(8);
        cases.push(v);
        let mut v = original.clone();
        v["primitives"][0]["geometryInstances"]["attributes"]["color"] = json!([1., 2., 0., 1.]);
        cases.push(v);
        let mut v = original.clone();
        v["primitives"][0]["appearance"]["flat"] = json!(false);
        cases.push(v);
        let mut v = original.clone();
        v["primitives"][0]["asynchronous"] = json!(true);
        cases.push(v);
        let mut v = original.clone();
        v["primitives"][0]["geometryInstances"] = json!([]);
        cases.push(v);
        let mut v = original.clone();
        v["primitives"] = json!(vec![original["primitives"][0].clone(); 257]);
        cases.push(v);
        for v in cases {
            assert!(decode(&serde_json::to_vec(&v).unwrap()).is_err());
        }
    }
    #[test]
    fn instance_arrays_ids_and_outline_topology() {
        for (id, segments) in [
            ("box-outline-dev", 12),
            ("cylinder-outline-dev", 272),
            ("sphere-outline-dev", 2540),
            ("ellipsoid-outline-dev", 3048),
        ] {
            let e = decode(&input(id)).unwrap();
            let p = crate::outline::primitive(&e[0]).unwrap().unwrap();
            assert_eq!(p.indices.len(), segments * 2, "{id}");
        }
        let mut v: serde_json::Value = serde_json::from_slice(&input("box-outline-dev")).unwrap();
        let mut a = v["primitives"][0]["geometryInstances"].clone();
        a["id"] = json!("first");
        let mut b = a.clone();
        b["id"] = json!("second");
        v["primitives"][0]["geometryInstances"] = json!([a, b]);
        let e = decode(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[1].id, "second");
        v["primitives"][0]["geometryInstances"][1]["id"] = json!("first");
        assert!(decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    #[test]
    fn geographic_world_outlines_follow_surface() {
        let e = decode(&input("wall-outline-dev")).unwrap().remove(0);
        let p = crate::outline::primitive(&e).unwrap().unwrap();
        let m = e.model(0.).unwrap();
        let coordinates: Vec<_> = p
            .vertices
            .iter()
            .map(|v| {
                Cartographic::from_ecef(
                    m.transform_point3(DVec3::from_array(v.position.map(f64::from))),
                )
                .unwrap()
            })
            .collect();
        assert!(coordinates.iter().any(|c| c.latitude.to_degrees() > 60.8));
        assert!(coordinates.iter().any(|c| (c.height - 500010.5).abs() < 1.));
        assert!(coordinates.iter().any(|c| (c.height - 10.5).abs() < 1.));
        let e = decode(&input("rectangle-outline-dev")).unwrap().remove(0);
        let p = crate::outline::primitive(&e).unwrap().unwrap();
        assert!(!p.indices.is_empty());
    }
}
