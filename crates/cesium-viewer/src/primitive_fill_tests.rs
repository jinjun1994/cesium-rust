use crate::{entities, primitive_instances};
use glam::DVec3;
use serde_json::{json, Value};
fn input(id: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../tests/sandcastle/full/{id}.primitive.json")),
    )
    .unwrap()
}
fn specs(id: &str) -> Vec<entities::Entity> {
    primitive_instances::decode(&input(id)).unwrap()
}
#[test]
fn complete_filled_instances_preserve_original_graph() {
    for id in [
        "box-dev",
        "cylinder-dev",
        "sphere-dev",
        "ellipsoid-dev",
        "wall-dev",
        "ellipse-dev",
    ] {
        let expected: Vec<entities::Entity> = serde_json::from_slice(
            &std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join(format!("../../tests/sandcastle/full/{id}.json")),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(specs(id)).unwrap(),
            serde_json::to_value(expected).unwrap(),
            "{id}"
        );
    }
}
#[test]
fn appearance_defaults_quantized_color_and_render_flags() {
    let e = specs("box-dev").remove(0);
    let a = e.surface_appearance.unwrap();
    assert!(!a.flat && a.closed && a.translucent);
    assert!((e.color[3] - 128. / 255.).abs() < 1e-6);
    let p = e.primitive().unwrap();
    assert!(p.material.alpha_blend && p.material.two_pass && p.material.lambert);
    assert!(!p.material.double_sided && !p.material.face_forward && !p.material.unlit);
    let e = specs("wall-dev").remove(0);
    let p = e.primitive().unwrap();
    assert!(p.material.double_sided && p.material.face_forward && p.material.lambert);
    assert!(!p.material.alpha_blend && !p.material.two_pass);
    let e = specs("cylinder-dev").remove(0);
    assert_eq!(e.color, [0., 128. / 255., 0., 1.]);
    let p = e.primitive().unwrap();
    assert!(!p.material.double_sided && !p.material.alpha_blend);
}
#[test]
fn cylinder_smooth_normals_and_solid_winding() {
    for e in specs("cylinder-dev") {
        let p = e.primitive().unwrap();
        for v in &p.vertices {
            let n = DVec3::from_array(v.normal.map(f64::from));
            assert!((n.length() - 1.).abs() < 1e-6);
            let q = DVec3::from_array(v.position.map(f64::from));
            if n.z.abs() < 0.999 && q.x.hypot(q.y) > 1. {
                assert!(n.x * q.x + n.y * q.y > 0.);
            }
        }
        for tri in p.indices.as_chunks::<3>().0 {
            let a = &p.vertices[tri[0] as usize];
            let b = &p.vertices[tri[1] as usize];
            let c = &p.vertices[tri[2] as usize];
            let v = |v: &cesium_formats::Vertex| DVec3::from_array(v.position.map(f64::from));
            assert!(
                (v(b) - v(a))
                    .cross(v(c) - v(a))
                    .dot(DVec3::from_array(a.normal.map(f64::from)))
                    > 0.
            );
        }
    }
}
#[test]
fn ellipse_filled_caps_rotation_and_extrusion() {
    let e = specs("ellipse-dev");
    assert_eq!(e.len(), 3);
    let entities::Shape::Ellipse {
        rotation, height, ..
    } = e[1].shape
    else {
        panic!()
    };
    assert_eq!(height, Some(200000.));
    assert!((rotation - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
    let p = e[2].primitive().unwrap();
    let m = e[2].model(0.).unwrap();
    let h: Vec<_> = p
        .vertices
        .iter()
        .map(|v| {
            cesium_math::Cartographic::from_ecef(
                m.transform_point3(DVec3::from_array(v.position.map(f64::from))),
            )
            .unwrap()
            .height
        })
        .collect();
    assert!(h.iter().any(|h| (*h - 10.).abs() < 1.));
    assert!(h.iter().any(|h| (*h - 200010.).abs() < 1.));
    assert!(!p.material.double_sided && p.material.lambert);
    for tri in p.indices.as_chunks::<3>().0 {
        let a = &p.vertices[tri[0] as usize];
        let b = &p.vertices[tri[1] as usize];
        let c = &p.vertices[tri[2] as usize];
        let v = |v: &cesium_formats::Vertex| DVec3::from_array(v.position.map(f64::from));
        assert!(
            (v(b) - v(a))
                .cross(v(c) - v(a))
                .dot(DVec3::from_array(a.normal.map(f64::from)))
                > 0.
        );
    }
}
#[test]
fn opaque_attribute_alpha_and_flat_override() {
    let mut e = specs("sphere-dev").remove(0);
    e.color[3] = 0.25;
    let p = e.primitive().unwrap();
    assert_eq!(p.material.base_color[3], 0.25);
    assert!(!p.material.alpha_blend);
    e.surface_appearance.as_mut().unwrap().flat = true;
    let p = e.primitive().unwrap();
    assert!(p.material.unlit && !p.material.lambert);
}
#[test]
fn filled_input_rejects_unmapped_and_ambiguous_parameters() {
    let original: Value = serde_json::from_slice(&input("box-dev")).unwrap();
    let mut cases = vec![];
    let mut v = original.clone();
    v["primitives"][0]["geometryInstances"]["geometry"]["vertexFormat"] = json!("POSITION_ONLY");
    cases.push(v);
    let mut v = original.clone();
    v["primitives"][0]["appearance"]["faceForward"] = json!(true);
    cases.push(v);
    let mut v = original;
    v["primitives"][0]["appearance"]["renderState"] = json!({"lineWidth":1});
    cases.push(v);
    let mut v: Value = serde_json::from_slice(&input("wall-dev")).unwrap();
    v["primitives"][0]["geometryInstances"][0]["geometry"]["maximumHeight"] = json!(100000);
    cases.push(v);
    for v in cases {
        assert!(primitive_instances::decode(&serde_json::to_vec(&v).unwrap()).is_err());
    }
}
