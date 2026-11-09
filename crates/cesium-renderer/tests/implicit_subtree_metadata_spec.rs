use serde_json::json;
use cesium_renderer::bounding_volume_semantics::MetadataClass;
use cesium_renderer::implicit_subtree_metadata::{
    ImplicitSubtreeMetadata, ImplicitSubtreeMetadataOptions,
};

fn make_subtree_class() -> MetadataClass {
    let json = json!({
        "id": "subtree",
        "class": {
            "properties": {
                "author": {
                    "type": "STRING",
                    "semantic": "AUTHOR"
                },
                "date": {
                    "type": "STRING"
                },
                "credits": {
                    "type": "STRING",
                    "array": true,
                    "semantic": "CREDITS"
                }
            }
        }
    });
    MetadataClass::from_json(&json)
}

#[test]
fn creates_subtree_metadata_with_default_values() {
    let subtree_metadata = ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        subtree_metadata: Some(json!({})),
        class: Some(MetadataClass::default()),
    })
    .expect("creation success");

    assert!(subtree_metadata.extras().is_none());
    assert!(subtree_metadata.extensions().is_none());
}

#[test]
fn creates_subtree_metadata_full() {
    let subtree_class = make_subtree_class();
    let extras = json!({ "other": 0 });
    let extensions = json!({ "EXT_other_extension": {} });
    let properties = json!({ "credits": ["A", "B", "C"] });

    let subtree_metadata = ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        class: Some(subtree_class.clone()),
        subtree_metadata: Some(json!({
            "class": "subtree",
            "properties": properties,
            "extensions": extensions,
            "extras": extras
        })),
    })
    .unwrap();

    assert_eq!(subtree_metadata.class_def().id, "subtree");
    assert_eq!(subtree_metadata.extras(), Some(&extras));
    assert_eq!(subtree_metadata.extensions(), Some(&extensions));
    assert_eq!(
        subtree_metadata.get_property(Some("credits")).unwrap(),
        Some(json!(["A", "B", "C"]))
    );
}

#[test]
fn constructor_throws_without_required_options() {
    let subtree_class = make_subtree_class();

    assert!(ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        subtree_metadata: None,
        class: Some(subtree_class.clone()),
    })
    .is_err());

    assert!(ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        subtree_metadata: Some(json!({})),
        class: None,
    })
    .is_err());
}

#[test]
fn has_property_and_has_property_by_semantic() {
    let subtree_class = make_subtree_class();
    let subtree_metadata = ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        class: Some(subtree_class),
        subtree_metadata: Some(json!({
            "properties": {
                "credits": ["A", "B", "C"]
            }
        })),
    })
    .unwrap();

    assert!(subtree_metadata.has_property(Some("credits")).unwrap());
    assert!(!subtree_metadata.has_property(Some("author")).unwrap());
    assert!(subtree_metadata.has_property(None).is_err());

    assert!(subtree_metadata.has_property_by_semantic(Some("CREDITS")).unwrap());
    assert!(!subtree_metadata.has_property_by_semantic(Some("AUTHOR")).unwrap());
    assert!(subtree_metadata.has_property_by_semantic(None).is_err());
}

#[test]
fn has_property_with_class_default() {
    let class_json = json!({
        "id": "subtree",
        "class": {
            "properties": {
                "credits": {
                    "type": "STRING",
                    "array": true,
                    "semantic": "CREDITS",
                    "default": []
                }
            }
        }
    });
    let subtree_class = MetadataClass::from_json(&class_json);
    let subtree_metadata = ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        class: Some(subtree_class),
        subtree_metadata: Some(json!({})),
    })
    .unwrap();

    assert!(subtree_metadata.has_property(Some("credits")).unwrap());
    assert!(subtree_metadata.has_property_by_semantic(Some("CREDITS")).unwrap());
}

#[test]
fn get_property_ids_works() {
    let subtree_class = make_subtree_class();
    let subtree_metadata = ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        class: Some(subtree_class),
        subtree_metadata: Some(json!({
            "properties": {
                "author": "Cesium",
                "date": "2022-02-10",
                "credits": ["A", "B", "C"]
            }
        })),
    })
    .unwrap();

    let mut ids = subtree_metadata.get_property_ids(None);
    ids.sort();
    assert_eq!(ids, vec!["author", "credits", "date"]);

    let mut results = Vec::new();
    subtree_metadata.get_property_ids(Some(&mut results));
    results.sort();
    assert_eq!(results, vec!["author", "credits", "date"]);
}

#[test]
fn get_property_and_defaults() {
    let subtree_class = make_subtree_class();
    let subtree_metadata = ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        class: Some(subtree_class),
        subtree_metadata: Some(json!({
            "properties": {
                "author": "Cesium"
            }
        })),
    })
    .unwrap();

    assert_eq!(
        subtree_metadata.get_property(Some("author")).unwrap(),
        Some(json!("Cesium"))
    );
    assert_eq!(subtree_metadata.get_property(Some("date")).unwrap(), None);
    assert!(subtree_metadata.get_property(Some("non_existent")).is_err());
    assert!(subtree_metadata.get_property(None).is_err());
}

#[test]
fn set_property_and_semantic_set() {
    let subtree_class = make_subtree_class();
    let mut subtree_metadata = ImplicitSubtreeMetadata::new(ImplicitSubtreeMetadataOptions {
        class: Some(subtree_class),
        subtree_metadata: Some(json!({
            "properties": {
                "author": "None"
            }
        })),
    })
    .unwrap();

    assert!(subtree_metadata.set_property(Some("author"), Some(json!("Cesium"))).unwrap());
    assert_eq!(
        subtree_metadata.get_property(Some("author")).unwrap(),
        Some(json!("Cesium"))
    );
    assert!(!subtree_metadata.set_property(Some("unknown"), Some(json!("val"))).unwrap());
    assert!(subtree_metadata.set_property(None, Some(json!("val"))).is_err());
    assert!(subtree_metadata.set_property(Some("author"), None).is_err());

    assert_eq!(
        subtree_metadata.get_property_by_semantic(Some("AUTHOR")).unwrap(),
        Some(json!("Cesium"))
    );
    assert!(subtree_metadata.get_property_by_semantic(Some("CREDITS")).unwrap().is_none());
    assert!(subtree_metadata.get_property_by_semantic(None).is_err());

    assert!(subtree_metadata
        .set_property_by_semantic(Some("AUTHOR"), Some(json!("Updated Cesium")))
        .unwrap());
    assert_eq!(
        subtree_metadata.get_property(Some("author")).unwrap(),
        Some(json!("Updated Cesium"))
    );
    assert!(!subtree_metadata
        .set_property_by_semantic(Some("UNKNOWN"), Some(json!("val")))
        .unwrap());
    assert!(subtree_metadata.set_property_by_semantic(None, Some(json!("val"))).is_err());
    assert!(subtree_metadata.set_property_by_semantic(Some("AUTHOR"), None).is_err());
}
