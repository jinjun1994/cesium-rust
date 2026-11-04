//! SphereGeometry and SphereOutlineGeometry matching CesiumJS `Core/SphereGeometry` and `Core/SphereOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    ellipsoid_geometry::{EllipsoidGeometry, EllipsoidOutlineGeometry},
    geometry::Geometry,
    Cartesian3, VertexFormat,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereGeometry {
    pub radius: f64,
    pub stack_partitions: i32,
    pub slice_partitions: i32,
    pub vertex_format: VertexFormat,
}

impl Default for SphereGeometry {
    fn default() -> Self {
        Self {
            radius: 1.0,
            stack_partitions: 64,
            slice_partitions: 64,
            vertex_format: VertexFormat::DEFAULT,
        }
    }
}

impl SphereGeometry {
    pub fn new(
        radius: Option<f64>,
        stack_partitions: Option<i32>,
        slice_partitions: Option<i32>,
        vertex_format: Option<VertexFormat>,
    ) -> Self {
        Self {
            radius: radius.unwrap_or(1.0),
            stack_partitions: stack_partitions.unwrap_or(64),
            slice_partitions: slice_partitions.unwrap_or(64),
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let r = self.radius;
        if r <= 0.0 {
            return None;
        }
        let ell = EllipsoidGeometry::new(
            Some(Cartesian3::new(r, r, r)),
            None,
            None,
            None,
            None,
            None,
            Some(self.stack_partitions),
            Some(self.slice_partitions),
            Some(self.vertex_format),
        );
        ell.create_geometry()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereOutlineGeometry {
    pub radius: f64,
    pub stack_partitions: i32,
    pub slice_partitions: i32,
    pub subdivisions: i32,
}

impl Default for SphereOutlineGeometry {
    fn default() -> Self {
        Self {
            radius: 1.0,
            stack_partitions: 10,
            slice_partitions: 8,
            subdivisions: 200,
        }
    }
}

impl SphereOutlineGeometry {
    pub fn new(
        radius: Option<f64>,
        stack_partitions: Option<i32>,
        slice_partitions: Option<i32>,
        subdivisions: Option<i32>,
    ) -> Self {
        Self {
            radius: radius.unwrap_or(1.0),
            stack_partitions: stack_partitions.unwrap_or(10),
            slice_partitions: slice_partitions.unwrap_or(8),
            subdivisions: subdivisions.unwrap_or(200),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        let r = self.radius;
        if r <= 0.0 {
            return None;
        }
        let ell = EllipsoidOutlineGeometry::new(
            Some(Cartesian3::new(r, r, r)),
            None,
            None,
            None,
            None,
            None,
            Some(self.stack_partitions),
            Some(self.slice_partitions),
            Some(self.subdivisions),
        );
        ell.create_geometry()
    }
}
