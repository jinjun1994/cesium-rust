//! VertexArray, VertexArrayFactory, and VertexArrayFacade matching CesiumJS (Apache-2.0).

use cesium_math::{ComponentDatatype, DeveloperError, Geometry};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BufferUsage {
    #[default]
    StaticDraw,
    StreamDraw,
    DynamicDraw,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Buffer {
    pub size_in_bytes: usize,
    pub usage: BufferUsage,
    pub data: Vec<u8>,
    pub is_destroyed: bool,
}

#[derive(Clone, Debug)]
pub struct BufferOptions {
    pub size_in_bytes: usize,
    pub usage: Option<BufferUsage>,
    pub typed_array: Option<Vec<u8>>,
}

impl Buffer {
    pub fn create_vertex_buffer(options: BufferOptions) -> Self {
        let data = options.typed_array.unwrap_or_else(|| vec![0u8; options.size_in_bytes]);
        let size_in_bytes = if options.size_in_bytes > 0 {
            options.size_in_bytes
        } else {
            data.len()
        };
        Self {
            size_in_bytes,
            usage: options.usage.unwrap_or(BufferUsage::StaticDraw),
            data,
            is_destroyed: false,
        }
    }

    pub fn create_index_buffer(options: BufferOptions) -> Self {
        Self::create_vertex_buffer(options)
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VertexAttribute {
    pub index: u32,
    pub enabled: bool,
    pub vertex_buffer: Option<Buffer>,
    pub components_per_attribute: u32,
    pub component_datatype: ComponentDatatype,
    pub normalize: bool,
    pub offset_in_bytes: usize,
    pub stride_in_bytes: usize,
    pub instance_divisor: u32,
}

#[derive(Clone, Debug)]
pub struct VertexAttributeOptions {
    pub index: Option<u32>,
    pub enabled: Option<bool>,
    pub vertex_buffer: Option<Buffer>,
    pub components_per_attribute: u32,
    pub component_datatype: Option<ComponentDatatype>,
    pub normalize: Option<bool>,
    pub offset_in_bytes: Option<usize>,
    pub stride_in_bytes: Option<usize>,
    pub instance_divisor: Option<u32>,
}

impl VertexAttribute {
    pub fn new(options: VertexAttributeOptions) -> Self {
        Self {
            index: options.index.unwrap_or(0),
            enabled: options.enabled.unwrap_or(true),
            vertex_buffer: options.vertex_buffer,
            components_per_attribute: options.components_per_attribute,
            component_datatype: options.component_datatype.unwrap_or(ComponentDatatype::Float),
            normalize: options.normalize.unwrap_or(false),
            offset_in_bytes: options.offset_in_bytes.unwrap_or(0),
            stride_in_bytes: options.stride_in_bytes.unwrap_or(0),
            instance_divisor: options.instance_divisor.unwrap_or(0),
        }
    }
}

/// A VertexArray managing attribute bindings and an optional index buffer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VertexArray {
    attributes: Vec<VertexAttribute>,
    pub index_buffer: Option<Buffer>,
    pub is_destroyed: bool,
}

#[derive(Clone, Debug, Default)]
pub struct VertexArrayOptions {
    pub attributes: Vec<VertexAttributeOptions>,
    pub index_buffer: Option<Buffer>,
}

#[derive(Clone, Debug, Default)]
pub struct VertexArrayFromGeometryOptions {
    pub geometry: Option<Geometry>,
    pub interleave: Option<bool>,
}

impl VertexArray {
    pub fn new(options: VertexArrayOptions) -> Self {
        let mut attributes = Vec::with_capacity(options.attributes.len());
        for (i, mut attr_opt) in options.attributes.into_iter().enumerate() {
            if attr_opt.index.is_none() {
                attr_opt.index = Some(i as u32);
            }
            attributes.push(VertexAttribute::new(attr_opt));
        }

        Self {
            attributes,
            index_buffer: options.index_buffer,
            is_destroyed: false,
        }
    }

    pub fn from_geometry(options: VertexArrayFromGeometryOptions) -> Result<Self, DeveloperError> {
        let geometry = match options.geometry {
            Some(g) => g,
            None => {
                return Ok(Self {
                    attributes: Vec::new(),
                    index_buffer: None,
                    is_destroyed: false,
                })
            }
        };

        let mut attributes = Vec::new();
        let mut index = 0u32;

        if let Some(pos) = geometry.attributes.position {
            let buffer = Buffer::create_vertex_buffer(BufferOptions {
                size_in_bytes: pos.values.len() * 4,
                usage: Some(BufferUsage::DynamicDraw),
                typed_array: None,
            });
            attributes.push(VertexAttribute::new(VertexAttributeOptions {
                index: Some(index),
                enabled: Some(true),
                vertex_buffer: Some(buffer),
                components_per_attribute: pos.components_per_attribute as u32,
                component_datatype: Some(pos.component_datatype),
                normalize: Some(false),
                offset_in_bytes: Some(0),
                stride_in_bytes: Some(0),
                instance_divisor: Some(0),
            }));
            index += 1;
        }

        if let Some(normal) = geometry.attributes.normal {
            let buffer = Buffer::create_vertex_buffer(BufferOptions {
                size_in_bytes: normal.values.len() * 4,
                usage: Some(BufferUsage::DynamicDraw),
                typed_array: None,
            });
            attributes.push(VertexAttribute::new(VertexAttributeOptions {
                index: Some(index),
                enabled: Some(true),
                vertex_buffer: Some(buffer),
                components_per_attribute: normal.components_per_attribute as u32,
                component_datatype: Some(normal.component_datatype),
                normalize: Some(false),
                offset_in_bytes: Some(0),
                stride_in_bytes: Some(0),
                instance_divisor: Some(0),
            }));
        }

        Ok(Self {
            attributes,
            index_buffer: None,
            is_destroyed: false,
        })
    }

    pub fn number_of_attributes(&self) -> usize {
        self.attributes.len()
    }

    pub fn get_attribute(&self, index: usize) -> &VertexAttribute {
        &self.attributes[index]
    }

    pub fn get_attribute_mut(&mut self, index: usize) -> &mut VertexAttribute {
        &mut self.attributes[index]
    }

    pub fn bind(&self) {}
    pub fn unbind(&self) {}

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        for attr in &mut self.attributes {
            if let Some(ref mut buf) = attr.vertex_buffer {
                buf.destroy();
            }
        }
        if let Some(ref mut idx) = self.index_buffer {
            idx.destroy();
        }
        self.is_destroyed = true;
    }
}

/// Description of an attribute managed by VertexArrayFacade.
#[derive(Clone, Debug)]
pub struct VertexArrayFacadeAttribute {
    pub index: u32,
    pub components_per_attribute: u32,
    pub component_datatype: ComponentDatatype,
    pub usage: BufferUsage,
}

pub struct VertexArrayFacadeItem {
    pub va: VertexArray,
}

/// Manages dynamic batching, resizing, and writing to VertexArrays.
pub struct VertexArrayFacade {
    attributes: Vec<VertexArrayFacadeAttribute>,
    num_vertices: usize,
    raw_buffer: Vec<u8>,
    stride_in_bytes: usize,
    pub va: Vec<VertexArrayFacadeItem>,
}

impl VertexArrayFacade {
    pub fn new(attributes: Vec<VertexArrayFacadeAttribute>, num_vertices: usize) -> Self {
        let mut stride = 0;
        for attr in &attributes {
            let comp_size = match attr.component_datatype {
                ComponentDatatype::Byte | ComponentDatatype::UnsignedByte => 1,
                ComponentDatatype::Short | ComponentDatatype::UnsignedShort => 2,
                _ => 4,
            };
            stride += (attr.components_per_attribute as usize) * comp_size;
        }

        let mut facade = Self {
            attributes,
            num_vertices,
            raw_buffer: vec![0u8; num_vertices * stride],
            stride_in_bytes: stride,
            va: Vec::new(),
        };

        facade.commit();
        facade
    }

    pub fn resize(&mut self, new_num_vertices: usize) {
        self.num_vertices = new_num_vertices;
        self.raw_buffer.resize(new_num_vertices * self.stride_in_bytes, 0);
    }

    pub fn write_float3(&mut self, vertex_index: usize, x: f32, y: f32, z: f32) {
        let offset = vertex_index * self.stride_in_bytes;
        if offset + 12 <= self.raw_buffer.len() {
            self.raw_buffer[offset..offset + 4].copy_from_slice(&x.to_ne_bytes());
            self.raw_buffer[offset + 4..offset + 8].copy_from_slice(&y.to_ne_bytes());
            self.raw_buffer[offset + 8..offset + 12].copy_from_slice(&z.to_ne_bytes());
        }
    }

    pub fn commit(&mut self) {
        let mut va_attrs = Vec::new();
        let buffer = Buffer::create_vertex_buffer(BufferOptions {
            size_in_bytes: self.raw_buffer.len(),
            usage: Some(self.attributes.first().map(|a| a.usage).unwrap_or(BufferUsage::StaticDraw)),
            typed_array: Some(self.raw_buffer.clone()),
        });

        let mut offset = 0;
        for attr in &self.attributes {
            let comp_size = match attr.component_datatype {
                ComponentDatatype::Byte | ComponentDatatype::UnsignedByte => 1,
                ComponentDatatype::Short | ComponentDatatype::UnsignedShort => 2,
                _ => 4,
            };
            va_attrs.push(VertexAttributeOptions {
                index: Some(attr.index),
                enabled: Some(true),
                vertex_buffer: Some(buffer.clone()),
                components_per_attribute: attr.components_per_attribute,
                component_datatype: Some(attr.component_datatype),
                normalize: Some(false),
                offset_in_bytes: Some(offset),
                stride_in_bytes: Some(self.stride_in_bytes),
                instance_divisor: Some(0),
            });
            offset += (attr.components_per_attribute as usize) * comp_size;
        }

        self.va = vec![VertexArrayFacadeItem {
            va: VertexArray::new(VertexArrayOptions {
                attributes: va_attrs,
                index_buffer: None,
            }),
        }];
    }
}
