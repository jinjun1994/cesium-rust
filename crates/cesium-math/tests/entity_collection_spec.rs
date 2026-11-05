//! Direct port and verification against CesiumJS `EntityCollectionSpec.js` (Apache-2.0).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use cesium_math::{
    Entity, EntityCollection, Iso8601, JulianDate, TimeInterval, TimeIntervalCollection,
};

#[test]
fn test_constructor_has_expected_defaults() {
    let collection = EntityCollection::new();
    assert!(!collection.id().is_empty());
    assert_eq!(collection.values().len(), 0);
    assert!(collection.is_empty());
    assert!(collection.show());
}

#[test]
fn test_add_remove_works() {
    let entity = Entity::new();
    let entity2 = Entity::new();
    let id1 = entity.id.clone();
    let id2 = entity2.id.clone();
    let mut collection = EntityCollection::new();

    collection.add(entity).unwrap();
    assert_eq!(collection.values().len(), 1);

    collection.add(entity2).unwrap();
    assert_eq!(collection.values().len(), 2);

    assert!(collection.remove_by_id(&id2));
    assert_eq!(collection.values().len(), 1);

    assert!(collection.remove_by_id(&id1));
    assert_eq!(collection.values().len(), 0);
    assert!(collection.is_empty());
}

#[test]
fn test_add_remove_raises_expected_events() {
    let entity = Entity::new();
    let entity2 = Entity::new();
    let id1 = entity.id.clone();
    let id2 = entity2.id.clone();
    let mut collection = EntityCollection::new();

    let times_called = Arc::new(AtomicUsize::new(0));
    let last_added_count = Arc::new(AtomicUsize::new(0));
    let last_removed_count = Arc::new(AtomicUsize::new(0));
    let last_changed_count = Arc::new(AtomicUsize::new(0));

    let tc = times_called.clone();
    let la = last_added_count.clone();
    let lr = last_removed_count.clone();
    let lc = last_changed_count.clone();

    collection.collection_changed.add_listener(move |(added, removed, changed)| {
        tc.fetch_add(1, Ordering::SeqCst);
        la.store(added.len(), Ordering::SeqCst);
        lr.store(removed.len(), Ordering::SeqCst);
        lc.store(changed.len(), Ordering::SeqCst);
    });

    collection.add(entity).unwrap();
    assert_eq!(times_called.load(Ordering::SeqCst), 1);
    assert_eq!(last_added_count.load(Ordering::SeqCst), 1);
    assert_eq!(last_removed_count.load(Ordering::SeqCst), 0);
    assert_eq!(last_changed_count.load(Ordering::SeqCst), 0);

    collection.notify_entity_changed(&id1);
    assert_eq!(times_called.load(Ordering::SeqCst), 2);
    assert_eq!(last_added_count.load(Ordering::SeqCst), 0);
    assert_eq!(last_removed_count.load(Ordering::SeqCst), 0);
    assert_eq!(last_changed_count.load(Ordering::SeqCst), 1);

    collection.add(entity2).unwrap();
    assert_eq!(times_called.load(Ordering::SeqCst), 3);
    assert_eq!(last_added_count.load(Ordering::SeqCst), 1);

    collection.remove_by_id(&id2);
    assert_eq!(times_called.load(Ordering::SeqCst), 4);
    assert_eq!(last_removed_count.load(Ordering::SeqCst), 1);

    collection.remove_by_id(&id1);
    assert_eq!(times_called.load(Ordering::SeqCst), 5);
    assert_eq!(last_removed_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_suspended_add_remove_raises_expected_events() {
    let entity = Entity::new();
    let entity2 = Entity::new();
    let entity3 = Entity::new();
    let id2 = entity2.id.clone();
    let id3 = entity3.id.clone();
    let mut collection = EntityCollection::new();

    let times_called = Arc::new(AtomicUsize::new(0));
    let last_added_count = Arc::new(AtomicUsize::new(0));
    let last_removed_count = Arc::new(AtomicUsize::new(0));

    let tc = times_called.clone();
    let la = last_added_count.clone();
    let lr = last_removed_count.clone();

    collection.collection_changed.add_listener(move |(added, removed, _changed)| {
        tc.fetch_add(1, Ordering::SeqCst);
        la.store(added.len(), Ordering::SeqCst);
        lr.store(removed.len(), Ordering::SeqCst);
    });

    collection.suspend_events();
    collection.suspend_events();
    collection.add(entity).unwrap();
    collection.add(entity2).unwrap();
    collection.add(entity3).unwrap();
    collection.remove_by_id(&id2);

    assert_eq!(times_called.load(Ordering::SeqCst), 0);
    collection.resume_events().unwrap();

    assert_eq!(times_called.load(Ordering::SeqCst), 0);
    collection.resume_events().unwrap();

    assert_eq!(times_called.load(Ordering::SeqCst), 1);
    assert_eq!(last_added_count.load(Ordering::SeqCst), 2);
    assert_eq!(last_removed_count.load(Ordering::SeqCst), 0);

    collection.suspend_events();
    collection.remove_by_id(&id3);
    collection.resume_events().unwrap();

    assert_eq!(times_called.load(Ordering::SeqCst), 2);
    assert_eq!(last_removed_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_remove_all_works_and_raises_events() {
    let entity = Entity::new();
    let entity2 = Entity::new();
    let mut collection = EntityCollection::new();

    collection.add(entity).unwrap();
    collection.add(entity2).unwrap();
    assert_eq!(collection.values().len(), 2);

    let times_called = Arc::new(AtomicUsize::new(0));
    let last_removed_count = Arc::new(AtomicUsize::new(0));
    let tc = times_called.clone();
    let lr = last_removed_count.clone();

    collection.collection_changed.add_listener(move |(_added, removed, _changed)| {
        tc.fetch_add(1, Ordering::SeqCst);
        lr.store(removed.len(), Ordering::SeqCst);
    });

    collection.remove_all();
    assert_eq!(collection.values().len(), 0);
    assert_eq!(times_called.load(Ordering::SeqCst), 1);
    assert_eq!(last_removed_count.load(Ordering::SeqCst), 2);

    // Calling remove_all on empty does not fire again
    collection.remove_all();
    assert_eq!(times_called.load(Ordering::SeqCst), 1);
}

#[test]
fn test_get_by_id_and_get_or_create() {
    let mut collection = EntityCollection::new();
    assert!(collection.get_by_id("123").is_none());

    let created = collection.get_or_create_entity("test").unwrap();
    assert_eq!(created.id, "test");
    assert_eq!(collection.values().len(), 1);

    let retrieved = collection.get_by_id("test");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "test");

    let same = collection.get_or_create_entity("test").unwrap();
    assert_eq!(same.id, "test");
    assert_eq!(collection.values().len(), 1);
}

#[test]
fn test_compute_availability() {
    let mut collection = EntityCollection::new();
    let availability = collection.compute_availability();
    assert_eq!(availability.start, Iso8601::minimum_value());
    assert_eq!(availability.stop, Iso8601::maximum_value());

    let e1 = collection.get_or_create_entity("1").unwrap();
    let mut avail1 = TimeIntervalCollection::new();
    avail1.add_interval(TimeInterval::new(
        JulianDate::from_iso8601("2012-08-01T00:00:00Z").unwrap(),
        JulianDate::from_iso8601("2012-08-02T00:00:00Z").unwrap(),
        true,
        true,
    ));
    e1.availability = Some(avail1);

    let e2 = collection.get_or_create_entity("2").unwrap();
    let mut avail2 = TimeIntervalCollection::new();
    avail2.add_interval(TimeInterval::new(
        JulianDate::from_iso8601("2012-08-05T00:00:00Z").unwrap(),
        JulianDate::from_iso8601("2012-08-06T00:00:00Z").unwrap(),
        true,
        true,
    ));
    e2.availability = Some(avail2);

    let _e3 = collection.get_or_create_entity("3").unwrap();

    let computed = collection.compute_availability();
    assert_eq!(
        computed.start,
        JulianDate::from_iso8601("2012-08-01T00:00:00Z").unwrap()
    );
    assert_eq!(
        computed.stop,
        JulianDate::from_iso8601("2012-08-06T00:00:00Z").unwrap()
    );
}

#[test]
fn test_compute_availability_with_infinite_bounds() {
    let mut collection = EntityCollection::new();
    let e1 = collection.get_or_create_entity("1").unwrap();
    let mut avail1 = TimeIntervalCollection::new();
    avail1.add_interval(TimeInterval::new(
        JulianDate::from_iso8601("2012-08-01T00:00:00Z").unwrap(),
        Iso8601::maximum_value(),
        true,
        true,
    ));
    e1.availability = Some(avail1);

    let e2 = collection.get_or_create_entity("2").unwrap();
    let mut avail2 = TimeIntervalCollection::new();
    avail2.add_interval(TimeInterval::new(
        Iso8601::minimum_value(),
        JulianDate::from_iso8601("2012-08-06T00:00:00Z").unwrap(),
        true,
        true,
    ));
    e2.availability = Some(avail2);

    let computed = collection.compute_availability();
    assert_eq!(
        computed.start,
        JulianDate::from_iso8601("2012-08-01T00:00:00Z").unwrap()
    );
    assert_eq!(
        computed.stop,
        JulianDate::from_iso8601("2012-08-06T00:00:00Z").unwrap()
    );
}

#[test]
fn test_error_handling() {
    let mut collection = EntityCollection::new();
    assert!(collection.resume_events().is_err());

    let e1 = Entity::with_id("dup");
    let e2 = Entity::with_id("dup");
    collection.add(e1).unwrap();
    assert!(collection.add(e2).is_err());

    assert!(!collection.remove_by_id("non_existent"));
    assert!(!collection.remove_by_id(""));
    assert!(collection.get_or_create_entity("").is_err());
}
