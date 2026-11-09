use cesium_math::Cartesian2;
use cesium_renderer::BatchTableHierarchy;
use serde_json::json;

fn hierarchy_extension() -> serde_json::Value {
    json!({
        "classes": [
            {
                "name": "Wall",
                "length": 6,
                "instances": {
                    "color": ["white", "red", "yellow", "gray", "brown", "black"]
                }
            },
            {
                "name": "Building",
                "length": 3,
                "instances": {
                    "name": ["unit29", "unit20", "unit93"],
                    "address": ["100 Main St", "102 Main St", "104 Main St"]
                }
            },
            {
                "name": "Owner",
                "length": 3,
                "instances": {
                    "type": ["city", "resident", "commercial"],
                    "id": [1120, 1250, 6445]
                }
            }
        ],
        "instancesLength": 12,
        "classIds": [0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 2, 2],
        "parentCounts": [1, 3, 2, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        "parentIds": [6, 6, 10, 11, 7, 11, 7, 8, 8, 10, 10, 9]
    })
}

fn binary_hierarchy() -> (serde_json::Value, Vec<u8>) {
    let json = json!({
        "classes": [
            {
                "name": "Box",
                "length": 3,
                "instances": {
                    "items": {
                        "type": "SCALAR",
                        "componentType": "UNSIGNED_BYTE",
                        "byteOffset": 0
                    },
                    "coordinates": {
                        "type": "VEC2",
                        "componentType": "UNSIGNED_BYTE",
                        "byteOffset": 4
                    }
                }
            },
            {
                "name": "Pallet",
                "length": 1,
                "instances": {
                    "boxCount": [1]
                }
            }
        ],
        "instancesLength": 4,
        "classIds": [0, 1, 0, 0],
        "parentIds": [1, 1, 2, 3]
    });
    let body = vec![1, 2, 3, 0, 1, 0, 1, 2, 3, 2];
    (json, body)
}

fn binary_hierarchy_with_binary_ids() -> (serde_json::Value, Vec<u8>) {
    let json = json!({
        "classes": [
            {
                "name": "Box",
                "length": 3,
                "instances": {
                    "items": {
                        "type": "SCALAR",
                        "componentType": "UNSIGNED_BYTE",
                        "byteOffset": 8
                    },
                    "coordinates": {
                        "type": "VEC2",
                        "componentType": "UNSIGNED_BYTE",
                        "byteOffset": 12
                    }
                }
            },
            {
                "name": "Pallet",
                "length": 1,
                "instances": {
                    "boxCount": [1]
                }
            }
        ],
        "instancesLength": 4,
        "classIds": {
            "byteOffset": 18,
            "componentType": "UNSIGNED_BYTE"
        },
        "parentIds": {
            "byteOffset": 22,
            "componentType": "UNSIGNED_BYTE"
        },
        "parentCounts": {
            "byteOffset": 26,
            "componentType": "UNSIGNED_BYTE"
        }
    });
    let body = vec![
        // padding
        0, 0, 0, 0, 0, 0, 0, 0,
        // binary property: items (offset 8)
        1, 2, 3, 0,
        // binary property: coordinates (offset 12)
        1, 0, 1, 2, 3, 2,
        // class IDs (offset 18)
        0, 1, 0, 0,
        // parent IDs (offset 22)
        1, 1, 2, 3,
        // parent Counts (offset 26)
        1, 1, 1, 1,
    ];
    (json, body)
}

#[test]
fn test_throws_without_extension() {
    let res = BatchTableHierarchy::new(None, None);
    assert!(res.is_err());
    assert!(res.unwrap_err().is_developer_error());
}

#[test]
fn test_throws_for_invalid_binary_property() {
    let missing_type = json!({
        "classes": [
            {
                "name": "Resources",
                "length": 2,
                "instances": {
                    "foodUnits": {
                        "byteOffset": 0,
                        "componentType": "UNSIGNED_SHORT"
                    },
                    "waterUnits": [15, 13]
                }
            }
        ],
        "instancesLength": 2,
        "classIds": [0, 0],
        "parentIds": [1, 1]
    });
    let binary_body = vec![10, 0, 20, 0];
    let res1 = BatchTableHierarchy::new(Some(&missing_type), Some(&binary_body));
    assert!(res1.is_err());
    assert!(res1.unwrap_err().is_runtime_error());

    let missing_component_type = json!({
        "classes": [
            {
                "name": "Resources",
                "length": 2,
                "instances": {
                    "foodUnits": {
                        "byteOffset": 0,
                        "type": "SCALAR"
                    },
                    "waterUnits": [15, 13]
                }
            }
        ],
        "instancesLength": 2,
        "classIds": [0, 0],
        "parentIds": [1, 1]
    });
    let res2 = BatchTableHierarchy::new(Some(&missing_component_type), Some(&binary_body));
    assert!(res2.is_err());
    assert!(res2.unwrap_err().is_runtime_error());
}

#[test]
fn test_throws_if_binary_body_is_needed_and_not_provided() {
    let hierarchy_ext = json!({
        "classes": [
            {
                "name": "Resources",
                "length": 2,
                "instances": {
                    "foodUnits": {
                        "byteOffset": 0,
                        "type": "SCALAR",
                        "componentType": "UNSIGNED_SHORT"
                    },
                    "waterUnits": [15, 13]
                }
            }
        ],
        "instancesLength": 2,
        "classIds": [0, 0],
        "parentIds": [1, 1]
    });
    let res = BatchTableHierarchy::new(Some(&hierarchy_ext), None);
    assert!(res.is_err());
    assert!(res.unwrap_err().is_runtime_error());
}

#[test]
fn test_has_property() {
    let ext = hierarchy_extension();
    let hierarchy = BatchTableHierarchy::new(Some(&ext), None).unwrap();

    assert!(hierarchy.has_property(0, "color"));
    assert!(!hierarchy.has_property(0, "height"));
    assert!(!hierarchy.has_property(6, "color"));
}

#[test]
fn test_property_exists() {
    let ext = hierarchy_extension();
    let hierarchy = BatchTableHierarchy::new(Some(&ext), None).unwrap();

    assert!(hierarchy.property_exists("color"));
    assert!(!hierarchy.property_exists("other"));
}

#[test]
fn test_get_property() {
    let ext = hierarchy_extension();
    let hierarchy = BatchTableHierarchy::new(Some(&ext), None).unwrap();

    assert_eq!(hierarchy.get_property(0, "color"), Some(json!("white")));
    assert_eq!(hierarchy.get_property(0, "name"), Some(json!("unit29")));
    assert_eq!(hierarchy.get_property(0, "address"), Some(json!("100 Main St")));
    assert_eq!(hierarchy.get_property(0, "type"), Some(json!("resident")));
    assert_eq!(hierarchy.get_property(0, "id"), Some(json!(1250)));
    assert_eq!(hierarchy.get_property(0, "occupancy"), None);
}

#[test]
fn test_get_property_binary() {
    let (ext, body) = binary_hierarchy();
    let hierarchy = BatchTableHierarchy::new(Some(&ext), Some(&body)).unwrap();

    assert_eq!(hierarchy.get_property(0, "items"), Some(json!(1.0)));
    assert_eq!(
        hierarchy.get_property_cartesian2(0, "coordinates"),
        Some(Cartesian2::new(1.0, 0.0))
    );
    assert_eq!(hierarchy.get_property(0, "boxCount"), Some(json!(1)));

    assert_eq!(hierarchy.get_property(1, "items"), None);
    assert_eq!(hierarchy.get_property_cartesian2(1, "coordinates"), None);
    assert_eq!(hierarchy.get_property(1, "boxCount"), Some(json!(1)));

    assert_eq!(hierarchy.get_property(2, "items"), Some(json!(2.0)));
    assert_eq!(
        hierarchy.get_property_cartesian2(2, "coordinates"),
        Some(Cartesian2::new(1.0, 2.0))
    );
    assert_eq!(hierarchy.get_property(2, "boxCount"), None);

    assert_eq!(hierarchy.get_property(3, "items"), Some(json!(3.0)));
    assert_eq!(
        hierarchy.get_property_cartesian2(3, "coordinates"),
        Some(Cartesian2::new(3.0, 2.0))
    );
    assert_eq!(hierarchy.get_property(3, "boxCount"), None);
}

#[test]
fn test_get_property_binary_with_ids() {
    let (ext, body) = binary_hierarchy_with_binary_ids();
    let hierarchy = BatchTableHierarchy::new(Some(&ext), Some(&body)).unwrap();

    assert_eq!(hierarchy.get_property(0, "items"), Some(json!(1.0)));
    assert_eq!(
        hierarchy.get_property_cartesian2(0, "coordinates"),
        Some(Cartesian2::new(1.0, 0.0))
    );
    assert_eq!(hierarchy.get_property(0, "boxCount"), Some(json!(1)));

    assert_eq!(hierarchy.get_property(1, "items"), None);
    assert_eq!(hierarchy.get_property_cartesian2(1, "coordinates"), None);
    assert_eq!(hierarchy.get_property(1, "boxCount"), Some(json!(1)));

    assert_eq!(hierarchy.get_property(2, "items"), Some(json!(2.0)));
    assert_eq!(
        hierarchy.get_property_cartesian2(2, "coordinates"),
        Some(Cartesian2::new(1.0, 2.0))
    );
    assert_eq!(hierarchy.get_property(2, "boxCount"), None);

    assert_eq!(hierarchy.get_property(3, "items"), Some(json!(3.0)));
    assert_eq!(
        hierarchy.get_property_cartesian2(3, "coordinates"),
        Some(Cartesian2::new(3.0, 2.0))
    );
    assert_eq!(hierarchy.get_property(3, "boxCount"), None);
}

#[test]
fn test_set_property_throws_inherited() {
    let ext = hierarchy_extension();
    let mut hierarchy = BatchTableHierarchy::new(Some(&ext), None).unwrap();

    let res = hierarchy.set_property(0, "type", json!("city"));
    assert!(res.is_err());
    assert!(res.unwrap_err().is_developer_error());
}

#[test]
fn test_set_property_non_existent() {
    let ext = hierarchy_extension();
    let mut hierarchy = BatchTableHierarchy::new(Some(&ext), None).unwrap();

    let res = hierarchy.set_property(0, "occupancy", json!(100)).unwrap();
    assert!(!res);
}

#[test]
fn test_set_property_value() {
    let ext = hierarchy_extension();
    let mut hierarchy = BatchTableHierarchy::new(Some(&ext), None).unwrap();

    assert_eq!(hierarchy.get_property(0, "color"), Some(json!("white")));
    assert!(hierarchy.set_property(0, "color", json!("brown")).unwrap());
    assert_eq!(hierarchy.get_property(0, "color"), Some(json!("brown")));
}

#[test]
fn test_set_property_binary() {
    let (ext, body) = binary_hierarchy();
    let mut hierarchy = BatchTableHierarchy::new(Some(&ext), Some(&body)).unwrap();

    assert_eq!(hierarchy.get_property(0, "items"), Some(json!(1.0)));
    assert!(hierarchy.set_property(0, "items", json!(5.0)).unwrap());
    assert_eq!(hierarchy.get_property(0, "items"), Some(json!(5.0)));

    assert_eq!(
        hierarchy.get_property_cartesian2(2, "coordinates"),
        Some(Cartesian2::new(1.0, 2.0))
    );
    let position = Cartesian2::new(5.0, 5.0);
    assert!(hierarchy.set_property_cartesian2(2, "coordinates", position).unwrap());
    assert_eq!(
        hierarchy.get_property_cartesian2(2, "coordinates"),
        Some(position)
    );
}

#[test]
fn test_set_property_binary_with_ids() {
    let (ext, body) = binary_hierarchy_with_binary_ids();
    let mut hierarchy = BatchTableHierarchy::new(Some(&ext), Some(&body)).unwrap();

    assert_eq!(hierarchy.get_property(0, "items"), Some(json!(1.0)));
    assert!(hierarchy.set_property(0, "items", json!(5.0)).unwrap());
    assert_eq!(hierarchy.get_property(0, "items"), Some(json!(5.0)));

    assert_eq!(
        hierarchy.get_property_cartesian2(2, "coordinates"),
        Some(Cartesian2::new(1.0, 2.0))
    );
    let position = Cartesian2::new(5.0, 5.0);
    assert!(hierarchy.set_property_cartesian2(2, "coordinates", position).unwrap());
    assert_eq!(
        hierarchy.get_property_cartesian2(2, "coordinates"),
        Some(position)
    );
}

#[test]
fn test_validates_hierarchy_with_multiple_parents() {
    let extension = json!({
        "instancesLength": 4,
        "classIds": [0, 1, 1, 2],
        "parentCounts": [2, 1, 1, 0],
        "parentIds": [1, 2, 3, 3],
        "classes": [
            {
                "name": "window",
                "length": 1,
                "instances": {
                    "window_name": ["window0"]
                }
            },
            {
                "name": "door",
                "length": 2,
                "instances": {
                    "door_name": ["door0", "door1"]
                }
            },
            {
                "name": "building",
                "length": 1,
                "instances": {
                    "building_name": ["building0"]
                }
            }
        ]
    });

    let hierarchy = BatchTableHierarchy::new(Some(&extension), None).unwrap();
    let mut ids = hierarchy.get_property_ids(0);
    ids.sort();
    assert_eq!(ids, vec!["building_name", "door_name", "window_name"]);
}

#[test]
fn test_validates_hierarchy_with_multiple_parents_2() {
    let extension = json!({
        "instancesLength": 5,
        "classIds": [0, 1, 1, 2, 3],
        "parentCounts": [3, 1, 2, 1, 0],
        "parentIds": [1, 2, 4, 3, 3, 4, 4],
        "classes": [
            {
                "name": "window",
                "length": 1,
                "instances": {
                    "window_name": ["window0"]
                }
            },
            {
                "name": "door",
                "length": 2,
                "instances": {
                    "door_name": ["door0", "door1"]
                }
            },
            {
                "name": "building",
                "length": 1,
                "instances": {
                    "building_name": ["building0"]
                }
            },
            {
                "name": "zone",
                "length": 1,
                "instances": {
                    "zone_name": ["zone0"]
                }
            }
        ]
    });

    let hierarchy = BatchTableHierarchy::new(Some(&extension), None).unwrap();
    let mut ids = hierarchy.get_property_ids(0);
    ids.sort();
    assert_eq!(
        ids,
        vec!["building_name", "door_name", "window_name", "zone_name"]
    );
    assert!(hierarchy.has_property(1, "zone_name"));
    assert!(hierarchy.has_property(2, "zone_name"));
}

#[test]
fn test_throws_circular_dependency() {
    let extension = json!({
        "instancesLength": 3,
        "classIds": [0, 1, 2],
        "parentIds": [1, 2, 0],
        "classes": [
            {
                "name": "window",
                "length": 1,
                "instances": {
                    "window_name": ["window0"]
                }
            },
            {
                "name": "door",
                "length": 1,
                "instances": {
                    "door_name": ["door0"]
                }
            },
            {
                "name": "building",
                "length": 1,
                "instances": {
                    "building_name": ["building0"]
                }
            }
        ]
    });

    let res = BatchTableHierarchy::new(Some(&extension), None);
    assert!(res.is_err());
    assert!(res.unwrap_err().is_developer_error());
}

#[test]
fn test_throws_circular_dependency_2() {
    let extension = json!({
        "instancesLength": 4,
        "classIds": [0, 1, 2, 0],
        "parentIds": [1, 2, 3, 1],
        "classes": [
            {
                "name": "window",
                "length": 2,
                "instances": {
                    "window_name": ["window0", "window1"]
                }
            },
            {
                "name": "door",
                "length": 1,
                "instances": {
                    "door_name": ["door0"]
                }
            },
            {
                "name": "building",
                "length": 1,
                "instances": {
                    "building_name": ["building0"]
                }
            }
        ]
    });

    let res = BatchTableHierarchy::new(Some(&extension), None);
    assert!(res.is_err());
    assert!(res.unwrap_err().is_developer_error());
}

#[test]
fn test_throws_parent_id_exceeds_length() {
    let extension = json!({
        "instancesLength": 2,
        "classIds": [0, 1],
        "parentIds": [1, 2],
        "classes": [
            {
                "name": "window",
                "length": 1,
                "instances": {
                    "window_name": ["window0"]
                }
            },
            {
                "name": "door",
                "length": 1,
                "instances": {
                    "door_name": ["door0"]
                }
            }
        ]
    });

    let res = BatchTableHierarchy::new(Some(&extension), None);
    assert!(res.is_err());
    assert!(res.unwrap_err().is_developer_error());
}

#[test]
fn test_computes_byte_length() {
    let (ext, body) = binary_hierarchy_with_binary_ids();
    let hierarchy = BatchTableHierarchy::new(Some(&ext), Some(&body)).unwrap();

    let class_ids_size = 4;
    let parent_counts_size = 4;
    let parent_indexes_size = 4 * 2;
    let parent_ids_size = 4;
    let binary_properties_size = 3 + 6;
    let class_indexes_size = 4 * 2;

    let expected_byte_length = class_ids_size
        + parent_counts_size
        + parent_indexes_size
        + parent_ids_size
        + binary_properties_size
        + class_indexes_size;

    assert_eq!(hierarchy.byte_length(), expected_byte_length);
}

#[test]
fn test_is_class_and_class_name() {
    let ext = hierarchy_extension();
    let hierarchy = BatchTableHierarchy::new(Some(&ext), None).unwrap();

    assert_eq!(hierarchy.get_class_name(0), Some("Wall"));
    assert_eq!(hierarchy.get_class_name(6), Some("Building"));
    assert_eq!(hierarchy.get_class_name(9), Some("Owner"));
    assert_eq!(hierarchy.get_class_name(999), None);

    assert!(hierarchy.is_class(0, "Wall"));
    assert!(hierarchy.is_class(0, "Building"));
    assert!(hierarchy.is_class(0, "Owner"));
    assert!(!hierarchy.is_class(6, "Wall"));
    assert!(hierarchy.is_class(6, "Building"));
}
