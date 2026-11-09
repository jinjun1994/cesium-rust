use cesium_math::Cartesian2;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Binary property storage for batch table hierarchy.
#[derive(Debug, Clone)]
struct BinaryProperty {
    #[allow(dead_code)]
    byte_offset: usize,
    #[allow(dead_code)]
    type_name: String,
    component_type: String,
    component_count: usize,
    component_size: usize,
    data: Vec<u8>,
}

impl BinaryProperty {
    #[allow(dead_code)]
    fn byte_length(&self) -> usize {
        self.data.len()
    }

    fn get_value(&self, index: usize) -> Option<Value> {
        let stride = self.component_count * self.component_size;
        let start = index * stride;
        let end = start + stride;
        if end > self.data.len() {
            return None;
        }
        let slice = &self.data[start..end];

        let read_component = |offset: usize| -> f64 {
            match self.component_type.as_str() {
                "BYTE" => slice[offset] as i8 as f64,
                "UNSIGNED_BYTE" => slice[offset] as f64,
                "SHORT" => i16::from_le_bytes([slice[offset], slice[offset + 1]]) as f64,
                "UNSIGNED_SHORT" => u16::from_le_bytes([slice[offset], slice[offset + 1]]) as f64,
                "INT" => i32::from_le_bytes([
                    slice[offset],
                    slice[offset + 1],
                    slice[offset + 2],
                    slice[offset + 3],
                ]) as f64,
                "UNSIGNED_INT" => u32::from_le_bytes([
                    slice[offset],
                    slice[offset + 1],
                    slice[offset + 2],
                    slice[offset + 3],
                ]) as f64,
                "FLOAT" => f32::from_le_bytes([
                    slice[offset],
                    slice[offset + 1],
                    slice[offset + 2],
                    slice[offset + 3],
                ]) as f64,
                "DOUBLE" => f64::from_le_bytes([
                    slice[offset],
                    slice[offset + 1],
                    slice[offset + 2],
                    slice[offset + 3],
                    slice[offset + 4],
                    slice[offset + 5],
                    slice[offset + 6],
                    slice[offset + 7],
                ]),
                _ => 0.0,
            }
        };

        match self.component_count {
            1 => Some(Value::from(read_component(0))),
            2 => {
                let x = read_component(0);
                let y = read_component(self.component_size);
                Some(serde_json::json!({ "x": x, "y": y }))
            }
            3 => {
                let x = read_component(0);
                let y = read_component(self.component_size);
                let z = read_component(self.component_size * 2);
                Some(serde_json::json!({ "x": x, "y": y, "z": z }))
            }
            4 => {
                let x = read_component(0);
                let y = read_component(self.component_size);
                let z = read_component(self.component_size * 2);
                let w = read_component(self.component_size * 3);
                Some(serde_json::json!({ "x": x, "y": y, "z": z, "w": w }))
            }
            _ => None,
        }
    }

    fn set_value(&mut self, index: usize, value: &Value) -> bool {
        let stride = self.component_count * self.component_size;
        let start = index * stride;
        let end = start + stride;
        if end > self.data.len() {
            return false;
        }

        let mut write_component = |offset: usize, num: f64| {
            match self.component_type.as_str() {
                "BYTE" => self.data[start + offset] = num as i8 as u8,
                "UNSIGNED_BYTE" => self.data[start + offset] = num as u8,
                "SHORT" => {
                    let bytes = (num as i16).to_le_bytes();
                    self.data[start + offset..start + offset + 2].copy_from_slice(&bytes);
                }
                "UNSIGNED_SHORT" => {
                    let bytes = (num as u16).to_le_bytes();
                    self.data[start + offset..start + offset + 2].copy_from_slice(&bytes);
                }
                "INT" => {
                    let bytes = (num as i32).to_le_bytes();
                    self.data[start + offset..start + offset + 4].copy_from_slice(&bytes);
                }
                "UNSIGNED_INT" => {
                    let bytes = (num as u32).to_le_bytes();
                    self.data[start + offset..start + offset + 4].copy_from_slice(&bytes);
                }
                "FLOAT" => {
                    let bytes = (num as f32).to_le_bytes();
                    self.data[start + offset..start + offset + 4].copy_from_slice(&bytes);
                }
                "DOUBLE" => {
                    let bytes = num.to_le_bytes();
                    self.data[start + offset..start + offset + 8].copy_from_slice(&bytes);
                }
                _ => {}
            }
        };

        if self.component_count == 1 {
            if let Some(n) = value.as_f64() {
                write_component(0, n);
                return true;
            }
        } else if self.component_count == 2 {
            if let Some(obj) = value.as_object() {
                if let (Some(x), Some(y)) = (obj.get("x").and_then(Value::as_f64), obj.get("y").and_then(Value::as_f64)) {
                    write_component(0, x);
                    write_component(self.component_size, y);
                    return true;
                }
            } else if let Some(arr) = value.as_array() {
                if arr.len() >= 2 {
                    if let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64()) {
                        write_component(0, x);
                        write_component(self.component_size, y);
                        return true;
                    }
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
enum PropertyStorage {
    JsonArray(Vec<Value>),
    Binary(BinaryProperty),
}

impl PropertyStorage {
    fn get_value(&self, index: usize) -> Option<Value> {
        match self {
            Self::JsonArray(arr) => arr.get(index).cloned(),
            Self::Binary(bin) => bin.get_value(index),
        }
    }

    fn set_value(&mut self, index: usize, value: Value) -> bool {
        match self {
            Self::JsonArray(arr) => {
                if index < arr.len() {
                    arr[index] = value;
                    true
                } else {
                    false
                }
            }
            Self::Binary(bin) => bin.set_value(index, &value),
        }
    }
}

/// A class in the batch table hierarchy.
#[derive(Debug, Clone)]
pub struct BatchTableHierarchyClass {
    pub name: String,
    pub length: usize,
    properties: HashMap<String, PropertyStorage>,
}

/// Error type for BatchTableHierarchy.
#[derive(Debug, thiserror::Error)]
pub enum BatchTableHierarchyError {
    #[error("DeveloperError: {0}")]
    Developer(String),
    #[error("RuntimeError: {0}")]
    Runtime(String),
}

impl BatchTableHierarchyError {
    pub fn is_developer_error(&self) -> bool {
        matches!(self, Self::Developer(_))
    }

    pub fn is_runtime_error(&self) -> bool {
        matches!(self, Self::Runtime(_))
    }
}

/// Object for handling the `3DTILES_batch_table_hierarchy` extension.
#[derive(Debug, Clone)]
pub struct BatchTableHierarchy {
    classes: Vec<BatchTableHierarchyClass>,
    class_ids: Vec<usize>,
    class_indexes: Vec<usize>,
    parent_counts: Option<Vec<usize>>,
    parent_indexes: Option<Vec<usize>>,
    parent_ids: Option<Vec<usize>>,
    byte_length: usize,
}

impl BatchTableHierarchy {
    /// Creates a new `BatchTableHierarchy` from JSON extension and optional binary body.
    pub fn new(
        extension: Option<&Value>,
        binary_body: Option<&[u8]>,
    ) -> Result<Self, BatchTableHierarchyError> {
        let ext = extension.ok_or_else(|| {
            BatchTableHierarchyError::Developer("options.extension is required".to_string())
        })?;
        let ext_obj = ext.as_object().ok_or_else(|| {
            BatchTableHierarchyError::Developer("options.extension must be an object".to_string())
        })?;

        let instances_length = ext_obj
            .get("instancesLength")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                BatchTableHierarchyError::Developer("instancesLength is required".to_string())
            })? as usize;

        let mut byte_length = 0usize;

        // Parse classIds
        let (class_ids, class_ids_bytes) = Self::parse_id_array(
            ext_obj.get("classIds"),
            instances_length,
            binary_body,
            "classIds",
        )?;
        byte_length += class_ids_bytes;

        // Parse parentCounts
        let (parent_counts, parent_counts_bytes, parent_indexes) =
            if let Some(pc_val) = ext_obj.get("parentCounts") {
                let (counts, bytes) = Self::parse_id_array(
                    Some(pc_val),
                    instances_length,
                    binary_body,
                    "parentCounts",
                )?;
                let mut indexes = Vec::with_capacity(instances_length);
                let mut offset = 0usize;
                for &cnt in &counts {
                    indexes.push(offset);
                    offset += cnt;
                }
                // parentIndexes: Uint16Array of instances_length
                let parent_indexes_bytes = instances_length * 2;
                (Some(counts), bytes + parent_indexes_bytes, Some(indexes))
            } else {
                (None, 0, None)
            };
        byte_length += parent_counts_bytes;

        // Parse parentIds
        let parent_ids_length = if let Some(ref counts) = parent_counts {
            counts.iter().sum()
        } else {
            instances_length
        };

        let (parent_ids, parent_ids_bytes) = if let Some(pi_val) = ext_obj.get("parentIds") {
            let (ids, bytes) = Self::parse_id_array(
                Some(pi_val),
                parent_ids_length,
                binary_body,
                "parentIds",
            )?;
            (Some(ids), bytes)
        } else {
            (None, 0)
        };
        byte_length += parent_ids_bytes;

        // Parse classes
        let classes_array = ext_obj
            .get("classes")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                BatchTableHierarchyError::Developer("classes array is required".to_string())
            })?;

        let mut classes = Vec::with_capacity(classes_array.len());
        for class_val in classes_array {
            let class_obj = class_val.as_object().ok_or_else(|| {
                BatchTableHierarchyError::Developer("class must be an object".to_string())
            })?;
            let name = class_obj
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let length = class_obj
                .get("length")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;

            let mut properties = HashMap::new();
            if let Some(instances_val) = class_obj.get("instances").and_then(Value::as_object) {
                for (prop_name, prop_val) in instances_val {
                    if let Some(arr) = prop_val.as_array() {
                        properties.insert(
                            prop_name.clone(),
                            PropertyStorage::JsonArray(arr.clone()),
                        );
                    } else if let Some(desc) = prop_val.as_object() {
                        let type_name = desc
                            .get("type")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                BatchTableHierarchyError::Runtime(format!(
                                    "Property '{prop_name}' is missing type"
                                ))
                            })?;
                        let component_type = desc
                            .get("componentType")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                BatchTableHierarchyError::Runtime(format!(
                                    "Property '{prop_name}' is missing componentType"
                                ))
                            })?;
                        let byte_offset = desc
                            .get("byteOffset")
                            .and_then(Value::as_u64)
                            .ok_or_else(|| {
                                BatchTableHierarchyError::Runtime(format!(
                                    "Property '{prop_name}' is missing byteOffset"
                                ))
                            })? as usize;

                        let body = binary_body.ok_or_else(|| {
                            BatchTableHierarchyError::Runtime(
                                "binaryBody is needed and not provided".to_string(),
                            )
                        })?;

                        let component_count = match type_name {
                            "SCALAR" => 1,
                            "VEC2" => 2,
                            "VEC3" => 3,
                            "VEC4" => 4,
                            other => {
                                return Err(BatchTableHierarchyError::Runtime(format!(
                                    "Unsupported type: {other}"
                                )))
                            }
                        };

                        let component_size = match component_type {
                            "BYTE" | "UNSIGNED_BYTE" => 1,
                            "SHORT" | "UNSIGNED_SHORT" => 2,
                            "INT" | "UNSIGNED_INT" | "FLOAT" => 4,
                            "DOUBLE" => 8,
                            other => {
                                return Err(BatchTableHierarchyError::Runtime(format!(
                                    "Unsupported componentType: {other}"
                                )))
                            }
                        };

                        let total_prop_bytes = length * component_count * component_size;
                        if byte_offset + total_prop_bytes > body.len() {
                            return Err(BatchTableHierarchyError::Runtime(
                                "Binary body too short for property".to_string(),
                            ));
                        }

                        let data = body[byte_offset..byte_offset + total_prop_bytes].to_vec();
                        byte_length += total_prop_bytes;

                        properties.insert(
                            prop_name.clone(),
                            PropertyStorage::Binary(BinaryProperty {
                                byte_offset,
                                type_name: type_name.to_string(),
                                component_type: component_type.to_string(),
                                component_count,
                                component_size,
                                data,
                            }),
                        );
                    }
                }
            }

            classes.push(BatchTableHierarchyClass {
                name,
                length,
                properties,
            });
        }

        // Compute classIndexes: Uint16Array of instances_length
        let mut class_counts = vec![0usize; classes.len()];
        let mut class_indexes = Vec::with_capacity(instances_length);
        for &class_id in &class_ids {
            if class_id >= classes.len() {
                return Err(BatchTableHierarchyError::Developer(format!(
                    "Invalid classId {class_id}"
                )));
            }
            class_indexes.push(class_counts[class_id]);
            class_counts[class_id] += 1;
        }
        byte_length += instances_length * 2; // Uint16Array

        let hierarchy = Self {
            classes,
            class_ids,
            class_indexes,
            parent_counts,
            parent_indexes,
            parent_ids,
            byte_length,
        };

        // Validate hierarchy
        hierarchy.validate()?;

        Ok(hierarchy)
    }

    fn parse_id_array(
        value: Option<&Value>,
        length: usize,
        binary_body: Option<&[u8]>,
        field_name: &str,
    ) -> Result<(Vec<usize>, usize), BatchTableHierarchyError> {
        let val = value.ok_or_else(|| {
            BatchTableHierarchyError::Developer(format!("{field_name} is required"))
        })?;

        if let Some(arr) = val.as_array() {
            let ids = arr
                .iter()
                .map(|v| v.as_u64().unwrap_or(0) as usize)
                .collect();
            Ok((ids, 0))
        } else if let Some(desc) = val.as_object() {
            let body = binary_body.ok_or_else(|| {
                BatchTableHierarchyError::Runtime(
                    "binaryBody is needed and not provided".to_string(),
                )
            })?;
            let byte_offset = desc
                .get("byteOffset")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    BatchTableHierarchyError::Runtime(format!(
                        "{field_name}.byteOffset is required"
                    ))
                })? as usize;
            let component_type = desc
                .get("componentType")
                .and_then(Value::as_str)
                .unwrap_or("UNSIGNED_SHORT");

            let (component_size, ids) = match component_type {
                "UNSIGNED_BYTE" => {
                    let total_bytes = length;
                    if byte_offset + total_bytes > body.len() {
                        return Err(BatchTableHierarchyError::Runtime(
                            "Binary body too short".to_string(),
                        ));
                    }
                    let slice = &body[byte_offset..byte_offset + total_bytes];
                    let ids = slice.iter().map(|&b| b as usize).collect();
                    (1, ids)
                }
                "UNSIGNED_SHORT" => {
                    let total_bytes = length * 2;
                    if byte_offset + total_bytes > body.len() {
                        return Err(BatchTableHierarchyError::Runtime(
                            "Binary body too short".to_string(),
                        ));
                    }
                    let slice = &body[byte_offset..byte_offset + total_bytes];
                    let ids = slice
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|c| u16::from_le_bytes([c[0], c[1]]) as usize)
                        .collect();
                    (2, ids)
                }
                "UNSIGNED_INT" => {
                    let total_bytes = length * 4;
                    if byte_offset + total_bytes > body.len() {
                        return Err(BatchTableHierarchyError::Runtime(
                            "Binary body too short".to_string(),
                        ));
                    }
                    let slice = &body[byte_offset..byte_offset + total_bytes];
                    let ids = slice
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]) as usize)
                        .collect();
                    (4, ids)
                }
                other => {
                    return Err(BatchTableHierarchyError::Runtime(format!(
                        "Unsupported componentType for ids: {other}"
                    )))
                }
            };
            Ok((ids, length * component_size))
        } else {
            Err(BatchTableHierarchyError::Developer(format!(
                "Invalid {field_name}"
            )))
        }
    }

    fn validate(&self) -> Result<(), BatchTableHierarchyError> {
        let instances_length = self.class_ids.len();
        let mut stack = Vec::new();

        for i in 0..instances_length {
            self.validate_instance(i, &mut stack)?;
        }
        Ok(())
    }

    fn validate_instance(
        &self,
        instance_index: usize,
        stack: &mut Vec<usize>,
    ) -> Result<(), BatchTableHierarchyError> {
        let instances_length = self.class_ids.len();
        if self.parent_ids.is_none() {
            return Ok(());
        }

        if instance_index >= instances_length {
            return Err(BatchTableHierarchyError::Developer(format!(
                "Parent index {instance_index} exceeds the total number of instances: {instances_length}"
            )));
        }

        if stack.contains(&instance_index) {
            return Err(BatchTableHierarchyError::Developer(
                "Circular dependency detected in the batch table hierarchy.".to_string(),
            ));
        }

        stack.push(instance_index);

        let parent_counts = self.parent_counts.as_ref();
        let parent_indexes = self.parent_indexes.as_ref();
        let parent_ids = self.parent_ids.as_ref().unwrap();

        let parent_count = parent_counts.map_or(1, |pc| pc[instance_index]);
        let parent_index = parent_indexes.map_or(instance_index, |pi| pi[instance_index]);

        for i in 0..parent_count {
            let parent_id = parent_ids[parent_index + i];
            if parent_id != instance_index {
                self.validate_instance(parent_id, stack)?;
            }
        }

        stack.pop();
        Ok(())
    }

    fn traverse_hierarchy<F, R>(&self, instance_index: usize, mut callback: F) -> Option<R>
    where
        F: FnMut(&Self, usize) -> Option<R>,
    {
        if self.parent_ids.is_none() {
            return callback(self, instance_index);
        }

        if let Some(ref parent_counts) = self.parent_counts {
            let parent_indexes = self.parent_indexes.as_ref().unwrap();
            let parent_ids = self.parent_ids.as_ref().unwrap();

            let mut visited = HashSet::new();
            let mut stack = vec![instance_index];

            while let Some(idx) = stack.pop() {
                if !visited.insert(idx) {
                    continue;
                }
                if let Some(r) = callback(self, idx) {
                    return Some(r);
                }
                let parent_count = parent_counts[idx];
                let parent_index = parent_indexes[idx];
                for i in 0..parent_count {
                    let parent_id = parent_ids[parent_index + i];
                    if parent_id != idx {
                        stack.push(parent_id);
                    }
                }
            }
            None
        } else {
            let parent_ids = self.parent_ids.as_ref().unwrap();
            let mut curr = instance_index;
            let mut has_parent = true;
            while has_parent {
                if let Some(r) = callback(self, curr) {
                    return Some(r);
                }
                let parent_id = parent_ids[curr];
                has_parent = parent_id != curr;
                curr = parent_id;
            }
            None
        }
    }

    /// The total byte length used by typed arrays.
    #[inline]
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }

    /// Returns whether the feature has this property.
    pub fn has_property(&self, batch_id: usize, property_id: &str) -> bool {
        self.traverse_hierarchy(batch_id, |hierarchy, idx| {
            let class_id = hierarchy.class_ids[idx];
            if hierarchy.classes[class_id]
                .properties
                .contains_key(property_id)
            {
                Some(true)
            } else {
                None
            }
        })
        .unwrap_or(false)
    }

    /// Returns whether any feature has this property.
    pub fn property_exists(&self, property_id: &str) -> bool {
        self.classes
            .iter()
            .any(|c| c.properties.contains_key(property_id))
    }

    /// Returns an array of property IDs for the given batchId.
    pub fn get_property_ids(&self, batch_id: usize) -> Vec<String> {
        let mut results = Vec::new();
        self.traverse_hierarchy(batch_id, |hierarchy, idx| {
            let class_id = hierarchy.class_ids[idx];
            for name in hierarchy.classes[class_id].properties.keys() {
                if !results.contains(name) {
                    results.push(name.clone());
                }
            }
            None::<()>
        });
        results
    }

    /// Returns a copy of the value of the property with the given ID.
    pub fn get_property(&self, batch_id: usize, property_id: &str) -> Option<Value> {
        self.traverse_hierarchy(batch_id, |hierarchy, idx| {
            let class_id = hierarchy.class_ids[idx];
            let index_in_class = hierarchy.class_indexes[idx];
            hierarchy.classes[class_id]
                .properties
                .get(property_id)
                .and_then(|prop| prop.get_value(index_in_class))
        })
    }

    /// Helper that returns property as `Cartesian2`.
    pub fn get_property_cartesian2(&self, batch_id: usize, property_id: &str) -> Option<Cartesian2> {
        let val = self.get_property(batch_id, property_id)?;
        if let Some(obj) = val.as_object() {
            let x = obj.get("x")?.as_f64()?;
            let y = obj.get("y")?.as_f64()?;
            Some(Cartesian2::new(x, y))
        } else if let Some(arr) = val.as_array() {
            if arr.len() >= 2 {
                let x = arr[0].as_f64()?;
                let y = arr[1].as_f64()?;
                Some(Cartesian2::new(x, y))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Sets the value of the property with the given ID.
    /// Only properties of the instance may be set; parent properties may not be set.
    pub fn set_property(
        &mut self,
        batch_id: usize,
        property_id: &str,
        value: Value,
    ) -> Result<bool, BatchTableHierarchyError> {
        // First traverse to find the instance defining the property
        let target_idx = self.traverse_hierarchy(batch_id, |hierarchy, idx| {
            let class_id = hierarchy.class_ids[idx];
            if hierarchy.classes[class_id]
                .properties
                .contains_key(property_id)
            {
                Some(idx)
            } else {
                None
            }
        });

        match target_idx {
            None => Ok(false),
            Some(idx) => {
                if idx != batch_id {
                    return Err(BatchTableHierarchyError::Developer(format!(
                        "Inherited property \"{property_id}\" is read-only."
                    )));
                }
                let class_id = self.class_ids[idx];
                let index_in_class = self.class_indexes[idx];
                let prop = self.classes[class_id]
                    .properties
                    .get_mut(property_id)
                    .unwrap();
                Ok(prop.set_value(index_in_class, value))
            }
        }
    }

    /// Helper for setting property from `Cartesian2`.
    pub fn set_property_cartesian2(
        &mut self,
        batch_id: usize,
        property_id: &str,
        value: Cartesian2,
    ) -> Result<bool, BatchTableHierarchyError> {
        self.set_property(
            batch_id,
            property_id,
            serde_json::json!({ "x": value.x, "y": value.y }),
        )
    }

    /// Check if a feature belongs to a class with the given name.
    pub fn is_class(&self, batch_id: usize, class_name: &str) -> bool {
        self.traverse_hierarchy(batch_id, |hierarchy, idx| {
            let class_id = hierarchy.class_ids[idx];
            if hierarchy.classes[class_id].name == class_name {
                Some(true)
            } else {
                None
            }
        })
        .unwrap_or(false)
    }

    /// Get the name of the class a given feature belongs to.
    pub fn get_class_name(&self, batch_id: usize) -> Option<&str> {
        let class_id = *self.class_ids.get(batch_id)?;
        self.classes.get(class_id).map(|c| c.name.as_str())
    }
}
