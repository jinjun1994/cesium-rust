use cesium_math::{JulianDate, TimeInterval, TimeIntervalCollection};

#[test]
fn test_time_interval_collection_default() {
    let collection: TimeIntervalCollection<i32> = TimeIntervalCollection::new();
    assert_eq!(collection.len(), 0);
    assert!(collection.is_empty());
    assert_eq!(collection.start(), None);
    assert_eq!(collection.stop(), None);
    assert!(!collection.is_start_included());
    assert!(!collection.is_stop_included());
}

#[test]
fn test_time_interval_collection_add_non_overlapping() {
    let mut collection = TimeIntervalCollection::new();
    let d1 = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let d2 = JulianDate::from_iso8601("2000-01-02T00:00:00Z").unwrap();
    let d3 = JulianDate::from_iso8601("2000-01-03T00:00:00Z").unwrap();
    let d4 = JulianDate::from_iso8601("2000-01-04T00:00:00Z").unwrap();

    let i1 = TimeInterval::with_data(d1, d2, true, false, 1);
    let i2 = TimeInterval::with_data(d3, d4, true, true, 2);

    collection.add_interval(i1);
    collection.add_interval(i2);

    assert_eq!(collection.len(), 2);
    assert_eq!(collection.start(), Some(d1));
    assert_eq!(collection.stop(), Some(d4));
    assert!(collection.is_start_included());
    assert!(collection.is_stop_included());

    assert_eq!(collection.find_data_for_interval_containing_date(&d1), Some(&1));
    assert_eq!(collection.find_data_for_interval_containing_date(&d2), None);
    assert_eq!(collection.find_data_for_interval_containing_date(&d3), Some(&2));
    assert_eq!(collection.find_data_for_interval_containing_date(&d4), Some(&2));
}

#[test]
fn test_time_interval_collection_merge_same_data() {
    let mut collection = TimeIntervalCollection::new();
    let d1 = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let d2 = JulianDate::from_iso8601("2000-01-02T00:00:00Z").unwrap();
    let d3 = JulianDate::from_iso8601("2000-01-03T00:00:00Z").unwrap();

    let i1 = TimeInterval::with_data(d1, d2, true, true, 10);
    let i2 = TimeInterval::with_data(d2, d3, true, true, 10);

    collection.add_interval(i1);
    collection.add_interval(i2);

    assert_eq!(collection.len(), 1);
    assert_eq!(collection.start(), Some(d1));
    assert_eq!(collection.stop(), Some(d3));
}

#[test]
fn test_time_interval_collection_split_different_data() {
    let mut collection = TimeIntervalCollection::new();
    let d1 = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let d2 = JulianDate::from_iso8601("2000-01-02T00:00:00Z").unwrap();
    let d3 = JulianDate::from_iso8601("2000-01-03T00:00:00Z").unwrap();
    let d4 = JulianDate::from_iso8601("2000-01-04T00:00:00Z").unwrap();

    let i_base = TimeInterval::with_data(d1, d4, true, true, 1);
    let i_middle = TimeInterval::with_data(d2, d3, true, true, 2);

    collection.add_interval(i_base);
    collection.add_interval(i_middle);

    assert_eq!(collection.len(), 3);
    assert_eq!(collection.get(0).unwrap().data, Some(1));
    assert_eq!(collection.get(1).unwrap().data, Some(2));
    assert_eq!(collection.get(2).unwrap().data, Some(1));
}

#[test]
fn test_time_interval_collection_remove_interval() {
    let mut collection = TimeIntervalCollection::new();
    let d1 = JulianDate::from_iso8601("2000-01-01T00:00:00Z").unwrap();
    let d2 = JulianDate::from_iso8601("2000-01-02T00:00:00Z").unwrap();
    let d3 = JulianDate::from_iso8601("2000-01-03T00:00:00Z").unwrap();
    let d4 = JulianDate::from_iso8601("2000-01-04T00:00:00Z").unwrap();

    let i1 = TimeInterval::with_data(d1, d4, true, true, 100);
    collection.add_interval(i1);

    let hole = TimeInterval::new(d2, d3, true, true);
    assert!(collection.remove_interval(&hole));
    assert_eq!(collection.len(), 2);
}
