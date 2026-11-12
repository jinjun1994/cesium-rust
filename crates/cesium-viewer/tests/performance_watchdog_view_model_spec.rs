use cesium_viewer::widgets::{
    PerformanceWatchdogViewModel, PerformanceWatchdogViewModelOptions, WatchdogScene,
    DEFAULT_LOW_FRAME_RATE_MESSAGE,
};
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn throws_when_constructed_without_a_scene() {
    let result1 = PerformanceWatchdogViewModel::new(None);
    assert!(result1.is_err());
    assert_eq!(result1.err().unwrap().message, "options.scene is required.");

    let result2 = PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
        scene: None,
        ..Default::default()
    }));
    assert!(result2.is_err());
    assert_eq!(result2.err().unwrap().message, "options.scene is required.");
}

#[test]
fn can_be_constructed_with_just_a_scene() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    let view_model = PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
        scene: Some(Rc::clone(&scene)),
        ..Default::default()
    }))
    .unwrap();

    assert_eq!(
        view_model.low_frame_rate_message,
        DEFAULT_LOW_FRAME_RATE_MESSAGE
    );
    assert!(!view_model.low_frame_rate_message_dismissed());
    assert!(!view_model.showing_low_frame_rate_message());
    assert!(Rc::ptr_eq(&view_model.scene(), &scene));
}

#[test]
fn honors_parameters_to_the_constructor() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    let view_model = PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
        scene: Some(Rc::clone(&scene)),
        low_frame_rate_message: Some("why so slow?".to_string()),
    }))
    .unwrap();

    assert_eq!(view_model.low_frame_rate_message, "why so slow?");
    assert!(Rc::ptr_eq(&view_model.scene(), &scene));
}

#[test]
fn shows_a_message_on_low_frame_rate() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    {
        let monitor = scene.borrow().monitor();
        let mut m = monitor.borrow_mut();
        m.quiet_period = 0.001;
        m.warmup_period = 0.001;
        m.sampling_window = 0.001;
        m.minimum_frame_rate_during_warmup = 1000.0;
        m.minimum_frame_rate_after_warmup = 1000.0;
    }

    let view_model = PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
        scene: Some(Rc::clone(&scene)),
        ..Default::default()
    }))
    .unwrap();

    assert!(!view_model.showing_low_frame_rate_message());

    // First frame starts quiet period
    scene.borrow().render_at(1000.0);

    // Frame past quiet period records first sample
    scene.borrow().render_at(1002.0);

    // Second sample over max frame time triggers low frame rate
    scene.borrow().render_at(1004.0);

    assert!(view_model.showing_low_frame_rate_message());
}

#[test]
fn does_not_report_a_low_frame_rate_during_the_quiet_period() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    {
        let monitor = scene.borrow().monitor();
        let mut m = monitor.borrow_mut();
        m.quiet_period = 1.0;
        m.warmup_period = 0.001;
        m.sampling_window = 0.001;
        m.minimum_frame_rate_during_warmup = 1000.0;
        m.minimum_frame_rate_after_warmup = 1000.0;
    }

    let view_model = PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
        scene: Some(Rc::clone(&scene)),
        ..Default::default()
    }))
    .unwrap();

    scene.borrow().render_at(1000.0);
    scene.borrow().render_at(1002.0);

    assert!(!view_model.showing_low_frame_rate_message());
}

#[test]
fn the_low_frame_rate_message_goes_away_after_warmup_if_frame_rate_returns_to_nominal() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    {
        let monitor = scene.borrow().monitor();
        let mut m = monitor.borrow_mut();
        m.quiet_period = 0.001;
        m.warmup_period = 0.001;
        m.sampling_window = 0.001;
        m.minimum_frame_rate_during_warmup = 10.0;
        m.minimum_frame_rate_after_warmup = 10.0;
    }

    let view_model = PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
        scene: Some(Rc::clone(&scene)),
        ..Default::default()
    }))
    .unwrap();

    assert!(!view_model.showing_low_frame_rate_message());

    scene.borrow().render_at(1000.0);
    scene.borrow().render_at(1002.0);
    scene.borrow().render_at(1122.0); // 120ms gap -> ~8.3 FPS < 10 FPS
    assert!(view_model.showing_low_frame_rate_message());

    // Render fast frames
    for i in 1..=50 {
        scene.borrow().render_at(1200.0 + i as f64 * 2.0); // 2ms per frame = 500 FPS > 10 FPS
    }
    assert!(!view_model.showing_low_frame_rate_message());
}

#[test]
fn does_not_show_low_frame_rate_message_again_once_it_is_dismissed() {
    let scene = Rc::new(RefCell::new(WatchdogScene::new()));
    {
        let monitor = scene.borrow().monitor();
        let mut m = monitor.borrow_mut();
        m.quiet_period = 0.001;
        m.warmup_period = 0.001;
        m.sampling_window = 0.001;
        m.minimum_frame_rate_during_warmup = 1000.0;
        m.minimum_frame_rate_after_warmup = 1000.0;
    }

    let view_model = PerformanceWatchdogViewModel::new(Some(PerformanceWatchdogViewModelOptions {
        scene: Some(Rc::clone(&scene)),
        ..Default::default()
    }))
    .unwrap();

    assert!(!view_model.showing_low_frame_rate_message());

    scene.borrow().render_at(1000.0);
    scene.borrow().render_at(1002.0);
    scene.borrow().render_at(1004.0);
    assert!(view_model.showing_low_frame_rate_message());

    assert!(view_model.dismiss_message().execute(()).is_ok());
    assert!(!view_model.showing_low_frame_rate_message());

    // Render more slow frames
    scene.borrow().render_at(1006.0);
    scene.borrow().render_at(1008.0);
    assert!(!view_model.showing_low_frame_rate_message());
}
