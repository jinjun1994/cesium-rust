//! PlaneGeometryUpdater basis: n dot p + distance = 0, in the entity ENU frame.
use glam::DVec3;
pub fn corners(dimensions: [f64; 2], normal: [f64; 3], distance: f64) -> [DVec3; 4] {
    let n = DVec3::from_array(normal);
    let up = if (n.z.abs() - 1.).abs() <= 1e-8 {
        DVec3::Y
    } else {
        DVec3::Z
    };
    let left = up.cross(n).normalize();
    let up = n.cross(left).normalize();
    let center = -distance * n;
    let x = left * dimensions[0] * 0.5;
    let y = up * dimensions[1] * 0.5;
    [
        center - x - y,
        center + x - y,
        center + x + y,
        center - x + y,
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn three_axes_and_offset_share_fill_outline_basis() {
        for n in [
            DVec3::X,
            DVec3::Y,
            DVec3::Z,
            DVec3::new(1., 2., 3.).normalize(),
        ] {
            let v = corners([400000., 300000.], n.to_array(), 120.);
            for p in v {
                assert!((n.dot(p) + 120.).abs() < 1e-8);
            }
            assert!((v[0].distance(v[1]) - 400000.).abs() < 1e-8);
            assert!((v[1].distance(v[2]) - 300000.).abs() < 1e-8);
            assert!((v[1] - v[0]).cross(v[2] - v[1]).normalize().dot(n) > 0.99999999);
        }
    }
    #[test]
    fn legacy_defaults_and_invalid_normals() {
        use crate::entities;
        let valid = serde_json::json!([{"id":"p","positionDegrees":[0,0,1000],"shape":{"type":"plane","dimensions":[10,20]}}]);
        assert!(entities::decode(&serde_json::to_vec(&valid).unwrap()).is_ok());
        for normal in [[0., 0., 0.], [0., 0., 2.]] {
            let mut bad = valid.clone();
            bad[0]["shape"]["normal"] = serde_json::json!(normal);
            assert!(entities::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
        }
    }
}
