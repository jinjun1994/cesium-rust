//! I3SSymbology matching CesiumJS `Scene/I3SSymbology.js` (Apache-2.0).

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use cesium_math::srgb_to_linear;

/// Edges/outline styling within an I3S symbol.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SSymbolEdges {
    pub color: Option<Vec<f64>>,
}

/// Material styling within an I3S symbol.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SSymbolMaterial {
    pub color_mix_mode: Option<String>,
    pub color: Option<Vec<f64>>,
}

/// Parsed symbol structure from drawingInfo.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SSymbol {
    pub edges: Option<I3SSymbolEdges>,
    pub material: Option<I3SSymbolMaterial>,
}

/// Class breaks hash structure for range binary search.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SClassBreaksHash {
    pub ranges: Vec<f64>,
    pub symbols: Vec<Option<I3SSymbol>>,
}

/// Result of evaluating symbology for a node.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SSymbologyResult {
    pub default_symbology: Option<I3SSymbol>,
    pub feature_symbols: HashMap<usize, I3SSymbol>,
}

fn convert_color(color: &[Value], transparency: Option<f64>) -> Vec<f64> {
    let mut converted = Vec::new();
    for (i, c) in color.iter().enumerate() {
        let byte_val = c.as_f64().unwrap_or(0.0);
        let float_color = byte_val / 255.0;
        if i < 3 {
            converted.push(srgb_to_linear(float_color));
        } else {
            converted.push(float_color);
        }
    }
    if converted.len() == 3 {
        if let Some(t) = transparency {
            converted.push(1.0 - t / 100.0);
        } else {
            converted.push(1.0);
        }
    }
    converted
}

fn parse_symbol(symbol: Option<&Value>, is_color_captured: bool) -> Option<I3SSymbol> {
    let sym = symbol?;
    let symbol_layers = sym.get("symbolLayers").and_then(Value::as_array)?;

    let mut edges = None;
    let mut material = None;

    for layer in symbol_layers {
        if layer.get("type").and_then(Value::as_str) == Some("Fill") {
            let edges_val = layer.get("edges");
            let outline_val = layer.get("outline");

            if let Some(e) = edges_val {
                let color = e.get("color").and_then(Value::as_array).map(|c| {
                    let trans = e.get("transparency").and_then(Value::as_f64);
                    convert_color(c, trans)
                });
                edges = Some(I3SSymbolEdges { color });
            } else if let Some(o) = outline_val {
                let color = o.get("color").and_then(Value::as_array).map(|c| {
                    let trans = o.get("transparency").and_then(Value::as_f64);
                    convert_color(c, trans)
                });
                edges = Some(I3SSymbolEdges { color });
            }

            if !is_color_captured {
                if let Some(m) = layer.get("material") {
                    let color_mix_mode = m.get("colorMixMode").and_then(Value::as_str).map(|s| s.to_string());
                    let color = m.get("color").and_then(Value::as_array).map(|c| {
                        let trans = m.get("transparency").and_then(Value::as_f64);
                        convert_color(c, trans)
                    });
                    material = Some(I3SSymbolMaterial {
                        color_mix_mode,
                        color,
                    });
                }
            }
            break;
        }
    }

    Some(I3SSymbol { edges, material })
}

fn build_unique_value_hash(renderer: &Value, is_color_captured: bool) -> Option<Value> {
    if let Some(groups) = renderer.get("uniqueValueGroups").and_then(Value::as_array) {
        let mut root_hash = serde_json::Map::new();
        for group in groups {
            if let Some(classes) = group.get("classes").and_then(Value::as_array) {
                for cls in classes {
                    let class_sym = parse_symbol(cls.get("symbol"), is_color_captured);
                    let class_sym_val = serde_json::to_value(&class_sym).unwrap_or(Value::Null);

                    if let Some(values_arr) = cls.get("values").and_then(Value::as_array) {
                        for val_tuple in values_arr {
                            if let Some(field_vals) = val_tuple.as_array() {
                                let mut cur_map = &mut root_hash;
                                for (idx, f_val) in field_vals.iter().enumerate() {
                                    let key = match f_val {
                                        Value::String(s) => s.clone(),
                                        Value::Number(n) => n.to_string(),
                                        _ => f_val.to_string(),
                                    };
                                    if idx == field_vals.len() - 1 {
                                        cur_map.insert(key, class_sym_val.clone());
                                    } else {
                                        if !cur_map.contains_key(&key) {
                                            cur_map.insert(key.clone(), Value::Object(serde_json::Map::new()));
                                        }
                                        cur_map = cur_map.get_mut(&key).unwrap().as_object_mut().unwrap();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return Some(Value::Object(root_hash));
    }

    if let Some(infos) = renderer.get("uniqueValueInfos").and_then(Value::as_array) {
        let mut root_hash = serde_json::Map::new();
        for info in infos {
            if let Some(val) = info.get("value") {
                let key = match val {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    _ => val.to_string(),
                };
                let sym = parse_symbol(info.get("symbol"), is_color_captured);
                root_hash.insert(key, serde_json::to_value(&sym).unwrap_or(Value::Null));
            }
        }
        return Some(Value::Object(root_hash));
    }

    None
}

fn build_class_breaks_hash(renderer: &Value, is_color_captured: bool) -> Option<I3SClassBreaksHash> {
    let infos_raw = renderer.get("classBreakInfos").and_then(Value::as_array)?;

    let mut class_break_infos = infos_raw.clone();
    class_break_infos.sort_by(|a, b| {
        let a_max = a.get("classMaxValue")
            .or_else(|| a.get("classMinValue"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let b_max = b.get("classMaxValue")
            .or_else(|| b.get("classMinValue"))
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        a_max.partial_cmp(&b_max).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut ranges = Vec::new();
    let mut symbols = Vec::new();

    if let Some(min_val) = renderer.get("minValue").and_then(Value::as_f64) {
        ranges.push(min_val);
        symbols.push(None);
    }

    for info in &class_break_infos {
        if let Some(min_val) = info.get("classMinValue").and_then(Value::as_f64) {
            if ranges.is_empty() || min_val > *ranges.last().unwrap() {
                ranges.push(min_val);
                symbols.push(None);
            }
        }
        if let Some(max_val) = info.get("classMaxValue").and_then(Value::as_f64) {
            if ranges.is_empty() || max_val > *ranges.last().unwrap() {
                ranges.push(max_val);
                symbols.push(parse_symbol(info.get("symbol"), is_color_captured));
            }
        }
    }
    symbols.push(None);

    Some(I3SClassBreaksHash { ranges, symbols })
}

fn bisect(array: &[f64], value: f64) -> usize {
    let mut low = 0;
    let mut high = array.len();
    while low < high {
        let mid = (low + high) / 2;
        if array[mid] < value {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    low
}

/// I3S symbology evaluator for layers and nodes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SSymbology {
    pub default_symbology: Option<I3SSymbol>,
    pub value_fields: Vec<String>,
    pub unique_value_hash: Option<Value>,
    pub class_breaks_hash: Option<I3SClassBreaksHash>,
}

impl I3SSymbology {
    /// Creates a new `I3SSymbology` by parsing layer symbology definition.
    pub fn new(layer_obj: &Value) -> Self {
        let layer_data = layer_obj.get("data").unwrap_or(layer_obj);
        let drawing_info = layer_data.get("drawingInfo");

        let mut default_symbology = None;
        let mut value_fields = Vec::new();
        let mut unique_value_hash = None;
        let mut class_breaks_hash = None;

        if let Some(di) = drawing_info {
            if let Some(renderer) = di.get("renderer") {
                let cached_drawing_info = layer_data.get("cachedDrawingInfo");
                let is_color_captured = cached_drawing_info
                    .and_then(|c| c.get("color"))
                    .and_then(Value::as_bool)
                    == Some(true);

                let r_type = renderer.get("type").and_then(Value::as_str).unwrap_or("");
                match r_type {
                    "simple" => {
                        default_symbology = parse_symbol(renderer.get("symbol"), is_color_captured);
                    }
                    "uniqueValue" => {
                        default_symbology = parse_symbol(renderer.get("defaultSymbol"), is_color_captured);
                        if let Some(f1) = renderer.get("field1").and_then(Value::as_str) {
                            value_fields.push(f1.to_string());
                        }
                        if let Some(f2) = renderer.get("field2").and_then(Value::as_str) {
                            value_fields.push(f2.to_string());
                        }
                        if let Some(f3) = renderer.get("field3").and_then(Value::as_str) {
                            value_fields.push(f3.to_string());
                        }
                        unique_value_hash = build_unique_value_hash(renderer, is_color_captured);
                    }
                    "classBreaks" => {
                        default_symbology = parse_symbol(renderer.get("defaultSymbol"), is_color_captured);
                        if let Some(f) = renderer.get("field").and_then(Value::as_str) {
                            value_fields.push(f.to_string());
                        }
                        class_breaks_hash = build_class_breaks_hash(renderer, is_color_captured);
                    }
                    _ => {}
                }
            }
        }

        Self {
            default_symbology,
            value_fields,
            unique_value_hash,
            class_breaks_hash,
        }
    }

    /// Evaluates symbology for a set of node fields.
    pub fn get_symbology(&self, fields: &HashMap<String, Vec<Value>>) -> I3SSymbologyResult {
        let mut result = I3SSymbologyResult {
            default_symbology: self.default_symbology.clone(),
            feature_symbols: HashMap::new(),
        };

        if self.value_fields.is_empty() {
            return result;
        }

        let mut fields_values = Vec::new();
        for field_name in &self.value_fields {
            if let Some(vals) = fields.get(field_name) {
                fields_values.push(vals.as_slice());
            } else {
                fields_values.push(&[]);
            }
        }

        if let Some(uv_hash) = &self.unique_value_hash {
            if let Some(first_field) = fields_values.first() {
                for feat_idx in 0..first_field.len() {
                    let mut cur = uv_hash;
                    let mut matched_sym = None;

                    for level_vals in &fields_values {
                        if feat_idx < level_vals.len() {
                            let key = match &level_vals[feat_idx] {
                                Value::String(s) => s.clone(),
                                Value::Number(n) => n.to_string(),
                                _ => level_vals[feat_idx].to_string(),
                            };
                            if let Some(next_val) = cur.get(&key) {
                                if next_val.is_object() && (next_val.get("edges").is_some() || next_val.get("material").is_some()) {
                                    matched_sym = serde_json::from_value::<I3SSymbol>(next_val.clone()).ok();
                                    break;
                                } else {
                                    cur = next_val;
                                }
                            } else {
                                break;
                            }
                        }
                    }

                    if let Some(sym) = matched_sym {
                        result.feature_symbols.insert(feat_idx, sym);
                    }
                }
            }
        } else if let Some(cb_hash) = &self.class_breaks_hash {
            if let Some(first_field) = fields_values.first() {
                for (feat_idx, val) in first_field.iter().enumerate() {
                    let num_val = match val {
                        Value::Number(n) => n.as_f64().unwrap_or(0.0),
                        _ => 0.0,
                    };
                    let range_idx = bisect(&cb_hash.ranges, num_val);
                    if range_idx < cb_hash.symbols.len() {
                        if let Some(sym) = &cb_hash.symbols[range_idx] {
                            result.feature_symbols.insert(feat_idx, sym.clone());
                        }
                    }
                }
            }
        }

        result
    }
}
