use cesium_math::subdivide_array;

#[test]
fn test_splits_evenly_divided_arrays() {
    let values = vec![1, 2, 3, 4];
    let split_values = subdivide_array(&values, 4).unwrap();
    assert_eq!(split_values.len(), 4);
    assert_eq!(split_values[0], vec![1]);
    assert_eq!(split_values[1], vec![2]);
    assert_eq!(split_values[2], vec![3]);
    assert_eq!(split_values[3], vec![4]);
}

#[test]
fn test_splits_unevenly_divided_arrays() {
    let values = vec![1, 2, 3, 4, 5, 6];
    let split_values = subdivide_array(&values, 4).unwrap();
    assert_eq!(split_values.len(), 4);
    assert_eq!(split_values[0], vec![1, 2]);
    assert_eq!(split_values[1], vec![3, 4]);
    assert_eq!(split_values[2], vec![5]);
    assert_eq!(split_values[3], vec![6]);
}

#[test]
fn test_works_with_empty_arrays() {
    let empty: Vec<i32> = vec![];
    let split_values = subdivide_array(&empty, 4).unwrap();
    assert!(split_values.is_empty());
}

#[test]
fn test_errors_with_invalid_number_of_arrays() {
    let empty: Vec<i32> = vec![];
    assert!(subdivide_array(&empty, 0).is_err());
}
