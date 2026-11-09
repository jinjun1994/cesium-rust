use std::sync::{Arc, RwLock};
use cesium_math::Ray;
use glam::DVec3;
use cesium_renderer::imagery_layer::{
    ImageryLayer, ImageryLayerFeatureInfo, ImageryProvider,
};
use cesium_renderer::imagery_layer_collection::{
    ImageryLayerCollection, PickImageryFeaturesOptions,
};

struct FakeProvider {
    features: Option<Vec<ImageryLayerFeatureInfo>>,
}

impl FakeProvider {
    fn new() -> Self {
        Self { features: None }
    }

    fn with_feature(name: &str, desc: &str) -> Self {
        Self {
            features: Some(vec![ImageryLayerFeatureInfo {
                name: Some(name.to_string()),
                description: Some(desc.to_string()),
                position: None,
                data: None,
            }]),
        }
    }
}

impl ImageryProvider for FakeProvider {
    fn pick_features(
        &self,
        _x: u32,
        _y: u32,
        _level: u32,
        _longitude: f64,
        _latitude: f64,
    ) -> Option<Vec<ImageryLayerFeatureInfo>> {
        self.features.clone()
    }
}

fn create_layer() -> Arc<RwLock<ImageryLayer>> {
    Arc::new(RwLock::new(ImageryLayer::new(
        Some(Arc::new(FakeProvider::new())),
        None,
    )))
}

#[test]
fn tracks_the_base_layer_on_add() {
    let layer1 = create_layer();
    let layer2 = create_layer();
    let layer3 = create_layer();
    let mut collection = ImageryLayerCollection::new();

    assert!(!layer1.read().unwrap().is_base_layer());

    collection.add(layer1.clone(), None).unwrap();
    assert!(layer1.read().unwrap().is_base_layer());

    collection.add(layer2.clone(), None).unwrap();
    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());

    collection.add(layer3.clone(), Some(0)).unwrap();
    assert!(!layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
    assert!(layer3.read().unwrap().is_base_layer());
}

#[test]
fn tracks_the_base_layer_on_remove() {
    let layer1 = create_layer();
    let layer2 = create_layer();
    let layer3 = create_layer();
    let mut collection = ImageryLayerCollection::new();

    collection.add(layer1.clone(), None).unwrap();
    collection.add(layer2.clone(), None).unwrap();
    collection.add(layer3.clone(), None).unwrap();

    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());

    collection.remove(&layer1, false);
    assert!(layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());

    collection.remove(&layer3, false);
    assert!(layer2.read().unwrap().is_base_layer());
}

#[test]
fn updates_is_base_layer_on_re_add() {
    let layer1 = create_layer();
    let layer2 = create_layer();
    let mut collection = ImageryLayerCollection::new();

    layer1.write().unwrap().set_is_base_layer(true);
    layer2.write().unwrap().set_is_base_layer(true);

    collection.add(layer1.clone(), None).unwrap();
    collection.add(layer2.clone(), None).unwrap();

    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
}

#[test]
fn does_not_crash_when_raising_and_lowering_a_single_layer() {
    let layer1 = create_layer();
    let mut collection = ImageryLayerCollection::new();
    collection.add(layer1.clone(), None).unwrap();

    collection.raise(&layer1).unwrap();
    collection.lower(&layer1).unwrap();
    collection.raise_to_top(&layer1).unwrap();
    collection.lower_to_bottom(&layer1).unwrap();
}

#[test]
fn tracks_the_base_layer_on_raise_and_lower() {
    let layer1 = create_layer();
    let layer2 = create_layer();
    let layer3 = create_layer();
    let mut collection = ImageryLayerCollection::new();

    collection.add(layer1.clone(), None).unwrap();
    collection.add(layer2.clone(), None).unwrap();
    collection.add(layer3.clone(), None).unwrap();

    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());

    collection.lower(&layer1).unwrap();
    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());

    collection.raise(&layer1).unwrap();
    assert!(!layer1.read().unwrap().is_base_layer());
    assert!(layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());

    collection.lower(&layer1).unwrap();
    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());
}

#[test]
fn tracks_the_base_layer_on_raise_to_top_and_lower_to_bottom() {
    let layer1 = create_layer();
    let layer2 = create_layer();
    let layer3 = create_layer();
    let mut collection = ImageryLayerCollection::new();

    collection.add(layer1.clone(), None).unwrap();
    collection.add(layer2.clone(), None).unwrap();
    collection.add(layer3.clone(), None).unwrap();

    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());

    collection.raise_to_top(&layer1).unwrap();
    assert!(!layer1.read().unwrap().is_base_layer());
    assert!(layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());

    collection.lower_to_bottom(&layer1).unwrap();
    assert!(layer1.read().unwrap().is_base_layer());
    assert!(!layer2.read().unwrap().is_base_layer());
    assert!(!layer3.read().unwrap().is_base_layer());
}

#[test]
fn add_throws_when_index_is_outside_valid_range() {
    let mut collection = ImageryLayerCollection::new();
    let layer1 = create_layer();
    let layer2 = create_layer();

    assert!(collection.add(layer1.clone(), Some(1)).is_err());
    collection.add(layer1, Some(0)).unwrap();

    assert!(collection.add(layer2.clone(), Some(2)).is_err());
    collection.add(layer2, Some(0)).unwrap();
}

#[test]
fn remove_ignores_unregistered_layer() {
    let mut collection = ImageryLayerCollection::new();
    let layer1 = create_layer();
    assert!(!collection.remove(&layer1, false));
}

#[test]
fn contains_works_as_expected() {
    let mut collection = ImageryLayerCollection::new();
    let layer1 = create_layer();
    let layer2 = create_layer();

    assert!(!collection.contains(&layer1));
    assert!(!collection.contains(&layer2));

    collection.add(layer1.clone(), None).unwrap();
    assert!(collection.contains(&layer1));
    assert!(!collection.contains(&layer2));

    collection.add(layer2.clone(), None).unwrap();
    assert!(collection.contains(&layer1));
    assert!(collection.contains(&layer2));

    collection.remove(&layer1, false);
    assert!(!collection.contains(&layer1));
    assert!(collection.contains(&layer2));

    collection.remove(&layer2, false);
    assert!(!collection.contains(&layer1));
    assert!(!collection.contains(&layer2));
}

#[test]
fn get_and_raise_error_handling() {
    let mut collection = ImageryLayerCollection::new();
    assert!(collection.get(0).is_err());

    let layer1 = create_layer();
    assert!(collection.raise(&layer1).is_err());

    collection.add(layer1.clone(), None).unwrap();
    assert_eq!(collection.get(0).unwrap().read().unwrap().id(), layer1.read().unwrap().id());
}

#[test]
fn reports_whether_or_not_it_is_destroyed() {
    let mut collection = ImageryLayerCollection::new();
    assert!(!collection.is_destroyed());
    collection.destroy();
    assert!(collection.is_destroyed());
}

#[test]
fn pick_imagery_layers_behaviour() {
    let mut collection = ImageryLayerCollection::new();
    let ray = Ray::new(DVec3::ZERO, DVec3::Z).unwrap();

    // No intersection
    assert!(collection.pick_imagery_layers(&ray, false).is_none());

    // Intersecting but empty
    assert!(collection.pick_imagery_layers(&ray, true).is_none());

    // One layer
    let p1 = Arc::new(FakeProvider::new());
    let l1 = collection.add_imagery_provider(p1, None).unwrap();
    let picked1 = collection.pick_imagery_layers(&ray, true).unwrap();
    assert_eq!(picked1.len(), 1);
    assert_eq!(picked1[0].read().unwrap().id(), l1.read().unwrap().id());

    // Two layers (reversed order: top to bottom)
    let p2 = Arc::new(FakeProvider::new());
    let l2 = collection.add_imagery_provider(p2, None).unwrap();
    let picked2 = collection.pick_imagery_layers(&ray, true).unwrap();
    assert_eq!(picked2.len(), 2);
    assert_eq!(picked2[0].read().unwrap().id(), l2.read().unwrap().id());
    assert_eq!(picked2[1].read().unwrap().id(), l1.read().unwrap().id());
}

#[test]
fn pick_imagery_layer_features_behaviour() {
    let mut collection = ImageryLayerCollection::new();
    let ray = Ray::new(DVec3::ZERO, DVec3::Z).unwrap();

    let opts_no_intersect = PickImageryFeaturesOptions {
        surface_intersects: false,
        x: 0,
        y: 0,
        level: 0,
        longitude: 0.0,
        latitude: 0.0,
    };
    assert!(collection.pick_imagery_layer_features(&ray, opts_no_intersect).is_none());

    let p1 = Arc::new(FakeProvider::with_feature("Feature1", "Desc1"));
    let p2 = Arc::new(FakeProvider::with_feature("Feature2", "Desc2"));

    collection.add_imagery_provider(p1, None).unwrap();
    collection.add_imagery_provider(p2, None).unwrap();

    let opts_intersect = PickImageryFeaturesOptions {
        surface_intersects: true,
        x: 0,
        y: 0,
        level: 0,
        longitude: 0.0,
        latitude: 0.0,
    };
    let features = collection
        .pick_imagery_layer_features(&ray, opts_intersect)
        .expect("features should be picked");
    assert_eq!(features.len(), 2);
    assert_eq!(features[0].name.as_deref(), Some("Feature2"));
    assert_eq!(features[1].name.as_deref(), Some("Feature1"));
}
