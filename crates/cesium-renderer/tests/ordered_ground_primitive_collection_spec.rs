//! OrderedGroundPrimitiveCollectionSpec matching CesiumJS `Specs/Scene/OrderedGroundPrimitiveCollectionSpec.js` (Apache-2.0).

use std::sync::{Arc, Mutex};

use cesium_renderer::ordered_ground_primitive_collection::{
    GroundPrimitive, MockGroundPrimitive, OrderedGroundPrimitiveCollection,
};

#[test]
fn test_constructs() {
    let collection = OrderedGroundPrimitiveCollection::new();
    assert_eq!(collection.length(), 0);
    assert!(collection.is_empty());
    assert!(collection.show);
    assert!(!collection.is_destroyed());
}

#[test]
fn test_adds_a_primitive() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p = Arc::new(MockGroundPrimitive::new());
    collection.add(p, None).unwrap();
    assert_eq!(collection.length(), 1);
    assert!(!collection.is_empty());
}

#[test]
fn test_add_handles_multiple_z_indexes() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p3: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p4: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());

    collection.add(p1.clone(), None).unwrap();
    collection.add(p2.clone(), Some(0)).unwrap();
    collection.add(p3.clone(), Some(2)).unwrap();
    collection.add(p4.clone(), Some(1)).unwrap();

    assert_eq!(collection.length(), 4);

    let array = collection.collections_array();
    assert_eq!(array.len(), 3);

    // z-index 0
    assert_eq!(array[0].z_index, 0);
    assert_eq!(array[0].len(), 2);
    assert_eq!(array[0].get(0).unwrap().id(), p1.id());
    assert_eq!(array[0].get(1).unwrap().id(), p2.id());

    // z-index 1
    assert_eq!(array[1].z_index, 1);
    assert_eq!(array[1].len(), 1);
    assert_eq!(array[1].get(0).unwrap().id(), p4.id());

    // z-index 2
    assert_eq!(array[2].z_index, 2);
    assert_eq!(array[2].len(), 1);
    assert_eq!(array[2].get(0).unwrap().id(), p3.id());
}

#[test]
fn test_add_works_with_negative_z_indexes() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p3: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p4: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());

    collection.add(p1.clone(), Some(0)).unwrap();
    collection.add(p2.clone(), Some(-3)).unwrap();
    collection.add(p3.clone(), Some(-1)).unwrap();
    collection.add(p4.clone(), Some(-3)).unwrap();

    assert_eq!(collection.length(), 4);

    let array = collection.collections_array();
    assert_eq!(array.len(), 3);

    // z-index -3
    assert_eq!(array[0].z_index, -3);
    assert_eq!(array[0].len(), 2);
    assert_eq!(array[0].get(0).unwrap().id(), p2.id());
    assert_eq!(array[0].get(1).unwrap().id(), p4.id());

    // z-index -1
    assert_eq!(array[1].z_index, -1);
    assert_eq!(array[1].len(), 1);
    assert_eq!(array[1].get(0).unwrap().id(), p3.id());

    // z-index 0
    assert_eq!(array[2].z_index, 0);
    assert_eq!(array[2].len(), 1);
    assert_eq!(array[2].get(0).unwrap().id(), p1.id());
}

#[test]
fn test_set_adds_primitive_if_not_in_collection() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    collection.set(&p, 3).unwrap();
    assert_eq!(collection.length(), 1);
    assert!(collection.contains(&p));
}

#[test]
fn test_set_changes_primitive_index() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p3: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p4: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());

    collection.add(p1.clone(), Some(0)).unwrap();
    collection.add(p2.clone(), Some(0)).unwrap();
    collection.add(p3.clone(), Some(2)).unwrap();
    collection.add(p4.clone(), Some(1)).unwrap();
    assert_eq!(collection.length(), 4);

    collection.set(&p4, 2).unwrap();
    assert_eq!(collection.length(), 4);

    let array = collection.collections_array();
    assert_eq!(array.len(), 2);

    assert_eq!(array[0].z_index, 0);
    assert_eq!(array[0].len(), 2);
    assert_eq!(array[0].get(0).unwrap().id(), p1.id());
    assert_eq!(array[0].get(1).unwrap().id(), p2.id());

    assert_eq!(array[1].z_index, 2);
    assert_eq!(array[1].len(), 2);
    assert_eq!(array[1].get(0).unwrap().id(), p3.id());
    assert_eq!(array[1].get(1).unwrap().id(), p4.id());
}

#[test]
fn test_set_works_with_negative_indexes() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());

    collection.add(p1.clone(), Some(0)).unwrap();
    collection.add(p2.clone(), Some(0)).unwrap();
    assert_eq!(collection.length(), 2);

    collection.set(&p2, -1).unwrap();

    let array = collection.collections_array();
    assert_eq!(array.len(), 2);

    assert_eq!(array[0].z_index, -1);
    assert_eq!(array[0].len(), 1);
    assert_eq!(array[0].get(0).unwrap().id(), p2.id());

    assert_eq!(array[1].z_index, 0);
    assert_eq!(array[1].len(), 1);
    assert_eq!(array[1].get(0).unwrap().id(), p1.id());
}

#[test]
fn test_removes_a_primitive() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    collection.add(p.clone(), None).unwrap();

    let removed = collection.remove(&p, false);
    assert!(removed);
    assert_eq!(collection.length(), 0);
    assert_eq!(collection.collections_array().len(), 0);
    assert!(p.is_destroyed());
}

#[test]
fn test_removes_handles_multiple_z_indexes() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p3: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p4: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());

    collection.add(p1.clone(), Some(0)).unwrap();
    collection.add(p2.clone(), Some(2)).unwrap();
    collection.add(p3.clone(), Some(2)).unwrap();
    collection.add(p4.clone(), Some(1)).unwrap();
    assert_eq!(collection.length(), 4);

    assert_eq!(collection.collections_array().len(), 3);
    collection.remove(&p3, false);
    assert_eq!(collection.collections_array().len(), 3);

    collection.remove(&p2, false);
    assert_eq!(collection.collections_array().len(), 2);

    let array = collection.collections_array();
    assert_eq!(array[0].get(0).unwrap().id(), p1.id());
    assert_eq!(array[1].get(0).unwrap().id(), p4.id());
}

#[test]
fn test_remove_all_removes_and_destroys_all_primitives() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p3: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p4: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());

    collection.add(p1.clone(), Some(0)).unwrap();
    collection.add(p2.clone(), Some(0)).unwrap();
    collection.add(p3.clone(), Some(2)).unwrap();
    collection.add(p4.clone(), Some(1)).unwrap();
    assert_eq!(collection.length(), 4);

    collection.remove_all();
    assert_eq!(collection.length(), 0);
    assert_eq!(collection.collections_array().len(), 0);

    assert!(p1.is_destroyed());
    assert!(p2.is_destroyed());
    assert!(p3.is_destroyed());
    assert!(p4.is_destroyed());
}

#[test]
fn test_contains_primitive() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());

    collection.add(p1.clone(), None).unwrap();
    assert!(collection.contains(&p1));
    assert!(!collection.contains(&p2));
}

#[test]
fn test_update_is_called_in_correct_order() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let update_order = Arc::new(Mutex::new(Vec::new()));

    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::with_logger(update_order.clone()));
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::with_logger(update_order.clone()));
    let p3: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::with_logger(update_order.clone()));
    let p4: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::with_logger(update_order.clone()));

    collection.add(p1.clone(), Some(0)).unwrap();
    collection.add(p2.clone(), Some(3)).unwrap();
    collection.add(p3.clone(), Some(1)).unwrap();
    collection.add(p4.clone(), Some(2)).unwrap();

    collection.update();
    {
        let order = update_order.lock().unwrap();
        assert_eq!(*order, vec![p1.id(), p3.id(), p4.id(), p2.id()]);
    }

    update_order.lock().unwrap().clear();
    collection.set(&p1, 4).unwrap();
    collection.update();
    {
        let order = update_order.lock().unwrap();
        assert_eq!(*order, vec![p3.id(), p4.id(), p2.id(), p1.id()]);
    }

    update_order.lock().unwrap().clear();
    collection.set(&p2, 0).unwrap();
    collection.update();
    {
        let order = update_order.lock().unwrap();
        assert_eq!(*order, vec![p2.id(), p3.id(), p4.id(), p1.id()]);
    }

    update_order.lock().unwrap().clear();
    collection.set(&p4, -1).unwrap();
    collection.update();
    {
        let order = update_order.lock().unwrap();
        assert_eq!(*order, vec![p4.id(), p2.id(), p3.id(), p1.id()]);
    }
}

#[test]
fn test_update_is_not_called_when_show_is_false() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    let update_order = Arc::new(Mutex::new(Vec::new()));

    let p1: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::with_logger(update_order.clone()));
    let p2: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::with_logger(update_order.clone()));

    collection.add(p1, Some(0)).unwrap();
    collection.add(p2, Some(3)).unwrap();

    collection.show = false;
    collection.update();
    assert!(update_order.lock().unwrap().is_empty());
}

#[test]
fn test_destroys() {
    let mut collection = OrderedGroundPrimitiveCollection::new();
    assert!(!collection.is_destroyed());
    collection.destroy();
    assert!(collection.is_destroyed());

    // Calling add or set on destroyed collection returns DeveloperError
    let p: Arc<dyn GroundPrimitive> = Arc::new(MockGroundPrimitive::new());
    assert!(collection.add(p.clone(), None).is_err());
    assert!(collection.set(&p, 1).is_err());
}
