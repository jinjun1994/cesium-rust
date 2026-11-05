//! Direct port and verification against CesiumJS `DataSourceCollectionSpec.js` (Apache-2.0).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use cesium_math::{CustomDataSource, DataSourceCollection};

#[test]
fn test_contains_get_get_length_and_index_of_work() {
    let mut collection = DataSourceCollection::new();
    let source = CustomDataSource::new("target");
    let source_id = source.id().to_string();

    assert_eq!(collection.len(), 0);
    assert!(!collection.contains(&source));

    let ds1 = CustomDataSource::new("ds1");
    let ds3 = CustomDataSource::new("ds3");

    collection.add(ds1);
    collection.add(source.clone());
    collection.add(ds3);

    assert_eq!(collection.len(), 3);
    assert_eq!(collection.get(1).unwrap().id(), source_id);
    assert_eq!(collection.index_of(&source), Some(1));
    assert!(collection.contains(&source));

    let first_id = collection.get(0).unwrap().id().to_string();
    collection.remove_by_id(&first_id);
    assert_eq!(collection.index_of(&source), Some(0));

    assert!(collection.remove(&source));
    assert!(!collection.contains(&source));
}

#[test]
fn test_get_by_name() {
    let mut collection = DataSourceCollection::new();
    let ds1 = CustomDataSource::new("Name1");
    let ds2 = CustomDataSource::new("Name1");
    let ds3 = CustomDataSource::new("Name2");

    collection.add(ds1);
    collection.add(ds2);
    collection.add(ds3);

    let res = collection.get_by_name("Name1");
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].name(), "Name1");
    assert_eq!(res[1].name(), "Name1");

    let res2 = collection.get_by_name("Name2");
    assert_eq!(res2.len(), 1);
    assert_eq!(res2[0].name(), "Name2");

    let empty = collection.get_by_name("NonExistent");
    assert!(empty.is_empty());
}

#[test]
fn test_add_and_remove_events_work() {
    let mut collection = DataSourceCollection::new();
    let source = CustomDataSource::new("eventTest");

    let add_called = Arc::new(AtomicUsize::new(0));
    let remove_called = Arc::new(AtomicUsize::new(0));

    let ac = add_called.clone();
    collection.data_source_added.add_listener(move |_| {
        ac.fetch_add(1, Ordering::SeqCst);
    });

    let rc = remove_called.clone();
    collection.data_source_removed.add_listener(move |_| {
        rc.fetch_add(1, Ordering::SeqCst);
    });

    collection.add(source.clone());
    assert_eq!(add_called.load(Ordering::SeqCst), 1);
    assert_eq!(remove_called.load(Ordering::SeqCst), 0);

    assert!(collection.remove(&source));
    assert_eq!(add_called.load(Ordering::SeqCst), 1);
    assert_eq!(remove_called.load(Ordering::SeqCst), 1);
}

#[test]
fn test_move_events_work() {
    let mut collection = DataSourceCollection::new();
    let source = CustomDataSource::new("mover");

    let move_called = Arc::new(AtomicUsize::new(0));
    let mc = move_called.clone();
    collection.data_source_moved.add_listener(move |_| {
        mc.fetch_add(1, Ordering::SeqCst);
    });

    collection.add(source.clone());
    collection.raise(&source).unwrap();
    collection.lower(&source).unwrap();
    collection.raise_to_top(&source).unwrap();
    collection.lower_to_bottom(&source).unwrap();

    // Since length is 1, moving does not trigger moved event
    assert_eq!(move_called.load(Ordering::SeqCst), 0);

    collection.add(CustomDataSource::new("ds2"));
    collection.add(CustomDataSource::new("ds3"));

    // Length is now 3, index of source is 0
    collection.raise(&source).unwrap();
    assert_eq!(move_called.load(Ordering::SeqCst), 1);
    assert_eq!(collection.index_of(&source), Some(1));

    collection.lower(&source).unwrap();
    assert_eq!(move_called.load(Ordering::SeqCst), 2);
    assert_eq!(collection.index_of(&source), Some(0));

    collection.raise_to_top(&source).unwrap();
    assert_eq!(move_called.load(Ordering::SeqCst), 3);
    assert_eq!(collection.index_of(&source), Some(2));

    collection.lower_to_bottom(&source).unwrap();
    assert_eq!(move_called.load(Ordering::SeqCst), 4);
    assert_eq!(collection.index_of(&source), Some(0));
}

#[test]
fn test_remove_all_triggers_events() {
    let mut collection = DataSourceCollection::new();
    collection.add(CustomDataSource::new("ds1"));
    collection.add(CustomDataSource::new("ds2"));
    collection.add(CustomDataSource::new("ds3"));

    let remove_called = Arc::new(AtomicUsize::new(0));
    let rc = remove_called.clone();
    collection.data_source_removed.add_listener(move |_| {
        rc.fetch_add(1, Ordering::SeqCst);
    });

    collection.remove_all();
    assert_eq!(collection.len(), 0);
    assert_eq!(remove_called.load(Ordering::SeqCst), 3);
}
