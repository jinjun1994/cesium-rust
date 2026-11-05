use cesium_math::{
    is_power_of_two, next_power_of_two, resize_image_to_next_power_of_two, ImageBuffer,
    ImageDimensions,
};

#[test]
fn test_resizes_an_image_so_its_width_is_a_power_of_two() {
    let resized = resize_image_to_next_power_of_two((3, 2)).unwrap();
    assert_eq!(resized.width, 4);
    assert_eq!(resized.height, 2);
}

#[test]
fn test_resizes_an_image_so_its_height_is_a_power_of_two() {
    let resized = resize_image_to_next_power_of_two((4, 3)).unwrap();
    assert_eq!(resized.width, 4);
    assert_eq!(resized.height, 4);
}

#[test]
fn test_keeps_original_image_size_if_both_dimensions_are_already_powers_of_two() {
    let resized = resize_image_to_next_power_of_two((8, 8)).unwrap();
    assert_eq!(resized.width, 8);
    assert_eq!(resized.height, 8);
}

#[test]
fn test_image_dimensions_struct_input() {
    let dims = ImageDimensions::new(5, 7);
    let resized = resize_image_to_next_power_of_two(dims).unwrap();
    assert_eq!(resized.width, 8);
    assert_eq!(resized.height, 8);
}

#[test]
fn test_resamples_pixel_buffer_when_present() {
    // 2x2 image with 1 channel grayscale
    let data = vec![10u8, 20, 30, 40];
    let img = ImageBuffer::with_data(2, 2, data);
    let resized = resize_image_to_next_power_of_two(img).unwrap();
    assert_eq!(resized.width, 2);
    assert_eq!(resized.height, 2);
    assert_eq!(resized.data.unwrap(), vec![10, 20, 30, 40]);

    // 1x2 image resizes to 1x2 or 2x2: (1, 3) -> (1, 4)
    let img_non_pot = ImageBuffer::with_data(1, 3, vec![100, 150, 200]);
    let resized_non_pot = resize_image_to_next_power_of_two(img_non_pot).unwrap();
    assert_eq!(resized_non_pot.width, 1);
    assert_eq!(resized_non_pot.height, 4);
    assert_eq!(resized_non_pot.data.unwrap().len(), 4);
}

#[test]
fn test_next_power_of_two_math_spec() {
    assert_eq!(next_power_of_two(0.0).unwrap(), 0);
    assert_eq!(next_power_of_two(1.0).unwrap(), 1);
    assert_eq!(next_power_of_two(2.0).unwrap(), 2);
    assert_eq!(next_power_of_two(3.0).unwrap(), 4);
    assert_eq!(next_power_of_two(257.0).unwrap(), 512);
    assert_eq!(next_power_of_two(512.0).unwrap(), 512);
    assert_eq!(next_power_of_two(1023.0).unwrap(), 1024);
    assert_eq!(next_power_of_two(1073741825.0).unwrap(), 2147483648);
    assert_eq!(next_power_of_two(2147483647.0).unwrap(), 2147483648);
    assert_eq!(next_power_of_two(2147483648.0).unwrap(), 2147483648);

    assert!(next_power_of_two(-1.0).is_err());
    assert!(next_power_of_two(2147483649.0).is_err());
}

#[test]
fn test_is_power_of_two() {
    assert!(!is_power_of_two(0));
    assert!(is_power_of_two(1));
    assert!(is_power_of_two(2));
    assert!(!is_power_of_two(3));
    assert!(is_power_of_two(4));
    assert!(is_power_of_two(512));
    assert!(!is_power_of_two(513));
}
