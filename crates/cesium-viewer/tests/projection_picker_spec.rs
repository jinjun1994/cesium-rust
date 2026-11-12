use cesium_viewer::widgets::{
    ProjectionPicker, ProjectionPickerContainer, ProjectionPickerElement, ProjectionPickerScene,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn can_create_and_destroy() {
    let container = Rc::new(RefCell::new(ProjectionPickerContainer::new(
        "testContainer",
    )));
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let registry = [Rc::clone(&container)];

    let mut widget = ProjectionPicker::from_id("testContainer", Some(scene), |id| {
        registry.iter().find(|c| c.borrow().id == id).cloned()
    })
    .unwrap();

    assert_eq!(
        widget.container().unwrap().borrow().id,
        container.borrow().id
    );
    assert!(!widget.is_destroyed());

    widget.destroy();
    assert!(widget.is_destroyed());
}

#[test]
fn close_on_input_event_closes_dropdown_if_target_is_not_inside_container() {
    let container = Rc::new(RefCell::new(ProjectionPickerContainer::new(
        "testContainer",
    )));
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));

    let mut widget =
        ProjectionPicker::new(Some(Rc::clone(&container)), Some(scene)).unwrap();

    let outside_node = ProjectionPickerElement::new("body");

    widget.view_model_mut().set_drop_down_visible(true);
    widget.close_on_input(&outside_node);
    assert!(!widget.view_model().drop_down_visible());

    widget.view_model_mut().set_drop_down_visible(true);
    let inside_node = container.borrow().first_child().unwrap();
    widget.close_on_input(&inside_node);
    assert!(widget.view_model().drop_down_visible());

    widget.destroy();
}

#[test]
fn input_event_does_nothing_during_camera_flight() {
    let container = Rc::new(RefCell::new(ProjectionPickerContainer::new(
        "testContainer",
    )));
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));

    let mut widget =
        ProjectionPicker::new(Some(Rc::clone(&container)), Some(Rc::clone(&scene))).unwrap();

    scene.borrow_mut().camera.fly_home(100.0);

    widget.handle_button_click();
    assert!(!widget.view_model().drop_down_visible());

    scene.borrow_mut().camera.cancel_flight();
    widget.destroy();
}

#[test]
fn input_event_does_nothing_in_2d() {
    let container = Rc::new(RefCell::new(ProjectionPickerContainer::new(
        "testContainer",
    )));
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));

    let mut widget =
        ProjectionPicker::new(Some(Rc::clone(&container)), Some(Rc::clone(&scene))).unwrap();

    scene.borrow_mut().morph_to_2d(0.0);

    widget.handle_button_click();
    assert!(!widget.view_model().drop_down_visible());

    widget.destroy();
}

#[test]
fn constructor_throws_with_no_scene() {
    let container = Rc::new(RefCell::new(ProjectionPickerContainer::new("body")));
    let result = ProjectionPicker::new(Some(container), None);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "scene is required.");
}

#[test]
fn constructor_throws_with_no_element() {
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let result = ProjectionPicker::new(None, Some(scene));
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "container is required.");
}

#[test]
fn constructor_throws_with_string_element_that_does_not_exist() {
    let scene = Rc::new(RefCell::new(ProjectionPickerScene::new()));
    let result = ProjectionPicker::from_id("does not exist", Some(scene), |_| None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "Element with id \"does not exist\" does not exist in the document."
    );
}
