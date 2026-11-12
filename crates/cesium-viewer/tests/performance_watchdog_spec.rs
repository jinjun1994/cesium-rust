use cesium_viewer::widgets::{
    PerformanceWatchdog, PerformanceWatchdogContainer, PerformanceWatchdogOptions, WatchdogScene,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn can_create_and_destroy() {
    let container = Rc::new(RefCell::new(PerformanceWatchdogContainer::new(
        "testContainer",
    )));
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    let registry = [Rc::clone(&container)];

    let mut widget = PerformanceWatchdog::from_id(
        "testContainer",
        Some(PerformanceWatchdogOptions {
            container: None,
            scene: Some(scene),
            low_frame_rate_message: None,
        }),
        |id| registry.iter().find(|c| c.borrow().id == id).cloned(),
    )
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
fn throws_if_options_is_undefined() {
    let result = PerformanceWatchdog::new(None);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "options.container is required."
    );
}

#[test]
fn throws_if_options_container_is_undefined() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    let result = PerformanceWatchdog::new(Some(PerformanceWatchdogOptions {
        container: None,
        scene: Some(scene),
        low_frame_rate_message: None,
    }));
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "options.container is required."
    );
}

#[test]
fn throws_if_options_scene_is_undefined() {
    let container = Rc::new(RefCell::new(PerformanceWatchdogContainer::new(
        "testContainer",
    )));
    let result = PerformanceWatchdog::new(Some(PerformanceWatchdogOptions {
        container: Some(container),
        scene: None,
        low_frame_rate_message: None,
    }));
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().message, "options.scene is required.");
}

#[test]
fn constructor_throws_with_string_element_that_does_not_exist() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    let result = PerformanceWatchdog::from_id(
        "does not exist",
        Some(PerformanceWatchdogOptions {
            container: None,
            scene: Some(scene),
            low_frame_rate_message: None,
        }),
        |_| None,
    );
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap().message,
        "Element with id \"does not exist\" does not exist in the document."
    );
}
