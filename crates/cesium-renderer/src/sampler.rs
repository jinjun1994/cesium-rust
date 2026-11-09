//! Sampler, TextureWrap, TextureMinificationFilter, TextureMagnificationFilter matching CesiumJS `Renderer/Sampler.js` (Apache-2.0).

use cesium_math::DeveloperError;

/// Texture coordinate wrap modes matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureWrap {
    #[default]
    ClampToEdge = 0x812F,
    Repeat = 0x2901,
    MirroredRepeat = 0x8370,
}

impl TextureWrap {
    /// Validates whether the given raw value or mode is valid.
    pub fn is_valid(self) -> bool {
        matches!(
            self,
            Self::ClampToEdge | Self::Repeat | Self::MirroredRepeat
        )
    }

    /// Parses from string representation.
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name {
            "CLAMP_TO_EDGE" | "clamp_to_edge" => Some(Self::ClampToEdge),
            "REPEAT" | "repeat" => Some(Self::Repeat),
            "MIRRORED_REPEAT" | "mirrored_repeat" => Some(Self::MirroredRepeat),
            _ => None,
        }
    }

    /// Converts raw GL enum integer to TextureWrap.
    pub fn from_gl_enum(val: u32) -> Option<Self> {
        match val {
            0x812F => Some(Self::ClampToEdge),
            0x2901 => Some(Self::Repeat),
            0x8370 => Some(Self::MirroredRepeat),
            _ => None,
        }
    }
}

/// Minification filters matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureMinificationFilter {
    Nearest = 0x2600,
    #[default]
    Linear = 0x2601,
    NearestMipmapNearest = 0x2700,
    LinearMipmapNearest = 0x2701,
    NearestMipmapLinear = 0x2702,
    LinearMipmapLinear = 0x2703,
}

impl TextureMinificationFilter {
    /// Validates whether the given minification filter is valid.
    pub fn is_valid(self) -> bool {
        matches!(
            self,
            Self::Nearest
                | Self::Linear
                | Self::NearestMipmapNearest
                | Self::LinearMipmapNearest
                | Self::NearestMipmapLinear
                | Self::LinearMipmapLinear
        )
    }

    /// Parses from string representation.
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name {
            "NEAREST" | "nearest" => Some(Self::Nearest),
            "LINEAR" | "linear" => Some(Self::Linear),
            "NEAREST_MIPMAP_NEAREST" | "nearest_mipmap_nearest" => {
                Some(Self::NearestMipmapNearest)
            }
            "LINEAR_MIPMAP_NEAREST" | "linear_mipmap_nearest" => {
                Some(Self::LinearMipmapNearest)
            }
            "NEAREST_MIPMAP_LINEAR" | "nearest_mipmap_linear" => {
                Some(Self::NearestMipmapLinear)
            }
            "LINEAR_MIPMAP_LINEAR" | "linear_mipmap_linear" => {
                Some(Self::LinearMipmapLinear)
            }
            _ => None,
        }
    }

    /// Converts raw GL enum integer to TextureMinificationFilter.
    pub fn from_gl_enum(val: u32) -> Option<Self> {
        match val {
            0x2600 => Some(Self::Nearest),
            0x2601 => Some(Self::Linear),
            0x2700 => Some(Self::NearestMipmapNearest),
            0x2701 => Some(Self::LinearMipmapNearest),
            0x2702 => Some(Self::NearestMipmapLinear),
            0x2703 => Some(Self::LinearMipmapLinear),
            _ => None,
        }
    }

    /// Whether this filter requires mipmaps.
    pub fn requires_mipmap(self) -> bool {
        matches!(
            self,
            Self::NearestMipmapNearest
                | Self::LinearMipmapNearest
                | Self::NearestMipmapLinear
                | Self::LinearMipmapLinear
        )
    }
}

/// Magnification filters matching WebGLConstants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureMagnificationFilter {
    Nearest = 0x2600,
    #[default]
    Linear = 0x2601,
}

impl TextureMagnificationFilter {
    /// Validates whether the given magnification filter is valid.
    pub fn is_valid(self) -> bool {
        matches!(self, Self::Nearest | Self::Linear)
    }

    /// Parses from string representation.
    pub fn from_str_name(name: &str) -> Option<Self> {
        match name {
            "NEAREST" | "nearest" => Some(Self::Nearest),
            "LINEAR" | "linear" => Some(Self::Linear),
            _ => None,
        }
    }

    /// Converts raw GL enum integer to TextureMagnificationFilter.
    pub fn from_gl_enum(val: u32) -> Option<Self> {
        match val {
            0x2600 => Some(Self::Nearest),
            0x2601 => Some(Self::Linear),
            _ => None,
        }
    }
}

/// Options for configuring a texture Sampler.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SamplerOptions {
    pub wrap_r: Option<TextureWrap>,
    pub wrap_s: Option<TextureWrap>,
    pub wrap_t: Option<TextureWrap>,
    pub minification_filter: Option<TextureMinificationFilter>,
    pub magnification_filter: Option<TextureMagnificationFilter>,
    pub maximum_anisotropy: Option<f64>,
}

/// Immutable texture sampler descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct Sampler {
    pub wrap_r: TextureWrap,
    pub wrap_s: TextureWrap,
    pub wrap_t: TextureWrap,
    pub minification_filter: TextureMinificationFilter,
    pub magnification_filter: TextureMagnificationFilter,
    pub maximum_anisotropy: f64,
}

impl Default for Sampler {
    fn default() -> Self {
        Self {
            wrap_r: TextureWrap::ClampToEdge,
            wrap_s: TextureWrap::ClampToEdge,
            wrap_t: TextureWrap::ClampToEdge,
            minification_filter: TextureMinificationFilter::Linear,
            magnification_filter: TextureMagnificationFilter::Linear,
            maximum_anisotropy: 1.0,
        }
    }
}

impl Sampler {
    /// Sampler with nearest filtering and clamp to edge.
    pub const NEAREST: Self = Self {
        wrap_r: TextureWrap::ClampToEdge,
        wrap_s: TextureWrap::ClampToEdge,
        wrap_t: TextureWrap::ClampToEdge,
        minification_filter: TextureMinificationFilter::Nearest,
        magnification_filter: TextureMagnificationFilter::Nearest,
        maximum_anisotropy: 1.0,
    };

    /// Creates a new Sampler with validation of options.
    pub fn new(options: SamplerOptions) -> Result<Self, DeveloperError> {
        let maximum_anisotropy = options.maximum_anisotropy.unwrap_or(1.0);
        if maximum_anisotropy < 1.0 {
            return Err(DeveloperError::new(
                "maximumAnisotropy must be greater than or equal to 1.0.",
            ));
        }

        Ok(Self {
            wrap_r: options.wrap_r.unwrap_or(TextureWrap::ClampToEdge),
            wrap_s: options.wrap_s.unwrap_or(TextureWrap::ClampToEdge),
            wrap_t: options.wrap_t.unwrap_or(TextureWrap::ClampToEdge),
            minification_filter: options
                .minification_filter
                .unwrap_or(TextureMinificationFilter::Linear),
            magnification_filter: options
                .magnification_filter
                .unwrap_or(TextureMagnificationFilter::Linear),
            maximum_anisotropy,
        })
    }

    /// Compares two Samplers for equality.
    pub fn equals(&self, other: &Self) -> bool {
        self == other
    }
}
