use cesium_math::{BoundingSphere, Ellipsoid};
use glam::DVec3;

/// Computes a depth plane quad to mask back-facing regions of the globe.
#[derive(Debug, Clone)]
pub struct DepthPlane {
    /// Offset added to the ellipsoid radii.
    pub ellipsoid_offset: f64,
    /// Bounding volume of the depth plane.
    pub bounding_volume: BoundingSphere,
}

impl Default for DepthPlane {
    fn default() -> Self {
        Self::new(None)
    }
}

impl DepthPlane {
    /// Creates a new `DepthPlane` with an optional ellipsoid offset.
    pub fn new(ellipsoid_offset: Option<f64>) -> Self {
        let offset = ellipsoid_offset.unwrap_or(0.0);
        let base_radii = Ellipsoid::WGS84.radii();
        let max_r = (base_radii.x + offset)
            .max(base_radii.y + offset)
            .max(base_radii.z + offset);

        Self {
            ellipsoid_offset: offset,
            bounding_volume: BoundingSphere::new(DVec3::ZERO, max_r),
        }
    }

    /// Updates the depth plane against the ellipsoid and camera view, computing the 4-vertex quad positions.
    pub fn update(
        &mut self,
        ellipsoid: &Ellipsoid,
        camera_position_wc: Option<DVec3>,
        is_orthographic: bool,
    ) -> [f32; 12] {
        let radii = ellipsoid.radii() + DVec3::splat(self.ellipsoid_offset);
        let max_r = radii.x.max(radii.y).max(radii.z);
        self.bounding_volume = BoundingSphere::new(DVec3::ZERO, max_r);

        let mut quad = [0.0f32; 12];

        let (center, east_offset, north_offset) = match (is_orthographic, camera_position_wc) {
            (true, _) | (_, None) => (DVec3::ZERO, DVec3::X, DVec3::Y),
            (false, Some(p)) => {
                let one_over_radii = DVec3::ONE / radii;
                let q = p * one_over_radii;
                let q_mag = q.length();

                if q_mag <= 1.0 {
                    (DVec3::ZERO, DVec3::X, DVec3::Y)
                } else {
                    let q_unit = q / q_mag;
                    let z_cross_q = DVec3::Z.cross(q);
                    let e_unit = if z_cross_q.length_squared() > 1e-12 {
                        z_cross_q.normalize()
                    } else {
                        DVec3::X
                    };
                    let n_unit = q_unit.cross(e_unit).normalize();

                    let w_mag = (q_mag * q_mag - 1.0).sqrt();
                    let center = q_unit * (1.0 / q_mag);
                    let scalar = w_mag / q_mag;
                    let east_offset = e_unit * scalar;
                    let north_offset = n_unit * scalar;
                    (center, east_offset, north_offset)
                }
            }
        };

        // upperLeft = (center + northOffset - eastOffset) * radii
        let ul = (center + north_offset - east_offset) * radii;
        quad[0] = ul.x as f32;
        quad[1] = ul.y as f32;
        quad[2] = ul.z as f32;

        // lowerLeft = (center - northOffset - eastOffset) * radii
        let ll = (center - north_offset - east_offset) * radii;
        quad[3] = ll.x as f32;
        quad[4] = ll.y as f32;
        quad[5] = ll.z as f32;

        // upperRight = (center + northOffset + eastOffset) * radii
        let ur = (center + north_offset + east_offset) * radii;
        quad[6] = ur.x as f32;
        quad[7] = ur.y as f32;
        quad[8] = ur.z as f32;

        // lowerRight = (center - northOffset + eastOffset) * radii
        let lr = (center - north_offset + east_offset) * radii;
        quad[9] = lr.x as f32;
        quad[10] = lr.y as f32;
        quad[11] = lr.z as f32;

        quad
    }
}
