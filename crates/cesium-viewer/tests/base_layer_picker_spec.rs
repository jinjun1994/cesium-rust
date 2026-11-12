use std::cell::RefCell;
use std::rc::Rc;

use cesium_viewer::widgets::{
    BaseLayerPicker, BaseLayerPickerContainer, BaseLayerPickerViewModelOptions, StandaloneGlobe,
};

#[test]
fn can_create_and_destroy() {
    let container = BaseLayerPickerContainer::new("testContainer", true);
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));

    let widget: BaseLayerPicker = BaseLayerPicker::new(
        Some(container.clone()),
        Some(BaseLayerPickerViewModelOptions {
            globe: Some(Rc::clone(&globe)),
            ..Default::default()
        }),
    )
    .unwrap();

    assert_eq!(widget.container().id, "testContainer");
    assert!(Rc::ptr_eq(&widget.view_model().globe(), &globe));
    assert!(!widget.is_destroyed());

    widget.destroy();
    assert!(widget.is_destroyed());
}

#[test]
fn pointer_down_event_closes_dropdown_if_target_is_not_inside_container() {
    let container = BaseLayerPickerContainer::new("testContainer", true);
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));

    let widget: BaseLayerPicker = BaseLayerPicker::new(
        Some(container),
        Some(BaseLayerPickerViewModelOptions {
            globe: Some(globe),
            ..Default::default()
        }),
    )
    .unwrap();

    // Open dropdown
    widget.view_model().set_drop_down_visible(true);
    assert!(widget.view_model().drop_down_visible());

    // Click outside on document.body -> closes
    widget.handle_pointer_down("document.body");
    assert!(!widget.view_model().drop_down_visible());

    // Open dropdown again
    widget.view_model().set_drop_down_visible(true);
    assert!(widget.view_model().drop_down_visible());

    // Click inside container or container child -> stays open
    widget.handle_pointer_down("testContainer");
    assert!(widget.view_model().drop_down_visible());

    widget.handle_pointer_down("testContainer_child_btn");
    assert!(widget.view_model().drop_down_visible());

    widget.destroy();
}

#[test]
fn constructor_throws_with_no_options() {
    let container = BaseLayerPickerContainer::new("testContainer", true);
    let err = BaseLayerPicker::<StandaloneGlobe, ()>::new(Some(container), None).unwrap_err();
    assert!(err.message.contains("options are required."));
}

#[test]
fn constructor_throws_with_no_element() {
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));
    let options = BaseLayerPickerViewModelOptions {
        globe: Some(globe),
        ..Default::default()
    };

    let err = BaseLayerPicker::<StandaloneGlobe, ()>::new(None, Some(options)).unwrap_err();
    assert!(err.message.contains("container is required."));
}

#[test]
fn constructor_throws_with_string_element_that_does_not_exist() {
    let container = BaseLayerPickerContainer::new("does not exist", false);
    let globe = Rc::new(RefCell::new(StandaloneGlobe::new()));
    let options = BaseLayerPickerViewModelOptions {
        globe: Some(globe),
        ..Default::default()
    };

    let err = BaseLayerPicker::<StandaloneGlobe, ()>::new(Some(container), Some(options)).unwrap_err();
    assert!(err.message.contains("does not exist"));
}
