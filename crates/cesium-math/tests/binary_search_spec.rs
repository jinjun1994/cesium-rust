use cesium_math::{binary_search, binary_search_cesium};
use std::cmp::Ordering;

fn compare_f64(a: &f64, b: &f64) -> Ordering {
    a.partial_cmp(b).unwrap_or(Ordering::Equal)
}

#[test]
fn test_binary_search_for_zero() {
    let array = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let to_find = 0.0;
    let index = binary_search_cesium(&array, &to_find, compare_f64);
    assert_eq!(index, 0);

    let idx = binary_search(&array, &to_find, compare_f64).unwrap();
    assert_eq!(idx, 0);
}

#[test]
fn test_binary_search_for_item_in_the_list() {
    let array = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let to_find = 7.0;
    let index = binary_search_cesium(&array, &to_find, compare_f64);
    assert_eq!(index, 7);

    let idx = binary_search(&array, &to_find, compare_f64).unwrap();
    assert_eq!(idx, 7);
}

#[test]
fn test_binary_search_for_item_in_between_two_items() {
    let array = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let to_find = 3.5;
    let index = binary_search_cesium(&array, &to_find, compare_f64);
    assert_eq!(!index, 4);

    let err = binary_search(&array, &to_find, compare_f64).unwrap_err();
    assert_eq!(err, 4);
}

#[test]
fn test_binary_search_for_item_before_all_items() {
    let array = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let to_find = -2.0;
    let index = binary_search_cesium(&array, &to_find, compare_f64);
    assert_eq!(!index, 0);

    let err = binary_search(&array, &to_find, compare_f64).unwrap_err();
    assert_eq!(err, 0);
}

#[test]
fn test_binary_search_for_item_after_all_items() {
    let array = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let to_find = 12.0;
    let index = binary_search_cesium(&array, &to_find, compare_f64);
    assert_eq!(!index, 8);

    let err = binary_search(&array, &to_find, compare_f64).unwrap_err();
    assert_eq!(err, 8);
}
