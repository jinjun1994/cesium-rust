use std::collections::HashMap;

use cesium_math::{Cartesian3, Color};
use cesium_renderer::{
    BufferCollection, BufferPoint, BufferPointCollection, BufferPointMaterial, BufferPointOptions,
    Cesium3DTileVectorFeature, VectorGltf3DTileContent, VectorGltfFeature,
};
use serde_json::json;

#[test]
fn feature_id_and_pick_ids() {
    let mut content = VectorGltf3DTileContent::new();
    let mut points1 = BufferPointCollection::default();
    let mut points2 = BufferPointCollection::default();
    let mut dummy = BufferPoint::default();

    let mut feature = Cesium3DTileVectorFeature::new(123, 0);
    assert_eq!(feature.feature_id(), 123);

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_X),
            ..Default::default()
        },
        &mut dummy,
    );
    content.pick_ids.insert((0, 0), 101);
    feature.add_primitive_by_collection(0, 0);

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Y),
            ..Default::default()
        },
        &mut dummy,
    ); // Skip index 1.

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Z),
            ..Default::default()
        },
        &mut dummy,
    );
    content.pick_ids.insert((0, 2), 102);
    feature.add_primitive_by_collection(0, 2);

    points2.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_X),
            ..Default::default()
        },
        &mut dummy,
    );
    content.pick_ids.insert((1, 0), 103);
    feature.add_primitive_by_collection(1, 0);

    content.collections.push(BufferCollection::Point(points1));
    content.collections.push(BufferCollection::Point(points2));

    assert_eq!(feature.pick_ids(&content), vec![101, 102, 103]);
}

#[test]
fn show_propagates_across_collections() {
    let mut content = VectorGltf3DTileContent::new();
    let mut points1 = BufferPointCollection::default();
    let mut points2 = BufferPointCollection::default();
    let mut dummy = BufferPoint::default();

    let mut feature = Cesium3DTileVectorFeature::new(123, 0);

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_X),
            show: Some(false),
            ..Default::default()
        },
        &mut dummy,
    );
    feature.add_primitive_by_collection(0, 0);

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Y),
            ..Default::default()
        },
        &mut dummy,
    ); // Skip.

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Z),
            show: Some(false),
            ..Default::default()
        },
        &mut dummy,
    );
    feature.add_primitive_by_collection(0, 2);

    points2.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_X),
            show: Some(false),
            ..Default::default()
        },
        &mut dummy,
    );
    feature.add_primitive_by_collection(1, 0);

    content.collections.push(BufferCollection::Point(points1));
    content.collections.push(BufferCollection::Point(points2));

    assert!(!feature.show(&content));

    if let BufferCollection::Point(ref mut p) = content.collections[0] {
        p.shows[2] = true;
    }
    assert!(feature.show(&content));

    feature.set_show(&mut content, true);
    assert!(feature.show(&content));
    if let BufferCollection::Point(ref p) = content.collections[0] {
        assert!(p.shows[0]);
        assert!(p.shows[2]);
    }
    if let BufferCollection::Point(ref p) = content.collections[1] {
        assert!(p.shows[0]);
    }

    feature.set_show(&mut content, false);
    assert!(!feature.show(&content));
    if let BufferCollection::Point(ref p) = content.collections[0] {
        assert!(!p.shows[0]);
        assert!(!p.shows[2]);
    }
    if let BufferCollection::Point(ref p) = content.collections[1] {
        assert!(!p.shows[0]);
    }
}

#[test]
fn color_and_properties_work() {
    let mut content = VectorGltf3DTileContent::new();
    let mut points1 = BufferPointCollection::default();
    let mut points2 = BufferPointCollection::default();
    let mut dummy = BufferPoint::default();

    let mat = BufferPointMaterial {
        color: Color::YELLOW,
        ..Default::default()
    };

    let mut feature = Cesium3DTileVectorFeature::new(123, 2);
    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_X),
            material: Some(mat.clone()),
            ..Default::default()
        },
        &mut dummy,
    );
    feature.add_primitive_by_collection(0, 0);

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Y),
            ..Default::default()
        },
        &mut dummy,
    );

    points1.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Z),
            material: Some(mat.clone()),
            ..Default::default()
        },
        &mut dummy,
    );
    feature.add_primitive_by_collection(0, 2);

    points2.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_X),
            material: Some(mat),
            ..Default::default()
        },
        &mut dummy,
    );
    feature.add_primitive_by_collection(1, 0);

    content.collections.push(BufferCollection::Point(points1));
    content.collections.push(BufferCollection::Point(points2));

    assert_eq!(feature.color(&content), Color::YELLOW);
    feature.set_color(&mut content, Color::CYAN);
    assert_eq!(feature.color(&content), Color::CYAN);

    if let BufferCollection::Point(ref p) = content.collections[0] {
        assert_eq!(p.materials[0].color, Color::CYAN);
        assert_eq!(p.materials[2].color, Color::CYAN);
    }
    if let BufferCollection::Point(ref p) = content.collections[1] {
        assert_eq!(p.materials[0].color, Color::CYAN);
    }

    let mut f = VectorGltfFeature::new(123);
    f.properties.insert("my-prop".to_string(), json!("hello world"));
    f.properties.insert("height".to_string(), json!(42));
    let mut table_map = HashMap::new();
    table_map.insert(123, f);
    content.features_by_table_id.insert(Some(2), table_map);

    assert!(feature.has_property(&content, "my-prop"));
    assert!(!feature.has_property(&content, "missing"));
    assert_eq!(
        feature.get_property(&content, "my-prop"),
        Some(json!("hello world"))
    );
    assert_eq!(
        feature.get_property_ids(&content),
        vec!["height".to_string(), "my-prop".to_string()]
    );
}
