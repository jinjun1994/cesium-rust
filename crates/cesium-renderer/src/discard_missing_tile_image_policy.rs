use cesium_math::Cartesian2;
use thiserror::Error;

/// Errors thrown by `DiscardMissingTileImagePolicy`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DiscardMissingError {
    /// Thrown when `missing_image_url` is not provided.
    #[error("options.missing_image_url is required.")]
    MissingImageUrlRequired,
    /// Thrown when `pixels_to_check` is not provided.
    #[error("options.pixels_to_check is required.")]
    PixelsToCheckRequired,
    /// Thrown when `should_discard_image` is called before policy is ready.
    #[error("shouldDiscardImage must not be called before the discard policy is ready.")]
    NotReady,
}

/// Options for constructing a `DiscardMissingTileImagePolicy`.
#[derive(Debug, Clone)]
pub struct DiscardMissingTileImagePolicyOptions {
    /// The URL or identifier of the known missing image.
    pub missing_image_url: Option<String>,
    /// Array of pixel positions to compare against the missing image.
    pub pixels_to_check: Option<Vec<Cartesian2>>,
    /// If true, discard check is disabled when all checked pixels are transparent.
    pub disable_check_if_all_pixels_are_transparent: bool,
    /// Optional pre-resolved (width, height, rgba_pixels) for testing or synchronous usage.
    pub preloaded_image: Option<(u32, u32, Vec<u8>)>,
}

/// A policy for discarding tile images that match a known missing image.
#[derive(Debug, Clone)]
pub struct DiscardMissingTileImagePolicy {
    pixels_to_check: Vec<Cartesian2>,
    missing_pixels: Option<Vec<u8>>,
    missing_width: u32,
    missing_height: u32,
    is_ready: bool,
}

impl DiscardMissingTileImagePolicy {
    /// Creates a new `DiscardMissingTileImagePolicy`.
    pub fn new(
        options: DiscardMissingTileImagePolicyOptions,
    ) -> Result<Self, DiscardMissingError> {
        let _url = options
            .missing_image_url
            .ok_or(DiscardMissingError::MissingImageUrlRequired)?;
        let pixels_to_check = options
            .pixels_to_check
            .ok_or(DiscardMissingError::PixelsToCheckRequired)?;

        let mut policy = Self {
            pixels_to_check,
            missing_pixels: None,
            missing_width: 0,
            missing_height: 0,
            is_ready: false,
        };

        if let Some((width, height, pixels)) = options.preloaded_image {
            policy.set_missing_image(
                width,
                height,
                pixels,
                options.disable_check_if_all_pixels_are_transparent,
            );
        }

        Ok(policy)
    }

    /// Determines if the discard policy is ready to process images.
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.is_ready
    }

    /// Sets the resolved missing image pixel buffer.
    pub fn set_missing_image(
        &mut self,
        width: u32,
        height: u32,
        pixels: Vec<u8>,
        disable_check_if_all_pixels_are_transparent: bool,
    ) {
        if disable_check_if_all_pixels_are_transparent {
            let mut all_transparent = true;
            for pos in &self.pixels_to_check {
                let x = pos.x as u32;
                let y = pos.y as u32;
                let idx = ((y * width + x) * 4 + 3) as usize;
                if idx < pixels.len() && pixels[idx] > 0 {
                    all_transparent = false;
                    break;
                }
            }
            if all_transparent {
                self.missing_pixels = None;
                self.is_ready = true;
                return;
            }
        }

        self.missing_width = width;
        self.missing_height = height;
        self.missing_pixels = Some(pixels);
        self.is_ready = true;
    }

    /// Given an RGBA image buffer and dimensions, decides whether to discard that image.
    pub fn should_discard_image(
        &self,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<bool, DiscardMissingError> {
        if !self.is_ready {
            return Err(DiscardMissingError::NotReady);
        }

        let missing = match &self.missing_pixels {
            Some(p) => p,
            None => return Ok(false), // disabled check
        };

        for pos in &self.pixels_to_check {
            let x = pos.x as u32;
            let y = pos.y as u32;

            if x >= width || y >= height || x >= self.missing_width || y >= self.missing_height {
                return Ok(false);
            }

            let img_idx = ((y * width + x) * 4) as usize;
            let missing_idx = ((y * self.missing_width + x) * 4) as usize;

            if img_idx + 4 > pixels.len() || missing_idx + 4 > missing.len() {
                return Ok(false);
            }

            if pixels[img_idx..img_idx + 4] != missing[missing_idx..missing_idx + 4] {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
