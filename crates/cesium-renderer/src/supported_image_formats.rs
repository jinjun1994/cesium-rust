//! SupportedImageFormats matching CesiumJS `Scene/SupportedImageFormats.js` (Apache-2.0).

/// Construction options for [`SupportedImageFormats`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SupportedImageFormatsOptions {
    /// Whether the browser supports WebP images. Default is `false`.
    pub webp: Option<bool>,
    /// Whether the browser supports compressed textures required for KTX2 + Basis Universal images. Default is `false`.
    pub basis: Option<bool>,
}

/// Image formats supported by the runtime or browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct SupportedImageFormats {
    /// Whether WebP images are supported.
    pub webp: bool,
    /// Whether KTX2 + Basis Universal compressed textures are supported.
    pub basis: bool,
}

impl SupportedImageFormats {
    /// Creates a new `SupportedImageFormats` instance with default or provided options.
    pub fn new(options: Option<SupportedImageFormatsOptions>) -> Self {
        let options = options.unwrap_or_default();
        Self {
            webp: options.webp.unwrap_or(false),
            basis: options.basis.unwrap_or(false),
        }
    }

    /// Creates a new `SupportedImageFormats` with explicit flags.
    pub const fn from_flags(webp: bool, basis: bool) -> Self {
        Self { webp, basis }
    }

    /// Creates a new `SupportedImageFormats` with basis and webp flags.
    pub const fn with_flags(basis: bool, webp: bool) -> Self {
        Self { basis, webp }
    }
}
