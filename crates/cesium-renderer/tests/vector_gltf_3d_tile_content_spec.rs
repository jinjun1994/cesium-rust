use std::collections::HashMap;

use cesium_math::{Cartesian3, Color};
use cesium_renderer::buffer_primitive_material::{
    BufferPointMaterial, BufferPolygonMaterial, BufferPolylineMaterial,
};
use cesium_renderer::create_vector_tile_buffers::{
    BlendOption, BufferCollection, BufferPoint, BufferPointCollection, BufferPointOptions,
    BufferPolygon, BufferPolygonCollection, BufferPolygonOptions, BufferPolyline,
    BufferPolylineCollection, BufferPolylineOptions, FeatureTableInfo, VectorGltf3DTileContent,
    VectorGltfFeature, VectorTileset,
};
use cesium_renderer::model_3d_tile_content::Cesium3DTileStyle;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};

fn create_buffer_point_collection() -> BufferPointCollection {
    let mut collection = BufferPointCollection::default();
    let mut pt = BufferPoint::default();
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_X),
            ..Default::default()
        },
        &mut pt,
    );
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Y),
            ..Default::default()
        },
        &mut pt,
    );
    collection.add(
        BufferPointOptions {
            position: Some(Cartesian3::UNIT_Z),
            ..Default::default()
        },
        &mut pt,
    );
    collection
}

fn create_buffer_polyline_collection() -> BufferPolylineCollection {
    let mut collection = BufferPolylineCollection::default();
    let mut line = BufferPolyline::default();
    collection.add(
        BufferPolylineOptions {
            positions: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            ..Default::default()
        },
        &mut line,
    );
    collection.add(
        BufferPolylineOptions {
            positions: vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0],
            ..Default::default()
        },
        &mut line,
    );
    collection.add(
        BufferPolylineOptions {
            positions: vec![2.0, 2.0, 2.0, 3.0, 3.0, 3.0],
            ..Default::default()
        },
        &mut line,
    );
    collection.add(
        BufferPolylineOptions {
            positions: vec![3.0, 3.0, 3.0, 4.0, 4.0, 4.0],
            ..Default::default()
        },
        &mut line,
    );
    collection
}

fn create_buffer_polygon_collection() -> BufferPolygonCollection {
    let mut collection = BufferPolygonCollection::default();
    let mut poly = BufferPolygon::default();
    collection.add(
        BufferPolygonOptions {
            positions: vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0],
            triangles: vec![0, 1, 2],
            ..Default::default()
        },
        &mut poly,
    );
    collection.add(
        BufferPolygonOptions {
            positions: vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0],
            triangles: vec![0, 1, 2],
            ..Default::default()
        },
        &mut poly,
    );
    collection.add(
        BufferPolygonOptions {
            positions: vec![2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0],
            triangles: vec![0, 1, 2],
            ..Default::default()
        },
        &mut poly,
    );
    collection.add(
        BufferPolygonOptions {
            positions: vec![3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0],
            triangles: vec![0, 1, 2],
            ..Default::default()
        },
        &mut poly,
    );
    collection
}

#[test]
fn test_features_length() {
    let mut content = VectorGltf3DTileContent::new();
    assert_eq!(content.features_length(), 0);

    content.feature_tables.push(FeatureTableInfo {
        features_length: 10,
        batch_table_byte_length: 100,
    });
    content.feature_tables.push(FeatureTableInfo {
        features_length: 14,
        batch_table_byte_length: 156,
    });

    assert_eq!(content.features_length(), 24);
    assert_eq!(content.batch_table_byte_length(), 256);
}

#[test]
fn test_points_triangles_and_byte_length() {
    let mut content = VectorGltf3DTileContent::new();
    let points = create_buffer_point_collection();
    let lines = create_buffer_polyline_collection();
    let polygons = create_buffer_polygon_collection();

    let expected_byte_len = points.byte_length() + lines.byte_length() + polygons.byte_length();
    let expected_tri_len = 2 * lines.primitive_count() + polygons.triangle_count();

    content.collections.push(BufferCollection::Point(points));
    content.collections.push(BufferCollection::Polyline(lines));
    content.collections.push(BufferCollection::Polygon(polygons));

    assert_eq!(content.points_length(), 3);
    assert_eq!(content.triangles_length(), expected_tri_len);
    assert_eq!(content.geometry_byte_length(), expected_byte_len);
    assert_eq!(content.textures_byte_length(), 0);
}

#[test]
fn test_get_feature_and_has_property() {
    let mut content = VectorGltf3DTileContent::new();

    let mut feat1 = VectorGltfFeature::new(123);
    feat1
        .properties
        .insert("my-prop".to_string(), serde_json::json!(true));
    let feat2 = VectorGltfFeature::new(456);
    let feat3 = VectorGltfFeature::new(789);

    let mut table100 = HashMap::new();
    table100.insert(123, feat1);
    table100.insert(456, feat2);

    let mut default_table = HashMap::new();
    default_table.insert(789, feat3);

    content.features_by_table_id.insert(Some(100), table100);
    content.features_by_table_id.insert(None, default_table);

    assert!(content.get_feature(123, Some(100)).is_some());
    assert!(content.get_feature(456, Some(100)).is_some());
    assert!(content.get_feature(789, None).is_some());
    assert!(content.get_feature(999, Some(101)).is_none());

    assert!(content.has_property(123, "my-prop", Some(100)));
    assert!(!content.has_property(456, "my-prop", Some(100)));
    assert!(!content.has_property(789, "my-prop", None));
    assert!(!content.has_property(999, "my-prop", Some(101)));
}

#[test]
fn test_apply_debug_settings() {
    let mut content = VectorGltf3DTileContent::new();
    content
        .collections
        .push(BufferCollection::Point(create_buffer_point_collection()));
    content
        .collections
        .push(BufferCollection::Polyline(create_buffer_polyline_collection()));
    content
        .collections
        .push(BufferCollection::Polygon(create_buffer_polygon_collection()));

    content.apply_debug_settings(true, Color::RED);
    if let BufferCollection::Point(ref p) = content.collections[0] {
        let mut pt = BufferPoint::default();
        p.get(0, &mut pt);
        let mut mat = BufferPointMaterial::default();
        pt.get_material(&mut mat);
        assert_eq!(mat.color, Color::RED);
    }

    content.apply_debug_settings(false, Color::RED);
    if let BufferCollection::Point(ref p) = content.collections[0] {
        let mut pt = BufferPoint::default();
        p.get(0, &mut pt);
        let mut mat = BufferPointMaterial::default();
        pt.get_material(&mut mat);
        assert_eq!(mat.color, Color::WHITE);
    }
}

#[test]
fn test_apply_style() {
    let mut content = VectorGltf3DTileContent::new();
    content
        .collections
        .push(BufferCollection::Point(create_buffer_point_collection()));
    content
        .collections
        .push(BufferCollection::Polyline(create_buffer_polyline_collection()));
    content
        .collections
        .push(BufferCollection::Polygon(create_buffer_polygon_collection()));

    let style = Cesium3DTileStyle {
        show_expression: Some("false".to_string()),
        color_expression: Some("color('#ff0000', 0.5)".to_string()),
        ..Default::default()
    };
    content.apply_style(Some(&style));

    if let BufferCollection::Point(ref p) = content.collections[0] {
        let mut pt = BufferPoint::default();
        p.get(0, &mut pt);
        assert!(!pt.show);
        let mut mat = BufferPointMaterial::default();
        pt.get_material(&mut mat);
        assert_eq!(mat.color, Color::new(1.0, 0.0, 0.0, 0.5));
        assert_eq!(mat.size, 12);
        assert_eq!(mat.outline_width, 2);
    }

    if let BufferCollection::Polyline(ref l) = content.collections[1] {
        let mut line = BufferPolyline::default();
        l.get(0, &mut line);
        assert!(!line.show);
        let mut mat = BufferPolylineMaterial::default();
        line.get_material(&mut mat);
        assert_eq!(mat.color, Color::new(1.0, 0.0, 0.0, 0.5));
        assert_eq!(mat.width, 12.0);
    }

    if let BufferCollection::Polygon(ref poly) = content.collections[2] {
        let mut p = BufferPolygon::default();
        poly.get(0, &mut p);
        assert!(!p.show);
        let mut mat = BufferPolygonMaterial::default();
        p.get_material(&mut mat);
        assert_eq!(mat.color, Color::new(1.0, 0.0, 0.0, 0.5));
    }
}

#[test]
fn test_update_syncs_blend_option() {
    let mut content = VectorGltf3DTileContent::new();
    content
        .collections
        .push(BufferCollection::Point(create_buffer_point_collection()));

    content.tileset = VectorTileset {
        vector_blend_option: Some(BlendOption::Opaque),
        ..Default::default()
    };

    let mut frame_state = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: true,
            pick: false,
        },
        ..Default::default()
    };

    content.update(None, &mut frame_state);
    assert_eq!(content.collections[0].blend_option(), BlendOption::Opaque);
    assert_eq!(frame_state.command_list.len(), 1);

    content.update(Some(BlendOption::Translucent), &mut frame_state);
    assert_eq!(
        content.collections[0].blend_option(),
        BlendOption::Translucent
    );
}

#[test]
fn test_is_destroyed_and_destroy() {
    let mut content = VectorGltf3DTileContent::new();
    assert!(!content.is_destroyed());
    content.destroy();
    assert!(content.is_destroyed());
    assert!(!content.ready);
}
