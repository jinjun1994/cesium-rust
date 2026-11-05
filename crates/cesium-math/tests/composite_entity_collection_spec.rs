//! Direct port and verification against CesiumJS `CompositeEntityCollectionSpec.js` (Apache-2.0).

use cesium_math::{CompositeEntityCollection, Entity, EntityCollection};

#[test]
fn test_constructor_has_expected_defaults() {
    let composite = CompositeEntityCollection::new();
    assert!(!composite.id().is_empty());
    assert_eq!(composite.get_collections_length(), 0);
    assert_eq!(composite.values().len(), 0);
}

#[test]
fn test_add_remove_collection_works() {
    let mut entity_collection = EntityCollection::new();
    entity_collection.add(Entity::new()).unwrap();

    let mut entity_collection2 = EntityCollection::new();
    entity_collection2.add(Entity::new()).unwrap();

    let mut composite = CompositeEntityCollection::new();
    composite.add_collection(entity_collection.clone(), None).unwrap();
    assert_eq!(composite.get_collections_length(), 1);
    assert_eq!(composite.values().len(), 1);

    composite.add_collection(entity_collection2.clone(), None).unwrap();
    assert_eq!(composite.get_collections_length(), 2);
    assert_eq!(composite.values().len(), 2);

    assert!(composite.remove_collection(&entity_collection));
    assert_eq!(composite.values().len(), 1);

    assert!(composite.remove_collection(&entity_collection2));
    assert_eq!(composite.values().len(), 0);
    assert_eq!(composite.get_collections_length(), 0);

    assert!(!composite.remove_collection(&entity_collection));
}

#[test]
fn test_add_collection_with_index() {
    let collection1 = EntityCollection::new();
    let collection2 = EntityCollection::new();
    let collection3 = EntityCollection::new();

    let id1 = collection1.id().to_string();
    let id2 = collection2.id().to_string();
    let id3 = collection3.id().to_string();

    let mut composite = CompositeEntityCollection::new();
    composite.add_collection(collection1, None).unwrap();
    composite.add_collection(collection3, None).unwrap();

    composite.add_collection(collection2, Some(1)).unwrap();
    assert_eq!(composite.get_collection(0).unwrap().id(), id1);
    assert_eq!(composite.get_collection(1).unwrap().id(), id2);
    assert_eq!(composite.get_collection(2).unwrap().id(), id3);
}

#[test]
fn test_contains_and_contains_collection() {
    let mut collection = EntityCollection::new();
    let mut composite = CompositeEntityCollection::new();

    assert!(!composite.contains_collection(&collection));
    let entity = collection.get_or_create_entity("asd").unwrap().clone();

    composite.add_collection(collection.clone(), None).unwrap();
    assert!(composite.contains_collection(&collection));
    assert!(composite.contains(&entity));

    let other_entity = Entity::new();
    assert!(!composite.contains(&other_entity));
}

#[test]
fn test_index_of_collection_and_get_collection() {
    let collection1 = EntityCollection::new();
    let collection2 = EntityCollection::new();
    let mut composite = CompositeEntityCollection::new();

    assert_eq!(composite.index_of_collection(&collection1), None);

    composite.add_collection(collection1.clone(), None).unwrap();
    composite.add_collection(collection2.clone(), None).unwrap();

    assert_eq!(composite.index_of_collection(&collection1), Some(0));
    assert_eq!(composite.index_of_collection(&collection2), Some(1));

    assert!(composite.get_collection(0).is_some());
    assert!(composite.get_collection(1).is_some());
    assert!(composite.get_collection(2).is_none());

    composite.remove_collection(&collection1);
    assert_eq!(composite.index_of_collection(&collection2), Some(0));
}

#[test]
fn test_raise_lower_collection() {
    let collection1 = EntityCollection::new();
    let collection2 = EntityCollection::new();
    let collection3 = EntityCollection::new();
    let mut composite = CompositeEntityCollection::new();

    composite.add_collection(collection1.clone(), None).unwrap();
    composite.add_collection(collection2.clone(), None).unwrap();
    composite.add_collection(collection3.clone(), None).unwrap();

    assert_eq!(composite.index_of_collection(&collection2), Some(1));
    composite.raise_collection(&collection2).unwrap();
    assert_eq!(composite.index_of_collection(&collection2), Some(2));

    composite.lower_collection(&collection2).unwrap();
    assert_eq!(composite.index_of_collection(&collection2), Some(1));

    composite.lower_collection(&collection2).unwrap();
    assert_eq!(composite.index_of_collection(&collection2), Some(0));

    // Lowering at bottom does nothing
    composite.lower_collection(&collection2).unwrap();
    assert_eq!(composite.index_of_collection(&collection2), Some(0));

    composite.raise_collection_to_top(&collection2).unwrap();
    assert_eq!(composite.index_of_collection(&collection2), Some(2));

    composite.lower_collection_to_bottom(&collection2).unwrap();
    assert_eq!(composite.index_of_collection(&collection2), Some(0));
}

#[test]
fn test_recomposite_merges_with_precedence() {
    let mut col1 = EntityCollection::new();
    let mut col2 = EntityCollection::new();

    let mut e1 = Entity::with_id("test");
    e1.name = Some("First".to_string());
    col1.add(e1).unwrap();

    let mut e2 = Entity::with_id("test");
    e2.name = Some("Second".to_string());
    col2.add(e2).unwrap();

    let mut composite = CompositeEntityCollection::new();
    composite.add_collection(col1, None).unwrap();
    composite.add_collection(col2, None).unwrap();

    // In CompositeEntityCollection, later collections override earlier ones
    assert_eq!(composite.values().len(), 1);
    assert_eq!(composite.values()[0].name.as_deref(), Some("Second"));
}
