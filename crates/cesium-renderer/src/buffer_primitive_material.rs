use cesium_math::Color;

/// Base options for buffer primitive materials.
#[derive(Debug, Clone, Default)]
pub struct BufferPrimitiveMaterialOptions {
    /// Color of fill.
    pub color: Option<Color>,
    /// Color of outline.
    pub outline_color: Option<Color>,
    /// Width of outline, 0-255px.
    pub outline_width: Option<u8>,
}

/// Abstract/base material description for a buffer primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPrimitiveMaterial {
    /// Color of fill.
    pub color: Color,
    /// Color of outline.
    pub outline_color: Color,
    /// Width of outline, 0-255px.
    pub outline_width: u8,
}

impl Default for BufferPrimitiveMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            outline_color: Color::WHITE,
            outline_width: 0,
        }
    }
}

impl BufferPrimitiveMaterial {
    pub const PACKED_LENGTH: usize = 12;

    pub fn new(options: Option<BufferPrimitiveMaterialOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            color: opts.color.unwrap_or(Color::WHITE),
            outline_color: opts.outline_color.unwrap_or(Color::WHITE),
            outline_width: opts.outline_width.unwrap_or(0),
        }
    }

    pub fn pack(&self, view: &mut [u8], byte_offset: usize) {
        let color_bytes = self.color.to_rgba().to_le_bytes();
        let outline_bytes = self.outline_color.to_rgba().to_le_bytes();

        view[byte_offset..byte_offset + 4].copy_from_slice(&color_bytes);
        view[byte_offset + 4..byte_offset + 8].copy_from_slice(&outline_bytes);
        view[byte_offset + 8] = self.outline_width;
    }

    pub fn unpack(view: &[u8], byte_offset: usize, result: &mut Self) {
        let color_u32 = u32::from_le_bytes([
            view[byte_offset],
            view[byte_offset + 1],
            view[byte_offset + 2],
            view[byte_offset + 3],
        ]);
        let outline_u32 = u32::from_le_bytes([
            view[byte_offset + 4],
            view[byte_offset + 5],
            view[byte_offset + 6],
            view[byte_offset + 7],
        ]);

        result.color = Color::from_rgba(color_u32);
        result.outline_color = Color::from_rgba(outline_u32);
        result.outline_width = view[byte_offset + 8];
    }
}

/// Options for constructing a [`BufferPointMaterial`].
#[derive(Debug, Clone, Default)]
pub struct BufferPointMaterialOptions {
    pub color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<u8>,
    pub size: Option<u8>,
}

/// Material description for a `BufferPoint`.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPointMaterial {
    pub color: Color,
    pub outline_color: Color,
    pub outline_width: u8,
    pub size: u8,
}

impl Default for BufferPointMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            outline_color: Color::WHITE,
            outline_width: 0,
            size: 1,
        }
    }
}

impl BufferPointMaterial {
    pub const PACKED_LENGTH: usize = 16;

    pub fn new(options: Option<BufferPointMaterialOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            color: opts.color.unwrap_or(Color::WHITE),
            outline_color: opts.outline_color.unwrap_or(Color::WHITE),
            outline_width: opts.outline_width.unwrap_or(0),
            size: opts.size.unwrap_or(1),
        }
    }

    pub fn pack(&self, view: &mut [u8], byte_offset: usize) {
        let base = BufferPrimitiveMaterial {
            color: self.color,
            outline_color: self.outline_color,
            outline_width: self.outline_width,
        };
        base.pack(view, byte_offset);
        view[byte_offset + 12] = self.size;
    }

    pub fn unpack(view: &[u8], byte_offset: usize, result: &mut Self) {
        let mut base = BufferPrimitiveMaterial::default();
        BufferPrimitiveMaterial::unpack(view, byte_offset, &mut base);
        result.color = base.color;
        result.outline_color = base.outline_color;
        result.outline_width = base.outline_width;
        result.size = view[byte_offset + 12];
    }
}

/// Options for constructing a [`BufferPolygonMaterial`].
#[derive(Debug, Clone, Default)]
pub struct BufferPolygonMaterialOptions {
    pub color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<u8>,
}

/// Material description for a `BufferPolygon`.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPolygonMaterial {
    pub color: Color,
    pub outline_color: Color,
    pub outline_width: u8,
}

impl Default for BufferPolygonMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            outline_color: Color::WHITE,
            outline_width: 0,
        }
    }
}

impl BufferPolygonMaterial {
    pub const PACKED_LENGTH: usize = 12;

    pub fn new(options: Option<BufferPolygonMaterialOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            color: opts.color.unwrap_or(Color::WHITE),
            outline_color: opts.outline_color.unwrap_or(Color::WHITE),
            outline_width: opts.outline_width.unwrap_or(0),
        }
    }

    pub fn pack(&self, view: &mut [u8], byte_offset: usize) {
        let base = BufferPrimitiveMaterial {
            color: self.color,
            outline_color: self.outline_color,
            outline_width: self.outline_width,
        };
        base.pack(view, byte_offset);
    }

    pub fn unpack(view: &[u8], byte_offset: usize, result: &mut Self) {
        let mut base = BufferPrimitiveMaterial::default();
        BufferPrimitiveMaterial::unpack(view, byte_offset, &mut base);
        result.color = base.color;
        result.outline_color = base.outline_color;
        result.outline_width = base.outline_width;
    }
}

/// Options for constructing a [`BufferPolylineMaterial`].
#[derive(Debug, Clone, Default)]
pub struct BufferPolylineMaterialOptions {
    pub color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Option<u8>,
    pub width: Option<f32>,
}

/// Material description for a `BufferPolyline`.
#[derive(Debug, Clone, PartialEq)]
pub struct BufferPolylineMaterial {
    pub color: Color,
    pub outline_color: Color,
    pub outline_width: u8,
    pub width: f32,
}

impl Default for BufferPolylineMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            outline_color: Color::WHITE,
            outline_width: 0,
            width: 1.0,
        }
    }
}

impl BufferPolylineMaterial {
    pub const PACKED_LENGTH: usize = 16;

    pub fn new(options: Option<BufferPolylineMaterialOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            color: opts.color.unwrap_or(Color::WHITE),
            outline_color: opts.outline_color.unwrap_or(Color::WHITE),
            outline_width: opts.outline_width.unwrap_or(0),
            width: opts.width.unwrap_or(1.0),
        }
    }

    pub fn pack(&self, view: &mut [u8], byte_offset: usize) {
        let base = BufferPrimitiveMaterial {
            color: self.color,
            outline_color: self.outline_color,
            outline_width: self.outline_width,
        };
        base.pack(view, byte_offset);
        let width_bytes = self.width.to_le_bytes();
        view[byte_offset + 12..byte_offset + 16].copy_from_slice(&width_bytes);
    }

    pub fn unpack(view: &[u8], byte_offset: usize, result: &mut Self) {
        let mut base = BufferPrimitiveMaterial::default();
        BufferPrimitiveMaterial::unpack(view, byte_offset, &mut base);
        result.color = base.color;
        result.outline_color = base.outline_color;
        result.outline_width = base.outline_width;
        let width_bytes = [
            view[byte_offset + 12],
            view[byte_offset + 13],
            view[byte_offset + 14],
            view[byte_offset + 15],
        ];
        result.width = f32::from_le_bytes(width_bytes);
    }
}
