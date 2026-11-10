//! Port of CesiumJS `Scene/Model/ModelFeatureSpec.js` (Apache-2.0).

use cesium_math::Color;
use cesium_renderer::model_feature_table::{
    ModelFeature, ModelFeatureKind, ModelFeatureTable, ModelPropertyDefinition, ModelPropertyTable,
    ModelRuntimeTarget,
};
use cesium_renderer::model_type::ModelType;
use serde_json::json;

fn create_test_feature_table() -> ModelFeatureTable {
    let mock_model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };

    let mut property_table = ModelPropertyTable::new(1);
    property_table.add_property(
        "height",
        ModelPropertyDefinition {
            semantic: Some("HEIGHT_SEMANTIC".to_string()),
            property_type: Some("SCALAR".to_string()),
            component_type: Some("FLOAT32".to_string()),
        },
        vec![json!(1.0)],
    );
    property_table.add_property(
        "name",
        ModelPropertyDefinition {
            semantic: None,
            property_type: Some("STRING".to_string()),
            component_type: None,
        },
        vec![json!("A")],
    );
    property_table.add_property(
        "HEIGHT_SEMANTIC",
        ModelPropertyDefinition {
            semantic: None,
            property_type: Some("SCALAR".to_string()),
            component_type: Some("FLOAT32".to_string()),
        },
        vec![json!(3.0)],
    );

    ModelFeatureTable::new(mock_model, property_table)
}

#[test]
fn constructs() {
    let mock_model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let feature_table = create_test_feature_table();
    let feature = ModelFeature::new(
        mock_model.clone(),
        3,
        feature_table.get_feature(0).unwrap().feature_table(),
        ModelFeatureKind::ModelFeature,
    );

    assert_eq!(feature.primitive(), &mock_model);
    assert_eq!(feature.feature_id(), 3);
}

#[test]
fn gets_and_sets_show() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert!(feature.show());

    feature.set_show(false);
    assert!(!feature.show());
}

#[test]
fn gets_and_sets_color() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert_eq!(feature.color(), Color::WHITE);

    feature.set_color(Color::RED);
    assert_eq!(feature.color(), Color::RED);
}

#[test]
fn gets_primitive() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert_eq!(
        feature.primitive(),
        &ModelRuntimeTarget {
            model_type: ModelType::Gltf,
            has_content: false,
        }
    );
}

#[test]
fn gets_feature_id() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert_eq!(feature.feature_id(), 0);
}

#[test]
fn has_property_works() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert!(feature.has_property("height"));
    assert!(!feature.has_property("width"));
}

#[test]
fn get_property_works() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert_eq!(feature.get_property("height"), Some(json!(1.0)));
    assert_eq!(feature.get_property("name"), Some(json!("A")));
    assert_eq!(feature.get_property("HEIGHT_SEMANTIC"), Some(json!(3.0)));
}

#[test]
fn get_property_inherited_works() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert_eq!(feature.get_property_inherited("height"), Some(json!(1.0)));
    assert_eq!(feature.get_property_inherited("_height"), None);
}

#[test]
fn get_property_ids_works() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    let mut results = Vec::new();
    assert_eq!(
        feature.get_property_ids(&mut results),
        vec!["height", "name", "HEIGHT_SEMANTIC"]
    );
}

#[test]
fn set_property_works() {
    let feature_table = create_test_feature_table();
    let feature = feature_table.get_feature(0).unwrap();

    assert_eq!(feature.get_property("height"), Some(json!(1.0)));
    assert!(feature.set_property("height", 3.0));
    assert_eq!(feature.get_property("height"), Some(json!(3.0)));
}
