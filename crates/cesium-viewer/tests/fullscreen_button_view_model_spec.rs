use cesium_viewer::widgets::FullscreenButtonViewModel;

#[test]
fn constructor_sets_default_values() {
    let vm = FullscreenButtonViewModel::default();
    assert_eq!(vm.fullscreen_element(), "document.body");
    assert!(!vm.is_destroyed());
    assert!(vm.is_fullscreen_enabled());
    assert!(!vm.is_fullscreen());
    assert_eq!(vm.tooltip(), "Full screen");

    vm.destroy();
    assert!(vm.is_destroyed());
}

#[test]
fn constructor_sets_expected_values() {
    let vm = FullscreenButtonViewModel::new(Some("spanElement"));
    assert_eq!(vm.fullscreen_element(), "spanElement");
    vm.destroy();
}

#[test]
fn is_fullscreen_enabled_works_as_expected() {
    let vm = FullscreenButtonViewModel::default();
    assert!(vm.is_fullscreen_enabled());
    assert!(vm.command().can_execute());

    vm.set_is_fullscreen_enabled(false);
    assert!(!vm.is_fullscreen_enabled());
    assert!(!vm.command().can_execute());
    assert_eq!(vm.tooltip(), "Full screen unavailable");

    vm.set_is_fullscreen_enabled(true);
    assert!(vm.is_fullscreen_enabled());
    assert!(vm.command().can_execute());
}

#[test]
fn can_get_and_set_fullscreen_element() {
    let vm = FullscreenButtonViewModel::default();
    assert_eq!(vm.fullscreen_element(), "document.body");

    vm.set_fullscreen_element("canvas3d").unwrap();
    assert_eq!(vm.fullscreen_element(), "canvas3d");

    assert!(vm.set_fullscreen_element("").is_err());
    assert!(vm.set_fullscreen_element("   ").is_err());
}

#[test]
fn command_toggles_fullscreen_state_and_tooltip() {
    let vm = FullscreenButtonViewModel::default();
    assert!(!vm.is_fullscreen());
    assert_eq!(vm.tooltip(), "Full screen");

    vm.command().call().unwrap();
    assert!(vm.is_fullscreen());
    assert_eq!(vm.tooltip(), "Exit full screen");

    vm.command().call().unwrap();
    assert!(!vm.is_fullscreen());
    assert_eq!(vm.tooltip(), "Full screen");
}
