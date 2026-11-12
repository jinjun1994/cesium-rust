use cesium_viewer::widgets::selection_indicator::SelectionIndicator;
use cesium_viewer::widgets::selection_indicator_view_model::{
    SelectionIndicatorContainer, SelectionIndicatorScene,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_selection_indicator_constructor_sets_expected_values() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let container = Rc::new(RefCell::new(SelectionIndicatorContainer::new("root")));

    let mut indicator = SelectionIndicator::new(Some(Rc::clone(&container)), Some(Rc::clone(&scene)))
        .expect("Constructor should succeed");

    assert!(indicator.container().is_some());
    assert!(!indicator.is_destroyed());

    indicator.destroy();
    assert!(indicator.is_destroyed());
    assert!(indicator.container().is_none());
}

#[test]
fn test_selection_indicator_constructor_works_with_string_id_container() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let element = Rc::new(RefCell::new(SelectionIndicatorContainer::new("testElement")));

    let mut elements_map = HashMap::new();
    elements_map.insert("testElement".to_string(), Rc::clone(&element));

    let mut indicator = SelectionIndicator::from_id("testElement", Some(Rc::clone(&scene)), |id| {
        elements_map.get(id).cloned()
    })
    .expect("from_id should succeed");

    assert!(indicator.container().is_some());
    indicator.destroy();
    assert!(indicator.is_destroyed());
}

#[test]
fn test_selection_indicator_throws_if_container_is_undefined() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let result = SelectionIndicator::new(None, Some(scene));
    assert!(result.is_err());
}

#[test]
fn test_selection_indicator_throws_if_container_string_is_undefined() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let result = SelectionIndicator::from_id("testElement", Some(scene), |_| None);
    assert!(result.is_err());
}
