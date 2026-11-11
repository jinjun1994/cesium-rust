use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use cesium_math::{
    Clock, ClockOptions, ClockStep, JulianDate, Request, RequestType, TimeInterval,
    TimeIntervalCollection,
};
use cesium_renderer::time_dynamic_imagery::{TimeDynamicImagery, TimeDynamicImageryOptions};
use serde_json::{json, Value};

fn create_test_clock() -> Rc<RefCell<Clock>> {
    let clock = Clock::new(ClockOptions {
        current_time: Some(JulianDate::from_iso8601("2017-04-26T00:00:00Z").unwrap()),
        clock_step: Some(ClockStep::TickDependent),
        multiplier: Some(1.0),
        should_animate: Some(true),
        can_animate: Some(true),
        ..Default::default()
    })
    .unwrap();
    Rc::new(RefCell::new(clock))
}

fn create_test_times() -> TimeIntervalCollection<Value> {
    let mut collection = TimeIntervalCollection::new();
    let dates = [
        "2017-04-26T00:00:00Z",
        "2017-04-27T00:00:00Z",
        "2017-04-28T00:00:00Z",
        "2017-04-29T00:00:00Z",
        "2017-04-30T00:00:00Z",
        "2017-05-01T00:00:00Z",
    ];

    for i in 0..5 {
        let start = JulianDate::from_iso8601(dates[i]).unwrap();
        let stop = JulianDate::from_iso8601(dates[i + 1]).unwrap();
        let data = json!({ "Time": dates[i] });
        let interval = TimeInterval::with_data(start, stop, true, false, data);
        collection.add_interval(interval);
    }

    collection
}

#[test]
fn clock_is_required() {
    let times = create_test_times();
    let res = TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
        clock: None,
        times: Some(times),
        request_image_function: Some(Rc::new(|_, _, _, _, _| None)),
        reload_function: Some(Rc::new(|| {})),
    });
    assert!(res.is_err());
}

#[test]
fn times_is_required() {
    let clock = create_test_clock();
    let res = TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
        clock: Some(clock),
        times: None,
        request_image_function: Some(Rc::new(|_, _, _, _, _| None)),
        reload_function: Some(Rc::new(|| {})),
    });
    assert!(res.is_err());
}

#[test]
fn request_image_function_is_required() {
    let clock = create_test_clock();
    let times = create_test_times();
    let res = TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
        clock: Some(clock),
        times: Some(times),
        request_image_function: None,
        reload_function: Some(Rc::new(|| {})),
    });
    assert!(res.is_err());
}

#[test]
fn reload_function_is_required() {
    let clock = create_test_clock();
    let times = create_test_times();
    let res = TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
        clock: Some(clock),
        times: Some(times),
        request_image_function: Some(Rc::new(|_, _, _, _, _| None)),
        reload_function: None,
    });
    assert!(res.is_err());
}

#[test]
fn initialization() {
    let clock = create_test_clock();
    let times = create_test_times();
    let reload_count = Rc::new(AtomicUsize::new(0));
    let reload_rc = reload_count.clone();

    let time_dynamic_imagery = TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
        clock: Some(clock),
        times: Some(times),
        request_image_function: Some(Rc::new(|_, _, _, _, _| None)),
        reload_function: Some(Rc::new(move || {
            reload_rc.fetch_add(1, Ordering::SeqCst);
        })),
    })
    .unwrap();

    assert!(time_dynamic_imagery.tile_cache().is_empty());
    assert!(time_dynamic_imagery.tiles_requested_for_interval().is_empty());
    assert_eq!(time_dynamic_imagery.current_interval_index(), Some(0));
    assert_eq!(reload_count.load(Ordering::SeqCst), 1);
}

#[test]
fn changing_clock_causes_reload() {
    let clock = create_test_clock();
    let times = create_test_times();
    let reload_count = Rc::new(AtomicUsize::new(0));
    let reload_rc = reload_count.clone();

    let mut time_dynamic_imagery =
        TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
            clock: Some(clock),
            times: Some(times),
            request_image_function: Some(Rc::new(|_, _, _, _, _| None)),
            reload_function: Some(Rc::new(move || {
                reload_rc.fetch_add(1, Ordering::SeqCst);
            })),
        })
        .unwrap();

    let initial = reload_count.load(Ordering::SeqCst);
    time_dynamic_imagery.set_clock(create_test_clock());
    assert!(reload_count.load(Ordering::SeqCst) > initial);
}

#[test]
fn changing_times_causes_reload() {
    let clock = create_test_clock();
    let times = create_test_times();
    let reload_count = Rc::new(AtomicUsize::new(0));
    let reload_rc = reload_count.clone();

    let mut time_dynamic_imagery =
        TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
            clock: Some(clock),
            times: Some(times),
            request_image_function: Some(Rc::new(|_, _, _, _, _| None)),
            reload_function: Some(Rc::new(move || {
                reload_rc.fetch_add(1, Ordering::SeqCst);
            })),
        })
        .unwrap();

    let initial = reload_count.load(Ordering::SeqCst);
    time_dynamic_imagery.set_times(create_test_times());
    assert!(reload_count.load(Ordering::SeqCst) > initial);
}

#[test]
fn clock_tick_causes_reload() {
    let clock = create_test_clock();
    let times = create_test_times();
    let request_count = Rc::new(AtomicUsize::new(0));
    let request_rc = request_count.clone();
    let reload_count = Rc::new(AtomicUsize::new(0));
    let reload_rc = reload_count.clone();

    let mut time_dynamic_imagery =
        TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
            clock: Some(clock.clone()),
            times: Some(times),
            request_image_function: Some(Rc::new(move |_, _, _, _, _| {
                request_rc.fetch_add(1, Ordering::SeqCst);
                Some("image_data".to_string())
            })),
            reload_function: Some(Rc::new(move || {
                reload_rc.fetch_add(1, Ordering::SeqCst);
            })),
        })
        .unwrap();

    let request = Request::default();
    time_dynamic_imagery.check_approaching_interval(0, 1, 2, &request);

    let reloads_before = reload_count.load(Ordering::SeqCst);
    // Move clock close to end of day
    clock
        .borrow_mut()
        .set_current_time(JulianDate::from_iso8601("2017-04-26T23:59:59Z").unwrap());
    clock.borrow_mut().tick();

    // After tick, crossed into next day (2017-04-27), triggering reload
    assert_eq!(request_count.load(Ordering::SeqCst), 0);
    assert_eq!(reload_count.load(Ordering::SeqCst), reloads_before + 1);
}

#[test]
fn clock_tick_causes_preload() {
    let clock = create_test_clock();
    let times = create_test_times();
    let request_count = Rc::new(AtomicUsize::new(0));
    let request_rc = request_count.clone();
    let reload_count = Rc::new(AtomicUsize::new(0));
    let reload_rc = reload_count.clone();

    let mut time_dynamic_imagery =
        TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
            clock: Some(clock.clone()),
            times: Some(times),
            request_image_function: Some(Rc::new(move |_, _, _, _, _| {
                request_rc.fetch_add(1, Ordering::SeqCst);
                Some("image_data".to_string())
            })),
            reload_function: Some(Rc::new(move || {
                reload_rc.fetch_add(1, Ordering::SeqCst);
            })),
        })
        .unwrap();

    let request = Request {
        throttle: false,
        throttle_by_server: true,
        request_type: RequestType::Imagery,
        ..Default::default()
    };
    time_dynamic_imagery.check_approaching_interval(0, 1, 2, &request);

    let reloads_before = reload_count.load(Ordering::SeqCst);
    // 4 seconds before day boundary (less than 5.0 seconds threshold)
    clock
        .borrow_mut()
        .set_current_time(JulianDate::from_iso8601("2017-04-26T23:59:56Z").unwrap());
    clock.borrow_mut().tick();

    assert_eq!(request_count.load(Ordering::SeqCst), 1);
    let cache = time_dynamic_imagery.tile_cache();
    assert!(cache.get(&1).unwrap().get("0-1-2").is_some());
    assert_eq!(reload_count.load(Ordering::SeqCst), reloads_before);
}

#[test]
fn check_approaching_interval_causes_preload() {
    let clock = create_test_clock();
    let times = create_test_times();
    let request_count = Rc::new(AtomicUsize::new(0));
    let request_rc = request_count.clone();
    let reload_count = Rc::new(AtomicUsize::new(0));
    let reload_rc = reload_count.clone();

    let mut time_dynamic_imagery =
        TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
            clock: Some(clock.clone()),
            times: Some(times),
            request_image_function: Some(Rc::new(move |_, _, _, _, _| {
                request_rc.fetch_add(1, Ordering::SeqCst);
                Some("image_data".to_string())
            })),
            reload_function: Some(Rc::new(move || {
                reload_rc.fetch_add(1, Ordering::SeqCst);
            })),
        })
        .unwrap();

    let request = Request {
        throttle: false,
        throttle_by_server: true,
        request_type: RequestType::Imagery,
        ..Default::default()
    };

    let reloads_before = reload_count.load(Ordering::SeqCst);
    clock
        .borrow_mut()
        .set_current_time(JulianDate::from_iso8601("2017-04-26T23:59:56Z").unwrap());
    time_dynamic_imagery.check_approaching_interval(0, 1, 2, &request);

    assert_eq!(request_count.load(Ordering::SeqCst), 1);
    let cache = time_dynamic_imagery.tile_cache();
    assert!(cache.get(&1).unwrap().get("0-1-2").is_some());
    assert_eq!(reload_count.load(Ordering::SeqCst), reloads_before);
}

#[test]
fn check_approaching_interval_limits_number_of_requests() {
    let clock = create_test_clock();
    let times = create_test_times();

    let mut time_dynamic_imagery =
        TimeDynamicImagery::<Value, String>::new(TimeDynamicImageryOptions {
            clock: Some(clock),
            times: Some(times),
            request_image_function: Some(Rc::new(|_, _, _, _, _| Some("image".to_string()))),
            reload_function: Some(Rc::new(|| {})),
        })
        .unwrap();

    let request = Request {
        throttle: true,
        throttle_by_server: true,
        request_type: RequestType::Imagery,
        ..Default::default()
    };

    let mut count = 0;
    for level in 0..15 {
        for x in 0..(level + 1) {
            for y in 0..(level + 1) {
                time_dynamic_imagery.check_approaching_interval(x, y, level, &request);
                count += 1;

                if count == 512 {
                    count = 256;
                }

                let requested = time_dynamic_imagery.tiles_requested_for_interval();
                assert_eq!(requested.len(), count);
                assert_eq!(requested[count - 1].key, format!("{x}-{y}-{level}"));
            }
        }
    }
}
