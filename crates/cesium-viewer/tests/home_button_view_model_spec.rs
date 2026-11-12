use cesium_viewer::widgets::{HomeButtonViewModel, SceneMode};

#[test]
fn constructor_sets_default_values() {
    let vm = HomeButtonViewModel::new(Some("scene_01"), None).unwrap();
    assert_eq!(vm.scene_id(), "scene_01");
    assert_eq!(vm.duration(), None);
    assert_eq!(vm.tooltip(), "View Home");
    assert!(!vm.is_destroyed());
    vm.destroy();
    assert!(vm.is_destroyed());
}

#[test]
fn throws_if_scene_is_undefined() {
    assert!(HomeButtonViewModel::new(None::<&str>, None).is_err());
    assert!(HomeButtonViewModel::new(Some(""), None).is_err());
}

#[test]
fn works_in_all_scene_modes() {
    let vm = HomeButtonViewModel::new(Some("scene_01"), Some(2.0)).unwrap();

    // 3D
    vm.set_scene_mode(SceneMode::Scene3D);
    vm.command().call().unwrap();
    assert_eq!(vm.home_flight_count(), 1);

    // 2D
    vm.set_scene_mode(SceneMode::Scene2D);
    vm.command().call().unwrap();
    assert_eq!(vm.home_flight_count(), 2);

    // Columbus View
    vm.set_scene_mode(SceneMode::ColumbusView);
    vm.command().call().unwrap();
    assert_eq!(vm.home_flight_count(), 3);

    // Morphing
    vm.set_scene_mode(SceneMode::Morphing);
    vm.command().call().unwrap();
    assert_eq!(vm.home_flight_count(), 4);
}

#[test]
fn can_get_and_set_duration_with_validation() {
    let vm = HomeButtonViewModel::new(Some("scene_01"), None).unwrap();
    assert_eq!(vm.duration(), None);

    vm.set_duration(Some(3.5)).unwrap();
    assert_eq!(vm.duration(), Some(3.5));

    assert!(vm.set_duration(Some(-1.0)).is_err());
}
