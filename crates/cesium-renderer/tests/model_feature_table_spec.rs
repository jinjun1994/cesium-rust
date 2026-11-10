//! Port of CesiumJS `Scene/Model/ModelFeatureTableSpec.js` (Apache-2.0).

use cesium_renderer::model_feature_table::{
    ModelFeatureKind, ModelFeatureTable, ModelPropertyDefinition, ModelPropertyTable,
    ModelRuntimeTarget,
};
use cesium_renderer::model_type::ModelType;
use serde_json::json;

fn create_mock_property_table() -> ModelPropertyTable {
    let mut pt = ModelPropertyTable::new(2);
    pt.add_property(
        "height",
        ModelPropertyDefinition {
            semantic: Some("HEIGHT_SEMANTIC".to_string()),
            property_type: Some("SCALAR".to_string()),
            component_type: Some("FLOAT32".to_string()),
        },
        vec![json!(1.0), json!(2.0)],
    );
    pt.add_property(
        "name",
        ModelPropertyDefinition {
            semantic: None,
            property_type: Some("STRING".to_string()),
            component_type: None,
        },
        vec![json!("A"), json!("B")],
    );
    pt.add_property(
        "HEIGHT_SEMANTIC",
        ModelPropertyDefinition {
            semantic: None,
            property_type: Some("SCALAR".to_string()),
            component_type: Some("FLOAT32".to_string()),
        },
        vec![json!(3.0), json!(4.0)],
    );
    pt
}

#[test]
fn creates_model_features_when_model_does_not_have_content() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    assert_eq!(table.features_length(), 2);
    for i in 0..table.features_length() {
        let feature = table.get_feature(i).unwrap();
        assert_eq!(feature.kind(), ModelFeatureKind::ModelFeature);
        assert!(feature.is_model_feature());
    }
}

#[test]
fn creates_cesium_3d_tile_feature_when_model_has_content() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::TileGltf,
        has_content: true,
    };
    let table = ModelFeatureTable::new(model, pt);

    assert_eq!(table.features_length(), 2);
    for i in 0..table.features_length() {
        let feature = table.get_feature(i).unwrap();
        assert_eq!(feature.kind(), ModelFeatureKind::Cesium3DTileFeature);
        assert!(feature.is_cesium_3d_tile_feature());
    }
}

#[test]
fn has_property_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    for feature in table.features() {
        assert!(feature.has_property("height"));
        assert!(!feature.has_property("width"));
    }
}

#[test]
fn has_property_by_semantic_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    for i in 0..table.features_length() {
        assert!(table.has_property_by_semantic(i, "HEIGHT_SEMANTIC"));
        assert!(!table.has_property_by_semantic(i, "WIDTH_SEMANTIC"));
    }
}

#[test]
fn get_feature_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    assert_eq!(table.features_length(), 2);
    for i in 0..table.features_length() {
        let feature = table.get_feature(i).unwrap();
        assert_eq!(feature, table.features()[i]);
        assert_eq!(feature.kind(), ModelFeatureKind::ModelFeature);
    }
}

#[test]
fn get_property_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    let expected_heights = [json!(1.0), json!(2.0)];
    let expected_names = [json!("A"), json!("B")];
    let expected_semantics = [json!(3.0), json!(4.0)];

    for (i, feature) in table.features().iter().enumerate() {
        assert_eq!(
            feature.get_property("height"),
            Some(expected_heights[i].clone())
        );
        assert_eq!(
            feature.get_property("name"),
            Some(expected_names[i].clone())
        );
        assert_eq!(
            feature.get_property("HEIGHT_SEMANTIC"),
            Some(expected_semantics[i].clone())
        );
    }
}

#[test]
fn get_property_inherited_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    let expected_heights = [json!(1.0), json!(2.0)];

    for (i, feature) in table.features().iter().enumerate() {
        assert_eq!(
            feature.get_property_inherited("height"),
            Some(expected_heights[i].clone())
        );
        assert_eq!(feature.get_property_inherited("_height"), None);
    }

    // Check if the semantic is prioritized over the property name.
    for (i, feature) in table.features().iter().enumerate() {
        assert_eq!(
            feature.get_property_inherited("HEIGHT_SEMANTIC"),
            Some(expected_heights[i].clone())
        );
        assert_eq!(feature.get_property_inherited("_HEIGHT_"), None);
    }
}

#[test]
fn get_property_ids_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    for feature in table.features() {
        let mut results = Vec::new();
        assert_eq!(
            feature.get_property_ids(&mut results),
            vec!["height", "name", "HEIGHT_SEMANTIC"]
        );
    }
}

#[test]
fn set_property_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let table = ModelFeatureTable::new(model, pt);

    let feature = &table.features()[0];
    assert_eq!(feature.get_property("height"), Some(json!(1.0)));
    assert!(feature.set_property("height", 3.0));
    assert_eq!(feature.get_property("height"), Some(json!(3.0)));
}

#[test]
fn destroy_works() {
    let pt = create_mock_property_table();
    let model = ModelRuntimeTarget {
        model_type: ModelType::Gltf,
        has_content: false,
    };
    let mut table = ModelFeatureTable::new(model, pt);

    let batch_texture = table.batch_texture();
    assert!(!batch_texture.is_destroyed());
    assert!(!table.is_destroyed());

    assert!(table.destroy().is_ok());

    assert!(table.batch_texture().is_destroyed());
    assert!(table.is_destroyed());
}
