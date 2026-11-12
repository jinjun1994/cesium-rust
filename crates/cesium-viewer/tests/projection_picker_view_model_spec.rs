use cesium_viewer::widgets::{
    FrustumType, ProjectionPickerScene, ProjectionPickerViewModel, SceneMode,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn can_construct_and_destroy() {
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let mut view_model = ProjectionPickerViewModel::new(Some(Rc::clone(&scene))).unwrap();

    assert!(Rc::ptr_eq(&view_model.scene(), &scene));
    assert_eq!(scene.borrow().morph_complete.number_of_listeners(), 1);
    assert_eq!(scene.borrow().pre_render.number_of_listeners(), 1);
    assert!(!view_model.is_destroyed());

    view_model.destroy();
    assert!(view_model.is_destroyed());
    assert_eq!(scene.borrow().morph_complete.number_of_listeners(), 0);
    assert_eq!(scene.borrow().pre_render.number_of_listeners(), 0);
}

#[test]
fn drop_down_visible_and_toggle_drop_down_work() {
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let mut view_model = ProjectionPickerViewModel::new(Some(scene)).unwrap();

    assert!(!view_model.drop_down_visible());
    view_model.toggle_drop_down();
    assert!(view_model.drop_down_visible());
    view_model.set_drop_down_visible(false);
    assert!(!view_model.drop_down_visible());

    view_model.destroy();
}

#[test]
fn morphing_to_2d_calls_correct_transition() {
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let mut view_model = ProjectionPickerViewModel::new(Some(Rc::clone(&scene))).unwrap();

    assert_eq!(scene.borrow().mode, SceneMode::Scene3D);
    assert!(!view_model.is_orthographic_projection());

    scene.borrow_mut().morph_to_2d(0.0);
    assert_eq!(scene.borrow().mode, SceneMode::Scene2D);
    assert!(view_model.is_orthographic_projection());

    view_model.destroy();
}

#[test]
fn switching_projection_calls_correct_transition() {
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let mut view_model = ProjectionPickerViewModel::new(Some(Rc::clone(&scene))).unwrap();

    assert_eq!(scene.borrow().mode, SceneMode::Scene3D);
    assert!(!view_model.is_orthographic_projection());
    assert_eq!(scene.borrow().camera.frustum, FrustumType::Perspective);

    view_model.switch_to_orthographic();
    assert!(view_model.is_orthographic_projection());
    assert_eq!(scene.borrow().camera.frustum, FrustumType::Orthographic);

    view_model.switch_to_perspective();
    assert!(!view_model.is_orthographic_projection());
    assert_eq!(scene.borrow().camera.frustum, FrustumType::Perspective);

    view_model.destroy();
}

#[test]
fn selected_tooltip_changes_on_transition() {
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let mut view_model = ProjectionPickerViewModel::new(Some(scene)).unwrap();

    view_model.switch_to_orthographic();
    assert_eq!(view_model.selected_tooltip(), view_model.tooltip_orthographic);

    view_model.switch_to_perspective();
    assert_eq!(view_model.selected_tooltip(), view_model.tooltip_perspective);

    view_model.destroy();
}

#[test]
fn create_throws_with_undefined_scene() {
    let result = ProjectionPickerViewModel::new(None);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "scene is required.");
}
