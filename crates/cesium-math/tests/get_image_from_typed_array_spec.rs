use cesium_math::get_image_from_typed_array;

#[test]
fn test_returns_an_image_of_expected_size() {
    let pixel_array_3x2: Vec<u8> = (0..6).flat_map(|_| [0u8, 255, 0, 255]).collect();
    let img = get_image_from_typed_array(&pixel_array_3x2, 3, 2).unwrap();

    assert_eq!(img.width, 3);
    assert_eq!(img.height, 2);
}

#[test]
fn test_fills_the_image_with_expected_colors() {
    let pixel_array_3x2: Vec<u8> = (0..6).flat_map(|_| [0u8, 255, 0, 255]).collect();
    let img = get_image_from_typed_array(&pixel_array_3x2, 3, 2).unwrap();

    assert_eq!(img.data.as_deref(), Some(pixel_array_3x2.as_slice()));
}

#[test]
fn test_errors_when_length_does_not_match_dimensions() {
    let invalid_data = vec![0u8; 10]; // 3x2 requires 24 bytes
    let result = get_image_from_typed_array(&invalid_data, 3, 2);
    assert!(result.is_err());
}
