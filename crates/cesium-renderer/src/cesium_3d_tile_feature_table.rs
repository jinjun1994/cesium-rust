use cesium_math::ComponentDatatype;
use serde_json::Value;

/// Feature table for a 3D Tile payload (such as b3dm, i3dm, pnts).
#[derive(Debug, Clone, Default)]
pub struct Cesium3DTileFeatureTable {
    json: Value,
    buffer: Option<Vec<u8>>,
    /// Number of features in the tile.
    pub features_length: usize,
}

impl Cesium3DTileFeatureTable {
    /// Creates a new `Cesium3DTileFeatureTable` from JSON header and optional binary buffer.
    pub fn new(json: Value, buffer: Option<Vec<u8>>) -> Self {
        Self {
            json,
            buffer,
            features_length: 0,
        }
    }

    /// Checks if a property with the given semantic name exists.
    pub fn has_property(&self, semantic: &str) -> bool {
        self.json.get(semantic).is_some()
    }

    /// Gets a global property from the feature table.
    pub fn get_global_property(
        &self,
        semantic: &str,
        component_type: Option<ComponentDatatype>,
        component_length: Option<usize>,
    ) -> Option<Vec<f64>> {
        let json_value = self.json.get(semantic)?;
        if let Some(arr) = json_value.as_array() {
            return Some(arr.iter().filter_map(Value::as_f64).collect());
        }
        if let Some(obj) = json_value.as_object() {
            if let Some(offset) = obj.get("byteOffset").and_then(Value::as_u64) {
                let comp_type = component_type.unwrap_or(ComponentDatatype::UnsignedInt);
                let comp_len = component_length.unwrap_or(1);
                let buf = self.buffer.as_ref()?;
                return Self::read_from_buffer(buf, offset as usize, comp_type, comp_len);
            }
        }
        None
    }

    /// Gets all values for a feature property as a contiguous vector of floats.
    pub fn get_property_array(
        &self,
        semantic: &str,
        component_type: Option<ComponentDatatype>,
        component_length: usize,
    ) -> Option<Vec<f64>> {
        let json_value = self.json.get(semantic)?;
        if let Some(arr) = json_value.as_array() {
            return Some(arr.iter().filter_map(Value::as_f64).collect());
        }
        if let Some(obj) = json_value.as_object() {
            if let Some(offset) = obj.get("byteOffset").and_then(Value::as_u64) {
                let comp_type = obj
                    .get("componentType")
                    .and_then(Value::as_str)
                    .and_then(ComponentDatatype::from_name)
                    .or(component_type)
                    .unwrap_or(ComponentDatatype::UnsignedByte);
                let total_elements = self.features_length * component_length;
                let buf = self.buffer.as_ref()?;
                return Self::read_from_buffer(buf, offset as usize, comp_type, total_elements);
            }
        }
        None
    }

    /// Gets property components for a specific feature into the provided slice.
    pub fn get_property(
        &self,
        semantic: &str,
        component_type: Option<ComponentDatatype>,
        component_length: usize,
        feature_id: usize,
        result: &mut [f64],
    ) -> Option<()> {
        let array = self.get_property_array(semantic, component_type, component_length)?;
        if component_length == 1 {
            if feature_id < array.len() {
                if !result.is_empty() {
                    result[0] = array[feature_id];
                }
                return Some(());
            }
        } else {
            let start = component_length * feature_id;
            let end = start + component_length;
            if end <= array.len() && result.len() >= component_length {
                result[..component_length].copy_from_slice(&array[start..end]);
                return Some(());
            }
        }
        None
    }

    fn read_from_buffer(
        buffer: &[u8],
        byte_offset: usize,
        component_type: ComponentDatatype,
        count: usize,
    ) -> Option<Vec<f64>> {
        let elem_size = component_type.size_in_bytes();
        let total_bytes = count * elem_size;
        if byte_offset + total_bytes > buffer.len() {
            return None;
        }
        let slice = &buffer[byte_offset..byte_offset + total_bytes];
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let off = i * elem_size;
            let val = match component_type {
                ComponentDatatype::Byte => slice[off] as i8 as f64,
                ComponentDatatype::UnsignedByte => slice[off] as f64,
                ComponentDatatype::Short => {
                    i16::from_le_bytes([slice[off], slice[off + 1]]) as f64
                }
                ComponentDatatype::UnsignedShort => {
                    u16::from_le_bytes([slice[off], slice[off + 1]]) as f64
                }
                ComponentDatatype::Int => i32::from_le_bytes([
                    slice[off],
                    slice[off + 1],
                    slice[off + 2],
                    slice[off + 3],
                ]) as f64,
                ComponentDatatype::UnsignedInt => u32::from_le_bytes([
                    slice[off],
                    slice[off + 1],
                    slice[off + 2],
                    slice[off + 3],
                ]) as f64,
                ComponentDatatype::Float => f32::from_le_bytes([
                    slice[off],
                    slice[off + 1],
                    slice[off + 2],
                    slice[off + 3],
                ]) as f64,
                ComponentDatatype::Double => f64::from_le_bytes([
                    slice[off],
                    slice[off + 1],
                    slice[off + 2],
                    slice[off + 3],
                    slice[off + 4],
                    slice[off + 5],
                    slice[off + 6],
                    slice[off + 7],
                ]),
            };
            out.push(val);
        }
        Some(out)
    }
}
