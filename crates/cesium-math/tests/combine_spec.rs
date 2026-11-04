use cesium_math::combine;
use serde_json::json;

#[test]
fn test_can_combine_shallow_references() {
    let obj1 = json!({
        "x": 1,
        "y": 2,
        "other": {
            "value1": 0
        }
    });
    let obj2 = json!({
        "x": -1,
        "z": 3,
        "other": {
            "value2": 1
        }
    });

    let composite = combine(Some(&obj1), Some(&obj2), false);
    assert_eq!(
        composite,
        json!({
            "x": 1,
            "y": 2,
            "z": 3,
            "other": {
                "value1": 0
            }
        })
    );
}

#[test]
fn test_can_combine_deep_references() {
    let object1 = json!({
        "one": 1,
        "deep": {
            "value1": 10
        }
    });
    let object2 = json!({
        "two": 2,
        "deep": {
            "value1": 5,
            "value2": 11,
            "sub": {
                "val": "a"
            }
        }
    });

    let composite = combine(Some(&object1), Some(&object2), true);
    assert_eq!(
        composite,
        json!({
            "one": 1,
            "two": 2,
            "deep": {
                "value1": 10,
                "value2": 11,
                "sub": {
                    "val": "a"
                }
            }
        })
    );
}

#[test]
fn test_can_accept_undefined_as_either_object() {
    let object = json!({
        "one": 1,
        "deep": {
            "value1": 10
        }
    });

    assert_eq!(combine(None, Some(&object), false), object);
    assert_eq!(combine(None, Some(&object), true), object);
    assert_eq!(combine(Some(&object), None, false), object);
    assert_eq!(combine(Some(&object), None, true), object);

    let empty = json!({});
    assert_eq!(combine(None, None, false), empty);
    assert_eq!(combine(None, None, true), empty);
}
