use cesium_viewer::widgets::vr_button_view_model::{VRButtonViewModel, VRScene};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_vr_button_view_model_constructor_sets_default_values() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let vm = VRButtonViewModel::new(Some(scene), None::<&str>)
        .expect("Constructor should succeed");

    assert_eq!(vm.vr_element(), "document.body");
    assert!(!vm.is_destroyed());
    vm.destroy();
    assert!(vm.is_destroyed());
}

#[test]
fn test_vr_button_view_model_constructor_sets_expected_values() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let vm = VRButtonViewModel::new(Some(scene), Some("testElement"))
        .expect("Constructor should succeed");

    assert_eq!(vm.vr_element(), "testElement");
    vm.destroy();
}

#[test]
fn test_vr_button_view_model_is_vr_enabled() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let vm = VRButtonViewModel::new(Some(scene), None::<&str>).unwrap();

    assert!(vm.is_vr_enabled());
    vm.set_vr_enabled(false);
    assert!(!vm.is_vr_enabled());
    vm.destroy();
}

#[test]
fn test_vr_button_view_model_can_get_and_set_vr_element() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let vm = VRButtonViewModel::new(Some(scene), None::<&str>).unwrap();

    assert_ne!(vm.vr_element(), "testElement");
    vm.set_vr_element("testElement").unwrap();
    assert_eq!(vm.vr_element(), "testElement");
}

#[test]
fn test_vr_button_view_model_throws_when_constructed_without_scene() {
    let err = VRButtonViewModel::new(None, None::<&str>);
    assert!(err.is_err());
}

#[test]
fn test_vr_button_view_model_throws_if_vr_element_is_invalid() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let vm = VRButtonViewModel::new(Some(scene), None::<&str>).unwrap();

    let err = vm.set_vr_element("");
    assert!(err.is_err());
}

#[test]
fn test_vr_button_view_model_command_toggles_mode() {
    let scene = Rc::new(RefCell::new(VRScene::new()));
    let vm = VRButtonViewModel::new(Some(Rc::clone(&scene)), None::<&str>).unwrap();

    assert!(!vm.is_vr_mode());
    assert!(!scene.borrow().use_web_vr.get());

    assert!(vm.command().execute(()).is_ok());
    assert!(vm.is_vr_mode());
    assert!(scene.borrow().use_web_vr.get());

    assert!(vm.command().execute(()).is_ok());
    assert!(!vm.is_vr_mode());
    assert!(!scene.borrow().use_web_vr.get());
}
