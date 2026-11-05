//! Helper function matching CesiumJS `Core/resizeImageToNextPowerOfTwo`.

use crate::core_math;
use crate::errors::DeveloperError;

/// Image dimensions representing width and height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl From<(u32, u32)> for ImageDimensions {
    fn from(tuple: (u32, u32)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

/// A representation of an image with dimensions and optional pixel buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Option<Vec<u8>>,
}

impl ImageBuffer {
    pub fn from_size(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: None,
        }
    }

    pub fn with_data(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data: Some(data),
        }
    }
}

impl From<(u32, u32)> for ImageBuffer {
    fn from(tuple: (u32, u32)) -> Self {
        Self::from_size(tuple.0, tuple.1)
    }
}

impl From<ImageDimensions> for ImageBuffer {
    fn from(dims: ImageDimensions) -> Self {
        Self::from_size(dims.width, dims.height)
    }
}

/// Resizes an image or image dimensions so that both width and height are powers of 2.
pub fn resize_image_to_next_power_of_two(
    image: impl Into<ImageBuffer>,
) -> Result<ImageBuffer, DeveloperError> {
    let input = image.into();
    let new_width = core_math::next_power_of_two(input.width as f64)?;
    let new_height = core_math::next_power_of_two(input.height as f64)?;

    let resized_data = if let Some(ref data) = input.data {
        if input.width == 0 || input.height == 0 || new_width == 0 || new_height == 0 {
            Some(Vec::new())
        } else {
            let channels = data.len() / (input.width as usize * input.height as usize);
            let mut out = vec![0u8; new_width as usize * new_height as usize * channels];
            for y_out in 0..new_height {
                let y_in = ((y_out as f64 * input.height as f64) / new_height as f64) as u32;
                for x_out in 0..new_width {
                    let x_in = ((x_out as f64 * input.width as f64) / new_width as f64) as u32;
                    let src_idx = (y_in as usize * input.width as usize + x_in as usize) * channels;
                    let dst_idx = (y_out as usize * new_width as usize + x_out as usize) * channels;
                    out[dst_idx..dst_idx + channels]
                        .copy_from_slice(&data[src_idx..src_idx + channels]);
                }
            }
            Some(out)
        }
    } else {
        None
    };

    Ok(ImageBuffer {
        width: new_width,
        height: new_height,
        data: resized_data,
    })
}
