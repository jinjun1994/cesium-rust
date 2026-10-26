//! Additional bounded source GeometryInstances. Unknown fields are errors, never ignored.
use crate::entities::{self, Entity};
use serde_json::{json, Value};
const TYPES: &[&str] = &[
    "PolylineGeometry",
    "PolygonGeometry",
    "CoplanarPolygonGeometry",
    "CorridorGeometry",
    "CorridorOutlineGeometry",
    "CircleGeometry",
    "RectangleGeometry",
    "EllipseOutlineGeometry",
    "CircleOutlineGeometry",
    "PolygonOutlineGeometry",
    "CoplanarPolygonOutlineGeometry",
    "SimplePolylineGeometry",
    "PolylineVolumeGeometry",
    "PolylineVolumeOutlineGeometry",
];
pub fn handles(v: &Value) -> bool {
    v["primitives"].as_array().is_some_and(|ps| {
        ps.iter().any(|p| {
            p["appearance"]["type"] != "PerInstanceColorAppearance"
                || instances(&p["geometryInstances"])
                    .iter()
                    .any(|i| TYPES.contains(&i["geometry"]["type"].as_str().unwrap_or("")))
        })
    })
}
fn instances(v: &Value) -> Vec<&Value> {
    match v {
        Value::Array(a) => a.iter().collect(),
        _ => vec![v],
    }
}
fn keys(v: &Value, allowed: &[&str]) -> Result<(), String> {
    let m = v.as_object().ok_or("object required")?;
    if m.keys().any(|k| !allowed.contains(&k.as_str())) {
        return Err(format!("unmapped instance field in {v}"));
    }
    Ok(())
}
fn number(v: &Value, default: f64) -> Result<f64, String> {
    if v.is_null() {
        Ok(default)
    } else {
        v.as_f64()
            .filter(|n| n.is_finite())
            .ok_or_else(|| "finite number required".into())
    }
}
fn flag(v: &Value, default: bool) -> Result<bool, String> {
    if v.is_null() {
        Ok(default)
    } else {
        v.as_bool().ok_or_else(|| "boolean option required".into())
    }
}
fn ring(
    h: &Value,
    per_height: bool,
    height: f64,
    out: &mut Vec<Vec<[f64; 3]>>,
) -> Result<(), String> {
    keys(h, &["positions", "holes"])?;
    let mut p: Vec<[f64; 3]> =
        serde_json::from_value(h["positions"].clone()).map_err(|e| e.to_string())?;
    if !(3..=1024).contains(&p.len()) {
        return Err("polygon ring length".into());
    }
    if !per_height {
        for c in &mut p {
            c[2] = height;
        }
    }
    if p.first() != p.last() {
        p.push(p[0]);
    }
    out.push(p);
    if let Some(holes) = h.get("holes") {
        for h in holes.as_array().ok_or("hierarchy holes array")? {
            ring(h, per_height, height, out)?;
        }
    }
    if out.len() > 128 {
        return Err("hierarchy ring budget".into());
    }
    Ok(())
}
pub fn decode(v: Value) -> Result<Vec<Entity>, String> {
    keys(&v, &["primitives"])?;
    let ps = v["primitives"].as_array().ok_or("primitives array")?;
    if !(1..=256).contains(&ps.len()) {
        return Err("primitive count".into());
    }
    let mut out = vec![];
    for (pi, p) in ps.iter().enumerate() {
        keys(
            p,
            &[
                "geometryInstances",
                "appearance",
                "materialSupport",
                "translucent",
            ],
        )?;
        if p.get("materialSupport").is_some_and(|v| v != "TEXTURED")
            || p.get("translucent").is_some_and(|v| v != false)
        {
            return Err("unsupported Primitive option".into());
        }
        let a = &p["appearance"];
        keys(
            a,
            &[
                "type",
                "flat",
                "closed",
                "translucent",
                "renderState",
                "material",
                "materialSupport",
                "aboveGround",
            ],
        )?;
        let appearance = a["type"].as_str().ok_or("appearance type")?;
        if ![
            "PolylineColorAppearance",
            "PolylineMaterialAppearance",
            "PerInstanceColorAppearance",
            "MaterialAppearance",
            "EllipsoidSurfaceAppearance",
        ]
        .contains(&appearance)
        {
            return Err("unsupported appearance".into());
        }
        let ins = instances(&p["geometryInstances"]);
        if ins.is_empty() || ins.len() + out.len() > 256 {
            return Err("instance count".into());
        }
        for (ii, i) in ins.iter().enumerate() {
            keys(i, &["id", "geometry", "attributes", "modelMatrix"])?;
            let source_type = i["geometry"]["type"].as_str().ok_or("geometry type")?;
            if !TYPES.contains(&source_type) && appearance == "PerInstanceColorAppearance" {
                let bytes = serde_json::to_vec(
                    &json!({"primitives":[{"geometryInstances":i,"appearance":a}]}),
                )
                .map_err(|e| e.to_string())?;
                let mut compiled = crate::primitive_instances::decode(&bytes)?;
                let mut e = compiled.remove(0);
                if i.get("id").is_none() {
                    e.id = format!("primitive-{pi}-instance-{ii}");
                }
                e.properties = json!({"primitiveIndex":pi,"instanceIndex":ii});
                out.push(serde_json::to_value(e).map_err(|e| e.to_string())?);
                continue;
            }
            if i.get("modelMatrix").is_some() {
                return Err("world geometry requires no modelMatrix".into());
            }
            let g = &i["geometry"];
            let t = g["type"].as_str().ok_or("geometry type")?;
            if t == "PolylineGeometry" {
                let attrs = i.get("attributes").cloned().unwrap_or(json!({}));
                out.push(crate::wide_polyline::primitive_entity(
                    g,
                    a,
                    &attrs,
                    i["id"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or(format!("primitive-{pi}-instance-{ii}")),
                    pi,
                    ii,
                )?);
                continue;
            }
            let outline = t.contains("Outline") || t == "SimplePolylineGeometry";
            if outline {
                if a["flat"] != true || a["renderState"]["lineWidth"] != 1 {
                    return Err("flat lineWidth=1 required".into());
                }
                keys(&a["renderState"], &["lineWidth"])?;
            } else if a.get("renderState").is_some() {
                return Err("custom renderState unsupported".into());
            }
            let attrs = i.get("attributes").cloned().unwrap_or(json!({}));
            keys(&attrs, &["color"])?;
            let color = attrs.get("color").cloned().unwrap_or(json!([1, 1, 1, 1]));
            let mut pos = Value::Null;
            let shape = match t {
                "PolygonGeometry" | "CoplanarPolygonGeometry" => {
                    keys(
                        g,
                        &[
                            "type",
                            "positions",
                            "polygonHierarchy",
                            "vertexFormat",
                            "height",
                            "extrudedHeight",
                            "perPositionHeight",
                            "closeTop",
                            "closeBottom",
                            "arcType",
                            "stRotation",
                            "textureCoordinates",
                        ],
                    )?;
                    let hierarchy = g
                        .get("polygonHierarchy")
                        .cloned()
                        .unwrap_or(json!({"positions":g["positions"],"holes":[]}));
                    json!({"type":"polygonSurface","data":{"hierarchy":hierarchy,"textureCoordinates":g["textureCoordinates"],"height":number(&g["height"],0.)?,"perPositionHeight":flag(&g["perPositionHeight"],false)?,"extrudedHeight":g["extrudedHeight"],"closeTop":flag(&g["closeTop"],true)?,"closeBottom":flag(&g["closeBottom"],true)?,"coplanar":t.starts_with("Coplanar"),"arcType":g.get("arcType").cloned().unwrap_or(json!("GEODESIC")),"stRotation":number(&g["stRotation"],0.)?}})
                }
                "CircleGeometry" => {
                    keys(
                        g,
                        &[
                            "type",
                            "center",
                            "radius",
                            "height",
                            "extrudedHeight",
                            "vertexFormat",
                        ],
                    )?;
                    pos = g["center"].clone();
                    json!({"type":"ellipse","semiMajorAxis":g["radius"],"semiMinorAxis":g["radius"],"height":number(&g["height"],0.)?,"extrudedHeight":g["extrudedHeight"],"rotation":0})
                }
                "CorridorGeometry" | "CorridorOutlineGeometry" => {
                    keys(
                        g,
                        &[
                            "type",
                            "positions",
                            "width",
                            "height",
                            "extrudedHeight",
                            "cornerType",
                            "vertexFormat",
                        ],
                    )?;
                    json!({"type":"corridor","positionsDegrees":g["positions"],"width":g["width"],"height":number(&g["height"],0.)?,"extrudedHeight":g["extrudedHeight"],"cornerType":g.get("cornerType").cloned().unwrap_or(json!("ROUNDED"))})
                }
                "PolylineVolumeGeometry" | "PolylineVolumeOutlineGeometry" => {
                    keys(
                        g,
                        &[
                            "type",
                            "polylinePositions",
                            "shapePositions",
                            "cornerType",
                            "vertexFormat",
                        ],
                    )?;
                    let shape_pos: Vec<[f64; 2]> = g["shapePositions"]
                        .as_array()
                        .ok_or("shapePositions array")?
                        .iter()
                        .map(|pt| {
                            if let Some(arr) = pt.as_array() {
                                Ok([number(&arr[0], 0.)?, number(&arr[1], 0.)?])
                            } else if let Some(obj) = pt.as_object() {
                                Ok([number(&obj["x"], 0.)?, number(&obj["y"], 0.)?])
                            } else {
                                Err("invalid shape point".into())
                            }
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    json!({
                        "type": "polylineVolume",
                        "positionsDegrees": g["polylinePositions"],
                        "shapePositions": shape_pos,
                        "cornerType": g.get("cornerType").cloned().unwrap_or(json!("ROUNDED"))
                    })
                }

                "EllipseOutlineGeometry" | "CircleOutlineGeometry" => {
                    keys(
                        g,
                        &[
                            "type",
                            "center",
                            "semiMajorAxis",
                            "semiMinorAxis",
                            "radius",
                            "height",
                            "rotation",
                            "extrudedHeight",
                            "numberOfVerticalLines",
                        ],
                    )?;
                    pos = g["center"].clone();
                    let major = if t == "CircleOutlineGeometry" {
                        g["radius"].clone()
                    } else {
                        g["semiMajorAxis"].clone()
                    };
                    let minor = if t == "CircleOutlineGeometry" {
                        g["radius"].clone()
                    } else {
                        g["semiMinorAxis"].clone()
                    };
                    let n = g.get("numberOfVerticalLines").cloned().unwrap_or(json!(16));
                    if n.as_u64().is_none_or(|n| n > 256) {
                        return Err("vertical line limit".into());
                    }
                    json!({"type":"ellipse","height":number(&g["height"],0.)?,"semiMajorAxis":major,"semiMinorAxis":minor,"rotation":number(&g["rotation"],0.)?,"extrudedHeight":g["extrudedHeight"],"numberOfVerticalLines":n})
                }
                "RectangleGeometry" => {
                    keys(
                        g,
                        &[
                            "type",
                            "rectangle",
                            "vertexFormat",
                            "height",
                            "extrudedHeight",
                            "rotation",
                        ],
                    )?;
                    let [w, s, e, n]: [f64; 4] = serde_json::from_value(g["rectangle"].clone())
                        .map_err(|e| e.to_string())?;
                    let h = number(&g["height"], 0.)?;
                    let rotation = number(&g["rotation"], 0.)?;
                    if rotation == 0. {
                        json!({"type":"rectangle","coordinatesDegrees":[w,s,e,n],"height":h,"extrudedHeight":g["extrudedHeight"]})
                    } else {
                        let (sin, cos) = rotation.sin_cos();
                        let (cx, cy) = ((w + e) / 2., (s + n) / 2.);
                        let p = [[w, s], [e, s], [e, n], [w, n]].map(|[x, y]| {
                            [
                                cx + (x - cx) * cos - (y - cy) * sin,
                                cy + (x - cx) * sin + (y - cy) * cos,
                                h,
                            ]
                        });
                        json!({"type":"polygon","positionsDegrees":p,"holes":[],"extrudedHeight":g["extrudedHeight"]})
                    }
                }
                "PolygonOutlineGeometry" | "CoplanarPolygonOutlineGeometry" => {
                    keys(
                        g,
                        &[
                            "type",
                            "positions",
                            "polygonHierarchy",
                            "perPositionHeight",
                            "height",
                            "extrudedHeight",
                        ],
                    )?;
                    let hierarchy = if g.get("polygonHierarchy").is_some() {
                        g["polygonHierarchy"].clone()
                    } else {
                        json!({"positions":g["positions"]})
                    };
                    let per = t.starts_with("Coplanar") || flag(&g["perPositionHeight"], false)?;
                    let mut paths = vec![];
                    ring(&hierarchy, per, number(&g["height"], 0.)?, &mut paths)?;
                    if let Some(h) = g.get("extrudedHeight") {
                        let h = number(h, 0.)?;
                        for top in paths.clone() {
                            paths.push(top.iter().map(|c| [c[0], c[1], h]).collect());
                            for c in top.iter().take(top.len() - 1) {
                                paths.push(vec![*c, [c[0], c[1], h]]);
                            }
                        }
                    }
                    json!({"type":"lineSet","paths":paths,"colors":[],"geodesic":!t.starts_with("Coplanar"),"colorsPerVertex":false})
                }
                "SimplePolylineGeometry" => {
                    keys(
                        g,
                        &["type", "positions", "colors", "arcType", "colorsPerVertex"],
                    )?;
                    if g.get("arcType")
                        .is_some_and(|v| v != "NONE" && v != "GEODESIC")
                    {
                        return Err("line arc type".into());
                    }
                    json!({"type":"lineSet","paths":[g["positions"]],"colors":g.get("colors").map(|c|json!([c])).unwrap_or(json!([])),"geodesic":g["arcType"]!="NONE","colorsPerVertex":flag(&g["colorsPerVertex"],false)?})
                }
                "WallGeometry" => {
                    keys(g, &["type", "positions", "vertexFormat"])?;
                    json!({"type":"wall","positionsDegrees":g["positions"],"minimumHeights":[],"maximumHeights":[]})
                }
                _ => return Err("unsupported extended geometry".into()),
            };
            let mut e = json!({"id":i["id"].as_str().map(str::to_owned).unwrap_or(format!("primitive-{pi}-instance-{ii}")),"positionDegrees":pos,"shape":shape,"color":if outline{json!([1,1,1,1])}else{color.clone()},"fill":!outline,"outline":outline,"outlineColor":if outline{color}else{json!([0,0,0,1])},"properties":{"primitiveIndex":pi,"instanceIndex":ii}});
            if !outline {
                if appearance == "PerInstanceColorAppearance" {
                    if g["vertexFormat"] != "POSITION_NORMAL"
                        && g["vertexFormat"] != "POSITION_NORMAL_ST"
                    {
                        return Err("POSITION_NORMAL required".into());
                    }
                    e["surfaceAppearance"] = json!({"flat":flag(&a["flat"],false)?,"closed":flag(&a["closed"],false)?,"translucent":flag(&a["translucent"],true)?});
                } else {
                    if (appearance == "MaterialAppearance"
                        && (a["materialSupport"] != "TEXTURED"
                            || (g["vertexFormat"] != "POSITION_NORMAL_ST"
                                && g["vertexFormat"] != "ALL")))
                        || (appearance == "EllipsoidSurfaceAppearance"
                            && g["vertexFormat"] != "POSITION_ST"
                            && g["vertexFormat"] != "POSITION_NORMAL_ST")
                    {
                        return Err("textured vertexFormat/support required".into());
                    }
                    if a["material"]["type"] == "Image" {
                        keys(&a["material"], &["type", "image"])?;
                        e["material"] = json!({"type":"image","image":a["material"]["image"],"repeat":[1,1],"color":[1,1,1,1],"transparent":false});
                    } else {
                        keys(&a["material"], &["type"])?;
                        e["material"] = default_material(
                            a["material"]["type"].as_str().ok_or("material type")?,
                        )?;
                    }
                }
            }
            out.push(e);
        }
    }
    entities::decode(&serde_json::to_vec(&out).map_err(|e| e.to_string())?)
}
fn default_material(t: &str) -> Result<Value, String> {
    Ok(match t {
        "Stripe" => {
            json!({"type":"stripe","evenColor":[1,1,1,0.5],"oddColor":[0,0,1,0.5],"repeat":5,"offset":0,"orientation":"HORIZONTAL"})
        }
        "Checkerboard" => {
            json!({"type":"checkerboard","evenColor":[1,1,1,0.5],"oddColor":[0,0,0,0.5],"repeat":[5,5]})
        }
        "Grid" => {
            json!({"type":"grid","color":[0,1,0,1],"lineCount":[8,8],"lineThickness":[1,1],"cellAlpha":0.1})
        }
        "Dot" => {
            json!({"type":"dot","lightColor":[1,1,0,0.75],"darkColor":[0,1,1,0.75],"repeat":[5,5]})
        }
        _ => return Err("unsupported material type".into()),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(id: &str, extension: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../tests/sandcastle/full/{id}.{extension}")),
        )
        .unwrap()
    }
    #[test]
    fn all_extended_source_instances_match_independent_snapshot() {
        for id in [
            "ellipse-outline-dev",
            "circle-outline-dev",
            "rectangle-dev",
            "per-instance-color-dev",
            "polygon-outline-dev",
            "coplanar-polygon-outline-dev",
            "simple-polyline-dev",
            "material-dev",
            "ellipsoid-surface-dev",
        ] {
            let actual =
                crate::primitive_instances::decode(&fixture(id, "primitive.json")).unwrap();
            let expected: Vec<Entity> = serde_json::from_slice(&fixture(id, "json")).unwrap();
            assert_eq!(
                serde_json::to_value(&actual).unwrap(),
                serde_json::to_value(&expected).unwrap(),
                "{id}"
            );
            for e in &actual {
                let p = e.primitive().unwrap();
                assert!(!p.vertices.is_empty());
                if e.outline {
                    assert!(crate::outline::primitive(e).unwrap().is_some());
                }
            }
        }
    }
    #[test]
    fn line_arc_color_and_hierarchy_semantics() {
        let lines =
            crate::primitive_instances::decode(&fixture("simple-polyline-dev", "primitive.json"))
                .unwrap();
        let straight = lines[1].primitive().unwrap();
        assert_eq!(straight.indices.len(), 20);
        // Per segment endpoints keep the same RGB; the next segment has a new RGB.
        assert_eq!(straight.vertices[0].color, straight.vertices[1].color);
        assert_ne!(straight.vertices[1].color, straight.vertices[2].color);
        let gradient = lines[2].primitive().unwrap();
        assert!(gradient.vertices.len() > 100);
        assert_ne!(
            gradient.vertices.first().unwrap().color,
            gradient.vertices.last().unwrap().color
        );
        let coplanar = crate::primitive_instances::decode(&fixture(
            "coplanar-polygon-outline-dev",
            "primitive.json",
        ))
        .unwrap();
        let entities::Shape::LineSet {
            paths, geodesic, ..
        } = &coplanar[1].shape
        else {
            panic!()
        };
        assert!(!geodesic);
        assert_eq!(paths.len(), 3, "outer, hole, nested island all survive");
        let polygon =
            crate::primitive_instances::decode(&fixture("polygon-outline-dev", "primitive.json"))
                .unwrap();
        let entities::Shape::LineSet { paths, .. } = &polygon[1].shape else {
            panic!()
        };
        assert_eq!(
            paths.len(),
            12,
            "both extruded rings plus eight original vertical edges"
        );
    }
    #[test]
    fn invalid_extended_input_is_rejected() {
        let v: Value =
            serde_json::from_slice(&fixture("ellipse-outline-dev", "primitive.json")).unwrap();
        for (field, value) in [
            ("numberOfVerticalLines", json!(257)),
            ("semiMajorAxis", json!(-1)),
            ("unmapped", json!(true)),
        ] {
            let mut x = v.clone();
            x["primitives"][0]["geometryInstances"][0]["geometry"][field] = value;
            assert!(
                crate::primitive_instances::decode(&serde_json::to_vec(&x).unwrap()).is_err(),
                "{field}"
            );
        }
        let mut x: Value =
            serde_json::from_slice(&fixture("simple-polyline-dev", "primitive.json")).unwrap();
        x["primitives"][1]["geometryInstances"][1]["geometry"]["colors"] = json!([[1, 0, 0, 1]]);
        assert!(crate::primitive_instances::decode(&serde_json::to_vec(&x).unwrap()).is_err());
    }
    #[test]
    fn czml_byte_and_float_colors_and_inferred_clock() {
        let (colors, _) = entities::czml(&fixture("czml-colors", "czml")).unwrap();
        for id in ["czml-colors", "czml-point-time-dynamic"] {
            let (actual, _) = entities::czml(&fixture(id, "czml")).unwrap();
            let expected: Vec<Entity> = serde_json::from_slice(&fixture(id, "json")).unwrap();
            for (a, b) in actual.iter().zip(&expected) {
                assert_eq!(a.samples.len(), b.samples.len());
                for (x, y) in a.samples.iter().zip(&b.samples) {
                    assert_eq!(x.time, y.time);
                    for k in 0..3 {
                        assert!((x.cartesian[k] - y.cartesian[k]).abs() < 1e-6);
                    }
                }
                let mut av = serde_json::to_value(a).unwrap();
                let mut bv = serde_json::to_value(b).unwrap();
                av.as_object_mut().unwrap().remove("samples");
                bv.as_object_mut().unwrap().remove("samples");
                assert_eq!(av, bv, "{id}");
            }
        }
        assert_eq!(colors.len(), 2);
        assert!((colors[0].color[3] - 100. / 255.).abs() < 1e-6);
        assert_eq!(colors[1].color[3], 0.39);
        let (e, c) = entities::czml(&fixture("czml-point-time-dynamic", "czml")).unwrap();
        assert_eq!(c.stop_time - c.start_time, 300.);
        assert_eq!(c.current_time, c.start_time);
        assert_eq!(c.multiplier, 2.5);
        assert_eq!(e[0].samples.len(), 4);
        let mid = e[0].position(c.start_time + 50.).unwrap();
        let a = glam::DVec3::from_array(e[0].samples[0].cartesian);
        let b = glam::DVec3::from_array(e[0].samples[1].cartesian);
        assert!(mid.distance(a.lerp(b, 0.5)) < 1e-6);
        assert!(e[0].position(c.stop_time + 1.).is_none());
    }
}
