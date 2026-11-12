use cesium_math::{Clock, ClockOptions};
use cesium_viewer::widgets::clock_view_model::ClockViewModel;
use cesium_viewer::widgets::viewer::{Viewer, ViewerContainer, ViewerOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_viewer_constructor_sets_default_values() {
    let container = ViewerContainer::new("container", true);
    let mut viewer = Viewer::new(Some(container.clone()), None).expect("Viewer creation");

    assert_eq!(viewer.container().id, "container");
    assert!(viewer.animation().is_some());
    assert!(viewer.base_layer_picker().is_some());
    assert!(viewer.fullscreen_button().is_some());
    assert!(viewer.geocoder().is_some());
    assert!(viewer.home_button().is_some());
    assert!(viewer.info_box().is_some());
    assert!(viewer.scene_mode_picker().is_some());
    assert!(viewer.selection_indicator().is_some());
    assert!(viewer.timeline().is_some());
    assert!(viewer.navigation_help_button().is_some());
    assert!(viewer.use_browser_recommended_resolution);
    assert!(!viewer.is_destroyed());

    viewer.resize();
    viewer.render();

    viewer.destroy();
    assert!(viewer.is_destroyed());
}

#[test]
fn test_viewer_can_specify_custom_clock_view_model() {
    let container = ViewerContainer::new("container", true);
    let clk = Rc::new(RefCell::new(Clock::default()));
    let cvm = ClockViewModel::new(Some(clk));

    let options = ViewerOptions {
        clock_view_model: Some(cvm.clone()),
        ..Default::default()
    };

    let mut viewer = Viewer::new(Some(container), Some(options)).unwrap();
    assert_eq!(viewer.clock_view_model().multiplier(), cvm.multiplier());

    viewer.destroy();
}

#[test]
fn test_viewer_setting_should_animate_overrides_clock() {
    let container = ViewerContainer::new("container", true);
    let clk = Rc::new(RefCell::new(
        Clock::new(ClockOptions {
            can_animate: Some(false),
            ..Default::default()
        })
        .unwrap(),
    ));
    let cvm = ClockViewModel::new(Some(clk));

    let options = ViewerOptions {
        clock_view_model: Some(cvm),
        should_animate: true,
        ..Default::default()
    };

    let viewer = Viewer::new(Some(container), Some(options)).unwrap();
    assert!(viewer.clock().borrow().should_animate());
}

#[test]
fn test_viewer_constructor_works_with_container_id_string() {
    let container = ViewerContainer::new("container", true);
    let mut map = HashMap::new();
    map.insert("container".to_string(), container);

    let viewer = Viewer::from_id("container", None, |id| map.get(id).cloned())
        .expect("Viewer from id");
    assert_eq!(viewer.container().id, "container");
}

#[test]
fn test_viewer_can_shut_off_sub_widgets() {
    let container = ViewerContainer::new("container", true);
    let options = ViewerOptions {
        home_button: false,
        scene_mode_picker: false,
        base_layer_picker: false,
        ..Default::default()
    };

    let mut viewer = Viewer::new(Some(container), Some(options)).unwrap();
    assert!(viewer.home_button().is_none());
    assert!(viewer.scene_mode_picker().is_none());
    assert!(viewer.base_layer_picker().is_none());

    assert!(viewer.geocoder().is_some());
    assert!(viewer.animation().is_some());
    assert!(viewer.timeline().is_some());
    assert!(viewer.fullscreen_button().is_some());
    assert!(viewer.selection_indicator().is_some());

    viewer.resize();
    viewer.render();
    viewer.destroy();
}
