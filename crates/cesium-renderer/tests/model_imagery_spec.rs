use std::sync::{Arc, RwLock};

use cesium_math::Ellipsoid;
use cesium_renderer::{ImageryLayer, ModelImagery, ModelPrimitiveImagery};

#[test]
fn test_constructor_throws_without_model() {
    let result = ModelImagery::new(false, 4);
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(err_str.contains("model is required"));
}

#[test]
fn test_properly_reports_has_imagery() {
    let mut model_imagery = ModelImagery::new(true, 4).expect("valid creation");
    assert!(!model_imagery.has_imagery());

    let layer = Arc::new(RwLock::new(ImageryLayer::new(None, None)));
    model_imagery.add_layer(layer);

    // Expect imagery to be present
    assert!(model_imagery.has_imagery());

    // Clear the set of imagery layers
    model_imagery.remove_all_layers();

    // Now there is no imagery again
    assert!(!model_imagery.has_imagery());
}

#[test]
fn test_properly_reports_all_imagery_layers_ready() {
    let mut model_imagery = ModelImagery::new(true, 4).expect("valid creation");
    let layer = Arc::new(RwLock::new(ImageryLayer::new(None, None)));
    layer.write().unwrap().set_ready(true);
    model_imagery.add_layer(layer.clone());

    // All imagery layers should be ready now
    assert!(model_imagery.all_imagery_layers_ready());

    // Setting layer ready to false
    layer.write().unwrap().set_ready(false);

    // Now, it should report the imagery layers to not be ready
    assert!(!model_imagery.all_imagery_layers_ready());
}

#[test]
fn test_properly_handles_modifications_of_imagery_configurations() {
    let mut model_imagery = ModelImagery::new(true, 4).expect("valid creation");
    let layer = Arc::new(RwLock::new(ImageryLayer::new(None, None)));
    layer.write().unwrap().set_ready(true);
    model_imagery.add_layer(layer.clone());

    model_imagery.update();

    // Initially, _imageryConfigurationsModified is false (it was just updated)
    assert!(!model_imagery.imagery_configurations_modified());

    // Modify imagery configuration alpha
    layer.write().unwrap().set_alpha(0.5);

    // Now, _imageryConfigurationsModified is true
    assert!(model_imagery.imagery_configurations_modified());

    // Trigger an update
    model_imagery.check_for_modified_imagery_configurations();

    // Now, _imageryConfigurationsModified is false again
    assert!(!model_imagery.imagery_configurations_modified());
}

#[test]
fn test_considers_the_show_flag_as_part_of_the_imagery_configurations() {
    let mut model_imagery = ModelImagery::new(true, 4).expect("valid creation");
    let layer = Arc::new(RwLock::new(ImageryLayer::new(None, None)));
    layer.write().unwrap().set_ready(true);
    model_imagery.add_layer(layer.clone());

    model_imagery.update();

    // Initially, _imageryConfigurationsModified is false (it was just updated)
    assert!(!model_imagery.imagery_configurations_modified());

    // Modify imagery configuration show
    layer.write().unwrap().set_show(false);

    // Now, _imageryConfigurationsModified is true
    assert!(model_imagery.imagery_configurations_modified());

    // Trigger an update
    model_imagery.check_for_modified_imagery_configurations();

    // Now, _imageryConfigurationsModified is false again
    assert!(!model_imagery.imagery_configurations_modified());
}

#[test]
fn test_creates_one_model_primitive_imagery_for_each_primitive() {
    let mut model_imagery = ModelImagery::new(true, 4).expect("valid creation");
    let layer = Arc::new(RwLock::new(ImageryLayer::new(None, None)));
    layer.write().unwrap().set_ready(true);
    model_imagery.add_layer(layer);

    model_imagery.update();

    let model_primitive_imageries = model_imagery.model_primitive_imageries().unwrap();
    assert_eq!(model_primitive_imageries.len(), 4);
    for (i, prim) in model_primitive_imageries.iter().enumerate() {
        assert_eq!(prim.runtime_node_index, 0);
        assert_eq!(prim.runtime_primitive_index, i);
        assert!(prim.ready);
        assert!(!prim.is_destroyed());
    }
}

#[test]
fn test_removes_model_primitive_imagery_objects_when_imagery_layers_are_removed() {
    let mut model_imagery = ModelImagery::new(true, 4).expect("valid creation");
    let layer = Arc::new(RwLock::new(ImageryLayer::new(None, None)));
    layer.write().unwrap().set_ready(true);
    model_imagery.add_layer(layer);

    model_imagery.update();

    assert_eq!(model_imagery.model_primitive_imageries().unwrap().len(), 4);

    // Remove the imagery layer from the model, and trigger an update
    model_imagery.remove_all_layers();
    model_imagery.update();

    // The model imagery should no longer contain any modelPrimitiveImagery objects now
    assert!(model_imagery.model_primitive_imageries().is_none());
}

#[test]
fn test_compute_unique_ellipsoids() {
    assert!(ModelPrimitiveImagery::compute_unique_ellipsoids_from_slice(None).is_err());

    let wgs84 = Ellipsoid::WGS84;
    let unit = Ellipsoid::UNIT_SPHERE;
    let list = [wgs84, unit, wgs84, unit, wgs84];

    let unique = ModelPrimitiveImagery::compute_unique_ellipsoids_from_slice(Some(&list)).unwrap();
    assert_eq!(unique.len(), 2);
    assert_eq!(unique[0], wgs84);
    assert_eq!(unique[1], unit);
}

#[test]
fn test_model_imagery_lifecycle_and_destroy() {
    let mut model_imagery = ModelImagery::new(true, 2).expect("valid creation");
    let layer = Arc::new(RwLock::new(ImageryLayer::new(None, None)));
    layer.write().unwrap().set_ready(true);
    model_imagery.add_layer(layer);
    model_imagery.update();

    assert!(model_imagery.ready());
    assert!(!model_imagery.is_destroyed());

    model_imagery.destroy();
    assert!(model_imagery.is_destroyed());
    assert!(model_imagery.model_primitive_imageries().is_none());
}
