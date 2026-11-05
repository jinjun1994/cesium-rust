//! Direct port and verification against CesiumJS `CustomDataSourceSpec.js` (Apache-2.0).

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use cesium_math::{CustomDataSource, DataSourceClock};

#[test]
fn test_constructor_has_expected_defaults() {
    let data_source = CustomDataSource::new("");
    assert!(data_source.name().is_empty());
    assert!(data_source.clock().is_none());
    assert!(!data_source.is_loading());
    assert!(data_source.show());
    assert_eq!(data_source.entities().values().len(), 0);
}

#[test]
fn test_show_sets_underlying_entity_collection_show() {
    let mut data_source = CustomDataSource::new("myData");

    data_source.set_show(false);
    assert!(!data_source.show());
    assert!(!data_source.entities().show());

    data_source.set_show(true);
    assert!(data_source.show());
    assert!(data_source.entities().show());
}

#[test]
fn test_setting_name_raises_changed_event() {
    let mut data_source = CustomDataSource::new("initial");

    let times_called = Arc::new(AtomicUsize::new(0));
    let tc = times_called.clone();
    data_source.changed_event.add_listener(move |_| {
        tc.fetch_add(1, Ordering::SeqCst);
    });

    data_source.set_name("chester");
    assert_eq!(data_source.name(), "chester");
    assert_eq!(times_called.load(Ordering::SeqCst), 1);
}

#[test]
fn test_setting_clock_raises_changed_event() {
    let mut data_source = CustomDataSource::new("withClock");

    let times_called = Arc::new(AtomicUsize::new(0));
    let tc = times_called.clone();
    data_source.changed_event.add_listener(move |_| {
        tc.fetch_add(1, Ordering::SeqCst);
    });

    let mut new_clock = DataSourceClock::new();
    new_clock.multiplier = Some(5.0);
    data_source.set_clock(Some(new_clock));

    assert!(data_source.clock().is_some());
    assert_eq!(data_source.clock().unwrap().multiplier, Some(5.0));
    assert_eq!(times_called.load(Ordering::SeqCst), 1);
}

#[test]
fn test_setting_is_loading_raises_loading_event() {
    let mut data_source = CustomDataSource::new("loader");

    let last_state = Arc::new(AtomicBool::new(false));
    let times_called = Arc::new(AtomicUsize::new(0));

    let ls = last_state.clone();
    let tc = times_called.clone();
    data_source.loading_event.add_listener(move |loading| {
        tc.fetch_add(1, Ordering::SeqCst);
        ls.store(*loading, Ordering::SeqCst);
    });

    data_source.set_loading(true);
    assert!(data_source.is_loading());
    assert_eq!(times_called.load(Ordering::SeqCst), 1);
    assert!(last_state.load(Ordering::SeqCst));

    data_source.set_loading(false);
    assert!(!data_source.is_loading());
    assert_eq!(times_called.load(Ordering::SeqCst), 2);
    assert!(!last_state.load(Ordering::SeqCst));
}
