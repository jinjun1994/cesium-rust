/// 1x1 transparent PNG bytes corresponding to DiscardEmptyTileImagePolicy.EMPTY_IMAGE data URI:
/// "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII="
pub const EMPTY_IMAGE_BYTES: [u8; 68] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
    0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0x64,
    0x60, 0x00, 0x00, 0x00, 0x06, 0x00, 0x01, 0x20, 0x81, 0xd0, 0x2f, 0x00, 0x00, 0x00, 0x00,
    0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// A policy for discarding tile images that contain no data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiscardEmptyTileImagePolicy;

impl DiscardEmptyTileImagePolicy {
    /// Constant representation of empty image bytes.
    pub const EMPTY_IMAGE: &'static [u8] = &EMPTY_IMAGE_BYTES;

    /// Creates a new `DiscardEmptyTileImagePolicy`.
    pub fn new() -> Self {
        Self
    }

    /// Determines if the discard policy is ready to process images.
    #[inline]
    pub fn is_ready(&self) -> bool {
        true
    }

    /// Given a tile image raw buffer, decide whether to discard that image.
    #[inline]
    pub fn should_discard_image(&self, image_data: &[u8]) -> bool {
        image_data == Self::EMPTY_IMAGE
    }
}
