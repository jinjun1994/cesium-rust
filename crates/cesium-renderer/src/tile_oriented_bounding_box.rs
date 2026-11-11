//! TileOrientedBoundingBox matching CesiumJS `Scene/TileOrientedBoundingBox.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{
    BoxOutlineGeometry, Cartesian3, Color, ComponentDatatype, DeveloperError, GeometryInstance,
    GeometryInstanceAttribute, Matrix3, Matrix4, OrientedBoundingBox, Plane, PlaneIntersection,
    Sphere, EPSILON6, EPSILON7,
};
use glam::{DMat3, DVec3};

use crate::scene_primitive::{Primitive, PrimitiveAppearance, PrimitiveOptions};

fn compute_missing_vector(a: DVec3, b: DVec3) -> DVec3 {
    let result = a.cross(b);
    let magnitude = result.length();
    if magnitude > 0.0 {
        result * (EPSILON7 / magnitude)
    } else {
        DVec3::new(EPSILON7, 0.0, 0.0)
    }
}

fn find_orthogonal_vector(a: DVec3) -> DVec3 {
    let temp = a.normalize_or_zero();
    let b = if (temp - DVec3::X).abs().max_element() < EPSILON6 {
        DVec3::Y
    } else {
        DVec3::X
    };
    compute_missing_vector(a, b)
}

/// Checks half axes and replaces zero or missing vectors with orthogonal ones.
pub fn check_half_axes(half_axes: DMat3) -> DMat3 {
    let mut u = half_axes.x_axis;
    let mut v = half_axes.y_axis;
    let mut w = half_axes.z_axis;

    let u_zero = u == DVec3::ZERO;
    let v_zero = v == DVec3::ZERO;
    let w_zero = w == DVec3::ZERO;

    if !u_zero && !v_zero && !w_zero {
        return half_axes;
    }
    if u_zero && v_zero && w_zero {
        return DMat3::from_diagonal(DVec3::splat(EPSILON7));
    }
    if u_zero && !v_zero && !w_zero {
        u = compute_missing_vector(v, w);
    } else if !u_zero && v_zero && !w_zero {
        v = compute_missing_vector(u, w);
    } else if !u_zero && !v_zero && w_zero {
        w = compute_missing_vector(v, u);
    } else if !u_zero {
        v = find_orthogonal_vector(u);
        w = compute_missing_vector(v, u);
    } else if !v_zero {
        u = find_orthogonal_vector(v);
        w = compute_missing_vector(v, u);
    } else if !w_zero {
        u = find_orthogonal_vector(w);
        v = compute_missing_vector(w, u);
    }

    DMat3::from_cols(u, v, w)
}

/// Camera state for `TileOrientedBoundingBox::distance_to_camera`.
#[derive(Clone, Debug, Default)]
pub struct TileOrientedBoundingBoxCamera {
    pub position_wc: Cartesian3,
}

impl TileOrientedBoundingBoxCamera {
    pub fn new(position_wc: Cartesian3) -> Self {
        Self { position_wc }
    }

    pub fn position(&self) -> Cartesian3 {
        self.position_wc
    }

    pub fn set_position(&mut self, pos: Cartesian3) {
        self.position_wc = pos;
    }
}

/// Frame state passed to `TileOrientedBoundingBox::distance_to_camera`.
#[derive(Clone, Debug, Default)]
pub struct TileOrientedBoundingBoxFrameState {
    pub camera: TileOrientedBoundingBoxCamera,
}

impl TileOrientedBoundingBoxFrameState {
    pub fn new(camera: TileOrientedBoundingBoxCamera) -> Self {
        Self { camera }
    }
}

/// A tile bounding volume specified as an oriented bounding box.
#[derive(Clone, Debug, PartialEq)]
pub struct TileOrientedBoundingBox {
    oriented_bounding_box: OrientedBoundingBox,
    bounding_sphere: Sphere,
}

impl Default for TileOrientedBoundingBox {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl TileOrientedBoundingBox {
    pub fn new(center: Option<Cartesian3>, half_axes: Option<Matrix3>) -> Self {
        let center_dvec: DVec3 = center.unwrap_or(Cartesian3::ZERO).into();
        let half_axes_dmat: DMat3 = half_axes.unwrap_or(Matrix3::ZERO).into();
        let validated_axes = check_half_axes(half_axes_dmat);

        let obb = OrientedBoundingBox::new(center_dvec, validated_axes);
        let bs = Sphere::from_oriented_bounding_box(&obb)
            .unwrap_or_else(|_| Sphere::new(center_dvec, EPSILON7));

        Self {
            oriented_bounding_box: obb,
            bounding_sphere: bs,
        }
    }

    pub fn from_center_half_axes(center: Cartesian3, half_axes: Matrix3) -> Self {
        Self::new(Some(center), Some(half_axes))
    }

    pub fn bounding_volume(&self) -> &OrientedBoundingBox {
        &self.oriented_bounding_box
    }

    pub fn bounding_sphere(&self) -> &Sphere {
        &self.bounding_sphere
    }

    pub fn center(&self) -> Cartesian3 {
        self.oriented_bounding_box.center.into()
    }

    pub fn half_axes(&self) -> Matrix3 {
        self.oriented_bounding_box.half_axes.into()
    }

    pub fn distance_to_camera(
        &self,
        frame_state: Option<&TileOrientedBoundingBoxFrameState>,
    ) -> Result<f64, DeveloperError> {
        let frame_state = frame_state.ok_or_else(|| DeveloperError::new("frameState is required."))?;
        let dist_sq = self
            .oriented_bounding_box
            .distance_squared_to(frame_state.camera.position_wc.into())
            .unwrap_or(0.0);
        Ok(dist_sq.sqrt())
    }

    pub fn intersect_plane(&self, plane: Option<Plane>) -> Result<PlaneIntersection, DeveloperError> {
        let plane = plane.ok_or_else(|| DeveloperError::new("plane is required."))?;
        Ok(self.oriented_bounding_box.intersect_plane(plane))
    }

    pub fn update(&mut self, center: Cartesian3, half_axes: Matrix3) {
        let half_axes_dmat: DMat3 = half_axes.into();
        let validated_axes = check_half_axes(half_axes_dmat);
        let center_dvec: DVec3 = center.into();

        self.oriented_bounding_box.center = center_dvec;
        self.oriented_bounding_box.half_axes = validated_axes;
        self.bounding_sphere = Sphere::from_oriented_bounding_box(&self.oriented_bounding_box)
            .unwrap_or_else(|_| Sphere::new(center_dvec, EPSILON7));
    }

    pub fn create_debug_volume(&self, color: Option<Color>) -> Result<Primitive, DeveloperError> {
        let color = color.ok_or_else(|| DeveloperError::new("color is required."))?;
        let outline = BoxOutlineGeometry::new(
            Cartesian3::new(-1.0, -1.0, -1.0),
            Cartesian3::new(1.0, 1.0, 1.0),
        );
        let geometry = outline
            .create_geometry()
            .ok_or_else(|| DeveloperError::new("Failed to create box outline geometry"))?;

        let rot: Matrix3 = self.oriented_bounding_box.half_axes.into();
        let trans: Cartesian3 = self.oriented_bounding_box.center.into();
        let model_matrix = Matrix4::from_rotation_translation(&rot, trans);

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
            appearance: Some(PrimitiveAppearance::new(false, false)),
            asynchronous: Some(false),
            ..Default::default()
        }))
    }
}
