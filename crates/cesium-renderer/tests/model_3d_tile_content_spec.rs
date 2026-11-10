//! Tests for Model3DTileContent matching CesiumJS `Scene/Model/Model3DTileContentSpec.js` (Apache-2.0).

use serde_json::json;

use cesium_math::{Cartesian3, ClassificationType, Color, Matrix4, Ray};
use cesium_renderer::bounding_volume_semantics::MetadataClass;
use cesium_renderer::clipping_plane_collection::ClippingPlaneCollection;
use cesium_renderer::clipping_polygon_collection::ClippingPolygonCollection;
use cesium_renderer::content_metadata::{ContentMetadata, ContentMetadataOptions};
use cesium_renderer::group_metadata::{GroupMetadata, GroupMetadataOptions};
use cesium_renderer::material_pipeline_stage::PointCloudShading;
use cesium_renderer::model_3d_tile_content::{
    Cesium3DTileStyle, Model3DTileContent, Model3DTileContext, Model3DTilesetContext,
    ModelFeatureTable,
};

#[test]
fn test_geojson_rendering_and_picking() {
    let tileset = Model3DTilesetContext::default();
    let tile = Model3DTileContext {
        computed_transform: Matrix4::IDENTITY,
        ..Default::default()
    };

    let geojson = json!({
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.0, 0.0]]]
                },
                "properties": {
                    "building": "residential",
                    "height": 25.5
                }
            },
            {
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": [[0.0, 0.0], [2.0, 2.0]]
                },
                "properties": {
                    "road": "primary"
                }
            }
        ]
    });

    let content = Model3DTileContent::from_geojson(tileset, tile, "https://example.com/data.geojson".to_string(), geojson)
        .expect("GeoJSON should parse successfully");

    assert_eq!(content.features_length(), 2);
    assert_eq!(content.triangles_length(), 4);
    assert!(content.geometry_byte_length() > 0);
    assert!(content.ready);

    // Feature inspection
    let feature0 = content.get_feature(0).unwrap();
    assert_eq!(feature0.get_property("building").and_then(|v| v.as_str()), Some("residential"));
    assert_eq!(feature0.get_property("height").and_then(|v| v.as_f64()), Some(25.5));

    let feature1 = content.get_feature(1).unwrap();
    assert_eq!(feature1.get_property("road").and_then(|v| v.as_str()), Some("primary"));

    assert!(content.has_property(0, "building"));
    assert!(!content.has_property(0, "road"));
    assert!(content.has_property(1, "road"));

    // Picking
    let ray = Ray::new(
        Cartesian3::new(0.0, 0.0, 10.0).into(),
        Cartesian3::new(0.0, 0.0, -1.0).into(),
    ).unwrap();
    let hit = content.pick(&ray, 1.0, 0.0);
    assert!(hit.is_some());
}

#[test]
fn test_b3dm_content_features_and_memory() {
    let tileset = Model3DTilesetContext::default();
    let tile = Model3DTileContext::default();

    // Create synthetic b3dm header:
    // magic: "b3dm", version: 1, byteLength: 48, ftJsonLen: 8, ftBinLen: 0, btJsonLen: 12, btBinLen: 0
    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"b3dm");
    buffer.extend_from_slice(&1u32.to_le_bytes()); // version
    buffer.extend_from_slice(&48u32.to_le_bytes()); // byteLength
    buffer.extend_from_slice(&8u32.to_le_bytes()); // ftJsonLen
    buffer.extend_from_slice(&0u32.to_le_bytes()); // ftBinLen
    buffer.extend_from_slice(&12u32.to_le_bytes()); // btJsonLen
    buffer.extend_from_slice(&0u32.to_le_bytes()); // btBinLen
    buffer.resize(48, 0);

    let mut content = Model3DTileContent::from_b3dm(tileset, tile, "tile.b3dm".to_string(), &buffer, 0).unwrap();

    assert_eq!(content.geometry_byte_length(), 48);
    assert_eq!(content.batch_table_byte_length(), 20); // 8 + 12

    // Attach features
    if let Some(model) = &mut content.model {
        let mut ft = ModelFeatureTable::new(3);
        ft.get_feature_mut(0).unwrap().set_property("name", json!("Building A"));
        ft.get_feature_mut(1).unwrap().set_property("name", json!("Building B"));
        ft.get_feature_mut(2).unwrap().set_property("name", json!("Building C"));
        model.feature_tables.push(ft);
        model.feature_table_id = Some(0);
    }

    assert_eq!(content.features_length(), 3);
    assert_eq!(content.get_feature(0).unwrap().get_property("name").unwrap().as_str().unwrap(), "Building A");

    // Apply debug settings
    content.apply_debug_settings(true, Color::YELLOW);
    assert_eq!(content.get_feature(0).unwrap().color, Some(Color::YELLOW));
}

#[test]
fn test_i3dm_content_and_textures() {
    let tileset = Model3DTilesetContext::default();
    let tile = Model3DTileContext::default();

    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"i3dm");
    buffer.extend_from_slice(&1u32.to_le_bytes());
    buffer.extend_from_slice(&48u32.to_le_bytes());
    buffer.extend_from_slice(&16u32.to_le_bytes());
    buffer.extend_from_slice(&0u32.to_le_bytes());
    buffer.extend_from_slice(&16u32.to_le_bytes());
    buffer.extend_from_slice(&0u32.to_le_bytes());
    buffer.extend_from_slice(&1u32.to_le_bytes()); // gltfFormat
    buffer.resize(48, 0);

    let mut content = Model3DTileContent::from_i3dm(tileset, tile, "instances.i3dm".to_string(), &buffer, 0).unwrap();

    assert_eq!(content.geometry_byte_length(), 48);
    assert_eq!(content.batch_table_byte_length(), 32);

    if let Some(model) = &mut content.model {
        model.statistics.texture_ids = vec!["tex_diffuse".to_string(), "tex_normal".to_string()];
        model.statistics.texture_byte_lengths.insert("tex_diffuse".to_string(), 1024);
        model.statistics.texture_byte_lengths.insert("tex_normal".to_string(), 2048);
        model.statistics.textures_byte_length = 3072;
    }

    assert_eq!(content.textures_byte_length(), 3072);
    assert_eq!(content.get_texture_ids(), vec!["tex_diffuse", "tex_normal"]);
    assert_eq!(content.get_texture_byte_length_by_id("tex_diffuse"), Some(1024));
    assert_eq!(content.get_texture_byte_length_by_id("tex_unknown"), None);
}

#[test]
fn test_pnts_content_and_attenuation() {
    let tileset = Model3DTilesetContext {
        point_cloud_shading: Some(PointCloudShading {
            attenuation: true,
            geometric_error_scale: 1.5,
            maximum_attenuation: 16.0,
            base_resolution: 0.05,
            ..Default::default()
        }),
        ..Default::default()
    };

    let tile = Model3DTileContext::default();

    let mut buffer = Vec::new();
    buffer.extend_from_slice(b"pnts");
    buffer.extend_from_slice(&1u32.to_le_bytes());
    buffer.extend_from_slice(&48u32.to_le_bytes());
    buffer.extend_from_slice(&10u32.to_le_bytes());
    buffer.extend_from_slice(&0u32.to_le_bytes());
    buffer.extend_from_slice(&10u32.to_le_bytes());
    buffer.extend_from_slice(&0u32.to_le_bytes());
    buffer.resize(48, 0);

    let mut content = Model3DTileContent::from_pnts(tileset, tile, "points.pnts".to_string(), &buffer, 0).unwrap();
    if let Some(model) = &mut content.model {
        model.statistics.points_length = 5000;
    }

    assert_eq!(content.points_length(), 5000);

    // Apply styling
    let style = Cesium3DTileStyle {
        color_expression: Some("color('red')".to_string()),
        show_expression: Some("true".to_string()),
        point_size_expression: Some("5.0".to_string()),
    };
    content.apply_style(Some(style));
    assert!(content.model.as_ref().unwrap().style.is_some());

    // Per-frame update should synchronize attenuation
    content.update();
    let pcs = content.model.as_ref().unwrap().point_cloud_shading.as_ref().unwrap();
    assert!(pcs.attenuation);
    assert_eq!(pcs.maximum_attenuation, 16.0);
    assert_eq!(pcs.geometric_error_scale, 1.5);
    assert_eq!(pcs.base_resolution, 0.05);
}

#[test]
fn test_gltf_lifecycle_extensions_and_errors() {
    let tileset = Model3DTilesetContext::default();
    let tile = Model3DTileContext::default();

    let gltf = json!({
        "asset": { "version": "2.0" },
        "extensions": {
            "EXT_structural_metadata": {
                "schema": { "id": "testSchema" }
            }
        }
    });

    let mut content = Model3DTileContent::from_gltf(tileset, tile, "model.gltf".to_string(), gltf).unwrap();
    assert!(content.ready);

    let ext = content.get_extension("EXT_structural_metadata");
    assert!(ext.is_some());
    assert_eq!(ext.unwrap()["schema"]["id"], "testSchema");

    // Calling get_feature without selected feature table should throw DeveloperError
    let err = content.get_feature(0).unwrap_err();
    assert!(err.message.contains("No feature ID set is selected"));

    // Select feature table with 2 features
    if let Some(model) = &mut content.model {
        model.feature_tables.push(ModelFeatureTable::new(2));
        model.feature_table_id = Some(0);
    }

    assert!(content.get_feature(0).is_ok());
    assert!(content.get_feature(1).is_ok());

    // Calling with out-of-range index should throw DeveloperError
    let err_out = content.get_feature(2).unwrap_err();
    assert!(err_out.message.contains("featureId is required and must be between 0 and featuresLength - 1"));

    // Destroy
    content.destroy();
    assert!(content.is_destroyed);
    assert!(content.model.is_none());
}

#[test]
fn test_clipping_planes_and_polygons_synchronization() {
    let mut tileset = Model3DTilesetContext::default();
    let mut planes = ClippingPlaneCollection::default();
    planes.enabled = true;
    tileset.clipping_planes = Some(planes);

    let mut polygons = ClippingPolygonCollection::default();
    polygons.enabled = true;
    tileset.clipping_polygons = Some(polygons);

    let mut tile = Model3DTileContext {
        is_clipped: true,
        is_clipped_by_polygon: true,
        clipping_planes_dirty: true,
        clipping_polygons_dirty: true,
        clipping_polygons_need_rebake: true,
        ..Default::default()
    };

    let mut content = Model3DTileContent::from_gltf(tileset, tile.clone(), "test.glb".to_string(), json!({})).unwrap();

    content.update();

    let model = content.model.as_ref().unwrap();
    assert!(model.clipping_planes.is_some());
    assert!(model.clipping_polygons.is_some());
    assert!(model.clipping_polygons_need_rebake);

    // If tile is not clipped, clipping planes are decoupled
    tile.is_clipped = false;
    tile.is_clipped_by_polygon = false;
    tile.clipping_planes_dirty = true;
    tile.clipping_polygons_dirty = true;
    content.tile = tile;

    content.update();

    let model = content.model.as_ref().unwrap();
    assert!(model.clipping_planes.is_none());
    assert!(model.clipping_polygons.is_none());
}

#[test]
fn test_classification_and_metadata_assignment() {
    let tileset = Model3DTilesetContext {
        classification_type: Some(ClassificationType::Both),
        ..Default::default()
    };

    let tile = Model3DTileContext::default();
    let mut content = Model3DTileContent::from_gltf(tileset, tile, "classified.glb".to_string(), json!({})).unwrap();

    // Assign content metadata and group metadata
    let class_def = MetadataClass::default();
    let content_meta = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({ "properties": { "name": "TileA" } })),
        class_def: Some(class_def.clone()),
    }).unwrap();

    let group_meta = GroupMetadata::new(GroupMetadataOptions {
        id: Some("group0".to_string()),
        group: Some(json!({ "properties": { "layer": 1 } })),
        class_def: Some(class_def),
    }).unwrap();

    content.metadata = Some(content_meta);
    content.group = Some(group_meta);

    assert!(content.metadata.is_some());
    assert!(content.group.is_some());
}
