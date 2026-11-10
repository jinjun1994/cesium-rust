//! ModelReader matching CesiumJS `Scene/Model/ModelReader.js` (Apache-2.0).

use cesium_math::{AttributeCompression, AttributeType, ComponentDatatype, DeveloperError};

use crate::model_components::{Attribute, Quantization};

/// Utility for reading and decoding vertex attribute data.
pub struct ModelReader;

impl ModelReader {
    /// Reads the data of the attribute as a flat, compact floating-point array,
    /// deinterleaving, normalizing, dequantizing, and oct-decoding as necessary.
    pub fn read_attribute_as_typed_array(attribute: &Attribute) -> Result<Vec<f32>, DeveloperError> {
        let raw_bytes = Self::read_attribute_as_raw_compact_typed_array(attribute)?;
        let count = attribute.count;
        let attr_type = attribute
            .attribute_type
            .ok_or_else(|| DeveloperError::new("attribute_type is required"))?;
        let comp_type = attribute
            .component_datatype
            .unwrap_or(ComponentDatatype::Float);

        let num_components = attr_type.get_number_of_components();
        let total_components = count * num_components;

        // Convert raw bytes to f32 based on component datatype and normalization
        let mut float_data = Vec::with_capacity(total_components);

        if attribute.normalized {
            // Apply normalization
            match comp_type {
                ComponentDatatype::Byte => {
                    for &b in raw_bytes.iter().take(total_components) {
                        let val = b as i8;
                        float_data.push((val as f32 / 127.0).max(-1.0));
                    }
                }
                ComponentDatatype::UnsignedByte => {
                    for &b in raw_bytes.iter().take(total_components) {
                        float_data.push(b as f32 / 255.0);
                    }
                }
                ComponentDatatype::Short => {
                    for i in 0..total_components {
                        let offset = i * 2;
                        let val = i16::from_le_bytes([raw_bytes[offset], raw_bytes[offset + 1]]);
                        float_data.push((val as f32 / 32767.0).max(-1.0));
                    }
                }
                ComponentDatatype::UnsignedShort => {
                    for i in 0..total_components {
                        let offset = i * 2;
                        let val = u16::from_le_bytes([raw_bytes[offset], raw_bytes[offset + 1]]);
                        float_data.push(val as f32 / 65535.0);
                    }
                }
                _ => {
                    // Default read without normalization
                    for i in 0..total_components {
                        let offset = i * 4;
                        let val = f32::from_le_bytes([
                            raw_bytes[offset],
                            raw_bytes[offset + 1],
                            raw_bytes[offset + 2],
                            raw_bytes[offset + 3],
                        ]);
                        float_data.push(val);
                    }
                }
            }
        } else {
            // Raw reading into f32
            match comp_type {
                ComponentDatatype::Byte => {
                    for &b in raw_bytes.iter().take(total_components) {
                        float_data.push(b as i8 as f32);
                    }
                }
                ComponentDatatype::UnsignedByte => {
                    for &b in raw_bytes.iter().take(total_components) {
                        float_data.push(b as f32);
                    }
                }
                ComponentDatatype::Short => {
                    for i in 0..total_components {
                        let offset = i * 2;
                        let val = i16::from_le_bytes([raw_bytes[offset], raw_bytes[offset + 1]]);
                        float_data.push(val as f32);
                    }
                }
                ComponentDatatype::UnsignedShort => {
                    for i in 0..total_components {
                        let offset = i * 2;
                        let val = u16::from_le_bytes([raw_bytes[offset], raw_bytes[offset + 1]]);
                        float_data.push(val as f32);
                    }
                }
                ComponentDatatype::UnsignedInt => {
                    for i in 0..total_components {
                        let offset = i * 4;
                        let val = u32::from_le_bytes([
                            raw_bytes[offset],
                            raw_bytes[offset + 1],
                            raw_bytes[offset + 2],
                            raw_bytes[offset + 3],
                        ]);
                        float_data.push(val as f32);
                    }
                }
                _ => {
                    for i in 0..total_components {
                        let offset = i * 4;
                        let val = f32::from_le_bytes([
                            raw_bytes[offset],
                            raw_bytes[offset + 1],
                            raw_bytes[offset + 2],
                            raw_bytes[offset + 3],
                        ]);
                        float_data.push(val);
                    }
                }
            }
        }

        // Apply dequantization if present
        if let Some(quant) = &attribute.quantization {
            float_data = Self::dequantize(&float_data, count, attr_type, quant)?;
        }

        Ok(float_data)
    }

    /// Reads raw compact (deinterleaved) bytes of the attribute.
    pub fn read_attribute_as_raw_compact_typed_array(
        attribute: &Attribute,
    ) -> Result<Vec<u8>, DeveloperError> {
        let count = attribute.count;
        let attr_type = attribute
            .attribute_type
            .ok_or_else(|| DeveloperError::new("attribute_type is required"))?;
        let comp_type = if let Some(q) = &attribute.quantization {
            q.component_datatype
                .unwrap_or_else(|| attribute.component_datatype.unwrap_or(ComponentDatatype::Float))
        } else {
            attribute.component_datatype.unwrap_or(ComponentDatatype::Float)
        };

        let num_components = attr_type.get_number_of_components();
        let bytes_per_component = comp_type.size_in_bytes();
        let element_size = num_components * bytes_per_component;
        let total_bytes = count * element_size;

        if let Some(typed_array) = &attribute.typed_array {
            // Tightly packed in-memory typed array
            let stride = attribute.byte_stride.unwrap_or(element_size);
            if stride == element_size && attribute.byte_offset == 0 {
                return Ok(typed_array.clone());
            }

            let mut compact = Vec::with_capacity(total_bytes);
            for i in 0..count {
                let start = attribute.byte_offset + i * stride;
                compact.extend_from_slice(&typed_array[start..start + element_size]);
            }
            return Ok(compact);
        }

        Err(DeveloperError::new("No attribute buffer or typed_array available"))
    }

    /// Dequantizes the input floating point array according to Quantization parameters.
    pub fn dequantize(
        quantized: &[f32],
        count: usize,
        attr_type: AttributeType,
        quantization: &Quantization,
    ) -> Result<Vec<f32>, DeveloperError> {
        if quantization.oct_encoded {
            let norm_range = quantization
                .normalization_range
                .as_ref()
                .and_then(|r| r.first().copied())
                .unwrap_or(255.0);

            let mut decoded = Self::oct_decode(quantized, count, norm_range)?;
            if quantization.oct_encoded_zxy {
                decoded = Self::convert_zxy_to_xyz(&decoded, count);
            }
            return Ok(decoded);
        }

        let step_size = quantization
            .quantized_volume_step_size
            .as_deref()
            .unwrap_or(&[]);
        let offset = quantization
            .quantized_volume_offset
            .as_deref()
            .unwrap_or(&[]);

        match attr_type {
            AttributeType::Scalar => {
                let s = step_size.first().copied().unwrap_or(1.0);
                let o = offset.first().copied().unwrap_or(0.0);
                Ok(Self::dequantize_1d(quantized, count, s, o))
            }
            AttributeType::Vec2 => {
                let s = [
                    step_size.first().copied().unwrap_or(1.0),
                    step_size.get(1).copied().unwrap_or(1.0),
                ];
                let o = [
                    offset.first().copied().unwrap_or(0.0),
                    offset.get(1).copied().unwrap_or(0.0),
                ];
                Ok(Self::dequantize_2d(quantized, count, s, o))
            }
            AttributeType::Vec3 => {
                let s = [
                    step_size.first().copied().unwrap_or(1.0),
                    step_size.get(1).copied().unwrap_or(1.0),
                    step_size.get(2).copied().unwrap_or(1.0),
                ];
                let o = [
                    offset.first().copied().unwrap_or(0.0),
                    offset.get(1).copied().unwrap_or(0.0),
                    offset.get(2).copied().unwrap_or(0.0),
                ];
                Ok(Self::dequantize_3d(quantized, count, s, o))
            }
            AttributeType::Vec4 => {
                let s = [
                    step_size.first().copied().unwrap_or(1.0),
                    step_size.get(1).copied().unwrap_or(1.0),
                    step_size.get(2).copied().unwrap_or(1.0),
                    step_size.get(3).copied().unwrap_or(1.0),
                ];
                let o = [
                    offset.first().copied().unwrap_or(0.0),
                    offset.get(1).copied().unwrap_or(0.0),
                    offset.get(2).copied().unwrap_or(0.0),
                    offset.get(3).copied().unwrap_or(0.0),
                ];
                Ok(Self::dequantize_4d(quantized, count, s, o))
            }
            _ => Err(DeveloperError::new(format!(
                "Element type for dequantization must be SCALAR, VEC2, VEC3, or VEC4, but is {:?}",
                attr_type
            ))),
        }
    }

    /// Applies oct-decoding to quantized 2D normal vectors into 3D Cartesian coordinates.
    pub fn oct_decode(
        quantized: &[f32],
        count: usize,
        normalization_range: f64,
    ) -> Result<Vec<f32>, DeveloperError> {
        let mut decoded = Vec::with_capacity(count * 3);
        for i in 0..count {
            let x = quantized[i * 2] as f64;
            let y = quantized[i * 2 + 1] as f64;
            let normal = AttributeCompression::oct_decode_in_range(x, y, normalization_range)?;
            decoded.push(normal.x as f32);
            decoded.push(normal.y as f32);
            decoded.push(normal.z as f32);
        }
        Ok(decoded)
    }

    /// Swizzles (z, x, y) to (x, y, z).
    pub fn convert_zxy_to_xyz(input: &[f32], count: usize) -> Vec<f32> {
        let mut output = vec![0.0f32; count * 3];
        for i in 0..count {
            let offset = i * 3;
            let z = input[offset];
            let x = input[offset + 1];
            let y = input[offset + 2];
            output[offset] = x;
            output[offset + 1] = y;
            output[offset + 2] = z;
        }
        output
    }

    /// Dequantizes 1D scalar data: `output[i] = input[i] * step_size + offset`.
    pub fn dequantize_1d(quantized: &[f32], count: usize, step_size: f64, offset: f64) -> Vec<f32> {
        let mut output = Vec::with_capacity(count);
        for &val in quantized.iter().take(count) {
            output.push((val as f64 * step_size + offset) as f32);
        }
        output
    }

    /// Dequantizes 2D vector data: `output[i] = input[i] * step_size + offset`.
    pub fn dequantize_2d(
        quantized: &[f32],
        count: usize,
        step_size: [f64; 2],
        offset: [f64; 2],
    ) -> Vec<f32> {
        let mut output = Vec::with_capacity(count * 2);
        for i in 0..count {
            output.push((quantized[i * 2] as f64 * step_size[0] + offset[0]) as f32);
            output.push((quantized[i * 2 + 1] as f64 * step_size[1] + offset[1]) as f32);
        }
        output
    }

    /// Dequantizes 3D vector data: `output[i] = input[i] * step_size + offset`.
    pub fn dequantize_3d(
        quantized: &[f32],
        count: usize,
        step_size: [f64; 3],
        offset: [f64; 3],
    ) -> Vec<f32> {
        let mut output = Vec::with_capacity(count * 3);
        for i in 0..count {
            output.push((quantized[i * 3] as f64 * step_size[0] + offset[0]) as f32);
            output.push((quantized[i * 3 + 1] as f64 * step_size[1] + offset[1]) as f32);
            output.push((quantized[i * 3 + 2] as f64 * step_size[2] + offset[2]) as f32);
        }
        output
    }

    /// Dequantizes 4D vector data: `output[i] = input[i] * step_size + offset`.
    pub fn dequantize_4d(
        quantized: &[f32],
        count: usize,
        step_size: [f64; 4],
        offset: [f64; 4],
    ) -> Vec<f32> {
        let mut output = Vec::with_capacity(count * 4);
        for i in 0..count {
            output.push((quantized[i * 4] as f64 * step_size[0] + offset[0]) as f32);
            output.push((quantized[i * 4 + 1] as f64 * step_size[1] + offset[1]) as f32);
            output.push((quantized[i * 4 + 2] as f64 * step_size[2] + offset[2]) as f32);
            output.push((quantized[i * 4 + 3] as f64 * step_size[3] + offset[3]) as f32);
        }
        output
    }
}
