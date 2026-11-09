use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde_json::json;
use cesium_renderer::bounding_volume_semantics::MetadataClass;
use cesium_renderer::implicit_metadata_view::{
    ImplicitMetadataView, ImplicitMetadataViewOptions, MetadataTable,
};

fn create_schema_and_classes() -> (MetadataClass, MetadataClass, MetadataClass) {
    let tile_class_json = json!({
        "id": "tile",
        "class": {
            "properties": {
                "highlightColor": {
                    "type": "VEC3",
                    "componentType": "UINT8",
                    "semantic": "_HIGHLIGHT_COLOR"
                },
                "buildingCount": {
                    "type": "SCALAR",
                    "componentType": "UINT16"
                }
            }
        }
    });

    let building_class_json = json!({
        "id": "building",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "UINT16",
                    "semantic": "_HEIGHT"
                },
                "buildingType": {
                    "type": "STRING",
                    "semantic": "_BUILDING_TYPE"
                }
            }
        }
    });

    let tree_class_json = json!({
        "id": "tree",
        "class": {
            "properties": {
                "height": {
                    "type": "SCALAR",
                    "componentType": "UINT16",
                    "semantic": "_HEIGHT"
                },
                "species": {
                    "type": "STRING",
                    "semantic": "_TREE_SPECIES"
                }
            }
        }
    });

    (
        MetadataClass::from_json(&tile_class_json),
        MetadataClass::from_json(&building_class_json),
        MetadataClass::from_json(&tree_class_json),
    )
}

fn setup_fixtures() -> (
    ImplicitMetadataView,
    ImplicitMetadataView,
    ImplicitMetadataView,
    ImplicitMetadataView,
    ImplicitMetadataView,
    ImplicitMetadataView,
) {
    let (tile_class, building_class, tree_class) = create_schema_and_classes();

    let mut tile_props = HashMap::new();
    tile_props.insert(
        "highlightColor".to_string(),
        vec![
            json!([255, 0, 0]),
            json!([255, 255, 0]),
            json!([255, 0, 255]),
            json!([255, 255, 255]),
        ],
    );
    tile_props.insert(
        "buildingCount".to_string(),
        vec![json!(100), json!(350), json!(200), json!(500)],
    );
    let tile_table = Arc::new(RwLock::new(MetadataTable::new(tile_class.clone(), 4, tile_props)));

    let mut building_props = HashMap::new();
    building_props.insert(
        "height".to_string(),
        vec![json!(20), json!(30), json!(10)],
    );
    building_props.insert(
        "buildingType".to_string(),
        vec![json!("Residential"), json!("Commercial"), json!("Other")],
    );
    let building_table = Arc::new(RwLock::new(MetadataTable::new(building_class.clone(), 3, building_props)));

    let mut tree_props = HashMap::new();
    tree_props.insert("height".to_string(), vec![json!(3), json!(2), json!(4)]);
    tree_props.insert(
        "species".to_string(),
        vec![json!("Oak"), json!("Pine"), json!("Maple")],
    );
    let tree_table = Arc::new(RwLock::new(MetadataTable::new(tree_class.clone(), 3, tree_props)));

    let empty_json = json!({});

    let tile_view = ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(tile_table.clone()),
        class: Some(tile_class.clone()),
        entity_id: Some(0),
        property_table_json: Some(empty_json.clone()),
    })
    .unwrap();

    let second_tile_view = ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(tile_table),
        class: Some(tile_class),
        entity_id: Some(1),
        property_table_json: Some(empty_json.clone()),
    })
    .unwrap();

    let building_view = ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(building_table.clone()),
        class: Some(building_class.clone()),
        entity_id: Some(0),
        property_table_json: Some(empty_json.clone()),
    })
    .unwrap();

    let second_building_view = ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(building_table),
        class: Some(building_class),
        entity_id: Some(1),
        property_table_json: Some(empty_json.clone()),
    })
    .unwrap();

    let tree_view = ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(tree_table.clone()),
        class: Some(tree_class.clone()),
        entity_id: Some(0),
        property_table_json: Some(empty_json.clone()),
    })
    .unwrap();

    let second_tree_view = ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(tree_table),
        class: Some(tree_class),
        entity_id: Some(1),
        property_table_json: Some(empty_json),
    })
    .unwrap();

    (
        tile_view,
        second_tile_view,
        building_view,
        second_building_view,
        tree_view,
        second_tree_view,
    )
}

#[test]
fn creates_metadata_table_views_with_class_and_extras() {
    let (tile_view, _, building_view, _, tree_view, _) = setup_fixtures();

    assert_eq!(tile_view.class().id, "tile");
    assert!(tile_view.extras().is_none());
    assert!(tile_view.extensions().is_none());

    assert_eq!(building_view.class().id, "building");
    assert_eq!(tree_view.class().id, "tree");
}

#[test]
fn throws_without_required_options() {
    let (tile_class, _, _) = create_schema_and_classes();
    let empty_table = Arc::new(RwLock::new(MetadataTable::default()));

    assert!(ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: None,
        class: Some(tile_class.clone()),
        entity_id: Some(0),
        property_table_json: Some(json!({})),
    })
    .is_err());

    assert!(ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(empty_table.clone()),
        class: None,
        entity_id: Some(0),
        property_table_json: Some(json!({})),
    })
    .is_err());

    assert!(ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(empty_table.clone()),
        class: Some(tile_class.clone()),
        entity_id: None,
        property_table_json: Some(json!({})),
    })
    .is_err());

    assert!(ImplicitMetadataView::new(ImplicitMetadataViewOptions {
        metadata_table: Some(empty_table),
        class: Some(tile_class),
        entity_id: Some(0),
        property_table_json: None,
    })
    .is_err());
}

#[test]
fn has_property_and_has_property_by_semantic() {
    let (tile_view, _, building_view, _, tree_view, _) = setup_fixtures();

    assert!(tile_view.has_property("highlightColor"));
    assert!(building_view.has_property("height"));
    assert!(tree_view.has_property("species"));

    assert!(!tile_view.has_property("height"));
    assert!(!building_view.has_property("buildingCount"));
    assert!(!tree_view.has_property("buildingType"));

    assert!(tile_view.has_property_by_semantic("_HIGHLIGHT_COLOR"));
    assert!(building_view.has_property_by_semantic("_HEIGHT"));
    assert!(tree_view.has_property_by_semantic("_TREE_SPECIES"));

    assert!(!tile_view.has_property_by_semantic("_AREA"));
    assert!(!building_view.has_property_by_semantic("_HIGHLIGHT_COLOR"));
    assert!(!tree_view.has_property_by_semantic("_BUILDING_TYPE"));
}

#[test]
fn get_property_ids_works() {
    let (tile_view, _, building_view, _, tree_view, _) = setup_fixtures();

    let mut tile_ids = Vec::new();
    tile_view.get_property_ids(&mut tile_ids);
    tile_ids.sort();
    assert_eq!(tile_ids, vec!["buildingCount", "highlightColor"]);

    let mut building_ids = Vec::new();
    building_view.get_property_ids(&mut building_ids);
    building_ids.sort();
    assert_eq!(building_ids, vec!["buildingType", "height"]);

    let mut tree_ids = Vec::new();
    tree_view.get_property_ids(&mut tree_ids);
    tree_ids.sort();
    assert_eq!(tree_ids, vec!["height", "species"]);
}

#[test]
fn get_property_values() {
    let (tile_view, second_tile_view, building_view, second_building_view, tree_view, second_tree_view) =
        setup_fixtures();

    assert_eq!(tile_view.get_property("highlightColor"), Some(json!([255, 0, 0])));
    assert_eq!(tile_view.get_property("buildingCount"), Some(json!(100)));

    assert_eq!(building_view.get_property("height"), Some(json!(20)));
    assert_eq!(building_view.get_property("buildingType"), Some(json!("Residential")));

    assert_eq!(tree_view.get_property("height"), Some(json!(3)));
    assert_eq!(tree_view.get_property("species"), Some(json!("Oak")));

    assert_eq!(second_tile_view.get_property("highlightColor"), Some(json!([255, 255, 0])));
    assert_eq!(second_building_view.get_property("buildingType"), Some(json!("Commercial")));
    assert_eq!(second_tree_view.get_property("species"), Some(json!("Pine")));
}

#[test]
fn set_property_values() {
    let (mut tile_view, second_tile_view, mut building_view, _, mut tree_view, _) = setup_fixtures();

    assert!(!tile_view.set_property("height", json!(10)));
    assert!(!building_view.set_property("buildingCount", json!(10)));
    assert!(!tree_view.set_property("buildingType", json!("Other")));

    assert!(tile_view.set_property("buildingCount", json!(400)));
    assert_eq!(tile_view.get_property("buildingCount"), Some(json!(400)));
    assert_eq!(second_tile_view.get_property("buildingCount"), Some(json!(350)));

    assert!(building_view.set_property("buildingType", json!("Commercial")));
    assert_eq!(building_view.get_property("buildingType"), Some(json!("Commercial")));

    assert!(tree_view.set_property("species", json!("Chestnut")));
    assert_eq!(tree_view.get_property("species"), Some(json!("Chestnut")));
}

#[test]
fn semantic_get_and_set() {
    let (mut tile_view, second_tile_view, mut building_view, _, mut tree_view, _) = setup_fixtures();

    assert!(tile_view.get_property_by_semantic("_AREA").is_none());
    assert_eq!(
        tile_view.get_property_by_semantic("_HIGHLIGHT_COLOR"),
        Some(json!([255, 0, 0]))
    );
    assert_eq!(
        building_view.get_property_by_semantic("_BUILDING_TYPE"),
        Some(json!("Residential"))
    );
    assert_eq!(
        tree_view.get_property_by_semantic("_TREE_SPECIES"),
        Some(json!("Oak"))
    );

    assert!(tile_view.set_property_by_semantic("_HIGHLIGHT_COLOR", json!([0, 0, 0])));
    assert_eq!(
        tile_view.get_property_by_semantic("_HIGHLIGHT_COLOR"),
        Some(json!([0, 0, 0]))
    );
    assert_eq!(
        second_tile_view.get_property_by_semantic("_HIGHLIGHT_COLOR"),
        Some(json!([255, 255, 0]))
    );

    assert!(building_view.set_property_by_semantic("_BUILDING_TYPE", json!("Other")));
    assert_eq!(
        building_view.get_property_by_semantic("_BUILDING_TYPE"),
        Some(json!("Other"))
    );

    assert!(tree_view.set_property_by_semantic("_TREE_SPECIES", json!("Chestnut")));
    assert_eq!(
        tree_view.get_property_by_semantic("_TREE_SPECIES"),
        Some(json!("Chestnut"))
    );

    assert!(!tile_view.set_property_by_semantic("_AREA", json!(100)));
}
