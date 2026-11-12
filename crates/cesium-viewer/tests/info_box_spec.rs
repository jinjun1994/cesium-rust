use cesium_viewer::widgets::{InfoBox, InfoBoxContainer};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn constructor_sets_expected_values() {
    let test_container = Rc::new(RefCell::new(InfoBoxContainer::new("testContainer")));
    let mut info_box = InfoBox::new(Some(Rc::clone(&test_container))).unwrap();

    assert_eq!(
        info_box.container().unwrap().borrow().id,
        test_container.borrow().id
    );
    assert!(!info_box.is_destroyed());
    info_box.destroy();
    assert!(info_box.is_destroyed());
}

#[test]
fn can_set_description_body() {
    let test_container = Rc::new(RefCell::new(InfoBoxContainer::new("testContainer")));
    let mut info_box = InfoBox::new(Some(Rc::clone(&test_container))).unwrap();

    let info_element = test_container.borrow().first_child().unwrap();

    info_box.set_description("Please do not crash");
    let node = info_box
        .frame()
        .borrow()
        .content_document
        .first_child()
        .unwrap();

    assert_eq!(
        info_element
            .borrow()
            .style
            .get("background-color")
            .map(|s| s.as_str())
            .unwrap_or(""),
        ""
    );
    assert_eq!(node.borrow().inner_html, info_box.view_model().description);

    info_box.set_description(
        "<div style=\"background-color: rgb(255, 255, 255);\">Please do not crash</div>",
    );
    assert_eq!(
        info_element
            .borrow()
            .style
            .get("background-color")
            .map(|s| s.as_str()),
        Some("rgb(255, 255, 255)")
    );
    assert_eq!(node.borrow().inner_html, info_box.view_model().description);
}

#[test]
fn constructor_works_with_string_id_container() {
    let test_container = Rc::new(RefCell::new(InfoBoxContainer::new("testContainer")));
    let registry = [Rc::clone(&test_container)];

    let info_box = InfoBox::from_id("testContainer", |id| {
        registry.iter().find(|c| c.borrow().id == id).cloned()
    })
    .unwrap();

    assert_eq!(
        info_box.container().unwrap().borrow().id,
        test_container.borrow().id
    );
}

#[test]
fn throws_if_container_is_undefined() {
    let result = InfoBox::new(None);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "container is required.");
}

#[test]
fn throws_if_container_string_is_undefined() {
    let result = InfoBox::from_id("foo", |_| None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "Element with id \"foo\" does not exist in the document."
    );
}
