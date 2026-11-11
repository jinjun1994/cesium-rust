use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use cesium_renderer::primitive_collection::{
    MockPrimitive, PrimitiveCollection, PrimitiveCollectionOptions, ScenePrimitive,
};

#[test]
fn test_primitive_collection_constructs_with_options() {
    let collection = PrimitiveCollection::new(PrimitiveCollectionOptions {
        show: Some(false),
        destroy_primitives: Some(false),
        count_references: Some(false),
    });
    assert!(!collection.show);
    assert!(!collection.destroy_primitives);
    assert!(!collection.is_destroyed());
    collection.destroy();
    assert!(collection.is_destroyed());
}

#[test]
fn test_primitive_collection_defaults_and_add() {
    let mut primitives = PrimitiveCollection::default();
    assert!(primitives.show);
    assert!(primitives.destroy_primitives);
    assert_eq!(primitives.len(), 0);

    let p = Arc::new(MockPrimitive::new());
    primitives.add(p, None).unwrap();
    assert_eq!(primitives.len(), 1);
}

#[test]
fn test_primitive_collection_add_with_index() {
    let mut primitives = PrimitiveCollection::default();
    let p0 = Arc::new(MockPrimitive::new());
    let p1 = Arc::new(MockPrimitive::new());

    assert!(primitives.add(p0.clone(), Some(1)).is_err());

    primitives.add(p0.clone(), Some(0)).unwrap();
    assert_eq!(primitives.get(0).unwrap().id(), p0.id());

    assert!(primitives.add(p1.clone(), Some(2)).is_err());

    primitives.add(p1.clone(), Some(0)).unwrap();
    assert_eq!(primitives.get(0).unwrap().id(), p1.id());
    assert_eq!(primitives.get(1).unwrap().id(), p0.id());
}

#[test]
fn test_primitive_collection_removes_primitives() {
    let mut primitives = PrimitiveCollection::default();
    let p0 = Arc::new(MockPrimitive::new());
    let p1 = Arc::new(MockPrimitive::new());

    primitives.add(p0.clone(), None).unwrap();
    primitives.add(p1.clone(), None).unwrap();
    assert_eq!(primitives.len(), 2);

    assert!(primitives.remove(p0.as_ref()));
    assert_eq!(primitives.len(), 1);
    assert_eq!(primitives.get(0).unwrap().id(), p1.id());

    assert!(primitives.remove(p1.as_ref()));
    assert_eq!(primitives.len(), 0);

    assert!(!primitives.remove(p0.as_ref()));
}

#[test]
fn test_primitive_collection_contains_and_remove_all() {
    let mut primitives = PrimitiveCollection::default();
    let p0 = Arc::new(MockPrimitive::new());
    let p1 = Arc::new(MockPrimitive::new());

    primitives.add(p0.clone(), None).unwrap();
    assert!(primitives.contains(p0.as_ref()));
    assert!(!primitives.contains(p1.as_ref()));

    primitives.remove_all();
    assert_eq!(primitives.len(), 0);
    assert!(!primitives.contains(p0.as_ref()));
}

#[test]
fn test_primitive_collection_two_composites() {
    let p = Arc::new(MockPrimitive::new());

    let mut coll1 = PrimitiveCollection::new(PrimitiveCollectionOptions {
        destroy_primitives: Some(false),
        ..Default::default()
    });
    let mut coll2 = PrimitiveCollection::new(PrimitiveCollectionOptions {
        destroy_primitives: Some(false),
        ..Default::default()
    });

    coll1.add(p.clone(), None).unwrap();
    coll2.add(p.clone(), None).unwrap();

    assert!(coll1.contains(p.as_ref()));
    assert!(coll2.contains(p.as_ref()));

    assert!(coll1.remove(p.as_ref()));
    assert!(!coll1.contains(p.as_ref()));
    assert!(coll2.contains(p.as_ref()));

    assert!(coll2.remove(p.as_ref()));
    assert!(!coll1.contains(p.as_ref()));
    assert!(!coll2.contains(p.as_ref()));
    assert!(!coll1.remove(p.as_ref()));
    assert!(!coll2.remove(p.as_ref()));

    assert!(!p.is_destroyed());
    p.destroy();
    assert!(p.is_destroyed());
}

#[test]
fn test_primitive_collection_raise_and_lower() {
    let mut primitives = PrimitiveCollection::default();
    let p0 = Arc::new(MockPrimitive::new());
    let p1 = Arc::new(MockPrimitive::new());
    let p2 = Arc::new(MockPrimitive::new());

    primitives.add(p0.clone(), None).unwrap();
    primitives.add(p1.clone(), None).unwrap();
    primitives.add(p2.clone(), None).unwrap();

    // raise p0 up one
    primitives.raise(p0.as_ref()).unwrap();
    assert_eq!(primitives.get(0).unwrap().id(), p1.id());
    assert_eq!(primitives.get(1).unwrap().id(), p0.id());
    assert_eq!(primitives.get(2).unwrap().id(), p2.id());

    // raise p1 to top
    primitives.raise_to_top(p1.as_ref()).unwrap();
    assert_eq!(primitives.get(0).unwrap().id(), p0.id());
    assert_eq!(primitives.get(1).unwrap().id(), p2.id());
    assert_eq!(primitives.get(2).unwrap().id(), p1.id());

    // lower p1 down one
    primitives.lower(p1.as_ref()).unwrap();
    assert_eq!(primitives.get(0).unwrap().id(), p0.id());
    assert_eq!(primitives.get(1).unwrap().id(), p1.id());
    assert_eq!(primitives.get(2).unwrap().id(), p2.id());

    // lower p2 to bottom
    primitives.lower_to_bottom(p2.as_ref()).unwrap();
    assert_eq!(primitives.get(0).unwrap().id(), p2.id());
    assert_eq!(primitives.get(1).unwrap().id(), p0.id());
    assert_eq!(primitives.get(2).unwrap().id(), p1.id());

    // error when not in collection
    let alien = Arc::new(MockPrimitive::new());
    assert!(primitives.raise(alien.as_ref()).is_err());
    assert!(primitives.raise_to_top(alien.as_ref()).is_err());
    assert!(primitives.lower(alien.as_ref()).is_err());
    assert!(primitives.lower_to_bottom(alien.as_ref()).is_err());
}

#[test]
fn test_primitive_collection_nesting_and_destruction() {
    let mut parent = PrimitiveCollection::default();
    let mut child = PrimitiveCollection::default();
    let p = Arc::new(MockPrimitive::new());

    child.add(p.clone(), None).unwrap();
    parent.add(Arc::new(child), None).unwrap();

    assert!(!parent.is_destroyed());
    assert!(!p.is_destroyed());

    parent.destroy();
    assert!(parent.is_destroyed());
    assert!(p.is_destroyed());
}

#[test]
fn test_primitive_collection_events() {
    let mut primitives = PrimitiveCollection::default();
    let added_count = Arc::new(AtomicUsize::new(0));
    let removed_count = Arc::new(AtomicUsize::new(0));

    let ac = added_count.clone();
    primitives.add_on_primitive_added(move |_| {
        ac.fetch_add(1, Ordering::SeqCst);
    });

    let rc = removed_count.clone();
    primitives.add_on_primitive_removed(move |_| {
        rc.fetch_add(1, Ordering::SeqCst);
    });

    let p0 = Arc::new(MockPrimitive::new());
    let p1 = Arc::new(MockPrimitive::new());

    primitives.add(p0.clone(), None).unwrap();
    primitives.add(p1.clone(), None).unwrap();
    assert_eq!(added_count.load(Ordering::SeqCst), 2);
    assert_eq!(removed_count.load(Ordering::SeqCst), 0);

    primitives.remove(p0.as_ref());
    assert_eq!(removed_count.load(Ordering::SeqCst), 1);

    primitives.remove_all();
    assert_eq!(removed_count.load(Ordering::SeqCst), 2);
}

#[test]
fn test_primitive_collection_reference_counting_lifecycle() {
    let mut rc1 = PrimitiveCollection::new(PrimitiveCollectionOptions {
        count_references: Some(true),
        destroy_primitives: Some(false),
        ..Default::default()
    });
    let mut rc2 = PrimitiveCollection::new(PrimitiveCollectionOptions {
        count_references: Some(true),
        destroy_primitives: Some(true),
        ..Default::default()
    });

    let p = Arc::new(MockPrimitive::new());
    rc1.add(p.clone(), None).unwrap();
    assert_eq!(PrimitiveCollection::get_reference_count(p.as_ref()), Some(1));

    rc2.add(p.clone(), None).unwrap();
    assert_eq!(PrimitiveCollection::get_reference_count(p.as_ref()), Some(2));

    rc1.remove(p.as_ref());
    assert_eq!(PrimitiveCollection::get_reference_count(p.as_ref()), Some(1));
    assert!(!p.is_destroyed());

    rc2.remove(p.as_ref());
    assert_eq!(PrimitiveCollection::get_reference_count(p.as_ref()), Some(0));
    // Since rc2 has destroy_primitives = true and ref_count reached 0, it should be destroyed
    assert!(p.is_destroyed());
}
