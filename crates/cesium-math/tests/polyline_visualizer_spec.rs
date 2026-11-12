use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, ColorMaterialProperty,
    ConstantProperty, Entity, EntityCollection, JulianDate, PolylineGraphics,
    PolylineVisualizer, PolylineVisualizerScene,
};

#[test]
fn test_polyline_visualizer_can_create_and_destroy() {
    let scene = Rc::new(RefCell::new(PolylineVisualizerScene::new()));
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = PolylineVisualizer::new(Some(Rc::clone(&scene)), Some(objects)).unwrap();
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.borrow().primitives.borrow().len(), 0);
    assert!(!visualizer.is_destroyed());
    visualizer.destroy();
    assert!(visualizer.is_destroyed());
}

#[test]
fn test_polyline_visualizer_throws_if_no_scene() {
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let res = PolylineVisualizer::new(None, Some(objects));
    assert!(res.is_err());
}

#[test]
fn test_polyline_visualizer_update_throws_if_no_time() {
    let scene = Rc::new(RefCell::new(PolylineVisualizerScene::new()));
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let mut visualizer = PolylineVisualizer::new(Some(scene), Some(objects)).unwrap();
    assert!(visualizer.update(None).is_err());
}

#[test]
fn test_polyline_visualizer_creates_and_removes_static_polyline() {
    let scene = Rc::new(RefCell::new(PolylineVisualizerScene::new()));
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = PolylineVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&objects))).unwrap();

    let mut polyline = PolylineGraphics::default();
    polyline.positions = Some(Rc::new(ConstantProperty::new(Some(vec![
        Cartesian3::new(0.0, 0.0, 0.0),
        Cartesian3::new(10.0, 20.0, 30.0),
    ]))));
    polyline.material = Some(Rc::new(ColorMaterialProperty::default()));

    let mut entity = Entity::with_id("test");
    entity.polyline = Some(polyline);
    objects.borrow_mut().add(entity.clone()).unwrap();

    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.borrow().primitives.borrow().len(), 1);

    {
        let prim_binding = scene.borrow();
        let prims = prim_binding.primitives.borrow();
        let prim = &prims[0];
        let attrs = prim.get_geometry_instance_attributes(&entity);
        assert!(attrs.is_some());
        assert!(attrs.unwrap().contains_key("show"));
    }

    let mut bs = BoundingSphere::default();
    let state = visualizer.get_bounding_sphere(Some(&entity), Some(&mut bs)).unwrap();
    assert_eq!(state, BoundingSphereState::Done);

    // Remove polyline
    objects.borrow_mut().remove_by_id("test");
    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.borrow().primitives.borrow().len(), 0);
}

#[test]
fn test_polyline_visualizer_ground_polyline() {
    let scene = Rc::new(RefCell::new(PolylineVisualizerScene::new()));
    let objects = Rc::new(RefCell::new(EntityCollection::new()));
    let time = JulianDate::now();

    let mut visualizer = PolylineVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&objects))).unwrap();

    let mut polyline = PolylineGraphics::default();
    polyline.positions = Some(Rc::new(ConstantProperty::new(Some(vec![
        Cartesian3::new(10.0, 20.0, 0.0),
        Cartesian3::new(10.1, 20.1, 0.0),
    ]))));
    polyline.clamp_to_ground = Some(Rc::new(ConstantProperty::new(Some(true))));

    let mut entity = Entity::with_id("ground_test");
    entity.polyline = Some(polyline);
    objects.borrow_mut().add(entity).unwrap();

    assert!(visualizer.update(Some(time)).unwrap());
    assert_eq!(scene.borrow().primitives.borrow().len(), 0);
    assert_eq!(scene.borrow().ground_primitives.borrow().len(), 1);
    assert!(scene.borrow().ground_primitives.borrow()[0].clamp_to_ground);
}
