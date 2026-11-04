use cesium_math::get_string_from_typed_array;

#[test]
fn test_converts_typed_array_to_string() {
    let arr = [67, 101, 115, 105, 117, 109];
    let s = get_string_from_typed_array(&arr, None, None).unwrap();
    assert_eq!(s, "Cesium");

    let empty: [u8; 0] = [];
    let s_empty = get_string_from_typed_array(&empty, None, None).unwrap();
    assert_eq!(s_empty, "");
}

#[test]
fn test_converts_sub_region_of_typed_array() {
    let arr = [67, 101, 115, 105, 117, 109];
    let s = get_string_from_typed_array(&arr, Some(1), Some(3)).unwrap();
    assert_eq!(s, "esi");
}

#[test]
fn test_throws_if_sub_region_exceeds_array_bounds() {
    let arr = [67, 101, 115, 105, 117, 109];
    assert!(get_string_from_typed_array(&arr, Some(3), Some(4)).is_err());
    assert!(get_string_from_typed_array(&arr, Some(10), None).is_err());
}
