//! Pixel format and datatype representations matching CesiumJS `Core/PixelFormat` and `Core/PixelDatatype`.
//! Adapted from CesiumJS (Apache-2.0).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PixelDatatype {
    UnsignedByte = 5121,
    UnsignedShort = 5123,
    UnsignedInt = 5125,
    Float = 5126,
    HalfFloat = 0x8D61,
    UnsignedInt24_8 = 0x84FA,
    UnsignedShort4_4_4_4 = 0x8033,
    UnsignedShort5_5_5_1 = 0x8034,
    UnsignedShort5_6_5 = 0x8363,
}

impl PixelDatatype {
    pub const UNSIGNED_BYTE: Self = Self::UnsignedByte;
    pub const UNSIGNED_SHORT: Self = Self::UnsignedShort;
    pub const UNSIGNED_INT: Self = Self::UnsignedInt;
    pub const FLOAT: Self = Self::Float;
    pub const HALF_FLOAT: Self = Self::HalfFloat;
    pub const UNSIGNED_INT_24_8: Self = Self::UnsignedInt24_8;
    pub const UNSIGNED_SHORT_4_4_4_4: Self = Self::UnsignedShort4_4_4_4;
    pub const UNSIGNED_SHORT_5_5_5_1: Self = Self::UnsignedShort5_5_5_1;
    pub const UNSIGNED_SHORT_5_6_5: Self = Self::UnsignedShort5_6_5;

    pub const fn size_in_bytes(&self) -> usize {
        match self {
            Self::UnsignedByte => 1,
            Self::UnsignedShort
            | Self::UnsignedShort4_4_4_4
            | Self::UnsignedShort5_5_5_1
            | Self::UnsignedShort5_6_5
            | Self::HalfFloat => 2,
            Self::UnsignedInt | Self::Float | Self::UnsignedInt24_8 => 4,
        }
    }

    pub const fn is_packed(&self) -> bool {
        matches!(
            self,
            Self::UnsignedInt24_8
                | Self::UnsignedShort4_4_4_4
                | Self::UnsignedShort5_5_5_1
                | Self::UnsignedShort5_6_5
        )
    }

    pub const fn to_webgl_constant(&self, webgl2: bool) -> u32 {
        match self {
            Self::UnsignedByte => 0x1401,
            Self::UnsignedShort => 0x1403,
            Self::UnsignedInt => 0x1405,
            Self::Float => 0x1406,
            Self::HalfFloat => {
                if webgl2 {
                    0x140B
                } else {
                    0x8D61
                }
            }
            Self::UnsignedInt24_8 => 0x84FA,
            Self::UnsignedShort4_4_4_4 => 0x8033,
            Self::UnsignedShort5_5_5_1 => 0x8034,
            Self::UnsignedShort5_6_5 => 0x8363,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PixelFormat {
    DepthComponent = 0x1902,
    DepthStencil = 0x84F9,
    Alpha = 0x1906,
    Red = 0x1903,
    Rg = 0x8227,
    Rgb = 0x1907,
    Rgba = 0x1908,
    RedInteger = 0x8D94,
    RgInteger = 0x8228,
    RgbInteger = 0x8D98,
    RgbaInteger = 0x8D99,
    Luminance = 0x1909,
    LuminanceAlpha = 0x190A,
    RgbDxt1 = 0x83F0,
    RgbaDxt1 = 0x83F1,
    RgbaDxt3 = 0x83F2,
    RgbaDxt5 = 0x83F3,
    RgbPvrtc4bppv1 = 0x8C00,
    RgbPvrtc2bppv1 = 0x8C01,
    RgbaPvrtc4bppv1 = 0x8C02,
    RgbaPvrtc2bppv1 = 0x8C03,
    RgbaAstc = 0x93B0,
    RgbEtc1 = 0x8D64,
    Rgb8Etc2 = 0x9274,
    Rgba8Etc2Eac = 0x9278,
    RgbaBc7 = 0x8E8C,
}

impl PixelFormat {
    pub const DEPTH_COMPONENT: Self = Self::DepthComponent;
    pub const DEPTH_STENCIL: Self = Self::DepthStencil;
    pub const ALPHA: Self = Self::Alpha;
    pub const RED: Self = Self::Red;
    pub const RG: Self = Self::Rg;
    pub const RGB: Self = Self::Rgb;
    pub const RGBA: Self = Self::Rgba;
    pub const RED_INTEGER: Self = Self::RedInteger;
    pub const RG_INTEGER: Self = Self::RgInteger;
    pub const RGB_INTEGER: Self = Self::RgbInteger;
    pub const RGBA_INTEGER: Self = Self::RgbaInteger;
    pub const LUMINANCE: Self = Self::Luminance;
    pub const LUMINANCE_ALPHA: Self = Self::LuminanceAlpha;
    pub const RGB_DXT1: Self = Self::RgbDxt1;
    pub const RGBA_DXT1: Self = Self::RgbaDxt1;
    pub const RGBA_DXT3: Self = Self::RgbaDxt3;
    pub const RGBA_DXT5: Self = Self::RgbaDxt5;
    pub const RGB_PVRTC_4BPPV1: Self = Self::RgbPvrtc4bppv1;
    pub const RGB_PVRTC_2BPPV1: Self = Self::RgbPvrtc2bppv1;
    pub const RGBA_PVRTC_4BPPV1: Self = Self::RgbaPvrtc4bppv1;
    pub const RGBA_PVRTC_2BPPV1: Self = Self::RgbaPvrtc2bppv1;
    pub const RGBA_ASTC: Self = Self::RgbaAstc;
    pub const RGB_ETC1: Self = Self::RgbEtc1;
    pub const RGB8_ETC2: Self = Self::Rgb8Etc2;
    pub const RGBA8_ETC2_EAC: Self = Self::Rgba8Etc2Eac;
    pub const RGBA_BC7: Self = Self::RgbaBc7;

    pub const fn as_gl_enum(&self) -> u32 {
        *self as u32
    }

    pub const fn components_length(&self) -> usize {
        match self {
            Self::Rgb | Self::RgbInteger => 3,
            Self::Rgba | Self::RgbaInteger => 4,
            Self::LuminanceAlpha | Self::Rg | Self::RgInteger => 2,
            Self::Alpha | Self::Red | Self::RedInteger | Self::Luminance => 1,
            _ => 1,
        }
    }

    pub const fn is_color_format(&self) -> bool {
        matches!(
            self,
            Self::Red | Self::Alpha | Self::Rgb | Self::Rgba | Self::Luminance | Self::LuminanceAlpha
        )
    }

    pub const fn is_depth_format(&self) -> bool {
        matches!(self, Self::DepthComponent | Self::DepthStencil)
    }

    pub const fn is_compressed_format(&self) -> bool {
        matches!(
            self,
            Self::RgbDxt1
                | Self::RgbaDxt1
                | Self::RgbaDxt3
                | Self::RgbaDxt5
                | Self::RgbPvrtc4bppv1
                | Self::RgbPvrtc2bppv1
                | Self::RgbaPvrtc4bppv1
                | Self::RgbaPvrtc2bppv1
                | Self::RgbaAstc
                | Self::RgbEtc1
                | Self::Rgb8Etc2
                | Self::Rgba8Etc2Eac
                | Self::RgbaBc7
        )
    }

    pub const fn to_internal_format(&self, datatype: PixelDatatype, webgl2: bool) -> u32 {
        if !webgl2 {
            return self.as_gl_enum();
        }

        if matches!(self, Self::DepthStencil) {
            return 0x88F0; // DEPTH24_STENCIL8
        }

        if matches!(self, Self::DepthComponent) {
            match datatype {
                PixelDatatype::UnsignedShort => return 0x81A5, // DEPTH_COMPONENT16
                PixelDatatype::UnsignedInt => return 0x81A6,   // DEPTH_COMPONENT24
                _ => {}
            }
        }

        match datatype {
            PixelDatatype::Float => match self {
                Self::Rgba => 0x8814, // RGBA32F
                Self::Rgb => 0x8815,  // RGB32F
                Self::Rg => 0x8230,   // RG32F
                Self::Red => 0x822E,  // R32F
                _ => self.as_gl_enum(),
            },
            PixelDatatype::HalfFloat => match self {
                Self::Rgba => 0x881A, // RGBA16F
                Self::Rgb => 0x881B,  // RGB16F
                Self::Rg => 0x822F,   // RG16F
                Self::Red => 0x822D,  // R16F
                _ => self.as_gl_enum(),
            },
            PixelDatatype::UnsignedByte => match self {
                Self::Rgba => 0x8058, // RGBA8
                Self::Rgb => 0x8051,  // RGB8
                Self::Rg => 0x822B,   // RG8
                Self::Red => 0x8229,  // R8
                _ => self.as_gl_enum(),
            },
            _ => self.as_gl_enum(),
        }
    }

    pub fn flip_y<T: Copy>(
        buffer_view: &[T],
        pixel_format: PixelFormat,
        _pixel_datatype: PixelDatatype,
        width: usize,
        height: usize,
    ) -> Vec<T> {
        if height <= 1 {
            return buffer_view.to_vec();
        }

        let num_components = pixel_format.components_length();
        let row_elements = width * num_components;
        let mut flipped = vec![buffer_view[0]; buffer_view.len()];

        for i in 0..height {
            let row = i * row_elements;
            let flipped_row = (height - 1 - i) * row_elements;
            flipped[flipped_row..flipped_row + row_elements]
                .copy_from_slice(&buffer_view[row..row + row_elements]);
        }

        flipped
    }
}
