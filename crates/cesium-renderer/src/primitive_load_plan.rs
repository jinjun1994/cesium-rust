//! PrimitiveLoadPlan matching CesiumJS `Scene/PrimitiveLoadPlan.js` (Apache-2.0).

use crate::buffer::{Buffer, BufferOptions, BufferUsage, IndexBufferOptions};
use crate::context::Context;
use crate::model_components::{Attribute, Indices, Primitive};
use crate::primitive_outline_generator::{
    PrimitiveOutlineGenerator, PrimitiveOutlineGeneratorOptions,
};
use cesium_math::{AttributeType, ComponentDatatype, DeveloperError, IndexDatatype};

/// Simple struct for tracking whether an attribute will be loaded as a buffer
/// or typed array after post-processing.
#[derive(Clone, Debug, Default)]
pub struct AttributeLoadPlan {
    pub attribute: Attribute,
    pub load_buffer: bool,
    pub load_typed_array: bool,
}

impl AttributeLoadPlan {
    pub fn new(attribute: Attribute) -> Self {
        Self {
            attribute,
            load_buffer: false,
            load_typed_array: false,
        }
    }

    pub fn try_new(attribute: Option<Attribute>) -> Result<Self, DeveloperError> {
        let attribute = attribute.ok_or_else(|| DeveloperError::new("attribute is required."))?;
        Ok(Self::new(attribute))
    }
}

/// Simple struct for tracking whether an index buffer will be loaded as a buffer
/// or typed array after post-processing.
#[derive(Clone, Debug, Default)]
pub struct IndicesLoadPlan {
    pub indices: Indices,
    pub load_buffer: bool,
    pub load_typed_array: bool,
}

impl IndicesLoadPlan {
    pub fn new(indices: Indices) -> Self {
        Self {
            indices,
            load_buffer: false,
            load_typed_array: false,
        }
    }

    pub fn try_new(indices: Option<Indices>) -> Result<Self, DeveloperError> {
        let indices = indices.ok_or_else(|| DeveloperError::new("indices is required."))?;
        Ok(Self::new(indices))
    }
}

/// Primitives may need post-processing steps after their attributes and indices
/// have loaded, such as generating outlines for the CESIUM_primitive_outline glTF
/// extension. This object tracks what indices and attributes need to be
/// post-processed.
#[derive(Clone, Debug, Default)]
pub struct PrimitiveLoadPlan {
    pub primitive: Primitive,
    pub attribute_plans: Vec<AttributeLoadPlan>,
    pub indices_plan: Option<IndicesLoadPlan>,
    pub needs_outlines: bool,
    pub outline_indices: Option<Vec<u32>>,
    pub needs_gaussian_splats: bool,
}

impl PrimitiveLoadPlan {
    pub fn new(primitive: Primitive) -> Self {
        Self {
            primitive,
            attribute_plans: Vec::new(),
            indices_plan: None,
            needs_outlines: false,
            outline_indices: None,
            needs_gaussian_splats: false,
        }
    }

    pub fn try_new(primitive: Option<Primitive>) -> Result<Self, DeveloperError> {
        let primitive = primitive.ok_or_else(|| DeveloperError::new("primitive is required."))?;
        Ok(Self::new(primitive))
    }

    /// Apply post-processing steps that may modify geometry such as generating
    /// outline coordinates. If no post-processing steps are needed, this function
    /// is a no-op.
    pub fn post_process(&mut self, context: Option<&Context>) {
        if self.needs_outlines {
            self.generate_outlines();
            self.generate_buffers(context);
        }

        if self.needs_gaussian_splats {
            self.setup_gaussian_splat_buffers();
        }
    }

    fn setup_gaussian_splat_buffers(&mut self) {
        for plan in &mut self.attribute_plans {
            plan.load_buffer = false;
            plan.load_typed_array = true;
        }
    }

    fn generate_outlines(&mut self) {
        let original_attr_count = self.attribute_plans.len();
        let vertex_count = self
            .attribute_plans
            .first()
            .map(|p| p.attribute.count)
            .or_else(|| self.primitive.attributes.first().map(|a| a.count))
            .unwrap_or(0);

        let parsed_indices = if let Some(ref plan) = self.indices_plan {
            parse_indices(&plan.indices)
        } else if let Some(ref ind) = self.primitive.indices {
            parse_indices(ind)
        } else {
            Vec::new()
        };

        let outline_indices = self.outline_indices.clone().unwrap_or_default();

        let generator = PrimitiveOutlineGenerator::new(PrimitiveOutlineGeneratorOptions {
            triangle_indices: Some(parsed_indices),
            outline_indices: Some(outline_indices),
            original_vertex_count: Some(vertex_count),
        });

        // 1. Update indices
        let updated_triangle_indices = generator.updated_triangle_indices;
        let max_idx = updated_triangle_indices.iter().copied().max().unwrap_or(0);
        let current_dt = self
            .indices_plan
            .as_ref()
            .and_then(|p| p.indices.index_datatype)
            .or_else(|| self.primitive.indices.as_ref().and_then(|i| i.index_datatype));

        let new_dt = match current_dt {
            Some(IndexDatatype::UnsignedByte) if max_idx <= 255 => IndexDatatype::UnsignedByte,
            _ if max_idx <= 255 && current_dt == Some(IndexDatatype::UnsignedByte) => {
                IndexDatatype::UnsignedByte
            }
            _ if max_idx <= 65535 => IndexDatatype::UnsignedShort,
            _ => IndexDatatype::UnsignedInt,
        };

        let encoded_bytes = encode_indices(&updated_triangle_indices, new_dt);
        if let Some(ref mut plan) = self.indices_plan {
            plan.indices.typed_array = Some(encoded_bytes.clone());
            plan.indices.count = updated_triangle_indices.len();
            plan.indices.index_datatype = Some(new_dt);
        }
        if let Some(ref mut ind) = self.primitive.indices {
            ind.typed_array = Some(encoded_bytes);
            ind.count = updated_triangle_indices.len();
            ind.index_datatype = Some(new_dt);
        }

        // 2. Duplicate extra vertices for existing attributes
        let extra_count = generator.extra_vertices.len();
        for plan in self.attribute_plans.iter_mut().take(original_attr_count) {
            let attr = &mut plan.attribute;
            if let Some(ref data) = attr.typed_array {
                if let Some(stride) = data.len().checked_div(vertex_count) {
                    if stride > 0 {
                        let mut updated_data = Vec::with_capacity(data.len() + extra_count * stride);
                        updated_data.extend_from_slice(data);
                        for &src_idx in &generator.extra_vertices {
                            let start = src_idx * stride;
                            let end = start + stride;
                            if end <= data.len() {
                                updated_data.extend_from_slice(&data[start..end]);
                            }
                        }
                        attr.typed_array = Some(updated_data);
                        attr.count = vertex_count + extra_count;
                    }
                }
            }
        }

        // 3. Create outline coordinates attribute
        let outline_coords = generator.outline_coordinates;
        let mut coord_bytes = Vec::with_capacity(outline_coords.len() * 4);
        for &f in &outline_coords {
            coord_bytes.extend_from_slice(&f.to_le_bytes());
        }

        let outline_attr = Attribute {
            name: Some("_OUTLINE_COORDINATES".to_string()),
            component_datatype: Some(ComponentDatatype::Float),
            attribute_type: Some(AttributeType::Vec3),
            normalized: false,
            count: outline_coords.len() / 3,
            typed_array: Some(coord_bytes),
            ..Default::default()
        };

        let mut outline_plan = AttributeLoadPlan::new(outline_attr.clone());
        outline_plan.load_buffer = true;
        outline_plan.load_typed_array = false;

        self.attribute_plans.push(outline_plan);
        self.primitive.outline_coordinates = Some(outline_attr);
    }

    fn generate_buffers(&mut self, _context: Option<&Context>) {
        for plan in &mut self.attribute_plans {
            let attr = &mut plan.attribute;
            if plan.load_buffer {
                if let Some(ref bytes) = attr.typed_array {
                    let buffer = Buffer::create_vertex_buffer(BufferOptions {
                        size_in_bytes: 0,
                        usage: Some(BufferUsage::StaticDraw),
                        typed_array: Some(bytes.clone()),
                    })
                    .ok();
                    attr.buffer = buffer;
                }
            }
            if !plan.load_typed_array {
                attr.typed_array = None;
            }
        }

        if let Some(ref mut plan) = self.indices_plan {
            let ind = &mut plan.indices;
            if plan.load_buffer {
                if let Some(ref bytes) = ind.typed_array {
                    let buffer = Buffer::create_index_buffer(IndexBufferOptions {
                        size_in_bytes: 0,
                        usage: Some(BufferUsage::StaticDraw),
                        typed_array: Some(bytes.clone()),
                        index_datatype: ind.index_datatype.unwrap_or(IndexDatatype::UnsignedShort),
                    })
                    .ok();
                    ind.buffer = buffer;
                }
            }
            if !plan.load_typed_array {
                ind.typed_array = None;
            }
        }

        // Synchronize with primitive
        if let Some(ref mut ind_plan) = self.indices_plan {
            self.primitive.indices = Some(ind_plan.indices.clone());
        }
        if let Some(outline) = self.attribute_plans.iter().find(|p| {
            p.attribute
                .name
                .as_deref()
                .is_some_and(|n| n == "_OUTLINE_COORDINATES")
        }) {
            self.primitive.outline_coordinates = Some(outline.attribute.clone());
        }
    }
}

fn parse_indices(indices: &Indices) -> Vec<u32> {
    let Some(ref bytes) = indices.typed_array else {
        return Vec::new();
    };

    match indices.index_datatype {
        Some(IndexDatatype::UnsignedByte) => bytes.iter().map(|&b| b as u32).collect(),
        Some(IndexDatatype::UnsignedShort) => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|chunk| u16::from_le_bytes(*chunk) as u32)
            .collect(),
        Some(IndexDatatype::UnsignedInt) => bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect(),
        None => {
            if indices.count > 0 && bytes.len() == indices.count {
                bytes.iter().map(|&b| b as u32).collect()
            } else if indices.count > 0 && bytes.len() == indices.count * 2 {
                bytes
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|chunk| u16::from_le_bytes(*chunk) as u32)
                    .collect()
            } else if indices.count > 0 && bytes.len() == indices.count * 4 {
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|chunk| u32::from_le_bytes(*chunk))
                    .collect()
            } else {
                bytes.iter().map(|&b| b as u32).collect()
            }
        }
    }
}

fn encode_indices(indices: &[u32], datatype: IndexDatatype) -> Vec<u8> {
    match datatype {
        IndexDatatype::UnsignedByte => indices.iter().map(|&i| i as u8).collect(),
        IndexDatatype::UnsignedShort => {
            let mut out = Vec::with_capacity(indices.len() * 2);
            for &i in indices {
                out.extend_from_slice(&(i as u16).to_le_bytes());
            }
            out
        }
        IndexDatatype::UnsignedInt => {
            let mut out = Vec::with_capacity(indices.len() * 4);
            for &i in indices {
                out.extend_from_slice(&i.to_le_bytes());
            }
            out
        }
    }
}
