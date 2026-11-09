//! I3SField matching CesiumJS `Scene/I3SField.js` (Apache-2.0).

use cesium_math::RuntimeError;
use serde_json::Value;
use std::collections::HashMap;

/// Storage definition for a single attribute in I3SField.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SAttributeStorageInfo {
    pub key: String,
    pub name: String,
    pub header: Vec<I3SHeaderItem>,
    pub ordering: Vec<String>,
    pub attribute_values: Option<I3SValueDesc>,
    pub object_ids: Option<I3SValueDesc>,
    pub attribute_byte_counts: Option<I3SValueDesc>,
    pub custom_props: HashMap<String, I3SValueDesc>,
}

/// Header item descriptor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SHeaderItem {
    pub property: String,
    pub value_type: String,
}

/// Value descriptor.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SValueDesc {
    pub value_type: String,
    pub values_per_element: Option<usize>,
    pub encoding: Option<String>,
}

/// I3S Field representing custom attribute data attached to nodes.
#[derive(Clone, Debug, Default)]
pub struct I3SField {
    pub storage_info: I3SAttributeStorageInfo,
    pub header: Option<HashMap<String, u64>>,
    pub values: Vec<Value>,
    pub data: Option<Vec<u8>>,
}

fn get_numeric_type_size(value_type: &str) -> usize {
    match value_type {
        "UInt8" | "Int8" => 1,
        "UInt16" | "Int16" => 2,
        "UInt32" | "Int32" | "Oid32" | "Float32" => 4,
        "UInt64" | "Int64" | "Float64" => 8,
        _ => 0,
    }
}

fn get_value_type_size(value_type: &str) -> usize {
    if value_type == "String" {
        1
    } else {
        get_numeric_type_size(value_type)
    }
}

impl I3SField {
    /// Creates a new `I3SField`.
    pub fn new(storage_info: I3SAttributeStorageInfo) -> Self {
        Self {
            storage_info,
            header: None,
            values: Vec::new(),
            data: None,
        }
    }

    /// Gets the name of this field.
    pub fn name(&self) -> &str {
        &self.storage_info.name
    }

    /// Validates the binary header against required header items.
    pub fn validate_header(&self, buffer: &[u8]) -> Result<usize, RuntimeError> {
        let mut header_size = 0;
        for item in &self.storage_info.header {
            header_size += get_value_type_size(&item.value_type);
        }
        if buffer.len() < header_size {
            return Err(RuntimeError::new(format!(
                "Invalid attribute buffer size (field: {}, header: {}, actual: {})",
                self.name(),
                header_size,
                buffer.len()
            )));
        }
        Ok(header_size)
    }

    /// Parses the header properties into the header map.
    pub fn parse_header(&mut self, buffer: &[u8]) -> Result<usize, RuntimeError> {
        let mut offset = 0;
        let mut header_map = HashMap::new();

        for item in &self.storage_info.header {
            let (val, next_offset) = self.parse_value(buffer, &item.value_type, offset)?;
            header_map.insert(item.property.clone(), val.as_u64().unwrap_or(0));
            offset = next_offset;
        }

        self.header = Some(header_map);
        Ok(offset)
    }

    /// Calculates body offset aligned to the value data size.
    pub fn get_body_offset(&self, header_size: usize) -> usize {
        let mut value_size = 0;
        if let Some(desc) = &self.storage_info.attribute_values {
            value_size = get_numeric_type_size(&desc.value_type);
        } else if let Some(desc) = &self.storage_info.object_ids {
            value_size = get_numeric_type_size(&desc.value_type);
        }

        if value_size > 0 {
            let rem = header_size % value_size;
            if rem == 0 {
                header_size
            } else {
                header_size + (value_size - rem)
            }
        } else {
            header_size
        }
    }

    /// Validates body length and structure against count.
    pub fn validate_body(&self, buffer: &[u8], mut offset: usize) -> Result<(), RuntimeError> {
        let count = match &self.header {
            Some(h) => h.get("count").copied().ok_or_else(|| {
                RuntimeError::new(format!(
                    "Invalid attribute buffer (field: {}, count is missing)",
                    self.name()
                ))
            })?,
            None => {
                return Err(RuntimeError::new(format!(
                    "Invalid attribute buffer (field: {}, count is missing)",
                    self.name()
                )))
            }
        };

        if offset > buffer.len() {
            return Err(RuntimeError::new(format!(
                "Invalid attribute buffer size (field: {}, expected: {}, actual: {})",
                self.name(),
                offset,
                buffer.len()
            )));
        }

        let mut attribute_byte_counts_offset = None;

        for ordering_val in &self.storage_info.ordering {
            let item_key = if ordering_val == "ObjectIds" {
                "objectIds"
            } else {
                ordering_val.as_str()
            };

            let desc = self.get_desc_by_key(item_key).ok_or_else(|| {
                RuntimeError::new(format!(
                    "Invalid attribute buffer (field: {}, {} is missing)",
                    self.name(),
                    item_key
                ))
            })?;

            if desc.value_type != "String" {
                if item_key == "attributeByteCounts" {
                    attribute_byte_counts_offset = Some(offset);
                }
                let val_size = get_numeric_type_size(&desc.value_type);
                offset += val_size * (count as usize);
            } else {
                let mut byte_counts_off = attribute_byte_counts_offset.ok_or_else(|| {
                    RuntimeError::new(format!(
                        "Invalid attribute buffer (field: {}, attributeByteCounts is missing)",
                        self.name()
                    ))
                })?;

                let byte_counts_type = self
                    .storage_info
                    .attribute_byte_counts
                    .as_ref()
                    .map(|d| d.value_type.as_str())
                    .unwrap_or("UInt32");

                for _ in 0..count {
                    if byte_counts_off >= buffer.len() {
                        break;
                    }
                    let (parsed_count, next_off) =
                        self.parse_value(buffer, byte_counts_type, byte_counts_off)?;
                    offset += parsed_count.as_u64().unwrap_or(0) as usize;
                    byte_counts_off = next_off;
                }
            }
        }

        if buffer.len() < offset {
            return Err(RuntimeError::new(format!(
                "Invalid attribute buffer size (field: {}, expected: {}, actual: {})",
                self.name(),
                offset,
                buffer.len()
            )));
        }

        Ok(())
    }

    /// Parses body values.
    pub fn parse_body(&mut self, buffer: &[u8], mut offset: usize) -> Result<(), RuntimeError> {
        let count = match &self.header {
            Some(h) => h.get("count").copied().unwrap_or(0) as usize,
            None => 0,
        };

        let mut parsed_map: HashMap<String, Vec<Value>> = HashMap::new();

        for ordering_val in &self.storage_info.ordering {
            let item_key = if ordering_val == "ObjectIds" {
                "objectIds"
            } else {
                ordering_val.as_str()
            };

            if let Some(desc) = self.get_desc_by_key(item_key) {
                let mut list = Vec::with_capacity(count);
                if desc.value_type != "String" {
                    for _ in 0..count {
                        if offset >= buffer.len() {
                            break;
                        }
                        let (val, next_off) = self.parse_value(buffer, &desc.value_type, offset)?;
                        list.push(val);
                        offset = next_off;
                    }
                } else {
                    let byte_counts = parsed_map.get("attributeByteCounts").cloned().unwrap_or_default();
                    for idx in 0..count {
                        let string_len = byte_counts
                            .get(idx)
                            .and_then(Value::as_u64)
                            .unwrap_or(0) as usize;

                        let mut s = String::new();
                        for _ in 0..string_len {
                            if offset >= buffer.len() {
                                break;
                            }
                            let c = buffer[offset];
                            offset += 1;
                            if c != 0 {
                                s.push(c as char);
                            }
                        }
                        list.push(Value::String(s));
                    }
                }
                parsed_map.insert(item_key.to_string(), list);
            }
        }

        if let Some(vals) = parsed_map.remove("attributeValues") {
            self.values = vals;
        } else if let Some(vals) = parsed_map.remove("objectIds") {
            self.values = vals;
        }

        Ok(())
    }

    /// Loads binary buffer into field data and parses values.
    pub fn load_binary(&mut self, buffer: Vec<u8>) -> Result<(), RuntimeError> {
        if buffer.is_empty() || (buffer.len() >= 5 && buffer.starts_with(b"{404}")) {
            return Ok(());
        }

        self.data = Some(buffer.clone());
        if self.validate_header(&buffer).is_err() {
            return Ok(());
        }

        let header_size = self.parse_header(&buffer)?;
        let offset = self.get_body_offset(header_size);
        if self.validate_body(&buffer, offset).is_err() {
            return Ok(());
        }

        self.parse_body(&buffer, offset)?;
        Ok(())
    }

    fn get_desc_by_key(&self, key: &str) -> Option<I3SValueDesc> {
        if key == "attributeValues" {
            self.storage_info.attribute_values.clone()
        } else if key == "objectIds" {
            self.storage_info.object_ids.clone()
        } else if key == "attributeByteCounts" {
            self.storage_info.attribute_byte_counts.clone()
        } else {
            self.storage_info.custom_props.get(key).cloned()
        }
    }

    fn parse_value(
        &self,
        buffer: &[u8],
        value_type: &str,
        offset: usize,
    ) -> Result<(Value, usize), RuntimeError> {
        let size = get_value_type_size(value_type);
        if offset + size > buffer.len() {
            return Err(RuntimeError::new("Buffer overflow while parsing value"));
        }

        let slice = &buffer[offset..offset + size];
        let val = match value_type {
            "UInt8" => Value::from(slice[0]),
            "Int8" => Value::from(slice[0] as i8),
            "UInt16" => Value::from(u16::from_le_bytes(slice.try_into().unwrap())),
            "Int16" => Value::from(i16::from_le_bytes(slice.try_into().unwrap())),
            "UInt32" | "Oid32" => Value::from(u32::from_le_bytes(slice.try_into().unwrap())),
            "Int32" => Value::from(i32::from_le_bytes(slice.try_into().unwrap())),
            "UInt64" => Value::from(u64::from_le_bytes(slice.try_into().unwrap())),
            "Int64" => Value::from(i64::from_le_bytes(slice.try_into().unwrap())),
            "Float32" => Value::from(f32::from_le_bytes(slice.try_into().unwrap())),
            "Float64" => Value::from(f64::from_le_bytes(slice.try_into().unwrap())),
            "String" => Value::String(String::from(slice[0] as char)),
            _ => Value::Null,
        };

        Ok((val, offset + size))
    }
}
