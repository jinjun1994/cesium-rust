//! Tests adapted from CesiumJS Scene/I3SSymbologySpec.js (Apache-2.0).

use std::collections::HashMap;
use serde_json::{json, Value};

use cesium_renderer::i3s_symbology::I3SSymbology;

fn default_symbol() -> Value {
    json!({
        "symbolLayers": [
            {
                "type": "Fill",
                "edges": {
                    "color": [255, 255, 255],
                    "transparency": 0
                },
                "material": {
                    "colorMixMode": "replace",
                    "color": [0, 0, 0],
                    "transparency": 10
                }
            }
        ]
    })
}

fn simple_symbology_data() -> Value {
    json!({
        "data": {
            "drawingInfo": {
                "renderer": {
                    "type": "simple",
                    "symbol": default_symbol()
                }
            }
        }
    })
}

fn unique_value_infos_symbology_data() -> Value {
    json!({
        "data": {
            "drawingInfo": {
                "renderer": {
                    "type": "uniqueValue",
                    "defaultSymbol": default_symbol(),
                    "field1": "c",
                    "uniqueValueInfos": [
                        {
                            "value": "c1",
                            "symbol": {
                                "symbolLayers": [
                                    {
                                        "type": "Fill",
                                        "edges": {
                                            "color": [255, 255, 255, 255]
                                        }
                                    }
                                ]
                            }
                        },
                        {
                            "value": "c3",
                            "symbol": {
                                "symbolLayers": [
                                    {
                                        "type": "Fill",
                                        "edges": {},
                                        "material": {
                                            "colorMixMode": "replace"
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn unique_value_groups_symbology_data() -> Value {
    json!({
        "data": {
            "drawingInfo": {
                "renderer": {
                    "type": "uniqueValue",
                    "defaultSymbol": default_symbol(),
                    "field1": "a",
                    "field2": "c",
                    "field3": "b",
                    "uniqueValueGroups": [
                        {
                            "classes": [
                                {
                                    "values": [
                                        [10, "c1", "b2"],
                                        [15, "c2", "b2"]
                                    ],
                                    "symbol": {
                                        "symbolLayers": [
                                            {
                                                "type": "Fill",
                                                "edges": {
                                                    "color": [0, 0, 0]
                                                }
                                            }
                                        ]
                                    }
                                },
                                {
                                    "values": [[30, "c3", "b3"]],
                                    "symbol": {
                                        "symbolLayers": [
                                            {
                                                "type": "Fill",
                                                "edges": {
                                                    "color": [0, 0, 0, 0]
                                                }
                                            }
                                        ]
                                    }
                                }
                            ]
                        },
                        {
                            "classes": [
                                {
                                    "values": [[10, "c1", "b1"]],
                                    "symbol": {
                                        "symbolLayers": [
                                            {
                                                "type": "Fill",
                                                "material": {
                                                    "color": [0, 0, 0, 102]
                                                }
                                            }
                                        ]
                                    }
                                }
                            ]
                        }
                    ]
                }
            }
        }
    })
}

fn class_breaks_symbology_data() -> Value {
    json!({
        "data": {
            "drawingInfo": {
                "renderer": {
                    "type": "classBreaks",
                    "defaultSymbol": default_symbol(),
                    "field": "a",
                    "minValue": 12,
                    "classBreakInfos": [
                        {
                            "classMaxValue": 15,
                            "symbol": {
                                "symbolLayers": [
                                    {
                                        "type": "Fill",
                                        "outline": {
                                            "color": [0, 0, 0, 255]
                                        }
                                    }
                                ]
                            }
                        },
                        {
                            "classMinValue": 20,
                            "classMaxValue": 25,
                            "symbol": {
                                "symbolLayers": [
                                    {
                                        "type": "Fill",
                                        "outline": {}
                                    }
                                ]
                            }
                        },
                        {
                            "classMinValue": 25,
                            "classMaxValue": 26,
                            "symbol": {
                                "symbolLayers": [
                                    {
                                        "type": "Fill",
                                        "outline": {
                                            "color": [0, 255, 0, 255]
                                        }
                                    }
                                ]
                            }
                        },
                        {
                            "classMaxValue": 12,
                            "symbol": {
                                "symbolLayers": [
                                    {
                                        "type": "Fill",
                                        "outline": {
                                            "color": [0, 0, 255, 255]
                                        }
                                    }
                                ]
                            }
                        },
                        {
                            "classMinValue": 17,
                            "symbol": {
                                "symbolLayers": [
                                    {
                                        "type": "Fill",
                                        "outline": {
                                            "color": [255, 0, 0, 255]
                                        }
                                    }
                                ]
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn make_node_fields() -> HashMap<String, Vec<Value>> {
    let mut fields = HashMap::new();
    fields.insert("a".to_string(), vec![json!(10), json!(15), json!(30)]);
    fields.insert("b".to_string(), vec![json!("b1"), json!("b2"), json!("b3"), json!("b4")]);
    fields.insert("c".to_string(), vec![json!("c1"), json!("c2")]);
    fields
}

#[test]
fn get_simple_symbology() {
    let symbology = I3SSymbology::new(&simple_symbology_data());
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    let def = symbology.default_symbology.as_ref().unwrap();
    assert_eq!(def.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![1.0, 1.0, 1.0, 1.0]);
    assert_eq!(def.material.as_ref().unwrap().color_mix_mode.as_deref(), Some("replace"));
    assert_eq!(def.material.as_ref().unwrap().color.as_ref().unwrap(), &vec![0.0, 0.0, 0.0, 0.9]);

    let data_def = data.default_symbology.as_ref().unwrap();
    assert_eq!(data_def.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![1.0, 1.0, 1.0, 1.0]);
    assert_eq!(data_def.material.as_ref().unwrap().color_mix_mode.as_deref(), Some("replace"));
    assert_eq!(data_def.material.as_ref().unwrap().color.as_ref().unwrap(), &vec![0.0, 0.0, 0.0, 0.9]);
}

#[test]
fn symbology_without_drawing_info() {
    let symbology = I3SSymbology::new(&json!({ "data": {} }));
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(symbology.default_symbology.is_none());
    assert!(data.default_symbology.is_none());
}

#[test]
fn symbology_without_renderer() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["drawingInfo"] = json!({});
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(symbology.default_symbology.is_none());
    assert!(data.default_symbology.is_none());
}

#[test]
fn symbology_with_unsupported_renderer() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["type"] = json!("unsupported");
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(symbology.default_symbology.is_none());
    assert!(data.default_symbology.is_none());
}

#[test]
fn symbology_without_symbol() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["symbol"] = Value::Null;
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(symbology.default_symbology.is_none());
    assert!(data.default_symbology.is_none());
}

#[test]
fn symbology_without_symbol_layers() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["symbol"] = json!({});
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(symbology.default_symbology.is_none());
    assert!(data.default_symbology.is_none());
}

#[test]
fn symbology_with_empty_symbol_layers() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["symbol"] = json!({ "symbolLayers": [] });
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    let def = symbology.default_symbology.as_ref().unwrap();
    assert!(def.edges.is_none());
    assert!(def.material.is_none());
    assert!(data.default_symbology.is_some());
}

#[test]
fn symbology_with_unsupported_symbol_layer() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["symbol"] = json!({
        "symbolLayers": [{ "type": "unsupported" }]
    });
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    let def = symbology.default_symbology.as_ref().unwrap();
    assert!(def.edges.is_none());
    assert!(def.material.is_none());
    assert!(data.default_symbology.is_some());
}

#[test]
fn symbology_with_empty_cached_drawing_info() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["cachedDrawingInfo"] = json!({});
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    let def = symbology.default_symbology.as_ref().unwrap();
    assert_eq!(def.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![1.0, 1.0, 1.0, 1.0]);
    assert_eq!(def.material.as_ref().unwrap().color_mix_mode.as_deref(), Some("replace"));
    assert_eq!(def.material.as_ref().unwrap().color.as_ref().unwrap(), &vec![0.0, 0.0, 0.0, 0.9]);
    assert!(data.default_symbology.is_some());
}

#[test]
fn symbology_with_cached_drawing_info_for_color() {
    let mut sym_data = simple_symbology_data();
    sym_data["data"]["cachedDrawingInfo"] = json!({ "color": true });
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    let def = symbology.default_symbology.as_ref().unwrap();
    assert_eq!(def.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![1.0, 1.0, 1.0, 1.0]);
    assert!(def.material.is_none());

    let data_def = data.default_symbology.as_ref().unwrap();
    assert_eq!(data_def.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![1.0, 1.0, 1.0, 1.0]);
    assert!(data_def.material.is_none());
}

#[test]
fn get_unique_values_symbology_by_info() {
    let symbology = I3SSymbology::new(&unique_value_infos_symbology_data());
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());
    let f0 = data.feature_symbols.get(&0).unwrap();
    assert_eq!(f0.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![1.0, 1.0, 1.0, 1.0]);
    assert!(f0.material.is_none());
    assert!(!data.feature_symbols.contains_key(&1));
    assert!(!data.feature_symbols.contains_key(&2));
}

#[test]
fn get_unique_values_symbology_without_values() {
    let mut sym_data = unique_value_infos_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["uniqueValueInfos"] = Value::Null;
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());
    assert!(data.feature_symbols.is_empty());
}

#[test]
fn get_unique_values_symbology_by_group() {
    let symbology = I3SSymbology::new(&unique_value_groups_symbology_data());
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());

    let f0 = data.feature_symbols.get(&0).unwrap();
    assert!(f0.material.is_some());
    assert!(f0.material.as_ref().unwrap().color_mix_mode.is_none());
    let f0_color = f0.material.as_ref().unwrap().color.as_ref().unwrap();
    assert_eq!(f0_color[0], 0.0);
    assert_eq!(f0_color[1], 0.0);
    assert_eq!(f0_color[2], 0.0);
    assert!((f0_color[3] - 0.4).abs() < 1e-2);

    let f1 = data.feature_symbols.get(&1).unwrap();
    assert!(f1.edges.is_some());
    assert_eq!(f1.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![0.0, 0.0, 0.0, 1.0]);
    assert!(f1.material.is_none());

    assert!(!data.feature_symbols.contains_key(&2));
}

#[test]
fn get_unique_values_symbology_without_classes() {
    let mut sym_data = unique_value_groups_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["uniqueValueGroups"] = json!([{}]);
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());
    assert!(data.feature_symbols.is_empty());
}

#[test]
fn get_class_breaks_symbology() {
    let symbology = I3SSymbology::new(&class_breaks_symbology_data());
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());
    assert!(!data.feature_symbols.contains_key(&0));

    let f1 = data.feature_symbols.get(&1).unwrap();
    assert!(f1.edges.is_some());
    assert_eq!(f1.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![0.0, 0.0, 0.0, 1.0]);
    assert!(f1.material.is_none());

    assert!(!data.feature_symbols.contains_key(&2));
}

#[test]
fn get_class_breaks_symbology_without_min_value() {
    let mut sym_data = class_breaks_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["minValue"] = Value::Null;
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());

    let f0 = data.feature_symbols.get(&0).unwrap();
    assert!(f0.edges.is_some());
    assert_eq!(f0.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![0.0, 0.0, 1.0, 1.0]);

    let f1 = data.feature_symbols.get(&1).unwrap();
    assert!(f1.edges.is_some());
    assert_eq!(f1.edges.as_ref().unwrap().color.as_ref().unwrap(), &vec![0.0, 0.0, 0.0, 1.0]);

    assert!(!data.feature_symbols.contains_key(&2));
}

#[test]
fn get_class_breaks_symbology_without_infos() {
    let mut sym_data = class_breaks_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["classBreakInfos"] = Value::Null;
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());
    assert!(data.feature_symbols.is_empty());
}

#[test]
fn get_class_breaks_symbology_with_empty_infos() {
    let mut sym_data = class_breaks_symbology_data();
    sym_data["data"]["drawingInfo"]["renderer"]["classBreakInfos"] = json!([]);
    sym_data["data"]["drawingInfo"]["renderer"]["minValue"] = Value::Null;
    let symbology = I3SSymbology::new(&sym_data);
    let node_fields = make_node_fields();
    let data = symbology.get_symbology(&node_fields);

    assert!(data.default_symbology.is_some());
    assert!(data.feature_symbols.is_empty());
}
