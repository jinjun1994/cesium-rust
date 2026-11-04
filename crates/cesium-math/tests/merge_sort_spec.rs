use cesium_math::{merge_sort, BoundingSphere};
use glam::DVec3;

#[test]
fn test_merge_sort_sorts() {
    let mut array = [0, 9, 1, 8, 2, 7, 3, 6, 4, 5];
    merge_sort(&mut array, |a, b| a.cmp(b));
    let expected = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    assert_eq!(array, expected);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    id: usize,
    value: i32,
}

#[test]
fn test_merge_sort_is_stable() {
    let mut array = [
        Item { id: 0, value: 5 },
        Item { id: 1, value: 10 },
        Item { id: 2, value: 5 },
        Item { id: 3, value: 0 },
    ];
    merge_sort(&mut array, |a, b| a.value.cmp(&b.value));
    assert_eq!(array[0].id, 3);
    assert_eq!(array[1].id, 0);
    assert_eq!(array[2].id, 2);
    assert_eq!(array[3].id, 1);
}

#[test]
fn test_merge_sort_with_user_defined_object() {
    let mut array = [
        BoundingSphere::new(DVec3::new(-2.0, 0.0, 0.0), 1.0),
        BoundingSphere::new(DVec3::new(-1.0, 0.0, 0.0), 1.0),
        BoundingSphere::new(DVec3::new(-3.0, 0.0, 0.0), 1.0),
    ];
    let position = DVec3::ZERO;
    merge_sort(&mut array, |a, b| {
        let da = a.distance_squared_to(position);
        let db = b.distance_squared_to(position);
        db.partial_cmp(&da).unwrap()
    });

    assert_eq!(array[0].center, DVec3::new(-3.0, 0.0, 0.0));
    assert_eq!(array[1].center, DVec3::new(-2.0, 0.0, 0.0));
    assert_eq!(array[2].center, DVec3::new(-1.0, 0.0, 0.0));
}
