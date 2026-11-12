use cesium_viewer::widgets::viewer::{
    viewer_performance_watchdog_mixin, Viewer, ViewerContainer, ViewerPerformanceWatchdogOptions,
};

#[test]
fn test_viewer_performance_watchdog_mixin_sets_default_values() {
    let container = ViewerContainer::new("container", true);
    let mut viewer = Viewer::new(Some(container), None).unwrap();

    viewer_performance_watchdog_mixin(Some(&mut viewer), None).expect("mixin should succeed");
    assert!(viewer.performance_watchdog().is_some());

    viewer.destroy();
}

#[test]
fn test_viewer_performance_watchdog_mixin_sets_option_values() {
    let container = ViewerContainer::new("container", true);
    let mut viewer = Viewer::new(Some(container), None).unwrap();

    let options = ViewerPerformanceWatchdogOptions {
        low_frame_rate_message: Some("Foo".to_string()),
    };

    viewer_performance_watchdog_mixin(Some(&mut viewer), Some(options)).unwrap();
    let pwd = viewer.performance_watchdog().unwrap();
    assert_eq!(pwd.view_model().low_frame_rate_message, "Foo");

    viewer.destroy();
}

#[test]
fn test_viewer_performance_watchdog_mixin_throws_if_not_given_a_viewer() {
    let err = viewer_performance_watchdog_mixin(None, None);
    assert!(err.is_err());
}
