//! ClippingPlaneCollection matching CesiumJS `Scene/ClippingPlaneCollection.js` (Apache-2.0).

use crate::{Color, Matrix4, Plane};

/// Specifies a set of clipping planes.
#[derive(Debug, Clone, PartialEq)]
pub struct ClippingPlaneCollection {
    pub planes: Vec<Plane>,
    pub enabled: bool,
    pub model_matrix: Matrix4,
    pub union_clipping_regions: bool,
    pub edge_color: Color,
    pub edge_width: f64,
}

impl Default for ClippingPlaneCollection {
    fn default() -> Self {
        Self {
            planes: Vec::new(),
            enabled: true,
            model_matrix: Matrix4::IDENTITY,
            union_clipping_regions: false,
            edge_color: Color::WHITE,
            edge_width: 0.0,
        }
    }
}

impl ClippingPlaneCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_planes(planes: Vec<Plane>) -> Self {
        Self {
            planes,
            ..Default::default()
        }
    }

    pub fn len(&self) -> usize {
        self.planes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.planes.is_empty()
    }

    pub fn add(&mut self, plane: Plane) {
        self.planes.push(plane);
    }

    pub fn remove_all(&mut self) {
        self.planes.clear();
    }
}
