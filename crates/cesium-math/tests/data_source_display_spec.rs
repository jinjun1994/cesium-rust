//! Direct port and verification against CesiumJS `DataSourceDisplaySpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, CustomDataSource,
    DataSourceCollection, DataSourceDisplay, DataSourceDisplayOptions,
    DataSourceDisplayScene, Entity, JulianDate, Visualizer,
};

#[derive(Debug)]
struct MockVisualizer {
    updates_called: AtomicUsize,
    last_update_time: RefCell<Option<JulianDate>>,
    destroyed: AtomicBool,
    bounding_sphere_result: RefCell<Option<BoundingSphere>>,
    bounding_sphere_state: RefCell<BoundingSphereState>,
    update_return_value: AtomicBool,
}

impl MockVisualizer {
    fn new() -> Self {
        Self {
            updates_called: AtomicUsize::new(0),
            last_update_time: RefCell::new(None),
            destroyed: AtomicBool::new(false),
            bounding_sphere_result: RefCell::new(None),
            bounding_sphere_state: RefCell::new(BoundingSphereState::Done),
            update_return_value: AtomicBool::new(true),
        }
    }
}

impl Visualizer for MockVisualizer {
    fn update(&mut self, time: &JulianDate) -> bool {
        *self.last_update_time.borrow_mut() = Some(*time);
        self.updates_called.fetch_add(1, Ordering::SeqCst);
        self.update_return_value.load(Ordering::SeqCst)
    }

    fn get_bounding_sphere(
        &self,
        _entity: &Entity,
        result: &mut BoundingSphere,
    ) -> BoundingSphereState {
        if let Some(res) = *self.bounding_sphere_result.borrow() {
            *result = res;
        }
        *self.bounding_sphere_state.borrow()
    }

    fn is_destroyed(&self) -> bool {
        self.destroyed.load(Ordering::SeqCst)
    }

    fn destroy(&mut self) {
        self.destroyed.store(true, Ordering::SeqCst);
    }
}

#[test]
fn constructor_sets_expected_values() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: None,
    }))
    .unwrap();

    assert_eq!(display.data_sources().borrow().len(), 0);
    assert!(!display.is_destroyed());
    assert_eq!(display.default_data_source().name(), "default");

    display.destroy();
    assert!(display.is_destroyed());
}

#[test]
fn computes_complete_bounding_sphere() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let vis1_sphere = BoundingSphere::new(Cartesian3::new(1.0, 2.0, 3.0), 456.0);
    let vis2_sphere = BoundingSphere::new(Cartesian3::new(7.0, 8.0, 9.0), 1011.0);

    let callback = Arc::new(move |_scene: &DataSourceDisplayScene, _cluster: Option<&_>, _ds: &_| {
        let v1 = MockVisualizer::new();
        *v1.bounding_sphere_result.borrow_mut() = Some(vis1_sphere);
        *v1.bounding_sphere_state.borrow_mut() = BoundingSphereState::Done;

        let v2 = MockVisualizer::new();
        *v2.bounding_sphere_result.borrow_mut() = Some(vis2_sphere);
        *v2.bounding_sphere_state.borrow_mut() = BoundingSphereState::Done;

        vec![
            Box::new(v1) as Box<dyn Visualizer>,
            Box::new(v2) as Box<dyn Visualizer>,
        ]
    });

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: Some(callback),
    }))
    .unwrap();

    let entity = Entity::new();
    let mut ds = CustomDataSource::new("mock");
    let _ = ds.entities_mut().add(entity.clone());
    data_source_collection.borrow_mut().add(ds);

    let time = JulianDate::now();
    assert!(display.update(Some(time)).unwrap());

    let mut result = BoundingSphere::default();
    let state = display
        .get_bounding_sphere(Some(&entity), Some(true), Some(&mut result))
        .unwrap();

    let expected = vis1_sphere.union(vis2_sphere);
    assert_eq!(state, BoundingSphereState::Done);
    assert_eq!(result, expected);

    display.destroy();
}

#[test]
fn should_not_return_pending_when_ready_is_false_and_allow_partial_is_true() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let vis1_sphere = BoundingSphere::new(Cartesian3::new(1.0, 2.0, 3.0), 456.0);
    let vis2_sphere = BoundingSphere::new(Cartesian3::new(7.0, 8.0, 9.0), 1011.0);

    let callback = Arc::new(move |_scene: &DataSourceDisplayScene, _cluster: Option<&_>, _ds: &_| {
        let v1 = MockVisualizer::new();
        *v1.bounding_sphere_result.borrow_mut() = Some(vis1_sphere);
        *v1.bounding_sphere_state.borrow_mut() = BoundingSphereState::Pending;

        let v2 = MockVisualizer::new();
        *v2.bounding_sphere_result.borrow_mut() = Some(vis2_sphere);
        *v2.bounding_sphere_state.borrow_mut() = BoundingSphereState::Done;

        vec![
            Box::new(v1) as Box<dyn Visualizer>,
            Box::new(v2) as Box<dyn Visualizer>,
        ]
    });

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: Some(callback),
    }))
    .unwrap();

    let entity = Entity::new();
    let mut ds = CustomDataSource::new("mock");
    let _ = ds.entities_mut().add(entity.clone());
    data_source_collection.borrow_mut().add(ds);

    let time = JulianDate::now();
    display.update(Some(time)).unwrap();

    let mut result = BoundingSphere::default();
    let state = display
        .get_bounding_sphere(Some(&entity), Some(true), Some(&mut result))
        .unwrap();

    assert_eq!(state, BoundingSphereState::Done);
    assert_eq!(result, vis2_sphere);

    display.destroy();
}

#[test]
fn fails_complete_bounding_sphere_if_allow_partial_false() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let callback = Arc::new(move |_scene: &DataSourceDisplayScene, _cluster: Option<&_>, _ds: &_| {
        let v1 = MockVisualizer::new();
        *v1.bounding_sphere_result.borrow_mut() =
            Some(BoundingSphere::new(Cartesian3::new(1.0, 2.0, 3.0), 456.0));
        *v1.bounding_sphere_state.borrow_mut() = BoundingSphereState::Pending;

        let v2 = MockVisualizer::new();
        *v2.bounding_sphere_result.borrow_mut() =
            Some(BoundingSphere::new(Cartesian3::new(7.0, 8.0, 9.0), 1011.0));
        *v2.bounding_sphere_state.borrow_mut() = BoundingSphereState::Done;

        vec![
            Box::new(v1) as Box<dyn Visualizer>,
            Box::new(v2) as Box<dyn Visualizer>,
        ]
    });

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: Some(callback),
    }))
    .unwrap();

    let entity = Entity::new();
    let _ = display.default_data_source_mut().entities_mut().add(entity.clone());

    let time = JulianDate::now();
    display.update(Some(time)).unwrap();

    let mut result = BoundingSphere::default();
    let state = display
        .get_bounding_sphere(Some(&entity), Some(false), Some(&mut result))
        .unwrap();

    assert_eq!(state, BoundingSphereState::Pending);

    display.destroy();
}

#[test]
fn fails_bounding_sphere_for_entity_without_visualization() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let callback = Arc::new(|_scene: &DataSourceDisplayScene, _cluster: Option<&_>, _ds: &_| {
        vec![]
    });

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: Some(callback),
    }))
    .unwrap();

    let entity = Entity::new();
    let mut ds = CustomDataSource::new("mock");
    let _ = ds.entities_mut().add(entity.clone());
    data_source_collection.borrow_mut().add(ds);

    let time = JulianDate::now();
    display.update(Some(time)).unwrap();

    let mut result = BoundingSphere::default();
    let state = display
        .get_bounding_sphere(Some(&entity), Some(false), Some(&mut result))
        .unwrap();
    assert_eq!(state, BoundingSphereState::Failed);

    display.destroy();
}

#[test]
fn compute_bounding_sphere_throws_on_missing_args() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(data_source_collection),
        visualizers_callback: None,
    }))
    .unwrap();

    let entity = Entity::new();
    let mut result = BoundingSphere::default();

    assert!(display.get_bounding_sphere(None, Some(false), Some(&mut result)).is_err());
    assert!(display.get_bounding_sphere(Some(&entity), None, Some(&mut result)).is_err());
    assert!(display.get_bounding_sphere(Some(&entity), Some(false), None).is_err());
}

#[test]
fn destroy_does_not_destroy_underlying_data_sources() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let ds = CustomDataSource::new("mock");
    data_source_collection.borrow_mut().add(ds.clone());

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: None,
    }))
    .unwrap();

    assert!(!display.is_destroyed());
    display.destroy();
    assert!(display.is_destroyed());

    // DataSource is still intact in collection
    assert_eq!(data_source_collection.borrow().len(), 1);
}

#[test]
fn ready_is_true_once_data_sources_are_ready_and_stays_true() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let callback = Arc::new(|_scene: &DataSourceDisplayScene, _cluster: Option<&_>, _ds: &_| {
        let v1 = MockVisualizer::new();
        vec![Box::new(v1) as Box<dyn Visualizer>]
    });

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: Some(callback),
    }))
    .unwrap();

    assert!(!display.ready());

    let ds = CustomDataSource::new("mock");
    data_source_collection.borrow_mut().add(ds);

    let time = JulianDate::now();
    display.update(Some(time)).unwrap();
    assert!(display.ready());

    // Subsequent updates keep ready as true
    display.update(Some(time)).unwrap();
    assert!(display.ready());

    display.destroy();
}

#[test]
fn triggers_rendering_when_data_source_becomes_ready() {
    let mut scene = DataSourceDisplayScene::new();
    scene.request_render_mode = true;
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene.clone()),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: None,
    }))
    .unwrap();

    assert!(!display.ready());
    assert!(!*scene.render_requested.borrow());

    let ds = CustomDataSource::new("mock");
    data_source_collection.borrow_mut().add(ds);

    let time = JulianDate::now();
    display.update(Some(time)).unwrap();
    assert!(display.ready());
    assert!(*scene.render_requested.borrow());

    scene.render_for_specs();
    assert!(!*scene.render_requested.borrow());

    display.update(Some(time)).unwrap();
    assert!(!*scene.render_requested.borrow());

    display.destroy();
}

#[test]
fn constructor_throws_without_required_args() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    assert!(DataSourceDisplay::new(None).is_err());
    assert!(DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: None,
        data_source_collection: Some(data_source_collection.clone()),
        visualizers_callback: None,
    }))
    .is_err());
    assert!(DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: None,
        visualizers_callback: None,
    }))
    .is_err());
}

#[test]
fn update_throws_if_time_undefined() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(data_source_collection),
        visualizers_callback: None,
    }))
    .unwrap();

    assert!(display.update(None).is_err());
}

#[test]
fn sets_and_cleans_up_primitives_on_add_and_remove() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene.clone()),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: None,
    }))
    .unwrap();

    let ds = CustomDataSource::new("source1");
    data_source_collection.borrow_mut().add(ds.clone());

    display.update(Some(JulianDate::now())).unwrap();

    assert!(display.primitives.borrow().contains("source1-primitives"));
    assert!(scene.primitives.borrow().contains("display-primitives"));

    data_source_collection.borrow_mut().remove(&ds);
    display.update(Some(JulianDate::now())).unwrap();

    assert!(!display.primitives.borrow().contains("source1-primitives"));
    // Default data source primitive remains
    assert_eq!(display.primitives.borrow().len(), 1);

    display.destroy();
}

#[test]
fn reorders_primitives_on_raise_and_lower() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene),
        data_source_collection: Some(Rc::clone(&data_source_collection)),
        visualizers_callback: None,
    }))
    .unwrap();

    let source1 = CustomDataSource::new("src1");
    let source2 = CustomDataSource::new("src2");
    let source3 = CustomDataSource::new("src3");

    data_source_collection.borrow_mut().add(source1.clone());
    data_source_collection.borrow_mut().add(source2.clone());
    data_source_collection.borrow_mut().add(source3.clone());

    display.update(Some(JulianDate::now())).unwrap();

    // Index 0: default, 1: src1, 2: src2, 3: src3
    assert_eq!(display.primitives.borrow().get(1), Some("src1-primitives"));
    assert_eq!(display.primitives.borrow().get(2), Some("src2-primitives"));
    assert_eq!(display.primitives.borrow().get(3), Some("src3-primitives"));

    // Raise source1
    data_source_collection.borrow_mut().raise(&source1).unwrap();
    display.update(Some(JulianDate::now())).unwrap();
    assert_eq!(display.primitives.borrow().get(1), Some("src2-primitives"));
    assert_eq!(display.primitives.borrow().get(2), Some("src1-primitives"));
    assert_eq!(display.primitives.borrow().get(3), Some("src3-primitives"));

    // Lower source3
    data_source_collection.borrow_mut().lower(&source3).unwrap();
    display.update(Some(JulianDate::now())).unwrap();
    assert_eq!(display.primitives.borrow().get(1), Some("src2-primitives"));
    assert_eq!(display.primitives.borrow().get(2), Some("src3-primitives"));
    assert_eq!(display.primitives.borrow().get(3), Some("src1-primitives"));

    // RaiseToTop source2
    data_source_collection.borrow_mut().raise_to_top(&source2).unwrap();
    display.update(Some(JulianDate::now())).unwrap();
    assert_eq!(display.primitives.borrow().get(3), Some("src2-primitives"));

    // LowerToBottom source2
    data_source_collection.borrow_mut().lower_to_bottom(&source2).unwrap();
    display.update(Some(JulianDate::now())).unwrap();
    assert_eq!(display.primitives.borrow().get(1), Some("src2-primitives"));

    display.destroy();
}

#[test]
fn adds_primitives_to_scene_when_entities_added_to_default_data_source() {
    let scene = DataSourceDisplayScene::new();
    let data_source_collection = Rc::new(RefCell::new(DataSourceCollection::new()));

    let mut display = DataSourceDisplay::new(Some(DataSourceDisplayOptions {
        scene: Some(scene.clone()),
        data_source_collection: Some(data_source_collection),
        visualizers_callback: None,
    }))
    .unwrap();

    assert!(!scene.primitives.borrow().contains("display-primitives"));

    let _ = display.default_data_source_mut().entities_mut().add(Entity::new());
    display.update(Some(JulianDate::now())).unwrap();

    assert!(scene.primitives.borrow().contains("display-primitives"));
    display.destroy();
}

#[test]
fn has_expected_default_visualizers_and_registration() {
    let scene = DataSourceDisplayScene::new();
    let ds = CustomDataSource::new("mock");

    let visualizers = DataSourceDisplay::default_visualizers_callback(&scene, None, &ds);
    assert_eq!(visualizers.len(), 8);

    // Register extra visualizer
    DataSourceDisplay::register_visualizer("fake", Arc::new(|_| Box::new(MockVisualizer::new())));
    let visualizers2 = DataSourceDisplay::default_visualizers_callback(&scene, None, &ds);
    assert_eq!(visualizers2.len(), 9);

    // Unregister extra visualizer
    DataSourceDisplay::unregister_visualizer("fake");
    let visualizers3 = DataSourceDisplay::default_visualizers_callback(&scene, None, &ds);
    assert_eq!(visualizers3.len(), 8);
}
