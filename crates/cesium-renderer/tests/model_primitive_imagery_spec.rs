use std::sync::{Arc, RwLock};

use cesium_math::{Cartesian3, Cartesian4, Ellipsoid, Matrix4};
use cesium_renderer::axis::Axis;
use cesium_renderer::imagery_pipeline_stage::ImageryCoverage;
use cesium_renderer::model_primitive_imagery::ModelPrimitiveImagery;
use cesium_renderer::model_utility::{AttributeType, ModelAttributeDescription};
use cesium_renderer::{ImageryLayer, ImageryLayerCollection, ImageryLayerOptions};

#[test]
fn test_compute_unique_ellipsoids_throws_with_undefined() {
    assert!(ModelPrimitiveImagery::compute_unique_ellipsoids(None).is_err());
}

#[test]
fn test_compute_unique_ellipsoids() {
    let mut collection = ImageryLayerCollection::new();

    let layer0 = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            ellipsoid: Some(Ellipsoid::WGS84),
            ..Default::default()
        }),
    )));
    let layer1 = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
            ..Default::default()
        }),
    )));
    let layer2 = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            ellipsoid: Some(Ellipsoid::UNIT_SPHERE),
            ..Default::default()
        }),
    )));
    let layer3 = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            ellipsoid: Some(Ellipsoid::MOON),
            ..Default::default()
        }),
    )));

    collection.add(layer0, None).unwrap();
    collection.add(layer1, None).unwrap();
    collection.add(layer2, None).unwrap();
    collection.add(layer3, None).unwrap();

    let expected = [Ellipsoid::WGS84, Ellipsoid::UNIT_SPHERE, Ellipsoid::MOON];
    let actual = ModelPrimitiveImagery::compute_unique_ellipsoids(Some(&collection)).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_extract_projections_throws_with_undefined() {
    assert!(ModelPrimitiveImagery::extract_projections_checked(None).is_err());
}

#[test]
fn test_extract_projections() {
    let mut collection = ImageryLayerCollection::new();
    for _ in 0..4 {
        collection
            .add(Arc::new(RwLock::new(ImageryLayer::new(None, None))), None)
            .unwrap();
    }
    let actual = ModelPrimitiveImagery::extract_projections(&collection);
    assert_eq!(actual.len(), 4);
    for proj in actual {
        assert_eq!(proj, "WebMercator");
    }
}

#[test]
fn test_compute_primitive_position_transform_throws_without_model() {
    let runtime_computed = Matrix4::IDENTITY;
    assert!(ModelPrimitiveImagery::compute_primitive_position_transform(
        None,
        None,
        None,
        Some(&runtime_computed)
    )
    .is_err());
}

#[test]
fn test_compute_primitive_position_transform_throws_without_runtime_node() {
    let model_matrix = Matrix4::IDENTITY;
    assert!(ModelPrimitiveImagery::compute_primitive_position_transform(
        Some(&model_matrix),
        None,
        None,
        None
    )
    .is_err());
}

#[test]
fn test_compute_primitive_position_transform_computes_transform() {
    let model_matrix = Matrix4::IDENTITY;
    let components_transform = Matrix4::IDENTITY;
    let axis_correction_matrix = Axis::Y_UP_TO_Z_UP.multiply(&Axis::Z_UP_TO_X_UP);
    let runtime_node_computed_transform = Matrix4::IDENTITY;

    let actual = ModelPrimitiveImagery::compute_primitive_position_transform(
        Some(&model_matrix),
        Some(&components_transform),
        Some(&axis_correction_matrix),
        Some(&runtime_node_computed_transform),
    )
    .unwrap();

    assert_eq!(actual, axis_correction_matrix);
}

#[test]
fn test_obtain_primitive_position_attribute_throws_without_primitive() {
    assert!(ModelPrimitiveImagery::obtain_primitive_position_attribute(None).is_err());
}

#[test]
fn test_obtain_primitive_position_attribute_throws_for_primitive_without_position() {
    let attrs = [ModelAttributeDescription {
        name: None,
        semantic: Some("NORMAL".to_string()),
        set_index: None,
        attribute_type: AttributeType::Vec3,
        min: None,
        max: None,
        quantization: None,
    }];
    assert!(ModelPrimitiveImagery::obtain_primitive_position_attribute(Some(&attrs)).is_err());
}

#[test]
fn test_obtain_primitive_position_attribute_obtains_position() {
    let attrs = [
        ModelAttributeDescription {
            name: None,
            semantic: Some("POSITION".to_string()),
            set_index: None,
            attribute_type: AttributeType::Vec3,
            min: Some(Cartesian3::new(-0.5, 0.0, 0.0)),
            max: Some(Cartesian3::new(0.5, 2.0, 0.0)),
            quantization: None,
        },
        ModelAttributeDescription {
            name: None,
            semantic: Some("NORMAL".to_string()),
            set_index: None,
            attribute_type: AttributeType::Vec3,
            min: None,
            max: None,
            quantization: None,
        },
    ];
    let pos = ModelPrimitiveImagery::obtain_primitive_position_attribute(Some(&attrs)).unwrap();
    assert_eq!(pos.semantic.as_deref(), Some("POSITION"));
    assert_eq!(pos, attrs[0]);
}

#[test]
fn test_properly_reports_mapped_positions_need_update() {
    let mut mpi = ModelPrimitiveImagery::new();
    let mat = Matrix4::IDENTITY;
    assert!(mpi.mapped_positions_need_update(&mat));

    mpi.mapped_positions_per_ellipsoid = Some(Vec::new());
    mpi.mapped_positions_model_matrix = mat;

    assert!(!mpi.mapped_positions_need_update(&mat));

    let new_mat = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    assert!(mpi.mapped_positions_need_update(&new_mat));
}

#[test]
fn test_coverages_for_imagery_layer() {
    let mut mpi = ModelPrimitiveImagery::new();
    assert!(mpi.coverages_for_imagery_layer_by_index(0, 1).is_err());

    let cov = ImageryCoverage::new(0, 0, 0, Cartesian4::ZERO);
    mpi.coverages_per_layer = Some(vec![vec![cov]]);

    assert!(mpi.coverages_for_imagery_layer_by_index(0, 1).is_ok());
    assert!(mpi.coverages_for_imagery_layer_by_index(1, 1).is_err());
}

#[test]
fn test_lifecycle_and_destroy() {
    let mut mpi = ModelPrimitiveImagery::new();
    mpi.mapped_positions_per_ellipsoid = Some(Vec::new());
    assert!(!mpi.is_destroyed());

    mpi.destroy();
    assert!(mpi.is_destroyed());
    assert!(mpi.mapped_positions_per_ellipsoid.is_none());
}

#[test]
fn test_create_cartographic_positions_validation() {
    let dummy_positions = [0.0f32, 0.0, 0.0];
    let transform = Matrix4::IDENTITY;
    let ell = Ellipsoid::WGS84;

    assert!(ModelPrimitiveImagery::create_cartographic_positions(None, Some(&transform), Some(ell)).is_err());
    assert!(ModelPrimitiveImagery::create_cartographic_positions(Some(&dummy_positions), None, Some(ell)).is_err());
    assert!(ModelPrimitiveImagery::create_cartographic_positions(Some(&dummy_positions), Some(&transform), None).is_err());

    let cartographics = ModelPrimitiveImagery::create_cartographic_positions(
        Some(&dummy_positions),
        Some(&transform),
        Some(ell),
    )
    .unwrap();
    assert_eq!(cartographics.len(), 1);
}
