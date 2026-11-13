//! Conservative collection framing from the actual resident geometry, including outline-only shapes.
use crate::{camera_control::LookOffset, RenderError, Scene};
use glam::DVec3;
#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub center: DVec3,
    pub radius: f64,
}
pub fn bounds(points: &[DVec3]) -> Option<Bounds> {
    let first = *points.first()?;
    let (mut min, mut max) = (first, first);
    for &p in points {
        if !p.is_finite() {
            return None;
        }
        min = min.min(p);
        max = max.max(p);
    }
    let center = (min + max) * 0.5;
    Some(Bounds {
        center,
        radius: points.iter().map(|p| p.distance(center)).fold(0., f64::max),
    })
}
impl Scene {
    /// Empty ids selects all visible, currently available entities. Unknown explicit IDs fail atomically.
    pub fn zoom_to_entities(
        &mut self,
        ids: &[String],
        width: u32,
        height: u32,
    ) -> Result<(), RenderError> {
        if width == 0 || height == 0 || width > 16384 || height > 16384 || ids.len() > 256 {
            return Err(RenderError("entity framing viewport/id limit".into()));
        }
        if ids.iter().any(|id| !self.entities.contains_key(id)) {
            return Err(RenderError("entity framing unknown id".into()));
        }
        let mut points = Vec::new();
        for (id, e) in &self.entities {
            if !self.entity_group_visible(id) {
                continue;
            }
            if !ids.is_empty() && !ids.contains(id) {
                continue;
            }
            let Some(model) = e.spec.model(self.clock.current_time) else {
                continue;
            };
            let prims = e.spec.primitives().map_err(RenderError)?;
            if e.spec.shape.screen() {
                points.push(model.transform_point3(DVec3::ZERO));
            } else {
                for p in &prims {
                    let xform = model * p.transform;
                    for v in &p.vertices {
                        points.push(xform.transform_point3(DVec3::from_array(v.position.map(f64::from))));
                    }
                }
            }
        }
        let b = bounds(&points).ok_or_else(|| RenderError("no visible entity bounds".into()))?;
        let vertical = cesium_math::Camera::cesium_fovy(width, height) * 0.5;
        let horizontal = (vertical.tan() * width as f64 / height as f64).atan();
        let (center, target) = if b.center.length() < 100.0 {
            let c = cesium_math::Cartographic::degrees(0., 0., 0.).unwrap();
            let t = c.to_ecef();
            (c, t)
        } else {
            let c = cesium_math::Cartographic::from_ecef(b.center)
                .map_err(|e| RenderError(e.to_string()))?;
            (c, b.center)
        };
        // ECEF linear interpolation may put a sampled point below the ellipsoid.
        // Keep the eye above ground without changing the object's position/occlusion.
        let clearance_range = if center.height < 0. {
            (100. - center.height) * std::f64::consts::SQRT_2
        } else {
            50.
        };
        let range = (b.radius.max(10.) / vertical.min(horizontal).sin())
            .max(clearance_range)
            .min(29_000_000.);
        let orbit = LookOffset {
            heading: 0.,
            pitch: -std::f64::consts::FRAC_PI_4,
            range,
        }
        .orbit(target)?;
        self.release_camera();
        self.orbit = orbit;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_enclose_corners_and_empty_fails() {
        assert!(bounds(&[]).is_none());
        let p = [DVec3::new(-4., -3., -2.), DVec3::new(4., 3., 2.)];
        let b = bounds(&p).unwrap();
        assert_eq!(b.center, DVec3::ZERO);
        for v in p {
            assert!(v.distance(b.center) <= b.radius);
        }
        assert!(bounds(&[DVec3::splat(f64::NAN)]).is_none());
    }
}
