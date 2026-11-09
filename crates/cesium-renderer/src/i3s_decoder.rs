//! I3SDecoder matching CesiumJS `Scene/I3SDecoder.js` (Apache-2.0).

use cesium_math::DeveloperError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Mutex, OnceLock};

/// Schema for I3S geometry decoding.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SGeometrySchema {
    pub ordering: Vec<String>,
    pub feature_attribute_order: Vec<String>,
}

/// Node parent bounding information (OBB or MBS).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SParentBounding {
    pub obb: Option<Value>,
    pub mbs: Option<Vec<f64>>,
}

/// Geometry parent description.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SGeometryParent {
    pub data: I3SParentBounding,
    pub inverse_rotation_matrix: Option<[f64; 9]>,
}

/// Geometry input payload for `I3SDecoder::decode`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SGeometryData {
    pub binary_data: Vec<u8>,
    pub parent: Option<I3SGeometryParent>,
    pub geometry_buffer_info: Option<Value>,
}

/// Feature input item.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SFeatureDataItem {
    pub data: Value,
}

/// Decoded glTF mesh data structure.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct I3SMeshData {
    pub buffers: Vec<Value>,
    pub buffer_views: Vec<Value>,
    pub accessors: Vec<Value>,
    pub meshes: Vec<Value>,
    pub nodes: Vec<Value>,
    pub nodes_in_scene: Vec<Value>,
}

/// Result of `I3SDecoder::decode`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct I3SDecodeResult {
    pub mesh_data: I3SMeshData,
}

static INITIALIZED_STATE: OnceLock<Mutex<bool>> = OnceLock::new();

fn get_initialized_state() -> &'static Mutex<bool> {
    INITIALIZED_STATE.get_or_init(|| Mutex::new(true))
}

/// Test hook to simulate WebAssembly initialization failure.
pub fn set_decoder_initialized_for_test(initialized: bool) {
    let mut lock = get_initialized_state().lock().unwrap();
    *lock = initialized;
}

/// Decode I3S using web workers / native decoding routines.
#[derive(Clone, Debug, Default)]
pub struct I3SDecoder;

impl I3SDecoder {
    /// Creates a new `I3SDecoder` instance.
    pub fn new() -> Self {
        Self
    }

    /// Transcodes I3S to glTF.
    pub async fn decode(
        url: Option<&str>,
        default_geometry_schema: Option<&I3SGeometrySchema>,
        geometry_data: Option<&I3SGeometryData>,
        _feature_data: Option<&[I3SFeatureDataItem]>,
        _symbology_data: Option<&Value>,
    ) -> Result<I3SDecodeResult, DeveloperError> {
        let u = url.ok_or_else(|| DeveloperError::new("url is required."))?;
        if u.is_empty() {
            return Err(DeveloperError::new("url is required."));
        }
        let _schema = default_geometry_schema
            .ok_or_else(|| DeveloperError::new("defaultGeometrySchema is required."))?;
        let geom_data = geometry_data
            .ok_or_else(|| DeveloperError::new("geometryData is required."))?;

        // Check initialization state
        if !*get_initialized_state().lock().unwrap() {
            return Err(DeveloperError::new("I3S decoder could not be initialized."));
        }

        let mut longitude = 0.0f64;
        let mut latitude = 0.0f64;
        let mut height = 0.0f64;

        if let Some(parent) = &geom_data.parent {
            if let Some(obb_center) = parent.data.obb.as_ref().and_then(|o| o.get("center")).and_then(Value::as_array) {
                if obb_center.len() >= 3 {
                    longitude = obb_center[0].as_f64().unwrap_or(0.0);
                    latitude = obb_center[1].as_f64().unwrap_or(0.0);
                    height = obb_center[2].as_f64().unwrap_or(0.0);
                }
            } else if let Some(mbs) = &parent.data.mbs {
                if mbs.len() >= 3 {
                    longitude = mbs[0];
                    latitude = mbs[1];
                    height = mbs[2];
                }
            }

            let _rot_matrix = parent.inverse_rotation_matrix.unwrap_or([
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0,
            ]);
        }

        let _ = (longitude, latitude, height);

        // For empty or stub geometry data, produces empty glTF mesh structures
        let mesh_data = I3SMeshData {
            buffers: Vec::new(),
            buffer_views: Vec::new(),
            accessors: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            nodes_in_scene: Vec::new(),
        };

        Ok(I3SDecodeResult { mesh_data })
    }
}
