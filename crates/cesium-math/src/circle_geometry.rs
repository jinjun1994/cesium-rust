//! CircleGeometry and CircleOutlineGeometry matching CesiumJS `Core/CircleGeometry` and `Core/CircleOutlineGeometry`.
//! Adapted from CesiumJS (Apache-2.0).

use crate::{
    geometry::Geometry,
    Cartesian3, EllipseGeometry, EllipseOutlineGeometry, Ellipsoid, VertexFormat,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircleGeometry {
    pub center: Cartesian3,
    pub radius: f64,
    pub ellipsoid: Ellipsoid,
    pub height: f64,
    pub extruded_height: Option<f64>,
    pub granularity: f64,
    pub vertex_format: VertexFormat,
    pub st_rotation: f64,
}

impl CircleGeometry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: Cartesian3,
        radius: f64,
        ellipsoid: Option<Ellipsoid>,
        height: Option<f64>,
        extruded_height: Option<f64>,
        granularity: Option<f64>,
        vertex_format: Option<VertexFormat>,
        st_rotation: Option<f64>,
    ) -> Self {
        Self {
            center,
            radius,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
            height: height.unwrap_or(0.0),
            extruded_height,
            granularity: granularity.unwrap_or(0.02),
            vertex_format: vertex_format.unwrap_or(VertexFormat::DEFAULT),
            st_rotation: st_rotation.unwrap_or(0.0),
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        if self.radius <= 0.0 || self.granularity <= 0.0 {
            return None;
        }

        let ellipse = EllipseGeometry::new(
            self.center,
            self.radius,
            self.radius,
            Some(self.ellipsoid),
            Some(self.height),
            self.extruded_height,
            Some(self.granularity),
            Some(0.0),
            Some(self.st_rotation),
            Some(self.vertex_format),
        );
        ellipse.create_geometry()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircleOutlineGeometry {
    pub center: Cartesian3,
    pub radius: f64,
    pub ellipsoid: Ellipsoid,
    pub height: f64,
    pub extruded_height: Option<f64>,
    pub granularity: f64,
    pub number_of_vertical_lines: Option<usize>,
}

impl CircleOutlineGeometry {
    pub fn new(
        center: Cartesian3,
        radius: f64,
        ellipsoid: Option<Ellipsoid>,
        height: Option<f64>,
        extruded_height: Option<f64>,
        granularity: Option<f64>,
        number_of_vertical_lines: Option<usize>,
    ) -> Self {
        Self {
            center,
            radius,
            ellipsoid: ellipsoid.unwrap_or(Ellipsoid::WGS84),
            height: height.unwrap_or(0.0),
            extruded_height,
            granularity: granularity.unwrap_or(0.02),
            number_of_vertical_lines,
        }
    }

    pub fn create_geometry(&self) -> Option<Geometry> {
        if self.radius <= 0.0 || self.granularity <= 0.0 {
            return None;
        }

        let ellipse = EllipseOutlineGeometry::new(
            self.center,
            self.radius,
            self.radius,
            Some(self.ellipsoid),
            Some(self.height),
            self.extruded_height,
            Some(self.granularity),
            Some(0.0),
            self.number_of_vertical_lines,
        );
        ellipse.create_geometry()
    }
}
