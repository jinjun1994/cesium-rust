use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, ConstantPositionProperty,
    ConstantProperty, Entity, EntityCollection, JulianDate, ModelGraphics,
    ModelVisualizer, ModelVisualizerScene,
};

#[test]
fn test_model_visualizer_constructor_throws_if_no_scene() {
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let res = ModelVisualizer::new(None, Some(col));
    assert!(res.is_err());
}

#[test]
fn test_model_visualizer_constructor_throws_if_no_entity_collection() {
    let scene = Rc::new(RefCell::new(ModelVisualizerScene::new()));
    let res = ModelVisualizer::new(Some(scene), None);
    assert!(res.is_err());
}

#[test]
fn test_model_visualizer_update_throws_if_no_time() {
    let scene = Rc::new(RefCell::new(ModelVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = ModelVisualizer::new(Some(scene), Some(col)).unwrap();
    assert!(vis.update(None).is_err());
}

#[test]
fn test_model_visualizer_is_destroyed() {
    let scene = Rc::new(RefCell::new(ModelVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = ModelVisualizer::new(Some(scene), Some(col)).unwrap();
    assert!(!vis.is_destroyed());
    vis.destroy();
    assert!(vis.is_destroyed());
}

#[test]
fn test_model_visualizer_removes_listener_when_destroyed() {
    let scene = Rc::new(RefCell::new(ModelVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = ModelVisualizer::new(Some(scene), Some(Rc::clone(&col))).unwrap();
    assert_eq!(col.borrow().collection_changed.number_of_listeners(), 1);
    vis.destroy();
    assert_eq!(col.borrow().collection_changed.number_of_listeners(), 0);
}

#[test]
fn test_model_visualizer_object_with_no_model_does_not_create_one() {
    let scene = Rc::new(RefCell::new(ModelVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = ModelVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&col))).unwrap();

    let mut entity = Entity::with_id("test");
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(1234.0, 5678.0, 91011.0)),
        None,
    ));
    col.borrow_mut().add(entity).unwrap();

    vis.update(Some(JulianDate::now())).unwrap();
    assert_eq!(scene.borrow().primitives.borrow().len(), 0);
}

#[test]
fn test_model_visualizer_object_with_no_position_does_not_create_a_model() {
    let scene = Rc::new(RefCell::new(ModelVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = ModelVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&col))).unwrap();

    let mut entity = Entity::with_id("test");
    let mut mg = ModelGraphics::default();
    mg.uri = Some(Rc::new(ConstantProperty::new(Some("http://test.com/box.gltf".to_string()))));
    entity.model = Some(mg);
    col.borrow_mut().add(entity).unwrap();

    vis.update(Some(JulianDate::now())).unwrap();
    assert_eq!(scene.borrow().primitives.borrow().len(), 0);
}

#[test]
fn test_model_visualizer_creates_and_updates_primitive() {
    let scene = Rc::new(RefCell::new(ModelVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = ModelVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&col))).unwrap();

    let mut entity = Entity::with_id("test");
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(10.0, 20.0, 30.0)),
        None,
    ));
    let mut mg = ModelGraphics::default();
    mg.uri = Some(Rc::new(ConstantProperty::new(Some("http://test.com/box.gltf".to_string()))));
    mg.scale = Some(Rc::new(ConstantProperty::new(Some(2.0))));
    mg.minimum_pixel_size = Some(Rc::new(ConstantProperty::new(Some(24.0))));
    entity.model = Some(mg);
    col.borrow_mut().add(entity).unwrap();

    vis.update(Some(JulianDate::now())).unwrap();
    assert_eq!(scene.borrow().primitives.borrow().len(), 1);
    let scene_borrow = scene.borrow();
    let prims = scene_borrow.primitives.borrow();
    let prim = &prims[0];
    assert_eq!(prim.scale, 2.0);
    assert_eq!(prim.minimum_pixel_size, 24.0);

    let ent = col.borrow().get_by_id("test").unwrap().clone();
    let mut bs = BoundingSphere::default();
    let state = vis.get_bounding_sphere(Some(&ent), Some(&mut bs)).unwrap();
    assert_eq!(state, BoundingSphereState::Done);
}
