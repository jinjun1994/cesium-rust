use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, ConstantPositionProperty,
    ConstantProperty, Entity, EntityCollection, JulianDate, PathGraphics,
    PathVisualizer, PathVisualizerScene,
};

#[test]
fn test_path_visualizer_constructor_throws_if_no_scene() {
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let res = PathVisualizer::new(None, Some(col));
    assert!(res.is_err());
}

#[test]
fn test_path_visualizer_update_throws_if_no_time() {
    let scene = Rc::new(RefCell::new(PathVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = PathVisualizer::new(Some(scene), Some(col)).unwrap();
    assert!(vis.update(None).is_err());
}

#[test]
fn test_path_visualizer_is_destroyed() {
    let scene = Rc::new(RefCell::new(PathVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = PathVisualizer::new(Some(scene), Some(col)).unwrap();
    assert!(!vis.is_destroyed());
    vis.destroy();
    assert!(vis.is_destroyed());
}

#[test]
fn test_path_visualizer_removes_listener_when_destroyed() {
    let scene = Rc::new(RefCell::new(PathVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = PathVisualizer::new(Some(scene), Some(Rc::clone(&col))).unwrap();
    assert_eq!(col.borrow().collection_changed.number_of_listeners(), 1);
    vis.destroy();
    assert_eq!(col.borrow().collection_changed.number_of_listeners(), 0);
}

#[test]
fn test_path_visualizer_object_with_no_path_does_not_create_primitive() {
    let scene = Rc::new(RefCell::new(PathVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = PathVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&col))).unwrap();

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
fn test_path_visualizer_creates_and_updates_primitive() {
    let scene = Rc::new(RefCell::new(PathVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = PathVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&col))).unwrap();

    let mut entity = Entity::with_id("test");
    entity.position = Some(ConstantPositionProperty::new(
        Some(Cartesian3::new(10.0, 20.0, 30.0)),
        None,
    ));
    let mut pg = PathGraphics::default();
    pg.width = Some(Rc::new(ConstantProperty::new(Some(5.0))));
    pg.lead_time = Some(Rc::new(ConstantProperty::new(Some(10.0))));
    pg.trail_time = Some(Rc::new(ConstantProperty::new(Some(10.0))));
    pg.resolution = Some(Rc::new(ConstantProperty::new(Some(5.0))));
    entity.path = Some(pg);
    col.borrow_mut().add(entity).unwrap();

    vis.update(Some(JulianDate::now())).unwrap();
    assert_eq!(scene.borrow().primitives.borrow().len(), 1);

    let binding = scene.borrow();
    let prims = binding.primitives.borrow();
    let prim = &prims[0];
    assert!(prim.show);
    assert_eq!(prim.width, 5.0);
    assert!(!prim.positions.is_empty());

    let ent = col.borrow().get_by_id("test").unwrap().clone();
    let mut bs = BoundingSphere::default();
    let state = vis.get_bounding_sphere(Some(&ent), Some(&mut bs)).unwrap();
    assert_eq!(state, BoundingSphereState::Done);
}
