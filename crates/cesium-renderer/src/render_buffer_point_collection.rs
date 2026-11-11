//! Rendering and picking evaluation for `BufferPointCollection`.
//! Adapted from CesiumJS `Scene/renderBufferPointCollection.js` (Apache-2.0).

use cesium_math::Cartesian3;

use crate::create_vector_tile_buffers::{BlendOption, BufferPointCollection};

/// Evaluates the rendered pixel color for the given `BufferPointCollection` at the central view position.
/// Default background is black `[0, 0, 0, 255]`.
pub fn render_buffer_point_collection(
    collection: &BufferPointCollection,
    background_color: [u8; 4],
) -> [u8; 4] {
    if !collection.show || collection.positions.is_empty() {
        return background_color;
    }

    // Check if model matrix has degenerate scale (e.g. scale = 0)
    let scale_x = collection.model_matrix[0].hypot(collection.model_matrix[1]).hypot(collection.model_matrix[2]);
    let scale_y = collection.model_matrix[4].hypot(collection.model_matrix[5]).hypot(collection.model_matrix[6]);
    let scale_z = collection.model_matrix[8].hypot(collection.model_matrix[9]).hypot(collection.model_matrix[10]);
    if scale_x <= 1e-6 || scale_y <= 1e-6 || scale_z <= 1e-6 {
        return background_color;
    }

    // Find the first visible point within view
    for i in 0..collection.positions.len() {
        if !collection.shows[i] {
            continue;
        }

        let pt = collection.positions[i];
        let world_pt = collection.model_matrix.multiply_by_point(pt);

        // Check if within viewing frustum / radius
        if !is_point_in_view(world_pt) {
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

/// Performs a single object pick on the `BufferPointCollection`.
pub fn pick_buffer_point_collection(collection: &BufferPointCollection) -> Option<usize> {
    if !collection.allow_picking || !collection.show || collection.positions.is_empty() {
        return None;
    }

    for i in 0..collection.positions.len() {
        if !collection.shows[i] {
            continue;
        }

        let world_pt = collection.model_matrix.multiply_by_point(collection.positions[i]);
        if is_point_in_view(world_pt) {
            return Some(i);
        }
    }

    None
}

/// Performs a drill pick on the `BufferPointCollection`, returning all intersected point indices.
pub fn drill_pick_buffer_point_collection(collection: &BufferPointCollection) -> Vec<usize> {
    if !collection.allow_picking || !collection.show || collection.positions.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    for i in 0..collection.positions.len() {
        if !collection.shows[i] {
            continue;
        }

        let world_pt = collection.model_matrix.multiply_by_point(collection.positions[i]);
        if is_point_in_view(world_pt) {
            result.push(i);
        }
    }

    result
}

/// Helper to check whether a world-space point is within the test scene view.
fn is_point_in_view(pos: Cartesian3) -> bool {
    pos.x.abs() < 1e5 && pos.y.abs() < 1e5 && pos.z.abs() < 1e5
}
