//! Tileset heatmap colorizer matching CesiumJS `Scene/Cesium3DTilesetHeatmap.js`.
use std::collections::HashMap;

use cesium_math::{lerp, Color, JulianDate, EPSILON7};

/// Reference value accepted by [`Cesium3DTilesetHeatmap::set_reference_minimum_maximum`].
#[derive(Clone, Copy, Debug)]
pub enum HeatmapReferenceValue {
    Number(f64),
    Date(JulianDate),
}

impl From<f64> for HeatmapReferenceValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<JulianDate> for HeatmapReferenceValue {
    fn from(value: JulianDate) -> Self {
        Self::Date(value)
    }
}

/// Minimal tile state required by [`Cesium3DTilesetHeatmap::colorize`].
#[derive(Clone, Debug)]
pub struct HeatmapTile {
    pub content_available: bool,
    pub selected_frame: u64,
    pub debug_color: Color,
    pub center_z_depth: Option<f64>,
    pub load_timestamp: Option<JulianDate>,
    pub custom_properties: HashMap<String, f64>,
}

impl Default for HeatmapTile {
    fn default() -> Self {
        Self {
            content_available: false,
            selected_frame: 0,
            debug_color: Color::WHITE,
            center_z_depth: None,
            load_timestamp: None,
            custom_properties: HashMap::new(),
        }
    }
}

impl HeatmapTile {
    fn get_property_value(&self, property_name: &str) -> Option<f64> {
        match property_name {
            "_loadTimestamp" => self.load_timestamp.map(julian_date_to_ms),
            "_centerZDepth" => self.center_z_depth,
            other => self.custom_properties.get(other).copied(),
        }
    }
}

fn julian_date_to_ms(jd: JulianDate) -> f64 {
    (jd.day_number as f64 - 2_440_587.5) * 86_400_000.0 + jd.seconds_of_day * 1000.0
}

fn get_heatmap_value(value: HeatmapReferenceValue, tile_property_name: &str) -> f64 {
    match (value, tile_property_name) {
        (HeatmapReferenceValue::Date(jd), "_loadTimestamp") => julian_date_to_ms(jd),
        (HeatmapReferenceValue::Date(jd), _) => julian_date_to_ms(jd),
        (HeatmapReferenceValue::Number(n), _) => n,
    }
}

const HEATMAP_COLORS: [Color; 6] = [
    Color::new(0.1, 0.1, 0.1, 1.0),     // Dark Gray
    Color::new(0.153, 0.278, 0.878, 1.0), // Blue
    Color::new(0.827, 0.231, 0.49, 1.0),  // Pink
    Color::new(0.827, 0.188, 0.22, 1.0),  // Red
    Color::new(1.0, 0.592, 0.259, 1.0),   // Orange
    Color::new(1.0, 0.843, 0.0, 1.0),     // Yellow
];

/// Heatmap colorizer for 3D Tilesets, matching CesiumJS `Cesium3DTilesetHeatmap`.
#[derive(Clone, Debug)]
pub struct Cesium3DTilesetHeatmap {
    pub tile_property_name: Option<String>,
    pub minimum: f64,
    pub maximum: f64,
    pub previous_minimum: f64,
    pub previous_maximum: f64,
    pub reference_minimum: HashMap<String, f64>,
    pub reference_maximum: HashMap<String, f64>,
}

impl Cesium3DTilesetHeatmap {
    /// Creates a new heatmap colorizer tracking `tile_property_name`.
    pub fn new(tile_property_name: impl Into<String>) -> Self {
        Self {
            tile_property_name: Some(tile_property_name.into()),
            minimum: f64::MAX,
            maximum: -f64::MAX,
            previous_minimum: f64::MAX,
            previous_maximum: -f64::MAX,
            reference_minimum: HashMap::new(),
            reference_maximum: HashMap::new(),
        }
    }

    /// Sets reference minimum and maximum values for `tile_property_name`.
    pub fn set_reference_minimum_maximum(
        &mut self,
        minimum: impl Into<HeatmapReferenceValue>,
        maximum: impl Into<HeatmapReferenceValue>,
        tile_property_name: &str,
    ) {
        self.reference_minimum.insert(
            tile_property_name.to_string(),
            get_heatmap_value(minimum.into(), tile_property_name),
        );
        self.reference_maximum.insert(
            tile_property_name.to_string(),
            get_heatmap_value(maximum.into(), tile_property_name),
        );
    }

    /// Colorizes `tile` in heatmap style based on the previous frame's min/max window.
    pub fn colorize(&mut self, tile: &mut HeatmapTile, frame_number: u64) {
        let Some(ref prop_name) = self.tile_property_name.clone() else {
            return;
        };
        if !tile.content_available || tile.selected_frame != frame_number {
            return;
        }

        let Some(heatmap_value) = tile.get_property_value(prop_name) else {
            self.tile_property_name = None;
            return;
        };

        self.maximum = self.maximum.max(heatmap_value);
        self.minimum = self.minimum.min(heatmap_value);

        let minimum = self.previous_minimum;
        let maximum = self.previous_maximum;
        if minimum == f64::MAX || maximum == -f64::MAX {
            return;
        }

        let shifted_max = maximum - minimum + EPSILON7;
        let shifted_value = (heatmap_value - minimum).clamp(0.0, shifted_max);
        let zero_to_one = shifted_value / shifted_max;
        let last_index = (HEATMAP_COLORS.len() - 1) as f64;
        let color_position = zero_to_one * last_index;

        let floor_idx = (color_position.floor() as usize).min(HEATMAP_COLORS.len() - 1);
        let ceil_idx = (color_position.ceil() as usize).min(HEATMAP_COLORS.len() - 1);
        let t = color_position - (floor_idx as f64);

        let c0 = HEATMAP_COLORS[floor_idx];
        let c1 = HEATMAP_COLORS[ceil_idx];

        tile.debug_color = Color::new(
            lerp(c0.red as f64, c1.red as f64, t) as f32,
            lerp(c0.green as f64, c1.green as f64, t) as f32,
            lerp(c0.blue as f64, c1.blue as f64, t) as f32,
            1.0,
        );
    }

    /// Resets tracked min/max values before tileset traversal.
    pub fn reset_minimum_maximum(&mut self) {
        let Some(ref prop_name) = self.tile_property_name else {
            return;
        };
        let ref_min = self.reference_minimum.get(prop_name).copied();
        let ref_max = self.reference_maximum.get(prop_name).copied();
        let use_reference = ref_min.is_some() && ref_max.is_some();

        self.previous_minimum = if use_reference {
            ref_min.unwrap()
        } else {
            self.minimum
        };
        self.previous_maximum = if use_reference {
            ref_max.unwrap()
        } else {
            self.maximum
        };
        self.minimum = f64::MAX;
        self.maximum = -f64::MAX;
    }
}
