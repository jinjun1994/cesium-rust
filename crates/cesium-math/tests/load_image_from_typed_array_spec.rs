use cesium_math::{
    load_image_from_typed_array, load_image_from_typed_array_sync, LoadImageFromTypedArrayOptions,
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = Box::pin(fut);
    unsafe fn noop_clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    unsafe fn noop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);
    let raw_waker = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw_waker) };
    let mut cx = Context::from_waker(&waker);
    match Pin::as_mut(&mut fut).poll(&mut cx) {
        Poll::Ready(res) => res,
        Poll::Pending => panic!("Future did not complete immediately"),
    }
}

static BLUE_10X10_PNG: &[u8] = include_bytes!("../../../../cesium/Specs/Data/Images/Blue10x10.png");

#[test]
fn test_can_load_an_image() {
    let options = LoadImageFromTypedArrayOptions {
        uint8_array: BLUE_10X10_PNG.to_vec(),
        format: Some("image/png".to_string()),
        flip_y: false,
        skip_color_space_conversion: false,
        premultiply_alpha: false,
    };

    let image = block_on(load_image_from_typed_array(options)).unwrap();
    assert_eq!(image.width, 10);
    assert_eq!(image.height, 10);
    assert!(image.data.is_some());
    assert_eq!(image.data.unwrap().len(), 10 * 10 * 4);
}

#[test]
fn test_flips_image_only_when_flip_y_is_true() {
    let options_normal = LoadImageFromTypedArrayOptions {
        uint8_array: BLUE_10X10_PNG.to_vec(),
        format: Some("image/png".to_string()),
        flip_y: false,
        ..Default::default()
    };
    let normal_img = load_image_from_typed_array_sync(options_normal).unwrap();

    let options_flipped = LoadImageFromTypedArrayOptions {
        uint8_array: BLUE_10X10_PNG.to_vec(),
        format: Some("image/png".to_string()),
        flip_y: true,
        ..Default::default()
    };
    let flipped_img = load_image_from_typed_array_sync(options_flipped).unwrap();

    assert_eq!(normal_img.width, flipped_img.width);
    assert_eq!(normal_img.height, flipped_img.height);

    let norm_data = normal_img.data.unwrap();
    let flip_data = flipped_img.data.unwrap();
    let row_len = 10 * 4;

    // The top row of flipped image must match the bottom row of normal image
    let top_row_flipped = &flip_data[0..row_len];
    let bottom_row_normal = &norm_data[9 * row_len..10 * row_len];
    assert_eq!(top_row_flipped, bottom_row_normal);
}

#[test]
fn test_errors_on_invalid_image_buffer() {
    let options = LoadImageFromTypedArrayOptions {
        uint8_array: vec![67, 101, 115, 105, 117, 109], // Invalid PNG
        format: Some("image/png".to_string()),
        ..Default::default()
    };
    let result = load_image_from_typed_array_sync(options);
    assert!(result.is_err());
}
