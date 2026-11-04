use cesium_math::AssociativeArray;

#[test]
fn test_associative_array_defaults() {
    let array: AssociativeArray<String, i32> = AssociativeArray::new();
    assert_eq!(array.len(), 0);
    assert!(array.is_empty());
    assert_eq!(array.values(), &[]);
}

#[test]
fn test_associative_array_manipulate() {
    let mut array = AssociativeArray::new();

    assert!(!array.contains(&"key1".to_string()));

    array.set("key1".to_string(), 1);
    array.set("key2".to_string(), 2);
    array.set("key3".to_string(), 3);

    assert_eq!(array.get(&"key1".to_string()), Some(&1));
    assert_eq!(array.get(&"key2".to_string()), Some(&2));
    assert_eq!(array.get(&"key3".to_string()), Some(&3));
    assert_eq!(array.len(), 3);

    assert!(array.contains(&"key1".to_string()));
    assert!(array.contains(&"key2".to_string()));
    assert!(array.contains(&"key3".to_string()));

    let vals = array.values();
    assert!(vals.contains(&1));
    assert!(vals.contains(&2));
    assert!(vals.contains(&3));

    array.set("key2".to_string(), 4);
    assert_eq!(array.len(), 3);
    let vals = array.values();
    assert!(vals.contains(&1));
    assert!(!vals.contains(&2));
    assert!(vals.contains(&4));
    assert!(vals.contains(&3));

    assert!(array.remove(&"key1".to_string()));
    assert_eq!(array.get(&"key1".to_string()), None);
    assert!(!array.contains(&"key1".to_string()));
    assert_eq!(array.len(), 2);
    assert!(!array.remove(&"key1".to_string()));

    array.remove_all();
    assert_eq!(array.len(), 0);
    assert!(array.values().is_empty());
}
