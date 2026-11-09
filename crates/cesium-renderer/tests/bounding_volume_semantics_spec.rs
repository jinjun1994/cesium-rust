use cesium_renderer::{
    BoundingVolumeSemanticValue, BoundingVolumeSemantics, BoundingVolumeSemanticsResult,
    MetadataClass, TileMetadata,
};
use serde_json::json;
use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_6};

fn setup_fixtures() -> (
    Vec<f64>,
    Vec<f64>,
    Vec<f64>,
    f64,
    f64,
    MetadataClass,
    TileMetadata,
) {
    let bounding_box = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let bounding_region = vec![0.0, 0.0, FRAC_PI_6, FRAC_PI_2, 0.0, 50.0];
    let bounding_sphere = vec![0.0, 0.0, 0.0, 1.0];
    let minimum_height = -10.0;
    let maximum_height = 10.0;

    let tile_class_json = json!({
        "id": "tile",
        "class": {
            "properties": {
                "tileBoundingBox": {
                    "type": "SCALAR",
                    "componentType": "FLOAT64",
                    "array": true,
                    "count": 12,
                    "semantic": "TILE_BOUNDING_BOX"
                },
                "tileBoundingRegion": {
                    "type": "SCALAR",
                    "componentType": "FLOAT64",
                    "array": true,
                    "count": 6,
                    "semantic": "TILE_BOUNDING_REGION"
                },
                "tileBoundingSphere": {
                    "type": "SCALAR",
                    "componentType": "FLOAT64",
                    "array": true,
                    "count": 4,
                    "semantic": "TILE_BOUNDING_SPHERE"
                },
                "tileMinimumHeight": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "semantic": "TILE_MINIMUM_HEIGHT"
                },
                "tileMaximumHeight": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "semantic": "TILE_MAXIMUM_HEIGHT"
                }
            }
        }
    });
    let tile_class = MetadataClass::from_json(&tile_class_json);
    let empty_tile_metadata = TileMetadata::default();

    (
        bounding_box,
        bounding_region,
        bounding_sphere,
        minimum_height,
        maximum_height,
        tile_class,
        empty_tile_metadata,
    )
}

#[test]
fn test_parse_all_throws_without_prefix() {
    let (_, _, _, _, _, _, empty_metadata) = setup_fixtures();
    assert!(BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        None,
        Some(&empty_metadata)
    )
    .is_err());
}

#[test]
fn test_parse_all_throws_if_prefix_not_tile_or_content() {
    let (_, _, _, _, _, _, empty_metadata) = setup_fixtures();
    assert!(BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILESET"),
        Some(&empty_metadata)
    )
    .is_err());
}

#[test]
fn test_parse_all_throws_without_metadata() {
    assert!(
        BoundingVolumeSemantics::parse_all_bounding_volume_semantics(Some("TILE"), None).is_err()
    );
}

#[test]
fn test_parse_all_works_if_no_semantics_present() {
    let (_, _, _, _, _, _, empty_metadata) = setup_fixtures();
    let res = BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILE"),
        Some(&empty_metadata),
    )
    .unwrap();
    assert_eq!(res, BoundingVolumeSemanticsResult::default());
}

#[test]
fn test_parse_all_minimum_and_maximum_height() {
    let (_, _, _, min_h, max_h, tile_class, _) = setup_fixtures();
    let mut props = HashMap::new();
    props.insert("tileMinimumHeight".to_string(), json!(min_h));
    props.insert("tileMaximumHeight".to_string(), json!(max_h));
    let metadata = TileMetadata::new(Some(tile_class), props);

    let res = BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILE"),
        Some(&metadata),
    )
    .unwrap();
    assert_eq!(
        res,
        BoundingVolumeSemanticsResult {
            bounding_volume: None,
            minimum_height: Some(min_h),
            maximum_height: Some(max_h),
        }
    );
}

#[test]
fn test_parse_all_bounding_volumes() {
    let (box_vol, region_vol, sphere_vol, _, _, tile_class, _) = setup_fixtures();

    let mut props_box = HashMap::new();
    props_box.insert("tileBoundingBox".to_string(), json!(box_vol));
    let meta_box = TileMetadata::new(Some(tile_class.clone()), props_box);
    let res_box = BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILE"),
        Some(&meta_box),
    )
    .unwrap();
    assert_eq!(
        res_box,
        BoundingVolumeSemanticsResult {
            bounding_volume: Some(BoundingVolumeSemanticValue::Box(box_vol.clone())),
            minimum_height: None,
            maximum_height: None,
        }
    );

    let mut props_region = HashMap::new();
    props_region.insert("tileBoundingRegion".to_string(), json!(region_vol));
    let meta_region = TileMetadata::new(Some(tile_class.clone()), props_region);
    let res_region = BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILE"),
        Some(&meta_region),
    )
    .unwrap();
    assert_eq!(
        res_region,
        BoundingVolumeSemanticsResult {
            bounding_volume: Some(BoundingVolumeSemanticValue::Region(region_vol.clone())),
            minimum_height: None,
            maximum_height: None,
        }
    );

    let mut props_sphere = HashMap::new();
    props_sphere.insert("tileBoundingSphere".to_string(), json!(sphere_vol));
    let meta_sphere = TileMetadata::new(Some(tile_class), props_sphere);
    let res_sphere = BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILE"),
        Some(&meta_sphere),
    )
    .unwrap();
    assert_eq!(
        res_sphere,
        BoundingVolumeSemanticsResult {
            bounding_volume: Some(BoundingVolumeSemanticValue::Sphere(sphere_vol.clone())),
            minimum_height: None,
            maximum_height: None,
        }
    );
}

#[test]
fn test_parse_all_precedence() {
    let (box_vol, region_vol, sphere_vol, _, _, tile_class, _) = setup_fixtures();

    let mut props_box_region = HashMap::new();
    props_box_region.insert("tileBoundingBox".to_string(), json!(box_vol));
    props_box_region.insert("tileBoundingRegion".to_string(), json!(region_vol));
    let meta_box_region = TileMetadata::new(Some(tile_class.clone()), props_box_region);
    let res1 = BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILE"),
        Some(&meta_box_region),
    )
    .unwrap();
    assert_eq!(
        res1.bounding_volume,
        Some(BoundingVolumeSemanticValue::Box(box_vol))
    );

    let mut props_region_sphere = HashMap::new();
    props_region_sphere.insert("tileBoundingRegion".to_string(), json!(region_vol));
    props_region_sphere.insert("tileBoundingSphere".to_string(), json!(sphere_vol));
    let meta_region_sphere = TileMetadata::new(Some(tile_class), props_region_sphere);
    let res2 = BoundingVolumeSemantics::parse_all_bounding_volume_semantics(
        Some("TILE"),
        Some(&meta_region_sphere),
    )
    .unwrap();
    assert_eq!(
        res2.bounding_volume,
        Some(BoundingVolumeSemanticValue::Region(region_vol))
    );
}

#[test]
fn test_parse_bounding_volume_semantic_errors() {
    let (_, _, _, _, _, _, empty_metadata) = setup_fixtures();
    assert!(BoundingVolumeSemantics::parse_bounding_volume_semantic(
        None,
        Some(&empty_metadata)
    )
    .is_err());
    assert!(BoundingVolumeSemantics::parse_bounding_volume_semantic(
        Some("TILESET"),
        Some(&empty_metadata)
    )
    .is_err());
    assert!(
        BoundingVolumeSemantics::parse_bounding_volume_semantic(Some("TILE"), None).is_err()
    );
}

#[test]
fn test_parse_bounding_volume_semantic_values() {
    let (box_vol, region_vol, sphere_vol, _, _, tile_class, empty_metadata) = setup_fixtures();

    assert_eq!(
        BoundingVolumeSemantics::parse_bounding_volume_semantic(
            Some("TILE"),
            Some(&empty_metadata)
        )
        .unwrap(),
        None
    );

    let mut props_box = HashMap::new();
    props_box.insert("tileBoundingBox".to_string(), json!(box_vol));
    let meta_box = TileMetadata::new(Some(tile_class.clone()), props_box);
    assert_eq!(
        BoundingVolumeSemantics::parse_bounding_volume_semantic(Some("TILE"), Some(&meta_box))
            .unwrap(),
        Some(BoundingVolumeSemanticValue::Box(box_vol.clone()))
    );

    let mut props_region = HashMap::new();
    props_region.insert("tileBoundingRegion".to_string(), json!(region_vol));
    let meta_region = TileMetadata::new(Some(tile_class.clone()), props_region);
    assert_eq!(
        BoundingVolumeSemantics::parse_bounding_volume_semantic(
            Some("TILE"),
            Some(&meta_region)
        )
        .unwrap(),
        Some(BoundingVolumeSemanticValue::Region(region_vol.clone()))
    );

    let mut props_sphere = HashMap::new();
    props_sphere.insert("tileBoundingSphere".to_string(), json!(sphere_vol));
    let meta_sphere = TileMetadata::new(Some(tile_class.clone()), props_sphere);
    assert_eq!(
        BoundingVolumeSemantics::parse_bounding_volume_semantic(
            Some("TILE"),
            Some(&meta_sphere)
        )
        .unwrap(),
        Some(BoundingVolumeSemanticValue::Sphere(sphere_vol))
    );

    // Precedence
    let mut props_box_region = HashMap::new();
    props_box_region.insert("tileBoundingBox".to_string(), json!(box_vol));
    props_box_region.insert("tileBoundingRegion".to_string(), json!(region_vol));
    let meta_box_region = TileMetadata::new(Some(tile_class), props_box_region);
    assert_eq!(
        BoundingVolumeSemantics::parse_bounding_volume_semantic(
            Some("TILE"),
            Some(&meta_box_region)
        )
        .unwrap(),
        Some(BoundingVolumeSemanticValue::Box(box_vol))
    );
}

#[test]
fn test_parse_minimum_height() {
    let (_, _, _, min_h, _, tile_class, empty_metadata) = setup_fixtures();

    assert!(BoundingVolumeSemantics::parse_minimum_height(None, Some(&empty_metadata)).is_err());
    assert!(BoundingVolumeSemantics::parse_minimum_height(
        Some("TILESET"),
        Some(&empty_metadata)
    )
    .is_err());
    assert!(BoundingVolumeSemantics::parse_minimum_height(Some("TILE"), None).is_err());

    assert_eq!(
        BoundingVolumeSemantics::parse_minimum_height(Some("TILE"), Some(&empty_metadata))
            .unwrap(),
        None
    );

    let mut props = HashMap::new();
    props.insert("tileMinimumHeight".to_string(), json!(min_h));
    let meta = TileMetadata::new(Some(tile_class), props);
    assert_eq!(
        BoundingVolumeSemantics::parse_minimum_height(Some("TILE"), Some(&meta)).unwrap(),
        Some(min_h)
    );
}

#[test]
fn test_parse_maximum_height() {
    let (_, _, _, _, max_h, tile_class, empty_metadata) = setup_fixtures();

    assert!(BoundingVolumeSemantics::parse_maximum_height(None, Some(&empty_metadata)).is_err());
    assert!(BoundingVolumeSemantics::parse_maximum_height(
        Some("TILESET"),
        Some(&empty_metadata)
    )
    .is_err());
    assert!(BoundingVolumeSemantics::parse_maximum_height(Some("TILE"), None).is_err());

    assert_eq!(
        BoundingVolumeSemantics::parse_maximum_height(Some("TILE"), Some(&empty_metadata))
            .unwrap(),
        None
    );

    let mut props = HashMap::new();
    props.insert("tileMaximumHeight".to_string(), json!(max_h));
    let meta = TileMetadata::new(Some(tile_class), props);
    assert_eq!(
        BoundingVolumeSemantics::parse_maximum_height(Some("TILE"), Some(&meta)).unwrap(),
        Some(max_h)
    );
}
