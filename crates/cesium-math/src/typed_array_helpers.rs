use serde::de::DeserializeOwned;

/// Reads a UTF-8 string from a byte slice with optional offset and length.
pub fn get_string_from_typed_array(
    uint8_array: &[u8],
    byte_offset: Option<usize>,
    byte_length: Option<usize>,
) -> Result<String, &'static str> {
    let offset = byte_offset.unwrap_or(0);
    if offset > uint8_array.len() {
        return Err("sub-region exceeds array bounds.");
    }
    let length = byte_length.unwrap_or(uint8_array.len() - offset);
    if offset + length > uint8_array.len() {
        return Err("sub-region exceeds array bounds.");
    }
    let slice = &uint8_array[offset..offset + length];
    String::from_utf8(slice.to_vec()).map_err(|_| "String decoding failed.")
}

/// Parses JSON from a byte slice with optional offset and length.
pub fn get_json_from_typed_array<T: DeserializeOwned>(
    uint8_array: &[u8],
    byte_offset: Option<usize>,
    byte_length: Option<usize>,
) -> Result<T, String> {
    let s = get_string_from_typed_array(uint8_array, byte_offset, byte_length)?;
    serde_json::from_str(&s).map_err(|e| e.to_string())
}
