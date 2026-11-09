use cesium_renderer::{
    FrameRateMonitor, FrameRateMonitorOptions, DEFAULT_MIN_FPS_AFTER_WARMUP, DEFAULT_MIN_FPS_WARMUP,
    DEFAULT_QUIET_PERIOD, DEFAULT_SAMPLING_WINDOW, DEFAULT_WARMUP_PERIOD,
};
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_default_constructor() {
    let monitor = FrameRateMonitor::default();

    assert_eq!(monitor.sampling_window, DEFAULT_SAMPLING_WINDOW);
    assert_eq!(monitor.quiet_period, DEFAULT_QUIET_PERIOD);
    assert_eq!(monitor.warmup_period, DEFAULT_WARMUP_PERIOD);
    assert_eq!(monitor.minimum_frame_rate_during_warmup, DEFAULT_MIN_FPS_WARMUP);
    assert_eq!(monitor.minimum_frame_rate_after_warmup, DEFAULT_MIN_FPS_AFTER_WARMUP);
    assert_eq!(monitor.low_frame_rate().number_of_listeners(), 0);
    assert_eq!(monitor.nominal_frame_rate().number_of_listeners(), 0);
    assert_eq!(monitor.last_frames_per_second(), None);
}

#[test]
fn test_honors_constructor_parameters() {
    let monitor = FrameRateMonitor::new(Some(FrameRateMonitorOptions {
        sampling_window: Some(3.0),
        quiet_period: Some(1.0),
        warmup_period: Some(6.0),
        minimum_frame_rate_during_warmup: Some(1.0),
        minimum_frame_rate_after_warmup: Some(2.0),
    }));

    assert_eq!(monitor.sampling_window, 3.0);
    assert_eq!(monitor.quiet_period, 1.0);
    assert_eq!(monitor.warmup_period, 6.0);
    assert_eq!(monitor.minimum_frame_rate_during_warmup, 1.0);
    assert_eq!(monitor.minimum_frame_rate_after_warmup, 2.0);
}

#[test]
fn test_raises_low_frame_rate_event() {
    let mut monitor = FrameRateMonitor::new(Some(FrameRateMonitorOptions {
        quiet_period: Some(0.001),
        warmup_period: Some(0.001),
        sampling_window: Some(0.001),
        minimum_frame_rate_during_warmup: Some(1000.0),
        minimum_frame_rate_after_warmup: Some(1000.0),
    }));

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    let _ = monitor.low_frame_rate().add_listener(move |_fps| {
        called_clone.set(true);
    });

    let mut t = 0.0;
    // Frame 1 starts quiet period
    monitor.update(t).unwrap();

    // Past quiet period (1ms)
    t += 2.0;
    monitor.update(t).unwrap();

    // 2ms later, second sample with low frame rate (500 fps < 1000 fps)
    t += 2.0;
    monitor.update(t).unwrap();

    assert!(monitor.last_frames_per_second().unwrap() < 1000.0);
    assert!(called.get());
}

#[test]
fn test_does_not_monitor_while_paused() {
    let mut monitor = FrameRateMonitor::new(Some(FrameRateMonitorOptions {
        quiet_period: Some(0.001),
        warmup_period: Some(0.001),
        sampling_window: Some(0.001),
        minimum_frame_rate_during_warmup: Some(1000.0),
        minimum_frame_rate_after_warmup: Some(1000.0),
    }));

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    let _ = monitor.low_frame_rate().add_listener(move |_fps| {
        called_clone.set(true);
    });

    let mut t = 0.0;
    monitor.update(t).unwrap();

    t += 2.0;
    monitor.update(t).unwrap();

    monitor.pause();
    assert_eq!(monitor.pause_count(), 1);

    t += 2.0;
    monitor.update(t).unwrap();

    monitor.unpause();
    assert_eq!(monitor.pause_count(), 0);

    t += 0.5;
    monitor.update(t).unwrap();

    assert!(!called.get());
}

#[test]
fn test_pausing_multiple_times() {
    let mut monitor = FrameRateMonitor::new(Some(FrameRateMonitorOptions {
        quiet_period: Some(0.001),
        warmup_period: Some(0.001),
        sampling_window: Some(0.001),
        minimum_frame_rate_during_warmup: Some(1000.0),
        minimum_frame_rate_after_warmup: Some(1000.0),
    }));

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    let _ = monitor.low_frame_rate().add_listener(move |_fps| {
        called_clone.set(true);
    });

    monitor.pause();
    monitor.pause();
    assert_eq!(monitor.pause_count(), 2);
    monitor.unpause();
    assert_eq!(monitor.pause_count(), 1);

    let mut t = 0.0;
    monitor.update(t).unwrap();
    t += 2.0;
    monitor.update(t).unwrap();
    t += 2.0;
    monitor.update(t).unwrap();

    monitor.unpause();
    assert_eq!(monitor.pause_count(), 0);
    t += 0.5;
    monitor.update(t).unwrap();

    assert!(!called.get());
}

#[test]
fn test_does_not_report_during_quiet_period() {
    let mut monitor = FrameRateMonitor::new(Some(FrameRateMonitorOptions {
        quiet_period: Some(1.0), // 1 second quiet period
        warmup_period: Some(0.001),
        sampling_window: Some(0.001),
        minimum_frame_rate_during_warmup: Some(1000.0),
        minimum_frame_rate_after_warmup: Some(1000.0),
    }));

    let called = Rc::new(Cell::new(false));
    let called_clone = called.clone();
    let _ = monitor.low_frame_rate().add_listener(move |_fps| {
        called_clone.set(true);
    });

    let mut t = 0.0;
    monitor.update(t).unwrap();

    // 2ms later (still well within 1.0s quiet period)
    t += 2.0;
    monitor.update(t).unwrap();

    assert!(!called.get());
}

#[test]
fn test_nominal_frame_rate_event_raised() {
    let mut monitor = FrameRateMonitor::new(Some(FrameRateMonitorOptions {
        quiet_period: Some(0.001),
        warmup_period: Some(0.001),
        sampling_window: Some(0.001),
        minimum_frame_rate_during_warmup: Some(10.0),
        minimum_frame_rate_after_warmup: Some(10.0),
    }));

    let low_called = Rc::new(Cell::new(false));
    let low_clone = low_called.clone();
    let _ = monitor.low_frame_rate().add_listener(move |_fps| {
        low_clone.set(true);
    });

    let nominal_called = Rc::new(Cell::new(false));
    let nominal_clone = nominal_called.clone();
    let _ = monitor.nominal_frame_rate().add_listener(move |_fps| {
        nominal_clone.set(true);
    });

    let mut t = 0.0;
    monitor.update(t).unwrap();
    t += 2.0;
    monitor.update(t).unwrap();

    // 120ms gap -> 8.3 fps (< 10.0 fps)
    t += 120.0;
    monitor.update(t).unwrap();

    assert!(monitor.last_frames_per_second().unwrap() < 10.0);
    assert!(low_called.get());

    // Recover with fast frames: 10 frames of 10ms each (100 fps)
    for _ in 0..10 {
        t += 10.0;
        monitor.update(t).unwrap();
    }

    assert!(monitor.last_frames_per_second().unwrap() >= 10.0);
    assert!(nominal_called.get());
}

#[test]
fn test_destroy() {
    let mut monitor = FrameRateMonitor::default();
    assert!(!monitor.is_destroyed());
    monitor.destroy();
    assert!(monitor.is_destroyed());
    assert!(monitor.update(0.0).is_err());
}
