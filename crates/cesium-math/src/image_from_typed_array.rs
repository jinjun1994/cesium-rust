//! Typed array image construction functions matching CesiumJS `Core/getImageFromTypedArray` and `Core/loadImageFromTypedArray`.

use crate::errors::{DeveloperError, RuntimeError};
use crate::resize_image_to_next_power_of_two::ImageBuffer;

/// Constructs an ImageBuffer from a slice of pixel values (RGBA).
pub fn get_image_from_typed_array(
    typed_array: &[u8],
    width: u32,
    height: u32,
) -> Result<ImageBuffer, DeveloperError> {
    if typed_array.len() != (width as usize * height as usize * 4) {
        return Err(DeveloperError::new(
            "The length of typedArray does not match width * height * 4",
        ));
    }

    Ok(ImageBuffer {
        width,
        height,
        data: Some(typed_array.to_vec()),
    })
}

/// Options for `load_image_from_typed_array`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LoadImageFromTypedArrayOptions {
    pub uint8_array: Vec<u8>,
    pub format: Option<String>,
    pub flip_y: bool,
    pub skip_color_space_conversion: bool,
    pub premultiply_alpha: bool,
}

/// Asynchronously loads an image from a typed array containing encoded image data (e.g. PNG or JPEG).
pub async fn load_image_from_typed_array(
    options: LoadImageFromTypedArrayOptions,
) -> Result<ImageBuffer, RuntimeError> {
    load_image_from_typed_array_sync(options)
}

/// Synchronously loads an image from a typed array containing encoded image data (e.g. PNG or JPEG).
pub fn load_image_from_typed_array_sync(
    options: LoadImageFromTypedArrayOptions,
) -> Result<ImageBuffer, RuntimeError> {
    if options.uint8_array.is_empty() {
        return Err(RuntimeError::new("uint8_array cannot be empty"));
    }

    let format = options.format.as_deref().unwrap_or("");
    let img = if format == "image/png" || options.uint8_array.starts_with(b"\x89PNG\r\n\x1a\n") {
        image::load_from_memory_with_format(&options.uint8_array, image::ImageFormat::Png)
            .map_err(|e| RuntimeError::new(format!("Failed to decode PNG: {:?}", e)))?
    } else if format == "image/jpeg" || options.uint8_array.starts_with(&[0xFF, 0xD8]) {
        image::load_from_memory_with_format(&options.uint8_array, image::ImageFormat::Jpeg)
            .map_err(|e| RuntimeError::new(format!("Failed to decode JPEG: {:?}", e)))?
    } else {
        image::load_from_memory(&options.uint8_array)
            .map_err(|e| RuntimeError::new(format!("Failed to decode image: {:?}", e)))?
    };

    let width = img.width();
    let height = img.height();
    let mut rgba = img.to_rgba8().into_raw();

    if options.flip_y && height > 1 {
        let row_bytes = width as usize * 4;
        let mut temp_row = vec![0u8; row_bytes];
        for y in 0..(height / 2) as usize {
            let top_offset = y * row_bytes;
            let bottom_offset = (height as usize - 1 - y) * row_bytes;
            temp_row.copy_from_slice(&rgba[top_offset..top_offset + row_bytes]);
            rgba.copy_within(bottom_offset..bottom_offset + row_bytes, top_offset);
            rgba[bottom_offset..bottom_offset + row_bytes].copy_from_slice(&temp_row);
        }
    }

    Ok(ImageBuffer {
        width,
        height,
        data: Some(rgba),
    })
}
