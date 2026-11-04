use cesium_math::clone_value;
use serde_json::json;

#[test]
fn test_can_make_shallow_and_deep_clones() {
    let obj = json!({
        "a": 1,
        "b": "s",
        "c": {
            "d": 0,
            "e": {
                "f": {
                    "g": 1
                }
            }
        }
    });

    let shallow = clone_value(&obj, false);
    assert_eq!(shallow, obj);

    let deep = clone_value(&obj, true);
    assert_eq!(deep, obj);
}
