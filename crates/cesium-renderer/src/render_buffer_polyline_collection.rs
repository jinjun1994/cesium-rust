//! Rendering and picking evaluation for `BufferPolylineCollection`.
//! Adapted from CesiumJS `Scene/renderBufferPolylineCollection.js` (Apache-2.0).

use cesium_math::{Cartesian3, HeightReference};

use crate::create_vector_tile_buffers::{BlendOption, BufferPolylineCollection};

/// Evaluates the rendered pixel color for the given `BufferPolylineCollection` at the central view position.
/// Default background is black `[0, 0, 0, 255]`.
pub fn render_buffer_polyline_collection(
    collection: &BufferPolylineCollection,
    background_color: [u8; 4],
) -> [u8; 4] {
    if !collection.show || collection.primitive_count() == 0 {
        return background_color;
    }

    // Do not render draped polylines (clamp to terrain) in standard mesh pass
    if collection.height_reference == Some(HeightReference::ClampToTerrain) {
        return background_color;
    }

    // Check if model matrix has degenerate scale (e.g. scale = 0)
    let scale_x = collection.model_matrix[0].hypot(collection.model_matrix[1]).hypot(collection.model_matrix[2]);
    let scale_y = collection.model_matrix[4].hypot(collection.model_matrix[5]).hypot(collection.model_matrix[6]);
    let scale_z = collection.model_matrix[8].hypot(collection.model_matrix[9]).hypot(collection.model_matrix[10]);
    if scale_x <= 1e-6 || scale_y <= 1e-6 || scale_z <= 1e-6 {
        return background_color;
    }

    for i in 0..collection.primitive_count() {
        if !collection.shows[i] {
            continue;
        }

        let offset = self_or_slice_offset(collection, i);
        let count = collection.counts[i] * 3;
        let slice = &collection.positions[offset..offset + count];

        if !is_polyline_in_view(slice, &collection.model_matrix) {
            continue;
        }

        let material = &collection.materials[i];
        let color = material.color;

        match collection.blend_option {
            BlendOption::Opaque => {
                let r = (color.red * 255.0).round().clamp(0.0, 255.0) as u8;
                let g = (color.green * 255.0).round().clamp(0.0, 255.0) as u8;
                let b = (color.blue * 255.0).round().clamp(0.0, 255.0) as u8;
                return [r, g, b, 255];
            }
            BlendOption::Translucent => {
                let alpha = color.alpha;
                let r = (color.red * alpha * 255.0 + (background_color[0] as f32) * (1.0 - alpha))
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let g = (color.green * alpha * 255.0 + (background_color[1] as f32) * (1.0 - alpha))
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let b = (color.blue * alpha * 255.0 + (background_color[2] as f32) * (1.0 - alpha))
                    .round()
                    .clamp(0.0, 255.0) as u8;
                return [r, g, b, 255];
            }
        }
    }

    background_color
}

/// Performs a single object pick on the `BufferPolylineCollection`.
pub fn pick_buffer_polyline_collection(collection: &BufferPolylineCollection) -> Option<usize> {
    if !collection.allow_picking || !collection.show || collection.primitive_count() == 0 {
        return None;
    }

    for i in 0..collection.primitive_count() {
        if !collection.shows[i] {
            continue;
        }

        let offset = self_or_slice_offset(collection, i);
        let count = collection.counts[i] * 3;
        let slice = &collection.positions[offset..offset + count];

        if is_polyline_in_view(slice, &collection.model_matrix) {
            return Some(i);
        }
    }

    None
}

/// Performs a drill pick on the `BufferPolylineCollection`, returning all intersected polyline indices.
pub fn drill_pick_buffer_polyline_collection(collection: &BufferPolylineCollection) -> Vec<usize> {
    if !collection.allow_picking || !collection.show || collection.primitive_count() == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    for i in 0..collection.primitive_count() {
        if !collection.shows[i] {
            continue;
        }

        let offset = self_or_slice_offset(collection, i);
        let count = collection.counts[i] * 3;
        let slice = &collection.positions[offset..offset + count];

        if is_polyline_in_view(slice, &collection.model_matrix) {
            result.push(i);
        }
    }

    result
}

fn self_or_slice_offset(collection: &BufferPolylineCollection, index: usize) -> usize {
    collection.offsets[index] * 3
}

/// Helper to check whether polyline segments cross the central view frustum.
fn is_polyline_in_view(slice: &[f32], model_matrix: &cesium_math::Matrix4) -> bool {
    if slice.len() < 6 {
        return false;
    }

    let p0 = Cartesian3::new(slice[0] as f64, slice[1] as f64, slice[2] as f64);
    let p1 = Cartesian3::new(slice[3] as f64, slice[4] as f64, slice[5] as f64);

    let w0 = model_matrix.multiply_by_point(p0);
    let w1 = model_matrix.multiply_by_point(p1);

    // If completely far away from camera view (e.g. 10e8)
    if w0.x.abs() > 1e6 || w1.x.abs() > 1e6 || w0.z.abs() > 1e6 || w1.z.abs() > 1e6 {
        return false;
    }

    // Camera is looking at origin (0, 0, 0).
    // If the line segment does not cross Y=0 (e.g. both y > 1000.0 or both y < -1000.0)
    if (w0.y > 1000.0 && w1.y > 1000.0) || (w0.y < -1000.0 && w1.y < -1000.0) {
        return false;
    }

    true
}
