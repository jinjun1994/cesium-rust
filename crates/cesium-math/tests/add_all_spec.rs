use cesium_math::add_all_to_array;

#[test]
fn test_works_for_basic_arrays() {
    let source = vec![3, 4, 5];
    let mut target = vec![0, 1, 2];
    add_all_to_array(&mut target, Some(&source));
    assert_eq!(target, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn test_works_for_none_source() {
    let mut target = vec![0, 1, 2];
    add_all_to_array::<i32>(&mut target, None);
    assert_eq!(target, vec![0, 1, 2]);
}

#[test]
fn test_works_for_large_arrays() {
    let source = vec![42u32; 200_000];
    let mut target = vec![42u32; 200_000];
    let expected = vec![42u32; 400_000];
    add_all_to_array(&mut target, Some(&source));
    assert_eq!(target, expected);
}
