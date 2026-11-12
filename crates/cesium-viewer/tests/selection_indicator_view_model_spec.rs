use cesium_math::{Cartesian2, Cartesian3};
use cesium_viewer::widgets::selection_indicator_view_model::{
    SelectionIndicatorContainer, SelectionIndicatorElement, SelectionIndicatorScene,
    SelectionIndicatorViewModel, OFF_SCREEN_COORDINATE,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_selection_indicator_view_model_constructor_sets_expected_values() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let element = Rc::new(RefCell::new(SelectionIndicatorElement::new(20.0, 20.0)));
    let container = Rc::new(RefCell::new(SelectionIndicatorContainer::new("container")));

    let vm = SelectionIndicatorViewModel::new(
        Some(Rc::clone(&scene)),
        Some(Rc::clone(&element)),
        Some(Rc::clone(&container)),
    )
    .expect("Should succeed");

    assert_eq!(vm.scale(), 1.0);
    assert!(!vm.is_visible());
    assert_eq!(vm.screen_position_x(), OFF_SCREEN_COORDINATE);
    assert_eq!(vm.screen_position_y(), OFF_SCREEN_COORDINATE);
}

#[test]
fn test_selection_indicator_view_model_throws_if_params_undefined() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let element = Rc::new(RefCell::new(SelectionIndicatorElement::new(20.0, 20.0)));
    let container = Rc::new(RefCell::new(SelectionIndicatorContainer::new("container")));

    assert!(SelectionIndicatorViewModel::new(None, Some(Rc::clone(&element)), Some(Rc::clone(&container))).is_err());
    assert!(SelectionIndicatorViewModel::new(Some(Rc::clone(&scene)), None, Some(Rc::clone(&container))).is_err());
    assert!(SelectionIndicatorViewModel::new(Some(Rc::clone(&scene)), Some(Rc::clone(&element)), None).is_err());
}

#[test]
fn test_selection_indicator_view_model_animate_selection_element() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let element = Rc::new(RefCell::new(SelectionIndicatorElement::new(20.0, 20.0)));
    let container = Rc::new(RefCell::new(SelectionIndicatorContainer::new("container")));

    let mut vm = SelectionIndicatorViewModel::new(
        Some(scene),
        Some(element),
        Some(container),
    )
    .unwrap();

    vm.animate_appear();
    assert_eq!(vm.scale(), 1.0);
    vm.animate_depart();
    assert_eq!(vm.scale(), 1.5);
}

#[test]
fn test_selection_indicator_view_model_custom_screen_space_positions() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let element = Rc::new(RefCell::new(SelectionIndicatorElement::new(20.0, 20.0)));
    let container = Rc::new(RefCell::new(SelectionIndicatorContainer::new("container")));

    let mut vm = SelectionIndicatorViewModel::new(
        Some(scene),
        Some(element),
        Some(container),
    )
    .unwrap();

    vm.show_selection = true;
    vm.position = Some(Cartesian3::new(1.0, 2.0, 3.0));
    vm.set_compute_screen_space_position(|pos| Some(Cartesian2::new(pos.x, pos.y)));

    vm.update();
    assert_eq!(vm.screen_position_x(), "-9px");
    assert_eq!(vm.screen_position_y(), "-8px");
}

#[test]
fn test_selection_indicator_view_model_hides_when_position_unknown() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let element = Rc::new(RefCell::new(SelectionIndicatorElement::new(20.0, 20.0)));
    let container = Rc::new(RefCell::new(SelectionIndicatorContainer::new("container")));

    let mut vm = SelectionIndicatorViewModel::new(
        Some(scene),
        Some(element),
        Some(container),
    )
    .unwrap();

    assert!(!vm.is_visible());
    vm.show_selection = true;
    assert!(!vm.is_visible());
    vm.position = Some(Cartesian3::new(1.0, 2.0, 3.0));
    assert!(vm.is_visible());
    vm.show_selection = false;
    assert!(!vm.is_visible());
}

#[test]
fn test_selection_indicator_view_model_move_indicator_off_screen() {
    let scene = Rc::new(RefCell::new(SelectionIndicatorScene::new()));
    let element = Rc::new(RefCell::new(SelectionIndicatorElement::new(20.0, 20.0)));
    let container = Rc::new(RefCell::new(SelectionIndicatorContainer::new("container")));

    let mut vm = SelectionIndicatorViewModel::new(
        Some(scene),
        Some(element),
        Some(container),
    )
    .unwrap();

    vm.show_selection = true;
    vm.position = Some(Cartesian3::new(1.0, 2.0, 3.0));
    vm.set_compute_screen_space_position(|_| None);

    vm.update();
    assert_eq!(vm.screen_position_x(), OFF_SCREEN_COORDINATE);
    assert_eq!(vm.screen_position_y(), OFF_SCREEN_COORDINATE);
}
