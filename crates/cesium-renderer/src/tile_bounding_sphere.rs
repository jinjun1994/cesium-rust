use std::collections::HashMap;

use cesium_math::{
    Cartesian3, Color, ComponentDatatype, DeveloperError, GeometryInstance,
    GeometryInstanceAttribute, Matrix4, Plane, PlaneIntersection, Sphere, SphereOutlineGeometry,
    EPSILON7,
};

use crate::scene_primitive::{Primitive, PrimitiveAppearance, PrimitiveOptions};

/// Camera state for `TileBoundingSphere::distance_to_camera`.
#[derive(Clone, Debug)]
pub struct TileBoundingSphereCamera {
    pub position_wc: Cartesian3,
}

impl TileBoundingSphereCamera {
    pub fn new(position_wc: Cartesian3) -> Self {
        Self { position_wc }
    }
}

/// Frame state passed to `TileBoundingSphere::distance_to_camera`.
pub struct TileBoundingSphereFrameState {
    pub camera: TileBoundingSphereCamera,
}

/// A tile bounding volume specified as a sphere.
#[derive(Clone, Debug, PartialEq)]
pub struct TileBoundingSphere {
    bounding_sphere: Sphere,
}

impl Default for TileBoundingSphere {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl TileBoundingSphere {
    pub fn new(center: Option<Cartesian3>, radius: Option<f64>) -> Self {
        let center_c = center.unwrap_or(Cartesian3::ZERO);
        let mut r = radius.unwrap_or(0.0);
        if r == 0.0 {
            r = EPSILON7;
        }
        Self {
            bounding_sphere: Sphere::new(center_c, r),
        }
    }

    pub fn center(&self) -> Cartesian3 {
        self.bounding_sphere.center.into()
    }

    pub fn radius(&self) -> f64 {
        self.bounding_sphere.radius
    }

    pub fn bounding_volume(&self) -> &Sphere {
        &self.bounding_sphere
    }

    pub fn bounding_sphere(&self) -> &Sphere {
        &self.bounding_sphere
    }

    pub fn distance_to_camera(&self, frame_state: &TileBoundingSphereFrameState) -> f64 {
        0.0_f64.max(self.center().distance(&frame_state.camera.position_wc) - self.radius())
    }

    pub fn intersect_plane(&self, plane: Plane) -> PlaneIntersection {
        self.bounding_sphere
            .intersect_plane(plane.normal, plane.distance)
    }

    pub fn update(&mut self, center: Cartesian3, radius: f64) {
        self.bounding_sphere.center = glam::DVec3::new(center.x, center.y, center.z);
        self.bounding_sphere.radius = radius;
    }

    pub fn create_debug_volume(&self, color: Color) -> Result<Primitive, DeveloperError> {
        let outline = SphereOutlineGeometry::new(Some(self.radius()), None, None, None);
        let geometry = outline
            .create_geometry()
            .ok_or_else(|| DeveloperError::new("Failed to create sphere outline geometry"))?;

        let model_matrix = Matrix4::from_translation(self.center());

        let mut attributes = HashMap::new();
        attributes.insert(
            "color".to_string(),
            GeometryInstanceAttribute {
                component_datatype: ComponentDatatype::Float,
                components_per_attribute: 4,
                normalize: false,
                value: vec![
                    color.red as f64,
                    color.green as f64,
                    color.blue as f64,
                    color.alpha as f64,
                ],
            },
        );

        let instance = GeometryInstance::new(
            geometry,
            Some(model_matrix),
            Some("outline".to_string()),
            Some(attributes),
        );

        Ok(Primitive::new(PrimitiveOptions {
            geometry_instances: Some(vec![instance]),
            appearance: Some(PrimitiveAppearance {
                closed: false,
                translucent: false,
            }),
            asynchronous: Some(false),
            ..Default::default()
        }))
    }
}
