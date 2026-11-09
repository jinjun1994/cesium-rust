use std::collections::HashMap;
use cesium_renderer::json_metadata_table::JsonMetadataTable;
use serde_json::json;

fn create_sample_table() -> JsonMetadataTable {
    let mut props = HashMap::new();
    props.insert("priority".to_string(), vec![json!(2), json!(1), json!(0)]);
    props.insert(
        "labels".to_string(),
        vec![json!("Point Cloud"), json!("Mesh"), json!("Raster")],
    );
    props.insert(
        "uri".to_string(),
        vec![json!("tree.las"), json!("building.gltf"), json!("map.tif")],
    );
    props.insert(
        "sizeInfo".to_string(),
        vec![
            json!({ "pointCount": 100 }),
            json!({ "vertices": 3000, "faces": 1000 }),
            json!({ "width": 1024, "height": 1024 }),
        ],
    );
    props.insert(
        "mixedValues".to_string(),
        vec![json!("red"), json!(3), json!(false)],
    );

    JsonMetadataTable::new(3, props).unwrap()
}

#[test]
fn constructor_throws_with_zero_count() {
    assert!(JsonMetadataTable::new(0, HashMap::new()).is_err());
}

#[test]
fn has_property_and_get_property_ids() {
    let table = create_sample_table();
    assert!(table.has_property("priority"));
    assert!(!table.has_property("price"));

    let mut ids = table.get_property_ids();
    ids.sort();
    assert_eq!(ids, vec!["labels", "mixedValues", "priority", "sizeInfo", "uri"]);
}

#[test]
fn get_property_returns_value_and_heterogeneous_types() {
    let table = create_sample_table();

    assert_eq!(table.get_property(0, "priority").unwrap(), Some(json!(2)));
    assert_eq!(table.get_property(1, "priority").unwrap(), Some(json!(1)));
    assert_eq!(table.get_property(2, "priority").unwrap(), Some(json!(0)));

    // Unknown property
    assert_eq!(table.get_property(0, "color").unwrap(), None);

    // Out of bounds
    assert!(table.get_property(5, "priority").is_err());

    // Heterogeneous
    assert_eq!(table.get_property(0, "mixedValues").unwrap(), Some(json!("red")));
    assert_eq!(table.get_property(1, "mixedValues").unwrap(), Some(json!(3)));
    assert_eq!(table.get_property(2, "mixedValues").unwrap(), Some(json!(false)));
}

#[test]
fn set_property_creates_and_updates_and_checks_bounds() {
    let mut table = create_sample_table();

    // Out of bounds
    assert!(table.set_property(5, "priority", json!(3)).is_err());

    // Create new property
    assert_eq!(table.get_property(0, "color").unwrap(), None);
    table
        .set_property(0, "color", json!([255, 255, 255, 1.0]))
        .unwrap();
    assert_eq!(
        table.get_property(0, "color").unwrap(),
        Some(json!([255, 255, 255, 1.0]))
    );

    // Update existing property
    let new_size = json!({ "lengthBytes": 1024 });
    table.set_property(0, "sizeInfo", new_size.clone()).unwrap();
    assert_eq!(table.get_property(0, "sizeInfo").unwrap(), Some(new_size));
}
