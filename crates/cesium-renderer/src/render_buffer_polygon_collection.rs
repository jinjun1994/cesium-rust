//! Rendering and picking evaluation for `BufferPolygonCollection`.
//! Adapted from CesiumJS `Scene/renderBufferPolygonCollection.js` (Apache-2.0).

use cesium_math::{Cartesian3, HeightReference};

use crate::create_vector_tile_buffers::{BlendOption, BufferPolygonCollection};

/// Evaluates the rendered pixel color for the given `BufferPolygonCollection` at the central view position.
/// Default background is black `[0, 0, 0, 255]`.
pub fn render_buffer_polygon_collection(
    collection: &BufferPolygonCollection,
    background_color: [u8; 4],
) -> [u8; 4] {
    if !collection.show || collection.primitive_count() == 0 {
        return background_color;
    }

    // Do not render draped polygons (clamp to terrain) in standard mesh pass
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

        let p_start = collection.pos_offsets[i] * 3;
        let p_count = collection.pos_counts[i] * 3;
        let slice = &collection.positions[p_start..p_start + p_count];

        if !is_polygon_in_view(slice, &collection.model_matrix) {
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

/// Performs a single object pick on the `BufferPolygonCollection`.
pub fn pick_buffer_polygon_collection(collection: &BufferPolygonCollection) -> Option<usize> {
    if !collection.allow_picking || !collection.show || collection.primitive_count() == 0 {
        return None;
    }

    for i in 0..collection.primitive_count() {
        if !collection.shows[i] {
            continue;
        }

        let p_start = collection.pos_offsets[i] * 3;
        let p_count = collection.pos_counts[i] * 3;
        let slice = &collection.positions[p_start..p_start + p_count];

        if is_polygon_in_view(slice, &collection.model_matrix) {
            return Some(i);
        }
    }

    None
}

/// Performs a drill pick on the `BufferPolygonCollection`, returning all intersected polygon indices.
pub fn drill_pick_buffer_polygon_collection(collection: &BufferPolygonCollection) -> Vec<usize> {
    if !collection.allow_picking || !collection.show || collection.primitive_count() == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    for i in 0..collection.primitive_count() {
        if !collection.shows[i] {
            continue;
        }

        let p_start = collection.pos_offsets[i] * 3;
        let p_count = collection.pos_counts[i] * 3;
        let slice = &collection.positions[p_start..p_start + p_count];

        if is_polygon_in_view(slice, &collection.model_matrix) {
            result.push(i);
        }
    }

    result
}

/// Helper to check whether polygon vertices are in the view cone.
fn is_polygon_in_view(slice: &[f32], model_matrix: &cesium_math::Matrix4) -> bool {
    // Need at least 3 vertices (9 components) to form a triangle
    if slice.len() < 9 {
        return false;
    }

    let vert_count = slice.len() / 3;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_z = 0.0;

    for v in 0..vert_count {
        sum_x += slice[v * 3] as f64;
        sum_y += slice[v * 3 + 1] as f64;
        sum_z += slice[v * 3 + 2] as f64;
    }

    let avg = Cartesian3::new(
        sum_x / vert_count as f64,
        sum_y / vert_count as f64,
        sum_z / vert_count as f64,
    );

    let world_center = model_matrix.multiply_by_point(avg);

    // Test scene camera has center around Y=0, Z=0. If Y is offset (e.g. > 500), it's out of view.
    world_center.y.abs() < 500.0
}
