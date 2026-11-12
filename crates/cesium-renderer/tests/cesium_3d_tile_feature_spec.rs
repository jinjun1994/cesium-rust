use std::collections::HashMap;

use cesium_math::Color;
use cesium_renderer::bounding_volume_semantics::MetadataClass;
use cesium_renderer::model_feature_table::{
    ModelFeatureTable, ModelPropertyDefinition, ModelPropertyTable, ModelRuntimeTarget,
};
use cesium_renderer::tile_metadata::TileMetadata;
use cesium_renderer::tileset_metadata::TilesetMetadata;
use cesium_renderer::{
    Cesium3DTileFeature, Cesium3DTileFeatureContent, ContentMetadata, ContentMetadataOptions,
    GroupMetadata, GroupMetadataOptions,
};
use serde_json::json;

#[test]
fn polyline_positions_gets_positions_and_returns_none_for_non_polyline() {
    let mut content = Cesium3DTileFeatureContent::default();
    content
        .polyline_positions
        .insert(0, vec![6378136.806372941, -1113.194885441724, 1105.675261474196]);

    let f0 = Cesium3DTileFeature::new(0);
    let f1 = Cesium3DTileFeature::new(1);

    let pos = f0.polyline_positions(&content).unwrap();
    assert_eq!(pos.len(), 3);
    assert!((pos[0] - 6378136.806372941).abs() < 1e-7);
    assert!(f1.polyline_positions(&content).is_none());
}

#[test]
fn show_color_and_feature_properties_work() {
    let mut prop_table = ModelPropertyTable::new(2);
    prop_table.add_property(
        "height",
        ModelPropertyDefinition {
            semantic: Some("_BUILDING_HEIGHT".to_string()),
            property_type: Some("SCALAR".to_string()),
            component_type: Some("FLOAT64".to_string()),
        },
        vec![json!(10.0), json!(20.0)],
    );
    let feature_table = ModelFeatureTable::new(ModelRuntimeTarget::default(), prop_table);

    let mut content = Cesium3DTileFeatureContent {
        feature_table: Some(feature_table),
        ..Default::default()
    };

    let feature = Cesium3DTileFeature::new(0);
    assert_eq!(feature.feature_id(), 0);
    assert!(feature.show(&content));
    assert_eq!(feature.color(&content), Color::WHITE);

    feature.set_show(&mut content, false);
    feature.set_color(&mut content, Color::RED);
    assert!(!feature.show(&content));
    assert_eq!(feature.color(&content), Color::RED);

    assert!(feature.has_property(&content, "height"));
    assert_eq!(feature.get_property_ids(&content), vec!["height"]);
    assert_eq!(feature.get_property(&content, "height"), Some(json!(10.0)));

    feature.set_property(&mut content, "clicked", json!(true));
    assert!(content.feature_properties_dirty);
    assert_eq!(feature.get_property(&content, "clicked"), Some(json!(true)));
}

#[test]
fn get_property_inherited_resolves_priority_order_across_all_metadata_layers() {
    // 1. Feature table (semantic + property)
    let mut prop_table = ModelPropertyTable::new(1);
    prop_table.add_property(
        "height",
        ModelPropertyDefinition {
            semantic: Some("_FEATURE_HEIGHT".to_string()),
            property_type: Some("SCALAR".to_string()),
            component_type: Some("FLOAT64".to_string()),
        },
        vec![json!(15.0)],
    );
    let feature_table = ModelFeatureTable::new(ModelRuntimeTarget::default(), prop_table);

    // 2. Content metadata
    let content_class = MetadataClass::from_json(&json!({
        "id": "contentClass",
        "class": {
            "properties": {
                "contentProp": { "type": "SCALAR", "componentType": "INT32" },
                "height": { "type": "SCALAR", "componentType": "INT32" }
            }
        }
    }))
    .unwrap();
    let content_meta = ContentMetadata::new(ContentMetadataOptions {
        content: Some(json!({ "properties": { "contentProp": 100, "height": 999 } })),
        class_def: Some(content_class),
    })
    .unwrap();

    // 3. Tile metadata
    let tile_class = MetadataClass::from_json(&json!({
        "id": "tileClass",
        "class": {
            "properties": {
                "tileHeight": { "type": "SCALAR", "componentType": "INT32", "semantic": "_TILE_HEIGHT" },
                "contentProp": { "type": "SCALAR", "componentType": "INT32" }
            }
        }
    }))
    .unwrap();
    let mut tile_props = HashMap::new();
    tile_props.insert("tileHeight".to_string(), json!(200));
    tile_props.insert("contentProp".to_string(), json!(888));
    let tile_meta = TileMetadata::new(Some(tile_class), tile_props);

    // 4. Group metadata
    let group_class = MetadataClass::from_json(&json!({
        "id": "groupClass",
        "class": {
            "properties": {
                "groupName": { "type": "STRING" }
            }
        }
    }))
    .unwrap();
    let group_meta = GroupMetadata::new(GroupMetadataOptions {
        id: Some("buildings".to_string()),
        group: Some(json!({ "properties": { "groupName": "Downtown" } })),
        class_def: Some(group_class),
    })
    .unwrap();

    // 5. Tileset metadata
    let tileset_class = MetadataClass::from_json(&json!({
        "id": "tilesetClass",
        "class": {
            "properties": {
                "author": { "type": "STRING" },
                "groupName": { "type": "STRING" }
            }
        }
    }))
    .unwrap();
    let mut tileset_props = HashMap::new();
    tileset_props.insert("author".to_string(), json!("Cesium"));
    tileset_props.insert("groupName".to_string(), json!("Ignored"));
    let tileset_meta = TilesetMetadata::new(Some(tileset_class), tileset_props);

    let content = Cesium3DTileFeatureContent {
        feature_table: Some(feature_table),
        content_metadata: Some(content_meta),
        tile_metadata: Some(tile_meta),
        group_metadata: Some(group_meta),
        tileset_metadata: Some(tileset_meta),
        ..Default::default()
    };

    let feature = Cesium3DTileFeature::new(0);
    // Feature semantic beats everything
    assert_eq!(
        feature.get_property_inherited(&content, "_FEATURE_HEIGHT"),
        Some(json!(15.0))
    );
    // Feature property beats content property of same name
    assert_eq!(
        feature.get_property_inherited(&content, "height"),
        Some(json!(15.0))
    );
    // Content beats tile
    assert_eq!(
        feature.get_property_inherited(&content, "contentProp"),
        Some(json!(100))
    );
    // Tile semantic
    assert_eq!(
        feature
            .get_property_inherited(&content, "_TILE_HEIGHT")
            .and_then(|v| v.as_f64()),
        Some(200.0)
    );
    // Group beats tileset
    assert_eq!(
        feature.get_property_inherited(&content, "groupName"),
        Some(json!("Downtown"))
    );
    // Tileset fallback
    assert_eq!(
        feature.get_property_inherited(&content, "author"),
        Some(json!("Cesium"))
    );
    // Missing property returns None
    assert_eq!(feature.get_property_inherited(&content, "nonexistent"), None);
}
