//! MetadataPicking matching CesiumJS `Scene/MetadataPicking.js` (Apache-2.0).

use crate::metadata_class_property::MetadataClassProperty;
use crate::metadata_component_type::MetadataComponentType;
use crate::metadata_type::MetadataType;
use cesium_math::{Cartesian2, Cartesian3, Cartesian4, DeveloperError, Matrix2, Matrix3, Matrix4};

/// Decoded primitive numeric value from raw buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RawMetadataNumber {
    Int(i64),
    Uint(u64),
    Float(f64),
}

impl RawMetadataNumber {
    pub fn as_f64(&self) -> f64 {
        match *self {
            Self::Int(v) => v as f64,
            Self::Uint(v) => v as f64,
            Self::Float(v) => v,
        }
    }
}

/// Generic decoded metadata value representation.
#[derive(Clone, Debug, PartialEq)]
pub enum MetadataValue {
    Scalar(f64),
    Int(i64),
    Uint(u64),
    Boolean(bool),
    String(String),
    Cartesian2(Cartesian2),
    Cartesian3(Cartesian3),
    Cartesian4(Cartesian4),
    Matrix2(Matrix2),
    Matrix3(Matrix3),
    Matrix4(Matrix4),
    Array(Vec<MetadataValue>),
}

impl MetadataValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Scalar(v) => Some(*v),
            Self::Int(v) => Some(*v as f64),
            Self::Uint(v) => Some(*v as f64),
            _ => None,
        }
    }
}

impl From<f64> for MetadataValue {
    fn from(v: f64) -> Self {
        Self::Scalar(v)
    }
}

impl From<i64> for MetadataValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<i32> for MetadataValue {
    fn from(v: i32) -> Self {
        Self::Int(v as i64)
    }
}

impl From<u64> for MetadataValue {
    fn from(v: u64) -> Self {
        Self::Uint(v)
    }
}

impl From<u32> for MetadataValue {
    fn from(v: u32) -> Self {
        Self::Uint(v as u64)
    }
}

impl From<bool> for MetadataValue {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<String> for MetadataValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for MetadataValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<Cartesian2> for MetadataValue {
    fn from(v: Cartesian2) -> Self {
        Self::Cartesian2(v)
    }
}

impl From<Cartesian3> for MetadataValue {
    fn from(v: Cartesian3) -> Self {
        Self::Cartesian3(v)
    }
}

impl From<Cartesian4> for MetadataValue {
    fn from(v: Cartesian4) -> Self {
        Self::Cartesian4(v)
    }
}

impl From<Matrix2> for MetadataValue {
    fn from(v: Matrix2) -> Self {
        Self::Matrix2(v)
    }
}

impl From<Matrix3> for MetadataValue {
    fn from(v: Matrix3) -> Self {
        Self::Matrix3(v)
    }
}

impl From<Matrix4> for MetadataValue {
    fn from(v: Matrix4) -> Self {
        Self::Matrix4(v)
    }
}

impl<T: Into<MetadataValue>> From<Vec<T>> for MetadataValue {
    fn from(v: Vec<T>) -> Self {
        Self::Array(v.into_iter().map(Into::into).collect())
    }
}

/// Metadata property options for metadata picking with value transforms.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PickingMetadataProperty {
    pub has_value_transform: bool,
    pub offset: Option<MetadataValue>,
    pub scale: Option<MetadataValue>,
}

/// Utility functions for metadata picking.
pub struct MetadataPicking;

impl MetadataPicking {
    /// Returns the value at the specified byte index in the buffer with given component type.
    pub fn decode_raw_metadata_value(
        component_type: MetadataComponentType,
        buffer: &[u8],
        index: usize,
    ) -> Result<RawMetadataNumber, DeveloperError> {
        let size = MetadataComponentType::get_size_in_bytes(component_type);
        if index.checked_add(size).is_none_or(|end| end > buffer.len()) {
            return Err(DeveloperError::new("Index out of bounds"));
        }

        let slice = &buffer[index..index + size];
        match component_type {
            MetadataComponentType::Int8 => Ok(RawMetadataNumber::Int(slice[0] as i8 as i64)),
            MetadataComponentType::Uint8 => Ok(RawMetadataNumber::Uint(slice[0] as u64)),
            MetadataComponentType::Int16 => {
                let bytes = [slice[0], slice[1]];
                Ok(RawMetadataNumber::Int(i16::from_le_bytes(bytes) as i64))
            }
            MetadataComponentType::Uint16 => {
                let bytes = [slice[0], slice[1]];
                Ok(RawMetadataNumber::Uint(u16::from_le_bytes(bytes) as u64))
            }
            MetadataComponentType::Int32 => {
                let bytes = [slice[0], slice[1], slice[2], slice[3]];
                Ok(RawMetadataNumber::Int(i32::from_le_bytes(bytes) as i64))
            }
            MetadataComponentType::Uint32 => {
                let bytes = [slice[0], slice[1], slice[2], slice[3]];
                Ok(RawMetadataNumber::Uint(u32::from_le_bytes(bytes) as u64))
            }
            MetadataComponentType::Int64 => {
                let bytes = [
                    slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                ];
                Ok(RawMetadataNumber::Int(i64::from_le_bytes(bytes)))
            }
            MetadataComponentType::Uint64 => {
                let bytes = [
                    slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                ];
                Ok(RawMetadataNumber::Uint(u64::from_le_bytes(bytes)))
            }
            MetadataComponentType::Float32 => {
                let bytes = [slice[0], slice[1], slice[2], slice[3]];
                Ok(RawMetadataNumber::Float(f32::from_le_bytes(bytes) as f64))
            }
            MetadataComponentType::Float64 => {
                let bytes = [
                    slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                ];
                Ok(RawMetadataNumber::Float(f64::from_le_bytes(bytes)))
            }
        }
    }

    /// Decodes one component of a metadata value, applying normalization if enabled.
    pub fn decode_raw_metadata_value_component(
        class_property: &MetadataClassProperty,
        buffer: &[u8],
        offset: usize,
    ) -> Result<RawMetadataNumber, DeveloperError> {
        let component_type = class_property
            .component_type
            .ok_or_else(|| DeveloperError::new("componentType is required"))?;
        let component = Self::decode_raw_metadata_value(component_type, buffer, offset)?;
        if class_property.normalized {
            let normalized =
                MetadataComponentType::normalize(component.as_f64(), component_type)?;
            Ok(RawMetadataNumber::Float(normalized))
        } else {
            Ok(component)
        }
    }

    /// Decodes one element of a metadata value.
    pub fn decode_raw_metadata_value_element(
        class_property: &MetadataClassProperty,
        buffer: &[u8],
        element_index: usize,
    ) -> Result<MetadataValue, DeveloperError> {
        let component_type = class_property
            .component_type
            .ok_or_else(|| DeveloperError::new("componentType is required"))?;
        let component_size = MetadataComponentType::get_size_in_bytes(component_type);
        let component_count = class_property.r#type.get_component_count();
        let element_size = component_size * component_count;

        if element_index
            .checked_mul(element_size)
            .and_then(|start| start.checked_add(element_size))
            .is_none_or(|end| end > buffer.len())
        {
            return Err(DeveloperError::new("Element index out of bounds"));
        }

        if component_count > 1 {
            let mut list = Vec::with_capacity(component_count);
            for i in 0..component_count {
                let offset = element_index * element_size + i * component_size;
                let comp =
                    Self::decode_raw_metadata_value_component(class_property, buffer, offset)?;
                list.push(match comp {
                    RawMetadataNumber::Int(v) => MetadataValue::Int(v),
                    RawMetadataNumber::Uint(v) => MetadataValue::Uint(v),
                    RawMetadataNumber::Float(v) => MetadataValue::Scalar(v),
                });
            }
            Ok(MetadataValue::Array(list))
        } else {
            let offset = element_index * element_size;
            let comp =
                Self::decode_raw_metadata_value_component(class_property, buffer, offset)?;
            Ok(match comp {
                RawMetadataNumber::Int(v) => MetadataValue::Int(v),
                RawMetadataNumber::Uint(v) => MetadataValue::Uint(v),
                RawMetadataNumber::Float(v) => MetadataValue::Scalar(v),
            })
        }
    }

    /// Decodes raw values into an array-based form of a metadata property value.
    pub fn decode_raw_metadata_values(
        class_property: &MetadataClassProperty,
        raw_pixel_values: &[u8],
    ) -> Result<MetadataValue, DeveloperError> {
        if class_property.is_array {
            let array_length = class_property
                .array_length
                .ok_or_else(|| DeveloperError::new("arrayLength is required for array class property"))?;
            let mut list = Vec::with_capacity(array_length);
            for i in 0..array_length {
                let element =
                    Self::decode_raw_metadata_value_element(class_property, raw_pixel_values, i)?;
                list.push(element);
            }
            Ok(MetadataValue::Array(list))
        } else {
            Self::decode_raw_metadata_value_element(class_property, raw_pixel_values, 0)
        }
    }

    /// Converts array-based vectors and matrices into their strongly-typed object representation.
    pub fn convert_to_object_type(
        type_val: MetadataType,
        value: &MetadataValue,
    ) -> Result<MetadataValue, DeveloperError> {
        match type_val {
            MetadataType::Scalar | MetadataType::String | MetadataType::Boolean | MetadataType::Enum => {
                Ok(value.clone())
            }
            MetadataType::Vec2 => {
                let nums = Self::extract_numbers(value)?;
                if nums.len() < 2 {
                    return Err(DeveloperError::new("Insufficient values for VEC2"));
                }
                Cartesian2::unpack(&nums, 0)
                    .map(MetadataValue::Cartesian2)
                    .map_err(|e| DeveloperError::new(e.to_string()))
            }
            MetadataType::Vec3 => {
                let nums = Self::extract_numbers(value)?;
                if nums.len() < 3 {
                    return Err(DeveloperError::new("Insufficient values for VEC3"));
                }
                Cartesian3::unpack(&nums, 0)
                    .map(MetadataValue::Cartesian3)
                    .map_err(|e| DeveloperError::new(e.to_string()))
            }
            MetadataType::Vec4 => {
                let nums = Self::extract_numbers(value)?;
                if nums.len() < 4 {
                    return Err(DeveloperError::new("Insufficient values for VEC4"));
                }
                Cartesian4::unpack(&nums, 0)
                    .map(MetadataValue::Cartesian4)
                    .map_err(|e| DeveloperError::new(e.to_string()))
            }
            MetadataType::Mat2 => {
                let nums = Self::extract_numbers(value)?;
                if nums.len() < 4 {
                    return Err(DeveloperError::new("Insufficient values for MAT2"));
                }
                Matrix2::unpack(&nums, 0)
                    .map(MetadataValue::Matrix2)
                    .map_err(|e| DeveloperError::new(e.to_string()))
            }
            MetadataType::Mat3 => {
                let nums = Self::extract_numbers(value)?;
                if nums.len() < 9 {
                    return Err(DeveloperError::new("Insufficient values for MAT3"));
                }
                Matrix3::unpack(&nums, 0)
                    .map(MetadataValue::Matrix3)
                    .map_err(|e| DeveloperError::new(e.to_string()))
            }
            MetadataType::Mat4 => {
                let nums = Self::extract_numbers(value)?;
                if nums.len() < 16 {
                    return Err(DeveloperError::new("Insufficient values for MAT4"));
                }
                Matrix4::unpack(&nums, 0)
                    .map(MetadataValue::Matrix4)
                    .map_err(|e| DeveloperError::new(e.to_string()))
            }
        }
    }

    /// Converts strongly-typed object types into their array-based representation.
    pub fn convert_from_object_type(
        type_val: MetadataType,
        value: &MetadataValue,
    ) -> Result<MetadataValue, DeveloperError> {
        match type_val {
            MetadataType::Scalar | MetadataType::String | MetadataType::Boolean | MetadataType::Enum => {
                Ok(value.clone())
            }
            MetadataType::Vec2 => match value {
                MetadataValue::Cartesian2(c) => Ok(MetadataValue::Array(vec![
                    MetadataValue::Scalar(c.x),
                    MetadataValue::Scalar(c.y),
                ])),
                MetadataValue::Array(arr) => Ok(MetadataValue::Array(arr.clone())),
                _ => Err(DeveloperError::new("Expected Cartesian2 or array for VEC2")),
            },
            MetadataType::Vec3 => match value {
                MetadataValue::Cartesian3(c) => Ok(MetadataValue::Array(vec![
                    MetadataValue::Scalar(c.x),
                    MetadataValue::Scalar(c.y),
                    MetadataValue::Scalar(c.z),
                ])),
                MetadataValue::Array(arr) => Ok(MetadataValue::Array(arr.clone())),
                _ => Err(DeveloperError::new("Expected Cartesian3 or array for VEC3")),
            },
            MetadataType::Vec4 => match value {
                MetadataValue::Cartesian4(c) => Ok(MetadataValue::Array(vec![
                    MetadataValue::Scalar(c.x),
                    MetadataValue::Scalar(c.y),
                    MetadataValue::Scalar(c.z),
                    MetadataValue::Scalar(c.w),
                ])),
                MetadataValue::Array(arr) => Ok(MetadataValue::Array(arr.clone())),
                _ => Err(DeveloperError::new("Expected Cartesian4 or array for VEC4")),
            },
            MetadataType::Mat2 => match value {
                MetadataValue::Matrix2(m) => {
                    let mut arr = [0.0; 4];
                    m.pack(&mut arr, 0).map_err(|e| DeveloperError::new(e.to_string()))?;
                    Ok(MetadataValue::Array(arr.iter().map(|&v| MetadataValue::Scalar(v)).collect()))
                }
                MetadataValue::Array(arr) => Ok(MetadataValue::Array(arr.clone())),
                _ => Err(DeveloperError::new("Expected Matrix2 or array for MAT2")),
            },
            MetadataType::Mat3 => match value {
                MetadataValue::Matrix3(m) => {
                    let mut arr = [0.0; 9];
                    m.pack(&mut arr, 0).map_err(|e| DeveloperError::new(e.to_string()))?;
                    Ok(MetadataValue::Array(arr.iter().map(|&v| MetadataValue::Scalar(v)).collect()))
                }
                MetadataValue::Array(arr) => Ok(MetadataValue::Array(arr.clone())),
                _ => Err(DeveloperError::new("Expected Matrix3 or array for MAT3")),
            },
            MetadataType::Mat4 => match value {
                MetadataValue::Matrix4(m) => {
                    let mut arr = [0.0; 16];
                    m.pack(&mut arr, 0).map_err(|e| DeveloperError::new(e.to_string()))?;
                    Ok(MetadataValue::Array(arr.iter().map(|&v| MetadataValue::Scalar(v)).collect()))
                }
                MetadataValue::Array(arr) => Ok(MetadataValue::Array(arr.clone())),
                _ => Err(DeveloperError::new("Expected Matrix4 or array for MAT4")),
            },
        }
    }

    /// Decodes pixel values into metadata property values, applying value transform and type conversions.
    pub fn decode_metadata_values(
        class_property: &MetadataClassProperty,
        metadata_property: &PickingMetadataProperty,
        raw_pixel_values: &[u8],
    ) -> Result<MetadataValue, DeveloperError> {
        let mut array_based_result =
            Self::decode_raw_metadata_values(class_property, raw_pixel_values)?;

        if metadata_property.has_value_transform {
            let offset = metadata_property
                .offset
                .as_ref()
                .map(|v| Self::convert_from_object_type(class_property.r#type, v))
                .transpose()?;
            let scale = metadata_property
                .scale
                .as_ref()
                .map(|v| Self::convert_from_object_type(class_property.r#type, v))
                .transpose()?;

            array_based_result = Self::apply_transform(
                &array_based_result,
                offset.as_ref(),
                scale.as_ref(),
            )?;
        }

        if class_property.is_array {
            if let MetadataValue::Array(elements) = array_based_result {
                let mut converted = Vec::with_capacity(elements.len());
                for elem in &elements {
                    converted.push(Self::convert_to_object_type(class_property.r#type, elem)?);
                }
                Ok(MetadataValue::Array(converted))
            } else {
                Self::convert_to_object_type(class_property.r#type, &array_based_result)
            }
        } else {
            Self::convert_to_object_type(class_property.r#type, &array_based_result)
        }
    }

    fn extract_numbers(value: &MetadataValue) -> Result<Vec<f64>, DeveloperError> {
        match value {
            MetadataValue::Array(arr) => {
                let mut nums = Vec::with_capacity(arr.len());
                for item in arr {
                    if let Some(n) = item.as_f64() {
                        nums.push(n);
                    } else {
                        return Err(DeveloperError::new("Expected numeric element"));
                    }
                }
                Ok(nums)
            }
            MetadataValue::Scalar(v) => Ok(vec![*v]),
            MetadataValue::Int(v) => Ok(vec![*v as f64]),
            MetadataValue::Uint(v) => Ok(vec![*v as f64]),
            _ => Err(DeveloperError::new("Expected numeric value or array")),
        }
    }

    fn apply_transform(
        value: &MetadataValue,
        offset: Option<&MetadataValue>,
        scale: Option<&MetadataValue>,
    ) -> Result<MetadataValue, DeveloperError> {
        match value {
            MetadataValue::Array(arr) => {
                match (offset, scale) {
                    (Some(MetadataValue::Array(off_arr)), Some(MetadataValue::Array(scale_arr))) => {
                        let mut res = Vec::with_capacity(arr.len());
                        for (i, item) in arr.iter().enumerate() {
                            let off = off_arr.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let sc = scale_arr.get(i).and_then(|v| v.as_f64()).unwrap_or(1.0);
                            let v = item.as_f64().ok_or_else(|| DeveloperError::new("Expected numeric"))?;
                            res.push(MetadataValue::Scalar(off + v * sc));
                        }
                        Ok(MetadataValue::Array(res))
                    }
                    (Some(MetadataValue::Scalar(off)), Some(MetadataValue::Scalar(sc))) => {
                        let mut res = Vec::with_capacity(arr.len());
                        for item in arr {
                            let v = item.as_f64().ok_or_else(|| DeveloperError::new("Expected numeric"))?;
                            res.push(MetadataValue::Scalar(off + v * sc));
                        }
                        Ok(MetadataValue::Array(res))
                    }
                    _ => {
                        let off_num = offset.and_then(|o| o.as_f64()).unwrap_or(0.0);
                        let sc_num = scale.and_then(|s| s.as_f64()).unwrap_or(1.0);
                        let mut res = Vec::with_capacity(arr.len());
                        for item in arr {
                            let v = item.as_f64().ok_or_else(|| DeveloperError::new("Expected numeric"))?;
                            res.push(MetadataValue::Scalar(off_num + v * sc_num));
                        }
                        Ok(MetadataValue::Array(res))
                    }
                }
            }
            _ => {
                let v = value.as_f64().ok_or_else(|| DeveloperError::new("Expected numeric value"))?;
                let off = offset.and_then(|o| o.as_f64()).unwrap_or(0.0);
                let sc = scale.and_then(|s| s.as_f64()).unwrap_or(1.0);
                Ok(MetadataValue::Scalar(off + v * sc))
            }
        }
    }
}
