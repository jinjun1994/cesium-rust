//! glTF pipeline helper functions matching CesiumJS `packages/gltf/Source` (Apache-2.0).

use serde_json::{json, Value};

/// Adds an element to an array in a `Value` and returns the element's index.
///
/// If `check_duplicates` is true and an equal element is already found, returns its index
/// without adding a duplicate.
pub fn add_to_array(array: &mut Value, element: Value, check_duplicates: bool) -> usize {
    if !array.is_array() {
        *array = Value::Array(Vec::new());
    }
    let arr = array.as_array_mut().unwrap();

    if check_duplicates {
        if let Some(pos) = arr.iter().position(|item| *item == element) {
            return pos;
        }
    }

    arr.push(element);
    arr.len() - 1
}

/// Adds a buffer to glTF and returns the ID of the newly added bufferView.
///
/// # Arguments
/// * `gltf` - A mutable reference to the glTF JSON `Value`.
/// * `buffer` - A slice of raw byte data.
///
/// # Returns
/// The index (ID) of the newly created bufferView.
pub fn add_buffer(gltf: &mut Value, buffer: &[u8]) -> usize {
    if gltf.get("buffers").is_none() {
        gltf["buffers"] = Value::Array(Vec::new());
    }
    if gltf.get("bufferViews").is_none() {
        gltf["bufferViews"] = Value::Array(Vec::new());
    }

    let byte_length = buffer.len();
    let source_json = Value::Array(buffer.iter().map(|&b| json!(b)).collect());

    let new_buffer = json!({
        "byteLength": byte_length,
        "extras": {
            "_pipeline": {
                "source": source_json
            }
        }
    });

    let buffer_id = add_to_array(&mut gltf["buffers"], new_buffer, false);

    let buffer_view = json!({
        "buffer": buffer_id,
        "byteOffset": 0,
        "byteLength": byte_length
    });

    add_to_array(&mut gltf["bufferViews"], buffer_view, false)
}
