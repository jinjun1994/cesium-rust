//! SceneTransforms matching CesiumJS `Scene/SceneTransforms.js` (Apache-2.0).

use cesium_math::{
    BoundingRectangle, Cartographic, DeveloperError, MapProjection, SceneMode,
};
use glam::{DMat4, DVec2, DVec3, DVec4};

/// Options for transforming world coordinates to window coordinates.
#[derive(Debug)]
pub struct WorldToWindowCoordinatesOptions<'a, P: MapProjection + ?Sized> {
    pub position: DVec3,
    pub eye_offset: DVec3,
    pub mode: SceneMode,
    pub morph_time: f64,
    pub map_projection: &'a P,
    pub view_matrix: DMat4,
    pub projection_matrix: DMat4,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub is_orthographic: bool,
}

impl<'a, P: MapProjection + ?Sized> Clone for WorldToWindowCoordinatesOptions<'a, P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, P: MapProjection + ?Sized> Copy for WorldToWindowCoordinatesOptions<'a, P> {}

/// Functions that perform scene-dependent transforms between rendering-related coordinate systems.
pub struct SceneTransforms;

impl SceneTransforms {
    /// Transforms a position in world coordinates to the coordinate frame used to render the current scene mode.
    pub fn compute_actual_ellipsoid_position<P: MapProjection + ?Sized>(
        mode: SceneMode,
        morph_time: f64,
        projection: &P,
        position: DVec3,
    ) -> Option<DVec3> {
        if mode == SceneMode::Scene3D {
            return Some(position);
        }

        let cartographic = projection.ellipsoid().cartesian_to_cartographic(position)?;
        let projected = projection.project(cartographic)?;

        match mode {
            SceneMode::ColumbusView => Some(DVec3::new(projected.z, projected.x, projected.y)),
            SceneMode::Scene2D => Some(DVec3::new(0.0, projected.x, projected.y)),
            SceneMode::Morphing => {
                let t = morph_time.clamp(0.0, 1.0);
                Some(DVec3::new(
                    projected.z + (position.x - projected.z) * t,
                    projected.x + (position.y - projected.x) * t,
                    projected.y + (position.z - projected.y) * t,
                ))
            }
            _ => Some(position),
        }
    }

    /// Converts a position sampled from the actual ellipsoid/terrain surface in the current scene mode's
    /// coordinate frame into a mode-independent `Cartographic`.
    pub fn actual_ellipsoid_position_to_cartographic<P: MapProjection + ?Sized>(
        mode: SceneMode,
        projection: &P,
        position: DVec3,
    ) -> Result<Cartographic, DeveloperError> {
        if mode == SceneMode::Scene3D {
            projection
                .ellipsoid()
                .cartesian_to_cartographic(position)
                .ok_or_else(|| DeveloperError::new("Failed to convert Cartesian3 to Cartographic"))
        } else {
            // Un-swizzle back from (height, easting, northing) to (easting, northing, height)
            let projected = DVec3::new(position.y, position.z, position.x);
            projection
                .unproject(projected)
                .ok_or_else(|| DeveloperError::new("Failed to unproject map position"))
        }
    }

    /// Transforms position in world coordinates to clip coordinates.
    pub fn world_to_clip(
        position: DVec3,
        eye_offset: DVec3,
        view_matrix: &DMat4,
        projection_matrix: &DMat4,
    ) -> DVec4 {
        let mut position_ec = *view_matrix * DVec4::new(position.x, position.y, position.z, 1.0);

        let norm_ec = DVec3::new(position_ec.x, position_ec.y, position_ec.z).normalize_or_zero();
        let z_eye_offset = eye_offset * norm_ec;
        position_ec.x += eye_offset.x + z_eye_offset.x;
        position_ec.y += eye_offset.y + z_eye_offset.y;
        position_ec.z += z_eye_offset.z;

        *projection_matrix * position_ec
    }

    /// Transforms clip coordinates to GL window coordinates.
    pub fn clip_to_gl_window_coordinates(viewport: BoundingRectangle, position_cc: DVec4) -> DVec2 {
        let w = if position_cc.w != 0.0 {
            position_cc.w
        } else {
            1.0
        };
        let ndc_x = position_cc.x / w;
        let ndc_y = position_cc.y / w;

        let x = (ndc_x + 1.0) * 0.5 * viewport.width + viewport.x;
        let y = (ndc_y + 1.0) * 0.5 * viewport.height + viewport.y;
        DVec2::new(x, y)
    }

    /// Transforms a position in world coordinates to window coordinates.
    pub fn world_to_window_coordinates<P: MapProjection + ?Sized>(
        options: WorldToWindowCoordinatesOptions<'_, P>,
    ) -> Result<Option<DVec2>, DeveloperError> {
        if options.canvas_width <= 0.0 || options.canvas_height <= 0.0 {
            return Err(DeveloperError::new(
                "canvas_width and canvas_height must be positive.",
            ));
        }

        let actual_position = match Self::compute_actual_ellipsoid_position(
            options.mode,
            options.morph_time,
            options.map_projection,
            options.position,
        ) {
            Some(pos) => pos,
            None => return Ok(None),
        };

        let viewport = BoundingRectangle::new(0.0, 0.0, options.canvas_width, options.canvas_height);

        let position_cc = Self::world_to_clip(
            actual_position,
            options.eye_offset,
            &options.view_matrix,
            &options.projection_matrix,
        );

        // Clip-space point behind near plane or camera direction check
        if position_cc.z < 0.0 && !options.is_orthographic {
            return Ok(None);
        }

        // Additional check for points behind the camera in eye space:
        // In right-handed eye space (Cesium/OpenGL/Vulkan standard), camera looks toward -Z.
        // Therefore, vertices in front of the camera have eye_z < 0.
        // If eye_z > 0, the point is strictly behind the camera.
        let position_ec = options.view_matrix * DVec4::new(actual_position.x, actual_position.y, actual_position.z, 1.0);
        if position_ec.z > 0.0 && !options.is_orthographic {
            return Ok(None);
        }

        let mut gl_coords = Self::clip_to_gl_window_coordinates(viewport, position_cc);
        // Flip Y for standard window coordinates (top-left origin)
        gl_coords.y = options.canvas_height - gl_coords.y;

        Ok(Some(gl_coords))
    }

    /// Transforms a position in world coordinates to drawing buffer coordinates.
    pub fn world_to_drawing_buffer_coordinates<P: MapProjection + ?Sized>(
        options: WorldToWindowCoordinatesOptions<'_, P>,
        drawing_buffer_width: f64,
        drawing_buffer_height: f64,
    ) -> Result<Option<DVec2>, DeveloperError> {
        let window_coords = Self::world_to_window_coordinates(options)?;
        let Some(coords) = window_coords else {
            return Ok(None);
        };

        let x_scale = drawing_buffer_width / options.canvas_width;
        let y_scale = drawing_buffer_height / options.canvas_height;

        Ok(Some(DVec2::new(coords.x * x_scale, coords.y * y_scale)))
    }

    /// Transforms window coordinates to drawing buffer coordinates.
    pub fn transform_window_to_drawing_buffer(
        window_position: DVec2,
        canvas_width: f64,
        canvas_height: f64,
        drawing_buffer_width: f64,
        drawing_buffer_height: f64,
    ) -> DVec2 {
        let x_scale = drawing_buffer_width / canvas_width;
        let y_scale = drawing_buffer_height / canvas_height;
        DVec2::new(window_position.x * x_scale, window_position.y * y_scale)
    }

    /// Transforms drawing buffer coordinates and depth to world coordinates.
    pub fn drawing_buffer_to_world_coordinates(
        drawing_buffer_pos: DVec2,
        depth: f64,
        viewport: BoundingRectangle,
        inverse_view_projection: &DMat4,
    ) -> DVec3 {
        let ndc_x = ((drawing_buffer_pos.x - viewport.x) / viewport.width) * 2.0 - 1.0;
        let ndc_y = ((drawing_buffer_pos.y - viewport.y) / viewport.height) * 2.0 - 1.0;
        let ndc_z = depth * 2.0 - 1.0;

        let clip = DVec4::new(ndc_x, ndc_y, ndc_z, 1.0);
        let world = *inverse_view_projection * clip;
        let w = if world.w != 0.0 { world.w } else { 1.0 };
        DVec3::new(world.x / w, world.y / w, world.z / w)
    }
}
