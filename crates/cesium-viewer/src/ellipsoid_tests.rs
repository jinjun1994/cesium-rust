use crate::{entities, entity_material, outline};
use serde_json::json;
fn spec(stacks: u32, slices: u32) -> serde_json::Value {
    json!([{"id":"ellipsoid","positionDegrees":[110,25,1000],"shape":{"type":"ellipsoid","radii":[20,30,40],"stackPartitions":stacks,"slicePartitions":slices},"outline":true}])
}
#[test]
fn ellipsoid_partition_topology_and_smooth_normals() {
    let e = entities::decode(&serde_json::to_vec(&spec(36, 24)).unwrap())
        .unwrap()
        .remove(0);
    let p = e.primitive().unwrap();
    assert_eq!(p.indices.len(), 6 * 24 * 35);
    for v in &p.vertices {
        let [x, y, z] = v.position;
        let n = glam::DVec3::new(x as f64 / 400., y as f64 / 900., z as f64 / 1600.).normalize();
        assert!(n.dot(glam::DVec3::from_array(v.normal.map(f64::from))) > 0.999999);
        assert!(((x / 20.).powi(2) + (y / 30.).powi(2) + (z / 40.).powi(2) - 1.).abs() < 1e-5);
    }
    let lines = outline::primitive(&e).unwrap().unwrap();
    assert_eq!(lines.indices.len(), (35 + 25) * 127 * 2);
}
#[test]
fn ellipsoid_partition_bounds_and_legacy_default() {
    for n in [0, 2, 129, u32::MAX] {
        assert!(entities::decode(&serde_json::to_vec(&spec(n, 24)).unwrap()).is_err());
    }
    let mut v = spec(36, 24);
    v[0]["shape"]
        .as_object_mut()
        .unwrap()
        .remove("stackPartitions");
    v[0]["shape"]
        .as_object_mut()
        .unwrap()
        .remove("slicePartitions");
    let e = entities::decode(&serde_json::to_vec(&v).unwrap())
        .unwrap()
        .remove(0);
    assert_eq!(e.primitive().unwrap().indices.len(), 6 * 64 * 63);
    assert_eq!(
        outline::primitive(&e).unwrap().unwrap().indices.len(),
        (9 + 9) * 127 * 2
    );
}
#[test]
fn czml_float_colors_strict_encoding() {
    assert_eq!(
        entity_material::rgba(&json!({"rgbaf":[0.1,0.2,0.3,0.4]})).unwrap(),
        [0.1, 0.2, 0.3, 0.4]
    );
    for v in [
        json!({"rgba":[0,0,0,255],"rgbaf":[0,0,0,1]}),
        json!({"rgbaf":[0,0,0,2]}),
        json!({"rgbaf":[0,0,0]}),
        json!({"rgbaf":[0,0,0,1],"epoch":"sampled"}),
        json!({"rgba":[0,0,0,1.2]}),
    ] {
        assert!(entity_material::rgba(&v).is_err());
    }
}
#[test]
fn czml_partitions_and_rgbaf_outline() {
    let mut v = json!([{"id":"document","version":"1.0"},{"id":"e","position":{"cartographicDegrees":[110,25,1000]},"ellipsoid":{"radii":{"cartesian":[20,30,40]},"stackPartitions":{"number":36},"slicePartitions":24,"outline":true,"outlineColor":{"rgbaf":[0,0,0,1]}}}]);
    let (e, _) = entities::czml(&serde_json::to_vec(&v).unwrap()).unwrap();
    assert_eq!(e[0].outline_color, [0., 0., 0., 1.]);
    assert_eq!(e[0].primitive().unwrap().indices.len(), 6 * 24 * 35);
    for n in [2., 36.5, 129.] {
        v[1]["ellipsoid"]["slicePartitions"] = json!(n);
        assert!(entities::czml(&serde_json::to_vec(&v).unwrap()).is_err());
    }
}
#[test]
fn batch28_partial_ellipsoids_and_czml_rectangle() {
    let ellipsoids: Vec<entities::Entity> = entities::decode(include_bytes!(
        "../../../tests/sandcastle/full/partial-ellipsoids.json"
    ))
    .unwrap();
    assert_eq!(ellipsoids.len(), 12);
    for e in &ellipsoids {
        let prim = e.primitive().unwrap();
        assert!(!prim.vertices.is_empty());
        assert!(!prim.indices.is_empty());
        if e.outline {
            let out = outline::primitive(e).unwrap().unwrap();
            assert!(!out.indices.is_empty());
        }
        let model = e.model(0.0).unwrap();
        assert!(model.is_finite());
    }

    let json_rects: Vec<entities::Entity> = entities::decode(include_bytes!(
        "../../../tests/sandcastle/full/czml-rectangle.json"
    ))
    .unwrap();
    assert_eq!(json_rects.len(), 4);

    let base = url::Url::from_file_path(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/sandcastle/full/czml-rectangle.czml"),
    )
    .unwrap();
    let raw = include_bytes!("../../../tests/sandcastle/full/czml-rectangle.czml");
    let hydrated = crate::entity_resources::hydrate(raw, &base, |u, _| {
        std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
    })
    .unwrap();
    let (rectangles, _) = entities::czml(&hydrated).unwrap();
    assert_eq!(rectangles.len(), 4);
    assert_eq!(rectangles[0].id, "redRectangle");
    assert!(rectangles[0].fill);
    assert!(rectangles[0].outline);
    let p0 = rectangles[0].primitive().unwrap();
    assert!(!p0.vertices.is_empty());

    assert_eq!(rectangles[1].id, "stripeRectangle");
    assert!(matches!(
        rectangles[1].material,
        Some(crate::entity_material::SurfaceMaterial::Stripe { .. })
    ));

    assert_eq!(rectangles[2].id, "yellowRectangle");
    assert!(!rectangles[2].fill);
    assert!(rectangles[2].outline);
    let out2 = outline::primitive(&rectangles[2]).unwrap().unwrap();
    assert!(!out2.indices.is_empty());

    assert_eq!(rectangles[3].id, "textureRectangle");
    assert!(matches!(
        rectangles[3].material,
        Some(crate::entity_material::SurfaceMaterial::Image { .. })
    ));
}
