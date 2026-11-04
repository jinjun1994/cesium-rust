use cesium_math::ManagedArray;

#[test]
fn test_managed_array_constructor_default() {
    let array: ManagedArray<f64> = ManagedArray::default();
    assert_eq!(array.len(), 0);
}

#[test]
fn test_managed_array_constructor_initializes_length() {
    let array: ManagedArray<f64> = ManagedArray::new(10);
    assert_eq!(array.len(), 10);
    assert_eq!(array.values().len(), 10);
}

#[test]
fn test_managed_array_get_and_set() {
    let length = 10;
    let mut array = ManagedArray::new(length);
    for i in 0..length {
        array.set(i, (i * i) as i32);
    }
    for i in 0..length {
        let expected = (i * i) as i32;
        assert_eq!(array.get(i), Some(&expected));
        assert_eq!(array.values()[i], Some(expected));
    }
}

#[test]
fn test_managed_array_get_returns_none_if_out_of_bounds() {
    let mut array: ManagedArray<f64> = ManagedArray::new(0);
    array.reserve(5);
    assert_eq!(array.values().len(), 5);
    assert_eq!(array.get(5), None);
}

#[test]
fn test_managed_array_set_resizes() {
    let mut array: ManagedArray<String> = ManagedArray::default();
    array.set(0, "a".to_string());
    assert_eq!(array.len(), 1);
    array.set(5, "b".to_string());
    assert_eq!(array.len(), 6);
    array.set(2, "c".to_string());
    assert_eq!(array.len(), 6);
}

#[test]
fn test_managed_array_peek() {
    let mut array: ManagedArray<i32> = ManagedArray::default();
    assert_eq!(array.peek(), None);
    array.push(0);
    assert_eq!(array.peek(), Some(&0));
    array.push(1);
    array.push(2);
    assert_eq!(array.peek(), Some(&2));
}

#[test]
fn test_managed_array_push() {
    let mut array: ManagedArray<f64> = ManagedArray::default();
    let length = 10;
    for i in 0..length {
        let val = (i as f64) * 1.5;
        array.push(val);
        assert_eq!(array.len(), i + 1);
        assert_eq!(array.values().len(), i + 1);
        assert_eq!(array.get(i), Some(&val));
        assert_eq!(array.values()[i], Some(val));
    }
}

#[test]
fn test_managed_array_pop() {
    let length = 10;
    let mut array = ManagedArray::new(length);
    for i in 0..length {
        array.set(i, (i as f64) * 2.5);
    }
    for i in (0..length).rev() {
        let val = *array.get(i).unwrap();
        assert_eq!(array.pop(), Some(val));
        assert_eq!(array.len(), i);
        assert_eq!(array.values().len(), length);
    }
    assert_eq!(array.pop(), None);
}

#[test]
fn test_managed_array_pop_removes_trailing_references() {
    let length = 10;
    let mut array = ManagedArray::new(length);
    array.set(0, 1.0);
    array.set(1, 2.0);
    array.set(2, 3.0);
    array.pop();
    array.pop();
    for i in array.len()..array.values().len() {
        assert_eq!(array.values()[i], None);
    }
}

#[test]
fn test_managed_array_reserve() {
    let mut array: ManagedArray<i32> = ManagedArray::new(2);
    array.reserve(10);
    assert_eq!(array.values().len(), 10);
    assert_eq!(array.len(), 2);
    array.reserve(20);
    assert_eq!(array.values().len(), 20);
    assert_eq!(array.len(), 2);
    array.reserve(5);
    assert_eq!(array.values().len(), 20);
    assert_eq!(array.len(), 2);
}

#[test]
fn test_managed_array_resize() {
    let mut array: ManagedArray<i32> = ManagedArray::new(2);
    array.resize(10);
    assert_eq!(array.values().len(), 10);
    assert_eq!(array.len(), 10);
    array.resize(20);
    assert_eq!(array.values().len(), 20);
    assert_eq!(array.len(), 20);
    array.resize(5);
    assert_eq!(array.values().len(), 20);
    assert_eq!(array.len(), 5);
}

#[test]
fn test_managed_array_resize_removes_trailing_references() {
    let length = 10;
    let mut array = ManagedArray::new(length);
    array.set(0, 10.0);
    array.set(1, 20.0);
    array.set(2, 30.0);
    array.resize(1);
    for i in array.len()..array.values().len() {
        assert_eq!(array.values()[i], None);
    }
}

#[test]
fn test_managed_array_trim() {
    let mut array: ManagedArray<i32> = ManagedArray::new(2);
    array.reserve(10);
    assert_eq!(array.len(), 2);
    assert_eq!(array.values().len(), 10);
    array.trim(None);
    assert_eq!(array.values().len(), 2);
    array.trim(Some(5));
    assert_eq!(array.len(), 2);
    assert_eq!(array.values().len(), 5);
    array.trim(Some(3));
    assert_eq!(array.len(), 2);
    assert_eq!(array.values().len(), 3);
}
