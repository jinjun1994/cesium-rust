use serde_json::Value;

/// Clones a JSON value, either shallowly or deeply.
pub fn clone_value(value: &Value, deep: bool) -> Value {
    if !deep {
        if let Value::Object(map) = value {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k.clone(), v.clone());
            }
            return Value::Object(new_map);
        }
    }
    value.clone()
}
