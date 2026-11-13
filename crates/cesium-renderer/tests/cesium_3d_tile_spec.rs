use cesium_renderer::cesium_3d_tile::{
    Cesium3DTile, Cesium3DTileBoundingVolume, Cesium3DTileRefine, MockTileset,
};
use serde_json::json;

#[test]
fn destroys_and_is_destroyed() {
    let tileset = MockTileset::default();
    let header = json!({
        "geometricError": 1,
        "refine": "REPLACE",
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 5.0]
        }
    });

    let mut tile = Cesium3DTile::new(&tileset, "/some_url", &header, None).unwrap();
    assert!(!tile.is_destroyed());
    tile.destroy();
    assert!(tile.is_destroyed());
}

#[test]
fn errors_if_bounding_volume_missing_or_invalid() {
    let tileset = MockTileset::default();

    let missing_bv = json!({
        "geometricError": 1,
        "refine": "REPLACE"
    });
    assert!(Cesium3DTile::new(&tileset, "/some_url", &missing_bv, None).is_err());

    let invalid_bv = json!({
        "geometricError": 1,
        "refine": "REPLACE",
        "boundingVolume": {
            "unknown": [1, 2, 3]
        }
    });
    assert!(Cesium3DTile::new(&tileset, "/some_url", &invalid_bv, None).is_err());
}

#[test]
fn parses_refine_and_loads_empty_content_for_empty_uri() {
    let tileset = MockTileset::default();

    // Lowercase refine
    let header_replace = json!({
        "geometricError": 1,
        "refine": "replace",
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 5.0]
        }
    });
    let tile_replace = Cesium3DTile::new(&tileset, "/some_url", &header_replace, None).unwrap();
    assert_eq!(tile_replace.refine, Cesium3DTileRefine::Replace);

    // Empty content URI -> empty content created
    let header_empty = json!({
        "geometricError": 1,
        "refine": "ADD",
        "content": {
            "uri": ""
        },
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 5.0]
        }
    });
    let tile_empty = Cesium3DTile::new(&tileset, "/some_url", &header_empty, None).unwrap();
    assert_eq!(tile_empty.refine, Cesium3DTileRefine::Add);
    assert!(tile_empty.has_empty_content());
}

#[test]
fn geometric_error_and_transform_inherit_from_parent_or_tileset() {
    let tileset = MockTileset {
        geometric_error: 2.0,
        ..Default::default()
    };

    let parent_header = json!({
        "geometricError": 1.0,
        "refine": "REPLACE",
        "transform": [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            10.0, 0.0, 0.0, 1.0
        ],
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 5.0]
        }
    });
    let parent = Cesium3DTile::new(&tileset, "/some_url", &parent_header, None).unwrap();
    assert_eq!(parent.geometric_error, 1.0);
    assert_eq!(parent.computed_transform.values[12], 10.0);

    // Child missing geometricError inherits from parent
    let child_header = json!({
        "refine": "REPLACE",
        "transform": [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            5.0, 0.0, 0.0, 1.0
        ],
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 2.5]
        }
    });
    let child = Cesium3DTile::new(&tileset, "/some_url", &child_header, Some(&parent)).unwrap();
    assert_eq!(child.geometric_error, 1.0);
    assert_eq!(child.computed_transform.values[12], 15.0);

    // Root tile missing geometricError inherits from tileset
    let root_missing_ge = json!({
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 5.0]
        }
    });
    let root = Cesium3DTile::new(&tileset, "/some_url", &root_missing_ge, None).unwrap();
    assert_eq!(root.geometric_error, 2.0);
}

#[test]
fn supports_sphere_region_and_box_bounding_volumes() {
    let tileset = MockTileset::default();

    // Sphere
    let header_sphere = json!({
        "geometricError": 1,
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 5.0]
        },
        "content": {
            "url": "0/0.b3dm",
            "boundingVolume": {
                "sphere": [0.0, 0.0, 1.0, 5.0]
            }
        }
    });
    let tile_sphere = Cesium3DTile::new(&tileset, "/some_url", &header_sphere, None).unwrap();
    assert!(matches!(tile_sphere.bounding_volume, Cesium3DTileBoundingVolume::Sphere(_)));
    assert!(matches!(tile_sphere.content_bounding_volume(), Cesium3DTileBoundingVolume::Sphere(_)));

    // Fallback to tile bounding volume when content bounding volume is undefined
    let header_no_cbv = json!({
        "geometricError": 1,
        "boundingVolume": {
            "sphere": [0.0, 0.0, 0.0, 5.0]
        },
        "content": {
            "url": "0/0.b3dm"
        }
    });
    let tile_no_cbv = Cesium3DTile::new(&tileset, "/some_url", &header_no_cbv, None).unwrap();
    assert_eq!(tile_no_cbv.content_bounding_volume(), &tile_no_cbv.bounding_volume);

    // Region
    let header_region = json!({
        "geometricError": 1,
        "boundingVolume": {
            "region": [-1.2, -1.2, 0.0, 0.0, -34.0, -30.0]
        }
    });
    let tile_region = Cesium3DTile::new(&tileset, "/some_url", &header_region, None).unwrap();
    assert!(matches!(tile_region.bounding_volume, Cesium3DTileBoundingVolume::Region(_)));

    // Box
    let header_box = json!({
        "geometricError": 1,
        "boundingVolume": {
            "box": [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]
        }
    });
    let tile_box = Cesium3DTile::new(&tileset, "/some_url", &header_box, None).unwrap();
    assert!(matches!(tile_box.bounding_volume, Cesium3DTileBoundingVolume::Box(_)));
}
