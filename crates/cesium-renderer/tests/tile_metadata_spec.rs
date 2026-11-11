use cesium_renderer::metadata_class::MetadataClass;
use cesium_renderer::tile_metadata::{TileMetadata, TileMetadataOptions};
use serde_json::json;

fn create_tile_class_with_no_properties() -> MetadataClass {
    MetadataClass::from_json(&json!({
        "id": "tile",
        "class": {}
    }))
    .expect("class creation")
}

fn create_tile_class() -> MetadataClass {
    MetadataClass::from_json(&json!({
        "id": "tile",
        "class": {
            "properties": {
                "color": {
                    "type": "SCALAR",
                    "componentType": "FLOAT32",
                    "array": true,
                    "count": 8,
                    "semantic": "COLOR"
                },
                "isSquare": {
                    "description": "Is a square tile, rather than a rectangular partial tile",
                    "type": "BOOLEAN"
                }
            }
        }
    }))
    .expect("class creation")
}

#[test]
fn throws_without_tile() {
    let tile_class = create_tile_class();
    let res = TileMetadata::from_options(TileMetadataOptions {
        tile: None,
        class_def: Some(tile_class),
    });
    assert!(res.is_err());
}

#[test]
fn throws_without_class() {
    let res = TileMetadata::from_options(TileMetadataOptions {
        tile: Some(json!({})),
        class_def: None,
    });
    assert!(res.is_err());
}

#[test]
fn creates_tile_metadata_with_default_values() {
    let tile_class_no_props = create_tile_class_with_no_properties();
    let metadata = TileMetadata::from_options(TileMetadataOptions {
        tile: Some(json!({})),
        class_def: Some(tile_class_no_props),
    })
    .expect("metadata creation");

    assert!(metadata.extras().is_none());
    assert!(metadata.extensions().is_none());
}

#[test]
fn creates_tile_metadata() {
    let tile_class = create_tile_class();
    let properties = json!({
        "color": [0.0, 0.0, 1.0],
        "isSquare": false
    });
    let extras = json!({
        "version": "0.0"
    });
    let extensions = json!({
        "3DTILES_extension": {}
    });

    let tile = json!({
        "class": "tile",
        "properties": properties,
        "extras": extras,
        "extensions": extensions
    });

    let metadata = TileMetadata::from_options(TileMetadataOptions {
        tile: Some(tile),
        class_def: Some(tile_class.clone()),
    })
    .expect("metadata creation");

    assert_eq!(metadata.class_definition(), Some(&tile_class));
    assert_eq!(
        metadata.get_property("color").unwrap(),
        json!([0.0, 0.0, 1.0])
    );
    assert_eq!(metadata.get_property("isSquare").unwrap(), json!(false));
    assert_eq!(metadata.extras(), Some(&extras));
    assert_eq!(metadata.extensions(), Some(&extensions));
}

#[test]
fn property_checks_and_semantic_access() {
    let tile_class = create_tile_class();
    let tile_extension = json!({
        "class": "tile",
        "properties": {
            "color": [1.0, 0.5, 0.0],
            "isSquare": true
        }
    });

    let mut metadata = TileMetadata::from_options(TileMetadataOptions {
        tile: Some(tile_extension),
        class_def: Some(tile_class),
    })
    .expect("metadata creation");

    // hasProperty returns true if the tile has this property
    assert!(metadata.has_property("color"));
    // hasProperty returns false if the tile does not have this property
    assert!(!metadata.has_property("numberOfPoints"));

    // hasPropertyBySemantic returns true if the tile has a property with the given semantic
    assert!(metadata.has_property_by_semantic("COLOR"));
    // hasPropertyBySemantic returns false if the tile does not have a property with the given semantic
    assert!(!metadata.has_property_by_semantic("NUMBER_OF_POINTS"));

    // getPropertyIds returns array of property IDs
    let mut property_ids = metadata.get_property_ids(None);
    property_ids.sort();
    assert_eq!(property_ids, vec!["color", "isSquare"]);

    // getProperty returns throws if a property does not exist
    assert!(metadata.get_property("height").is_err());

    // getProperty returns the property value
    assert_eq!(
        metadata.get_property("color").unwrap(),
        json!([1.0, 0.5, 0.0])
    );
    assert_eq!(metadata.get_property("isSquare").unwrap(), json!(true));

    // setProperty returns false if property doesn't exist
    assert!(!metadata.set_property("numberOfPoints", json!(10)));

    // setProperty sets property value
    assert_eq!(metadata.get_property("isSquare").unwrap(), json!(true));
    assert!(metadata.set_property("isSquare", json!(false)));
    assert_eq!(metadata.get_property("isSquare").unwrap(), json!(false));

    // getPropertyBySemantic returns undefined when there's no property with the given semantic
    assert!(metadata
        .get_property_by_semantic("HORIZON_OCCLUSION_POINT")
        .is_none());

    // getPropertyBySemantic returns the property value
    assert_eq!(
        metadata.get_property_by_semantic("COLOR").unwrap(),
        json!([1.0, 0.5, 0.0])
    );

    // setPropertyBySemantic sets property value
    assert!(metadata.set_property_by_semantic("COLOR", json!([0.0, 0.0, 0.0])));
    assert_eq!(
        metadata.get_property_by_semantic("COLOR").unwrap(),
        json!([0.0, 0.0, 0.0])
    );

    // setPropertyBySemantic returns false if the semantic doesn't exist
    assert!(!metadata.set_property_by_semantic("NAME", json!("Test Tile")));
}
