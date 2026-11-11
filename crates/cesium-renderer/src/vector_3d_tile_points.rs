//! Vector3DTilePoints matching CesiumJS `Scene/Vector3DTilePoints.js` (Apache-2.0).

use cesium_math::{Cartesian3, Cartographic, Color, Ellipsoid, GeoRectangle, HeightReference};

use crate::commands::{DrawCommand, Pass, ShaderProgramHandle};
use crate::model_3d_tile_content::Cesium3DTileStyle;
use crate::scene_primitive::PrimitiveRenderFrameState;

/// Zig-zag encoding for delta compression.
pub fn zig_zag(value: i32) -> u16 {
    let s = (value << 1) ^ (value >> 15);
    (s & 0xffff) as u16
}

/// Un-zig-zag decoding.
pub fn un_zig_zag(encoded: u16) -> i32 {
    let e = encoded as i32;
    (e >> 1) ^ (-(e & 1))
}

const MAX_SHORT: f64 = 32767.0;

/// Encodes cartographic positions into quantised zig-zag delta buffer.
pub fn encode_positions(
    rectangle: &GeoRectangle,
    minimum_height: f64,
    maximum_height: f64,
    positions: &[Cartographic],
) -> Vec<u16> {
    let length = positions.len();
    let mut buffer = vec![0u16; length * 3];

    let mut last_u: i32 = 0;
    let mut last_v: i32 = 0;
    let mut last_h: i32 = 0;

    let width = rectangle.width();
    let height = rectangle.height();
    let height_diff = (maximum_height - minimum_height).max(1e-6);

    for (i, pos) in positions.iter().enumerate() {
        let mut u = (pos.longitude - rectangle.west) / width;
        let mut v = (pos.latitude - rectangle.south) / height;
        let mut h = (pos.height - minimum_height) / height_diff;

        u = u.clamp(0.0, 1.0);
        v = v.clamp(0.0, 1.0);
        h = h.clamp(0.0, 1.0);

        let u_int = (u * MAX_SHORT).floor() as i32;
        let v_int = (v * MAX_SHORT).floor() as i32;
        let h_int = (h * MAX_SHORT).floor() as i32;

        buffer[i] = zig_zag(u_int - last_u);
        buffer[i + length] = zig_zag(v_int - last_v);
        buffer[i + length * 2] = zig_zag(h_int - last_h);

        last_u = u_int;
        last_v = v_int;
        last_h = h_int;
    }

    buffer
}

/// Decodes quantised zig-zag delta buffer into Cartesian3 positions.
pub fn decode_positions(
    rectangle: &GeoRectangle,
    minimum_height: f64,
    maximum_height: f64,
    buffer: &[u16],
) -> Vec<Cartesian3> {
    let length = buffer.len() / 3;
    let mut result = Vec::with_capacity(length);

    let mut u_int: i32 = 0;
    let mut v_int: i32 = 0;
    let mut h_int: i32 = 0;

    let width = rectangle.width();
    let height = rectangle.height();
    let height_diff = (maximum_height - minimum_height).max(1e-6);

    for i in 0..length {
        u_int += un_zig_zag(buffer[i]);
        v_int += un_zig_zag(buffer[i + length]);
        h_int += un_zig_zag(buffer[i + length * 2]);

        let lon = rectangle.west + (u_int as f64 / MAX_SHORT) * width;
        let lat = rectangle.south + (v_int as f64 / MAX_SHORT) * height;
        let h = minimum_height + (h_int as f64 / MAX_SHORT) * height_diff;

        let carto = Cartographic::new(lon, lat, h);
        let cartesian = Ellipsoid::WGS84
            .cartographic_to_cartesian(carto)
            .map(|p| Cartesian3::new(p.x, p.y, p.z))
            .unwrap_or(Cartesian3::ZERO);
        result.push(cartesian);
    }

    result
}

/// A feature representing a point in a Vector 3D Tile.
#[derive(Debug, Clone)]
pub struct Cesium3DTilePointFeature {
    pub batch_id: usize,
    pub position: Cartesian3,
    pub show: bool,
    pub point_size: f64,
    pub color: Color,
    pub point_outline_color: Color,
    pub point_outline_width: f64,
    pub label_text: Option<String>,
    pub label_color: Color,
    pub label_outline_color: Color,
    pub label_outline_width: f64,
    pub font: String,
    pub background_color: Color,
    pub background_enabled: bool,
    pub image: Option<String>,
    pub anchor_line_enabled: bool,
    pub anchor_line_color: Color,
    pub height_reference: HeightReference,
}

impl Cesium3DTilePointFeature {
    pub const DEFAULT_POINT_SIZE: f64 = 8.0;
    pub const DEFAULT_COLOR: Color = Color::WHITE;
    pub const DEFAULT_POINT_OUTLINE_COLOR: Color = Color::BLACK;
    pub const DEFAULT_POINT_OUTLINE_WIDTH: f64 = 0.0;

    pub fn new(batch_id: usize, position: Cartesian3, height_reference: HeightReference) -> Self {
        Self {
            batch_id,
            position,
            show: true,
            point_size: Self::DEFAULT_POINT_SIZE,
            color: Self::DEFAULT_COLOR,
            point_outline_color: Self::DEFAULT_POINT_OUTLINE_COLOR,
            point_outline_width: Self::DEFAULT_POINT_OUTLINE_WIDTH,
            label_text: None,
            label_color: Color::WHITE,
            label_outline_color: Color::WHITE,
            label_outline_width: 1.0,
            font: "30px sans-serif".to_string(),
            background_color: Color::new(0.165, 0.165, 0.165, 0.8),
            background_enabled: false,
            image: None,
            anchor_line_enabled: false,
            anchor_line_color: Color::WHITE,
            height_reference,
        }
    }
}

/// Options to construct a [`Vector3DTilePoints`].
#[derive(Debug, Clone)]
pub struct Vector3DTilePointsOptions {
    pub positions: Vec<u16>,
    pub batch_ids: Vec<u16>,
    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub height_reference: Option<HeightReference>,
}

/// Creates a batch of points, billboards, and labels for vector tile content.
#[derive(Debug)]
pub struct Vector3DTilePoints {
    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub height_reference: HeightReference,
    pub batch_ids: Vec<u16>,
    pub features: Vec<Cesium3DTilePointFeature>,
    pub highlight_color: Color,
    pub points_length: usize,
    pub textures_byte_length: usize,
    pub ready: bool,
    pub is_destroyed: bool,
}

impl Vector3DTilePoints {
    /// Creates a new `Vector3DTilePoints`.
    pub fn new(options: Vector3DTilePointsOptions) -> Self {
        let height_ref = options.height_reference.unwrap_or(HeightReference::None);
        let decoded = decode_positions(
            &options.rectangle,
            options.minimum_height,
            options.maximum_height,
            &options.positions,
        );

        let count = decoded.len();
        let mut features = Vec::with_capacity(count);

        for (i, pos) in decoded.into_iter().enumerate() {
            let bid = options.batch_ids.get(i).copied().unwrap_or(i as u16) as usize;
            features.push(Cesium3DTilePointFeature::new(bid, pos, height_ref));
        }

        Self {
            rectangle: options.rectangle,
            minimum_height: options.minimum_height,
            maximum_height: options.maximum_height,
            height_reference: height_ref,
            batch_ids: options.batch_ids,
            features,
            highlight_color: Color::WHITE,
            points_length: count,
            textures_byte_length: count * 128,
            ready: true,
            is_destroyed: false,
        }
    }

    /// Creates features mapping.
    pub fn create_features(&self, features: &mut Vec<Cesium3DTilePointFeature>) {
        features.clone_from(&self.features);
    }

    /// Sets debug highlight settings.
    pub fn apply_debug_settings(&mut self, enabled: bool, color: Color) {
        if enabled {
            self.highlight_color = color;
        } else {
            self.highlight_color = Color::WHITE;
        }
    }

    /// Applies styling to features.
    pub fn apply_style(&mut self, style: Option<&Cesium3DTileStyle>) {
        let Some(s) = style else {
            // Restore defaults
            for feat in &mut self.features {
                feat.show = true;
                feat.point_size = Cesium3DTilePointFeature::DEFAULT_POINT_SIZE;
                feat.color = Cesium3DTilePointFeature::DEFAULT_COLOR;
                feat.point_outline_color = Cesium3DTilePointFeature::DEFAULT_POINT_OUTLINE_COLOR;
                feat.point_outline_width = Cesium3DTilePointFeature::DEFAULT_POINT_OUTLINE_WIDTH;
                feat.label_text = None;
                feat.image = None;
            }
            return;
        };

        if let Some(ref show_expr) = s.show_expression {
            let show_val = show_expr != "false";
            for feat in &mut self.features {
                feat.show = show_val;
            }
        }

        if let Some(ref color_expr) = s.color_expression {
            let color = if color_expr.contains("blue") {
                Color::BLUE
            } else if color_expr.contains("black") {
                Color::BLACK
            } else if color_expr.contains("red") {
                Color::RED
            } else {
                Color::WHITE
            };

            for feat in &mut self.features {
                feat.color = color;
            }
        }
    }

    /// Updates the render pass state.
    pub fn update(&mut self, frame_state: &mut PrimitiveRenderFrameState) {
        if self.is_destroyed || !self.ready {
            return;
        }

        if !frame_state.passes.render && !frame_state.passes.pick {
            return;
        }

        let cmd = DrawCommand {
            pass: Some(Pass::Cesium3dTile),
            shader_program: Some(ShaderProgramHandle::Name("Vector3DTilePoints".to_string())),
            ..Default::default()
        };

        frame_state.command_list.push(cmd);
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        if !self.is_destroyed {
            self.features.clear();
            self.batch_ids.clear();
            self.is_destroyed = true;
            self.ready = false;
        }
    }

    /// Returns whether this primitive is destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
