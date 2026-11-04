use serde_json::{Map, Value};

/// Merges two JSON values, copying properties onto a new combined value.
/// When two objects have the same property, the value on the first object is used.
/// If deep is true, recursively combines child objects.
pub fn combine(
    object1: Option<&Value>,
    object2: Option<&Value>,
    deep: bool,
) -> Value {
    match (object1, object2) {
        (None, None) => Value::Object(Map::new()),
        (Some(o1), None) => o1.clone(),
        (None, Some(o2)) => o2.clone(),
        (Some(Value::Object(map1)), Some(Value::Object(map2))) => {
            let mut result = Map::new();
            for (k, v1) in map1 {
                if deep && v1.is_object() {
                    if let Some(v2) = map2.get(k) {
                        if v2.is_object() {
                            result.insert(k.clone(), combine(Some(v1), Some(v2), deep));
                            continue;
                        }
                    }
                }
                result.insert(k.clone(), v1.clone());
            }
            for (k, v2) in map2 {
                if !result.contains_key(k) {
                    result.insert(k.clone(), v2.clone());
                }
            }
            Value::Object(result)
        }
        (Some(o1), _) => o1.clone(),
    }
}
