use cesium_viewer::widgets::{
    NavigationHelpButton, NavigationHelpButtonContainer, NavigationHelpButtonOptions,
    NavigationHelpElement,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn can_create_and_destroy() {
    let container = Rc::new(RefCell::new(NavigationHelpButtonContainer::new(
        "testContainer",
    )));
    let registry = [Rc::clone(&container)];

    let mut widget = NavigationHelpButton::from_id("testContainer", None, |id| {
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
fn does_not_show_instructions_by_default() {
    let container = Rc::new(RefCell::new(NavigationHelpButtonContainer::new("body")));
    let mut widget = NavigationHelpButton::new(Some(NavigationHelpButtonOptions {
        container: Some(container),
        instructions_initially_visible: None,
    }))
    .unwrap();

    assert!(!widget.view_model().show_instructions());
    widget.destroy();
}

#[test]
fn shows_instructions_by_default_if_told_to_do_so_in_the_constructor() {
    let container = Rc::new(RefCell::new(NavigationHelpButtonContainer::new("body")));
    let mut widget = NavigationHelpButton::new(Some(NavigationHelpButtonOptions {
        container: Some(container),
        instructions_initially_visible: Some(true),
    }))
    .unwrap();

    assert!(widget.view_model().show_instructions());
    widget.destroy();
}

#[test]
fn close_on_input_event_closes_dropdown_if_target_is_not_inside_container() {
    let container = Rc::new(RefCell::new(NavigationHelpButtonContainer::new(
        "testContainer",
    )));
    let mut widget = NavigationHelpButton::new(Some(NavigationHelpButtonOptions {
        container: Some(Rc::clone(&container)),
        instructions_initially_visible: None,
    }))
    .unwrap();

    let outside_node = NavigationHelpElement::new("body");

    widget.view_model_mut().set_show_instructions(true);
    widget.close_on_input(&outside_node);
    assert!(!widget.view_model().show_instructions());

    widget.view_model_mut().set_show_instructions(true);
    let inside_node = container.borrow().first_child().unwrap();
    widget.close_on_input(&inside_node);
    assert!(widget.view_model().show_instructions());

    widget.destroy();
}

#[test]
fn throws_if_container_is_undefined() {
    let result = NavigationHelpButton::new(Some(NavigationHelpButtonOptions {
        container: None,
        instructions_initially_visible: None,
    }));
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "options.container is required."
    );
}

#[test]
fn throws_if_options_is_undefined() {
    let result = NavigationHelpButton::new(None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "options.container is required."
    );
}

#[test]
fn throws_if_options_container_is_undefined() {
    let result = NavigationHelpButton::new(Some(NavigationHelpButtonOptions::default()));
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "options.container is required."
    );
}

#[test]
fn constructor_throws_with_string_element_that_does_not_exist() {
    let result = NavigationHelpButton::from_id("does not exist", None, |_| None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "Element with id \"does not exist\" does not exist in the document."
    );
}
