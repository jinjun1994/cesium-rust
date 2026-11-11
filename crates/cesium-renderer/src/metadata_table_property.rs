//! MetadataTableProperty matching CesiumJS `Scene/MetadataTableProperty.js` (Apache-2.0).

use crate::metadata_class_property::MetadataClassProperty;
use crate::metadata_component_type::MetadataComponentType;
use crate::metadata_type::MetadataType;
use cesium_math::DeveloperError;
use serde_json::Value;
use std::collections::HashMap;

/// Options for creating a [`MetadataTableProperty`].
#[derive(Clone, Debug, Default)]
pub struct MetadataTablePropertyOptions {
    pub count: usize,
    pub property: Value,
    pub class_property: MetadataClassProperty,
    pub buffer_views: HashMap<String, Vec<u8>>,
}

/// A binary property stored in a [`MetadataTable`].
#[derive(Clone, Debug, PartialEq)]
pub struct MetadataTableProperty {
    count: usize,
    class_property: MetadataClassProperty,
    byte_length: usize,
    extras: Option<Value>,
    extensions: Option<Value>,
    has_value_transform: bool,
    offset: Option<Value>,
    scale: Option<Value>,
    min: Option<Value>,
    max: Option<Value>,
    values_buffer: Vec<u8>,
    string_offsets: Option<Vec<usize>>,
    array_offsets: Option<Vec<usize>>,
    value_type: MetadataComponentType,
}

impl MetadataTableProperty {
    /// Creates a new `MetadataTableProperty`.
    pub fn new(options: MetadataTablePropertyOptions) -> Result<Self, DeveloperError> {
        let count = options.count;
        if count == 0 {
            return Err(DeveloperError::new("options.count must be greater than 0."));
        }

        let prop = &options.property;
        let class_prop = &options.class_property;

        let extras = prop.get("extras").cloned();
        let extensions = prop.get("extensions").cloned();
        let min = prop.get("min").cloned().or_else(|| class_prop.min.clone());
        let max = prop.get("max").cloned().or_else(|| class_prop.max.clone());

        let offset = prop.get("offset").cloned().or_else(|| class_prop.offset.clone());
        let scale = prop.get("scale").cloned().or_else(|| class_prop.scale.clone());
        let has_value_transform = class_prop.has_value_transform || offset.is_some() || scale.is_some();

        let mut byte_length = 0;

        // Resolve buffer views helper
        let get_bv = |key: &str| -> Option<&Vec<u8>> {
            let key_val = prop.get(key)?;
            if let Some(idx) = key_val.as_u64() {
                options.buffer_views.get(&idx.to_string())
            } else if let Some(s) = key_val.as_str() {
                options.buffer_views.get(s)
            } else {
                None
            }
        };

        // Variable length array offsets
        let mut array_offsets = None;
        if class_prop.is_variable_length_array {
            let offset_type_str = prop
                .get("arrayOffsetType")
                .or_else(|| prop.get("offsetType"))
                .and_then(|v| v.as_str())
                .unwrap_or("UINT32");
            let offset_type: MetadataComponentType = offset_type_str.parse().unwrap_or(MetadataComponentType::Uint32);

            let bv = get_bv("arrayOffsets")
                .or_else(|| get_bv("arrayOffsetBufferView"))
                .or_else(|| get_bv("offsetBufferView"));
            if let Some(buf) = bv {
                byte_length += buf.len();
                let offsets = Self::decode_offsets(buf, offset_type, count + 1)?;
                array_offsets = Some(offsets);
            }
        }

        // Strings offsets
        let mut string_offsets = None;
        if class_prop.r#type == MetadataType::String {
            let offset_type_str = prop
                .get("stringOffsetType")
                .or_else(|| prop.get("offsetType"))
                .and_then(|v| v.as_str())
                .unwrap_or("UINT32");
            let offset_type: MetadataComponentType = offset_type_str.parse().unwrap_or(MetadataComponentType::Uint32);

            let bv = get_bv("stringOffsets")
                .or_else(|| get_bv("stringOffsetBufferView"))
                .or_else(|| get_bv("offsetBufferView"));
            if let Some(buf) = bv {
                byte_length += buf.len();
                let comp_count = if let Some(ref a_offs) = array_offsets {
                    if a_offs.len() > count {
                        a_offs[count] - a_offs[0]
                    } else {
                        count
                    }
                } else if class_prop.is_array {
                    count * class_prop.array_length.unwrap_or(1)
                } else {
                    count
                };
                let offsets = Self::decode_offsets(buf, offset_type, comp_count + 1)?;
                string_offsets = Some(offsets);
            }
        }

        let value_type = if class_prop.r#type == MetadataType::String || class_prop.r#type == MetadataType::Boolean {
            MetadataComponentType::Uint8
        } else {
            class_prop
                .component_type
                .or(class_prop.value_type)
                .unwrap_or(MetadataComponentType::Uint8)
        };

        let values_bv = get_bv("values").or_else(|| get_bv("bufferView"));
        let values_buffer = if let Some(buf) = values_bv {
            byte_length += buf.len();
            buf.clone()
        } else {
            Vec::new()
        };

        Ok(Self {
            count,
            class_property: class_prop.clone(),
            byte_length,
            extras,
            extensions,
            has_value_transform,
            offset,
            scale,
            min,
            max,
            values_buffer,
            string_offsets,
            array_offsets,
            value_type,
        })
    }

    /// Number of entities.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Total byte length across all buffer views.
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Extras property.
    pub fn extras(&self) -> Option<&Value> {
        self.extras.as_ref()
    }

    /// Extensions property.
    pub fn extensions(&self) -> Option<&Value> {
        self.extensions.as_ref()
    }

    /// Class property definition.
    pub fn class_property(&self) -> &MetadataClassProperty {
        &self.class_property
    }

    /// Whether this property applies a value transform.
    pub fn has_value_transform(&self) -> bool {
        self.has_value_transform
    }

    /// Offset value transform.
    pub fn offset(&self) -> Option<&Value> {
        self.offset.as_ref()
    }

    /// Scale value transform.
    pub fn scale(&self) -> Option<&Value> {
        self.scale.as_ref()
    }

    /// Minimum value.
    pub fn min(&self) -> Option<&Value> {
        self.min.as_ref()
    }

    /// Maximum value.
    pub fn max(&self) -> Option<&Value> {
        self.max.as_ref()
    }

    /// Gets underlying typed array buffer bytes.
    pub fn get_typed_array(&self) -> Option<Vec<u8>> {
        if self.values_buffer.is_empty() {
            None
        } else {
            Some(self.values_buffer.clone())
        }
    }

    /// Gets the property value at `index`.
    pub fn get(&self, index: usize) -> Option<Value> {
        if index >= self.count {
            return None;
        }

        if self.class_property.r#type == MetadataType::String {
            if let Some(ref str_offsets) = self.string_offsets {
                if index + 1 < str_offsets.len() {
                    let start = str_offsets[index];
                    let end = str_offsets[index + 1];
                    if end <= self.values_buffer.len() && start <= end {
                        let slice = &self.values_buffer[start..end];
                        let s = String::from_utf8_lossy(slice).to_string();
                        return Some(Value::String(s));
                    }
                }
            }
        }

        // Numerical / boolean / enum single scalar fallback
        let size = MetadataComponentType::get_size_in_bytes(self.value_type);
        let byte_offset = index * size;
        if byte_offset + size <= self.values_buffer.len() {
            let slice = &self.values_buffer[byte_offset..byte_offset + size];
            return match self.value_type {
                MetadataComponentType::Int8 => Some(Value::from(slice[0] as i8)),
                MetadataComponentType::Uint8 => Some(Value::from(slice[0])),
                MetadataComponentType::Int16 => {
                    Some(Value::from(i16::from_le_bytes([slice[0], slice[1]])))
                }
                MetadataComponentType::Uint16 => {
                    Some(Value::from(u16::from_le_bytes([slice[0], slice[1]])))
                }
                MetadataComponentType::Int32 => {
                    Some(Value::from(i32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])))
                }
                MetadataComponentType::Uint32 => {
                    Some(Value::from(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])))
                }
                MetadataComponentType::Int64 => {
                    let val = i64::from_le_bytes([
                        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                    ]);
                    Some(Value::from(val))
                }
                MetadataComponentType::Uint64 => {
                    let val = u64::from_le_bytes([
                        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                    ]);
                    Some(Value::from(val))
                }
                MetadataComponentType::Float32 => {
                    let val = f32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]);
                    serde_json::Number::from_f64(val as f64).map(Value::Number)
                }
                MetadataComponentType::Float64 => {
                    let val = f64::from_le_bytes([
                        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                    ]);
                    serde_json::Number::from_f64(val).map(Value::Number)
                }
            };
        }

        self.class_property.default.clone()
    }

    /// Sets the property value at `index`.
    pub fn set(&mut self, index: usize, value: Value) -> Result<(), DeveloperError> {
        if index >= self.count {
            return Err(DeveloperError::new("index out of bounds"));
        }

        if self.class_property.r#type == MetadataType::String {
            let s = value.as_str().ok_or_else(|| DeveloperError::new("Expected string"))?;
            let s_bytes = s.as_bytes();
            if let Some(ref mut str_offsets) = self.string_offsets {
                if index + 1 < str_offsets.len() {
                    let start = str_offsets[index];
                    let end = str_offsets[index + 1];
                    let old_len = end.saturating_sub(start);
                    let new_len = s_bytes.len();

                    self.values_buffer.splice(start..end, s_bytes.iter().copied());

                    if new_len != old_len {
                        let diff = new_len as isize - old_len as isize;
                        for offset in &mut str_offsets[index + 1..] {
                            *offset = (*offset as isize + diff).max(0) as usize;
                        }
                    }
                    return Ok(());
                }
            }
        }

        let size = MetadataComponentType::get_size_in_bytes(self.value_type);
        let byte_offset = index * size;
        if byte_offset + size > self.values_buffer.len() {
            self.values_buffer.resize(byte_offset + size, 0);
        }

        match self.value_type {
            MetadataComponentType::Int8 => {
                let v = value.as_i64().ok_or_else(|| DeveloperError::new("Expected integer"))? as i8;
                self.values_buffer[byte_offset] = v as u8;
            }
            MetadataComponentType::Uint8 => {
                let v = value.as_u64().ok_or_else(|| DeveloperError::new("Expected unsigned integer"))? as u8;
                self.values_buffer[byte_offset] = v;
            }
            MetadataComponentType::Int16 => {
                let v = value.as_i64().ok_or_else(|| DeveloperError::new("Expected integer"))? as i16;
                self.values_buffer[byte_offset..byte_offset + 2].copy_from_slice(&v.to_le_bytes());
            }
            MetadataComponentType::Uint16 => {
                let v = value.as_u64().ok_or_else(|| DeveloperError::new("Expected unsigned integer"))? as u16;
                self.values_buffer[byte_offset..byte_offset + 2].copy_from_slice(&v.to_le_bytes());
            }
            MetadataComponentType::Int32 => {
                let v = value.as_i64().ok_or_else(|| DeveloperError::new("Expected integer"))? as i32;
                self.values_buffer[byte_offset..byte_offset + 4].copy_from_slice(&v.to_le_bytes());
            }
            MetadataComponentType::Uint32 => {
                let v = value.as_u64().ok_or_else(|| DeveloperError::new("Expected unsigned integer"))? as u32;
                self.values_buffer[byte_offset..byte_offset + 4].copy_from_slice(&v.to_le_bytes());
            }
            MetadataComponentType::Int64 => {
                let v = value.as_i64().ok_or_else(|| DeveloperError::new("Expected integer"))?;
                self.values_buffer[byte_offset..byte_offset + 8].copy_from_slice(&v.to_le_bytes());
            }
            MetadataComponentType::Uint64 => {
                let v = value.as_u64().ok_or_else(|| DeveloperError::new("Expected unsigned integer"))?;
                self.values_buffer[byte_offset..byte_offset + 8].copy_from_slice(&v.to_le_bytes());
            }
            MetadataComponentType::Float32 => {
                let v = value.as_f64().ok_or_else(|| DeveloperError::new("Expected float"))? as f32;
                if v.is_nan() || v.is_infinite() {
                    return Err(DeveloperError::new("Value cannot be NaN or Infinity"));
                }
                self.values_buffer[byte_offset..byte_offset + 4].copy_from_slice(&v.to_le_bytes());
            }
            MetadataComponentType::Float64 => {
                let v = value.as_f64().ok_or_else(|| DeveloperError::new("Expected float"))?;
                if v.is_nan() || v.is_infinite() {
                    return Err(DeveloperError::new("Value cannot be NaN or Infinity"));
                }
                self.values_buffer[byte_offset..byte_offset + 8].copy_from_slice(&v.to_le_bytes());
            }
        }
        Ok(())
    }

    fn decode_offsets(
        buf: &[u8],
        component_type: MetadataComponentType,
        expected_count: usize,
    ) -> Result<Vec<usize>, DeveloperError> {
        let size = MetadataComponentType::get_size_in_bytes(component_type);
        let mut offsets = Vec::with_capacity(expected_count);
        for i in 0..expected_count {
            let start = i * size;
            if start + size > buf.len() {
                break;
            }
            let slice = &buf[start..start + size];
            let offset_val: usize = match component_type {
                MetadataComponentType::Uint8 => slice[0] as usize,
                MetadataComponentType::Uint16 => u16::from_le_bytes([slice[0], slice[1]]) as usize,
                MetadataComponentType::Uint32 => {
                    u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]) as usize
                }
                MetadataComponentType::Uint64 => {
                    u64::from_le_bytes([
                        slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                    ]) as usize
                }
                _ => {
                    return Err(DeveloperError::new("Offset type must be unsigned integer"));
                }
            };
            offsets.push(offset_val);
        }
        Ok(offsets)
    }
}
