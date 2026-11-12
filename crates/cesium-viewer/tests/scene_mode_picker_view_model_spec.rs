use cesium_viewer::widgets::{SceneMode, SceneModePickerScene, SceneModePickerViewModel};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn can_construct_and_destroy() {
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let mut view_model =
        SceneModePickerViewModel::new(Some(Rc::clone(&scene)), Some(1.0)).unwrap();

    assert!(Rc::ptr_eq(&view_model.scene(), &scene));
    assert_eq!(view_model.duration(), 1.0);
    assert_eq!(scene.borrow().morph_start.number_of_listeners(), 1);
    assert!(!view_model.is_destroyed());

    view_model.destroy();
    assert!(view_model.is_destroyed());
    assert_eq!(scene.borrow().morph_start.number_of_listeners(), 0);
}

#[test]
fn drop_down_visible_and_toggle_drop_down_work() {
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let mut view_model = SceneModePickerViewModel::new(Some(scene), None).unwrap();

    assert!(!view_model.drop_down_visible());
    view_model.toggle_drop_down();
    assert!(view_model.drop_down_visible());
    view_model.set_drop_down_visible(false);
    assert!(!view_model.drop_down_visible());

    view_model.destroy();
}

#[test]
fn morphing_closes_the_drop_down() {
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let mut view_model = SceneModePickerViewModel::new(Some(scene), None).unwrap();

    view_model.set_drop_down_visible(true);
    view_model.morph_to_columbus_view();
    assert!(!view_model.drop_down_visible());

    view_model.set_drop_down_visible(true);
    view_model.morph_to_3d();
    assert!(!view_model.drop_down_visible());

    view_model.set_drop_down_visible(true);
    view_model.morph_to_2d();
    assert!(!view_model.drop_down_visible());

    view_model.destroy();
}

#[test]
fn morphing_calls_correct_transition() {
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let mut view_model = SceneModePickerViewModel::new(Some(Rc::clone(&scene)), None).unwrap();

    assert_eq!(scene.borrow().mode, SceneMode::Scene3D);

    view_model.morph_to_columbus_view();
    scene.borrow_mut().complete_morph();
    assert_eq!(scene.borrow().mode, SceneMode::ColumbusView);

    view_model.morph_to_3d();
    scene.borrow_mut().complete_morph();
    assert_eq!(scene.borrow().mode, SceneMode::Scene3D);

    view_model.morph_to_2d();
    scene.borrow_mut().complete_morph();
    assert_eq!(scene.borrow().mode, SceneMode::Scene2D);

    view_model.destroy();
}

#[test]
fn selected_tooltip_changes_on_transition() {
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let mut view_model = SceneModePickerViewModel::new(Some(scene), None).unwrap();

    view_model.morph_to_columbus_view();
    assert_eq!(
        view_model.selected_tooltip(),
        view_model.tooltip_columbus_view
    );

    view_model.morph_to_3d();
    assert_eq!(view_model.selected_tooltip(), view_model.tooltip_3d);

    view_model.morph_to_2d();
    assert_eq!(view_model.selected_tooltip(), view_model.tooltip_2d);

    view_model.destroy();
}

#[test]
fn create_throws_with_undefined_scene() {
    let result = SceneModePickerViewModel::new(None, None);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "scene is required.");
}
