use cesium_viewer::widgets::{
    SceneModePicker, SceneModePickerContainer, SceneModePickerElement, SceneModePickerScene,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn can_create_and_destroy() {
    let container = Rc::new(RefCell::new(SceneModePickerContainer::new(
        "testContainer",
    )));
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let registry = [Rc::clone(&container)];

    let mut widget = SceneModePicker::from_id("testContainer", Some(scene), None, |id| {
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
    let container = Rc::new(RefCell::new(SceneModePickerContainer::new(
        "testContainer",
    )));
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));

    let mut widget =
        SceneModePicker::new(Some(Rc::clone(&container)), Some(scene), None).unwrap();

    let outside_node = SceneModePickerElement::new("body");

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
fn constructor_throws_with_no_transitioner() {
    let container = Rc::new(RefCell::new(SceneModePickerContainer::new("body")));
    let result = SceneModePicker::new(Some(container), None, None);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "scene is required.");
}

#[test]
fn constructor_throws_with_no_element() {
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let result = SceneModePicker::new(None, Some(scene), None);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "container is required.");
}

#[test]
fn constructor_throws_with_string_element_that_does_not_exist() {
    let scene = Rc::new(RefCell::new(SceneModePickerScene::new()));
    let result = SceneModePicker::from_id("does not exist", Some(scene), None, |_| None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "Element with id \"does not exist\" does not exist in the document."
    );
}
