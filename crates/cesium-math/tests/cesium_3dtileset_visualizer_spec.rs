use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    BoundingSphere, BoundingSphereState, Cartesian3, Cesium3DTilesetGraphics,
    Cesium3DTilesetVisualizer, ConstantPositionProperty, ConstantProperty, Entity,
    EntityCollection, JulianDate, TilesetVisualizerScene,
};

#[test]
fn test_cesium_3dtileset_visualizer_constructor_throws_if_no_scene() {
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let res = Cesium3DTilesetVisualizer::new(None, Some(col));
    assert!(res.is_err());
}

#[test]
fn test_cesium_3dtileset_visualizer_update_throws_if_no_time() {
    let scene = Rc::new(RefCell::new(TilesetVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = Cesium3DTilesetVisualizer::new(Some(scene), Some(col)).unwrap();
    assert!(vis.update(None).is_err());
}

#[test]
fn test_cesium_3dtileset_visualizer_is_destroyed() {
    let scene = Rc::new(RefCell::new(TilesetVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = Cesium3DTilesetVisualizer::new(Some(scene), Some(col)).unwrap();
    assert!(!vis.is_destroyed());
    vis.destroy();
    assert!(vis.is_destroyed());
}

#[test]
fn test_cesium_3dtileset_visualizer_removes_listener_when_destroyed() {
    let scene = Rc::new(RefCell::new(TilesetVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = Cesium3DTilesetVisualizer::new(Some(scene), Some(Rc::clone(&col))).unwrap();
    assert_eq!(col.borrow().collection_changed.number_of_listeners(), 1);
    vis.destroy();
    assert_eq!(col.borrow().collection_changed.number_of_listeners(), 0);
}

#[test]
fn test_cesium_3dtileset_visualizer_object_with_no_tileset_does_not_create_one() {
    let scene = Rc::new(RefCell::new(TilesetVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = Cesium3DTilesetVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&col))).unwrap();

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
fn test_cesium_3dtileset_visualizer_position_and_model_matrix() {
    let scene = Rc::new(RefCell::new(TilesetVisualizerScene::new()));
    let col = Rc::new(RefCell::new(EntityCollection::new()));
    let mut vis = Cesium3DTilesetVisualizer::new(Some(Rc::clone(&scene)), Some(Rc::clone(&col))).unwrap();

    let mut entity = Entity::with_id("test");
    let mut tg = Cesium3DTilesetGraphics::new();
    tg.uri = Some(Rc::new(ConstantProperty::new(Some("http://test.com/tileset.json".to_string()))));
    tg.maximum_screen_space_error = Some(Rc::new(ConstantProperty::new(Some(32.0))));
    entity.tileset = Some(tg);
    col.borrow_mut().add(entity).unwrap();

    vis.update(Some(JulianDate::now())).unwrap();
    assert_eq!(scene.borrow().primitives.borrow().len(), 1);
    assert_eq!(scene.borrow().primitives.borrow()[0].maximum_screen_space_error, 32.0);
    assert!(scene.borrow().primitives.borrow()[0].model_matrix.is_none());

    // Now set position
    let ent = {
        let mut col_mut = col.borrow_mut();
        let ent_mut = col_mut.get_by_id_mut("test").unwrap();
        ent_mut.position = Some(ConstantPositionProperty::new(
            Some(Cartesian3::new(10.0, 20.0, 30.0)),
            None,
        ));
        ent_mut.clone()
    };

    vis.update(Some(JulianDate::now())).unwrap();
    assert!(scene.borrow().primitives.borrow()[0].model_matrix.is_some());

    let mut bs = BoundingSphere::default();
    let state = vis.get_bounding_sphere(Some(&ent), Some(&mut bs)).unwrap();
    assert_eq!(state, BoundingSphereState::Done);
}
