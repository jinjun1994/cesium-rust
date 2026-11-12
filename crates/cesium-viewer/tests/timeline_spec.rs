use cesium_math::{Clock, ClockOptions, JulianDate};
use cesium_viewer::widgets::timeline::{Timeline, TimelineContainer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_timeline_sanity_check() {
    let container = Rc::new(RefCell::new(TimelineContainer::new("container")));
    let clock = Rc::new(RefCell::new(Clock::default()));

    let mut timeline = Timeline::new(Some(Rc::clone(&container)), Some(clock))
        .expect("Timeline should initialize");
    timeline.resize();
    assert!(!timeline.is_destroyed());
    timeline.destroy();
    assert!(timeline.is_destroyed());
}

#[test]
fn test_timeline_from_id_and_validation() {
    let container = Rc::new(RefCell::new(TimelineContainer::new("timelineContainer")));
    let clock = Rc::new(RefCell::new(Clock::default()));

    let mut map = HashMap::new();
    map.insert("timelineContainer".to_string(), Rc::clone(&container));

    let timeline = Timeline::from_id("timelineContainer", Some(clock), |id| map.get(id).cloned())
        .expect("Timeline from id should succeed");

    assert!(timeline.container().is_some());

    let err_empty = Timeline::from_id("", None, |_| None);
    assert!(err_empty.is_err());

    let err_missing = Timeline::from_id("unknown", None, |_| None);
    assert!(err_missing.is_err());
}

#[test]
fn test_timeline_zoom_to_and_needle_position() {
    let container = Rc::new(RefCell::new(TimelineContainer::new("container")));
    container.borrow_mut().set_size(1000.0, 50.0);

    let start = JulianDate::new_tai(100, 0.0);
    let stop = JulianDate::new_tai(101, 0.0);
    let current = JulianDate::new_tai(100, 43200.0);

    let options = ClockOptions {
        start_time: Some(start),
        stop_time: Some(stop),
        current_time: Some(current),
        ..Default::default()
    };
    let clock = Rc::new(RefCell::new(Clock::new(options).unwrap()));

    let mut timeline = Timeline::new(Some(container), Some(clock)).unwrap();

    assert_eq!(timeline.needle_pos_px(), 500.0);

    let new_start = JulianDate::new_tai(100, 0.0);
    let new_stop = JulianDate::new_tai(102, 0.0);
    timeline.zoom_to(new_start, new_stop).unwrap();

    // 100 + 43200s (0.5 day) is 25% of the 2-day span (100 to 102)
    assert_eq!(timeline.needle_pos_px(), 250.0);
}

#[test]
fn test_timeline_add_highlight_and_tracks() {
    let container = Rc::new(RefCell::new(TimelineContainer::new("container")));
    let clock = Rc::new(RefCell::new(Clock::default()));

    let mut timeline = Timeline::new(Some(container), Some(clock)).unwrap();

    let hl = timeline.add_highlight_range("rgba(255, 0, 0, 0.5)", 20.0, 0.0);
    assert_eq!(hl.color, "rgba(255, 0, 0, 0.5)");

    let track = timeline.add_track(
        JulianDate::new_tai(1, 0.0),
        JulianDate::new_tai(2, 0.0),
        15.0,
        "#00FF00",
        "#000000",
    );
    assert_eq!(track.height, 15.0);
    assert_eq!(track.color, "#00FF00");
}
