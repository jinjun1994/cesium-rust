//! BatchTable matching CesiumJS `Scene/BatchTable.js` (Apache-2.0).

use cesium_math::{Cartesian2, Cartesian4, ComponentDatatype, DeveloperError};

use crate::framebuffer::PixelDatatype;

/// Describes a single batched per-instance attribute stored in a [`BatchTable`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchTableAttribute {
    pub function_name: String,
    pub component_datatype: ComponentDatatype,
    pub components_per_attribute: usize,
    pub normalize: bool,
}

impl BatchTableAttribute {
    pub fn new(
        function_name: impl Into<String>,
        component_datatype: ComponentDatatype,
        components_per_attribute: usize,
    ) -> Self {
        Self {
            function_name: function_name.into(),
            component_datatype,
            components_per_attribute,
            normalize: false,
        }
    }

    pub fn with_normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }
}

/// Texture/uniform state produced by [`BatchTable::update`].
#[derive(Clone, Debug)]
pub struct BatchTableTextureData {
    pub pixel_datatype: PixelDatatype,
    pub dimensions: Cartesian2,
    pub step: Cartesian4,
    pub u8_data: Vec<u8>,
    pub f32_data: Vec<f32>,
}

/// Manages per-instance attributes packed into a GPU-friendly 2D texture.
#[derive(Clone, Debug)]
pub struct BatchTable {
    attributes: Vec<BatchTableAttribute>,
    number_of_instances: usize,
    floating_point_texture: bool,
    pixel_datatype: PixelDatatype,
    /// Number of RGBA texels per instance.
    texels_per_instance: usize,
    /// Texel offset of each attribute within an instance.
    attribute_texel_offsets: Vec<usize>,
    /// Component offset within the texel for packed u8 scalar attributes.
    attribute_component_offsets: Vec<usize>,
    dimensions: Cartesian2,
    step: Cartesian4,
    u8_data: Vec<u8>,
    f32_data: Vec<f32>,
    dirty: bool,
    destroyed: bool,
}

impl BatchTable {
    pub fn new(
        attributes: Vec<BatchTableAttribute>,
        number_of_instances: usize,
        floating_point_texture: bool,
    ) -> Result<Self, DeveloperError> {
        if attributes.is_empty() {
            return Err(DeveloperError::new("attributes must not be empty."));
        }
        if number_of_instances == 0 {
            return Err(DeveloperError::new(
                "numberOfInstances must be greater than zero.",
            ));
        }

        let has_float = attributes
            .iter()
            .any(|a| a.component_datatype == ComponentDatatype::Float);
        let pixel_datatype = if has_float && floating_point_texture {
            PixelDatatype::Float
        } else {
            PixelDatatype::UnsignedByte
        };

        let mut texels_per_instance = 0usize;
        let mut attribute_texel_offsets = Vec::with_capacity(attributes.len());
        let mut attribute_component_offsets = Vec::with_capacity(attributes.len());

        for attr in &attributes {
            if attr.components_per_attribute == 0 || attr.components_per_attribute > 4 {
                return Err(DeveloperError::new(
                    "componentsPerAttribute must be between 1 and 4.",
                ));
            }
            attribute_texel_offsets.push(texels_per_instance);
            attribute_component_offsets.push(0);
            if pixel_datatype == PixelDatatype::UnsignedByte
                && attr.component_datatype == ComponentDatatype::Float
            {
                // Each float is packed into 4 bytes (1 RGBA texel).
                texels_per_instance += attr.components_per_attribute;
            } else {
                texels_per_instance += 1;
            }
        }

        let total_texels = number_of_instances * texels_per_instance;
        let max_width = 4096usize;
        let width = total_texels.min(max_width).max(1);
        let height = total_texels.div_ceil(width).max(1);
        let padded_texels = width * height;

        let dimensions = Cartesian2::new(width as f64, height as f64);
        let step = Cartesian4::new(
            1.0 / width as f64,
            1.0 / height as f64,
            1.0 / (width as f64 * 2.0),
            1.0 / (height as f64 * 2.0),
        );

        let (u8_data, f32_data) = match pixel_datatype {
            PixelDatatype::Float => (Vec::new(), vec![0.0f32; padded_texels * 4]),
            _ => (vec![0u8; padded_texels * 4], Vec::new()),
        };

        Ok(Self {
            attributes,
            number_of_instances,
            floating_point_texture,
            pixel_datatype,
            texels_per_instance,
            attribute_texel_offsets,
            attribute_component_offsets,
            dimensions,
            step,
            u8_data,
            f32_data,
            dirty: true,
            destroyed: false,
        })
    }

    pub fn attributes(&self) -> &[BatchTableAttribute] {
        &self.attributes
    }

    pub fn number_of_instances(&self) -> usize {
        self.number_of_instances
    }

    pub fn pixel_datatype(&self) -> PixelDatatype {
        self.pixel_datatype
    }

    pub fn texture_dimensions(&self) -> Cartesian2 {
        self.dimensions
    }

    pub fn texture_step(&self) -> Cartesian4 {
        self.step
    }

    pub fn floating_point_texture(&self) -> bool {
        self.floating_point_texture
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn destroy(&mut self) {
        self.u8_data.clear();
        self.f32_data.clear();
        self.destroyed = true;
    }

    fn validate_indices(
        &self,
        instance_index: usize,
        attribute_index: usize,
    ) -> Result<(), DeveloperError> {
        if instance_index >= self.number_of_instances {
            return Err(DeveloperError::new(format!(
                "instanceIndex ({}) out of range [0, {}).",
                instance_index, self.number_of_instances
            )));
        }
        if attribute_index >= self.attributes.len() {
            return Err(DeveloperError::new(format!(
                "attributeIndex ({}) out of range [0, {}).",
                attribute_index,
                self.attributes.len()
            )));
        }
        Ok(())
    }

    pub fn set_batched_attribute(
        &mut self,
        instance_index: usize,
        attribute_index: usize,
        value: Cartesian4,
    ) -> Result<(), DeveloperError> {
        self.validate_indices(instance_index, attribute_index)?;
        let attr = &self.attributes[attribute_index];
        let base_texel =
            instance_index * self.texels_per_instance + self.attribute_texel_offsets[attribute_index];
        let comp_offset = self.attribute_component_offsets[attribute_index];

        let vals = [value.x, value.y, value.z, value.w];
        if self.pixel_datatype == PixelDatatype::Float {
            let base = base_texel * 4 + comp_offset;
            for (i, v) in vals.iter().take(attr.components_per_attribute).enumerate() {
                self.f32_data[base + i] = *v as f32;
            }
        } else if attr.component_datatype == ComponentDatatype::Float {
            // Forced packing: 1 float per RGBA texel (4 bytes).
            for (c, v) in vals.iter().take(attr.components_per_attribute).enumerate() {
                let bytes = (*v as f32).to_le_bytes();
                let base = (base_texel + c) * 4;
                self.u8_data[base..base + 4].copy_from_slice(&bytes);
            }
        } else {
            let base = base_texel * 4 + comp_offset;
            for (i, v) in vals.iter().take(attr.components_per_attribute).enumerate() {
                self.u8_data[base + i] = v.clamp(0.0, 255.0).round() as u8;
            }
        }
        self.dirty = true;
        Ok(())
    }

    pub fn set_batched_scalar(
        &mut self,
        instance_index: usize,
        attribute_index: usize,
        value: f64,
    ) -> Result<(), DeveloperError> {
        self.set_batched_attribute(
            instance_index,
            attribute_index,
            Cartesian4::new(value, 0.0, 0.0, 0.0),
        )
    }

    pub fn get_batched_attribute(
        &self,
        instance_index: usize,
        attribute_index: usize,
    ) -> Result<Cartesian4, DeveloperError> {
        self.validate_indices(instance_index, attribute_index)?;
        let attr = &self.attributes[attribute_index];
        let base_texel =
            instance_index * self.texels_per_instance + self.attribute_texel_offsets[attribute_index];
        let comp_offset = self.attribute_component_offsets[attribute_index];

        let mut out = [0.0f64; 4];
        if self.pixel_datatype == PixelDatatype::Float {
            let base = base_texel * 4 + comp_offset;
            for (i, slot) in out.iter_mut().take(attr.components_per_attribute).enumerate() {
                *slot = self.f32_data[base + i] as f64;
            }
        } else if attr.component_datatype == ComponentDatatype::Float {
            for (c, slot) in out.iter_mut().take(attr.components_per_attribute).enumerate() {
                let base = (base_texel + c) * 4;
                let bytes: [u8; 4] = self.u8_data[base..base + 4]
                    .try_into()
                    .unwrap_or([0; 4]);
                *slot = f32::from_le_bytes(bytes) as f64;
            }
        } else {
            let base = base_texel * 4 + comp_offset;
            for (i, slot) in out.iter_mut().take(attr.components_per_attribute).enumerate() {
                *slot = self.u8_data[base + i] as f64;
            }
        }
        Ok(Cartesian4::new(out[0], out[1], out[2], out[3]))
    }

    /// Updates the texture snapshot if dirty and returns the current texture data.
    pub fn update(&mut self) -> BatchTableTextureData {
        self.dirty = false;
        BatchTableTextureData {
            pixel_datatype: self.pixel_datatype,
            dimensions: self.dimensions,
            step: self.step,
            u8_data: self.u8_data.clone(),
            f32_data: self.f32_data.clone(),
        }
    }

    /// Injects uniform declarations and attribute getter functions into a vertex shader.
    pub fn get_vertex_shader_callback(&self, shader: &str) -> String {
        let mut header = String::from(
            "uniform sampler2D batchTexture;\n\
             uniform vec2 batchTextureDimensions;\n\
             uniform vec4 batchTextureStep;\n",
        );
        for (idx, attr) in self.attributes.iter().enumerate() {
            let ret_type = match attr.components_per_attribute {
                1 => "float",
                2 => "vec2",
                3 => "vec3",
                _ => "vec4",
            };
            let texel_offset = self.attribute_texel_offsets[idx];
            header.push_str(&format!(
                "{ret_type} {fn_name}(float batchId) {{\n\
                 \tfloat index = batchId * {stride}.0 + {offset}.0;\n\
                 \tfloat y = floor(index / batchTextureDimensions.x);\n\
                 \tfloat x = index - y * batchTextureDimensions.x;\n\
                 \tvec2 uv = vec2((x + 0.5) * batchTextureStep.x, (y + 0.5) * batchTextureStep.y);\n\
                 \tvec4 value = texture2D(batchTexture, uv);\n\
                 \treturn {ret_expr};\n\
                 }}\n",
                ret_type = ret_type,
                fn_name = attr.function_name,
                stride = self.texels_per_instance,
                offset = texel_offset,
                ret_expr = match attr.components_per_attribute {
                    1 => "value.x",
                    2 => "value.xy",
                    3 => "value.xyz",
                    _ => "value",
                }
            ));
        }
        format!("{}{}", header, shader)
    }
}
