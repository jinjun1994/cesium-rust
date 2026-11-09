use cesium_math::{Cartesian3, MathError, Plane};

/// A callback invoked when a clipping plane's distance or normal changes.
pub type ClippingPlaneChangeCallback = Box<dyn FnMut(i32) + Send + Sync>;

/// A Plane in Hessian Normal form used with clipping planes collections.
pub struct ClippingPlane {
    normal: Cartesian3,
    distance: f64,
    /// The index of this clipping plane inside a collection.
    pub index: i32,
    /// Optional callback invoked when normal or distance changes.
    pub on_change_callback: Option<ClippingPlaneChangeCallback>,
}

impl Clone for ClippingPlane {
    fn clone(&self) -> Self {
        Self {
            normal: self.normal,
            distance: self.distance,
            index: -1,
            on_change_callback: None,
        }
    }
}

impl std::fmt::Debug for ClippingPlane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClippingPlane")
            .field("normal", &self.normal)
            .field("distance", &self.distance)
            .field("index", &self.index)
            .finish()
    }
}

impl PartialEq for ClippingPlane {
    fn eq(&self, other: &Self) -> bool {
        self.normal == other.normal && self.distance == other.distance
    }
}

impl ClippingPlane {
    /// Creates a new `ClippingPlane` with given normal and distance.
    pub fn new(normal: Cartesian3, distance: f64) -> Self {
        Self {
            normal,
            distance,
            index: -1,
            on_change_callback: None,
        }
    }

    /// Gets the plane's normal.
    #[inline]
    pub fn normal(&self) -> Cartesian3 {
        self.normal
    }

    /// Gets the distance of the plane from the origin.
    #[inline]
    pub fn distance(&self) -> f64 {
        self.distance
    }

    /// Sets the distance, triggering `on_change_callback` if value changes.
    pub fn set_distance(&mut self, value: f64) {
        if (value - self.distance).abs() > 0.0 {
            if let Some(cb) = &mut self.on_change_callback {
                cb(self.index);
            }
            self.distance = value;
        }
    }

    /// Sets the normal, triggering `on_change_callback` if value changes.
    pub fn set_normal(&mut self, value: Cartesian3) {
        if self.normal != value {
            if let Some(cb) = &mut self.on_change_callback {
                cb(self.index);
            }
            self.normal = value;
        }
    }

    /// Sets normal.x, triggering `on_change_callback` if value changes.
    pub fn set_normal_x(&mut self, x: f64) {
        if (self.normal.x - x).abs() > 0.0 {
            if let Some(cb) = &mut self.on_change_callback {
                cb(self.index);
            }
            self.normal.x = x;
        }
    }

    /// Sets normal.y, triggering `on_change_callback` if value changes.
    pub fn set_normal_y(&mut self, y: f64) {
        if (self.normal.y - y).abs() > 0.0 {
            if let Some(cb) = &mut self.on_change_callback {
                cb(self.index);
            }
            self.normal.y = y;
        }
    }

    /// Sets normal.z, triggering `on_change_callback` if value changes.
    pub fn set_normal_z(&mut self, z: f64) {
        if (self.normal.z - z).abs() > 0.0 {
            if let Some(cb) = &mut self.on_change_callback {
                cb(self.index);
            }
            self.normal.z = z;
        }
    }

    /// Creates a `ClippingPlane` from a `Plane`.
    pub fn from_plane(plane: &Plane) -> Self {
        Self::new(
            Cartesian3::new(plane.normal.x, plane.normal.y, plane.normal.z),
            plane.distance,
        )
    }

    /// Copies plane parameters into a result `ClippingPlane`.
    pub fn from_plane_to(plane: &Plane, result: &mut Self) {
        result.set_normal(Cartesian3::new(
            plane.normal.x,
            plane.normal.y,
            plane.normal.z,
        ));
        result.set_distance(plane.distance);
    }

    /// Clones parameters into another `ClippingPlane`.
    pub fn clone_to(&self, result: &mut Self) {
        result.set_normal(self.normal);
        result.set_distance(self.distance);
    }

    /// Converts this `ClippingPlane` to a `Plane`.
    pub fn to_plane(&self) -> Result<Plane, MathError> {
        Plane::new(
            glam::DVec3::new(self.normal.x, self.normal.y, self.normal.z),
            self.distance,
        )
    }
}
