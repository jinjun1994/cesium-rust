//! Column vectors, metres, radians, right-handed ECEF and negative-Z eye space.
use glam::{DMat3, DMat4, DVec3, Mat4, Vec4};
use thiserror::Error;

pub const WGS84_A: f64 = 6_378_137.0;
pub const WGS84_B: f64 = 6_356_752.314_245_179;
const E2: f64 = 1.0 - (WGS84_B * WGS84_B) / (WGS84_A * WGS84_A);

#[derive(Debug, Error)]
#[error("invalid coordinate, camera, or degenerate transform")]
pub struct MathError;

#[derive(Clone, Copy, Debug)]
pub struct Cartographic {
    pub longitude: f64,
    pub latitude: f64,
    pub height: f64,
}
impl Cartographic {
    pub fn radians(longitude: f64, latitude: f64, height: f64) -> Result<Self, MathError> {
        if !longitude.is_finite()
            || !latitude.is_finite()
            || !height.is_finite()
            || latitude.abs() > std::f64::consts::FRAC_PI_2
        {
            return Err(MathError);
        }
        Ok(Self {
            longitude,
            latitude,
            height,
        })
    }
    pub fn degrees(lon: f64, lat: f64, height: f64) -> Result<Self, MathError> {
        Self::radians(lon.to_radians(), lat.to_radians(), height)
    }
    pub fn to_ecef(self) -> DVec3 {
        let (sin_lat, cos_lat) = self.latitude.sin_cos();
        let (sin_lon, cos_lon) = self.longitude.sin_cos();
        let n = WGS84_A / (1.0 - E2 * sin_lat * sin_lat).sqrt();
        DVec3::new(
            (n + self.height) * cos_lat * cos_lon,
            (n + self.height) * cos_lat * sin_lon,
            (n * (1.0 - E2) + self.height) * sin_lat,
        )
    }
    /// Rejects the ambiguous deep interior (<1 metre from centre).
    pub fn from_ecef(p: DVec3) -> Result<Self, MathError> {
        if !p.is_finite() || p.length() < 1.0 {
            return Err(MathError);
        }
        let xy = p.x.hypot(p.y);
        if xy < 1e-9 {
            return Self::radians(
                0.0,
                p.z.signum() * std::f64::consts::FRAC_PI_2,
                p.z.abs() - WGS84_B,
            );
        }
        // Bowring initial estimate, followed by a fixed-point correction.
        let theta = (p.z * WGS84_A).atan2(xy * WGS84_B);
        let ep2 = (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B);
        let mut lat = (p.z + ep2 * WGS84_B * theta.sin().powi(3))
            .atan2(xy - E2 * WGS84_A * theta.cos().powi(3));
        for _ in 0..12 {
            let n = WGS84_A / (1.0 - E2 * lat.sin().powi(2)).sqrt();
            let next = (p.z + E2 * n * lat.sin()).atan2(xy);
            if (next - lat).abs() < 1e-15 {
                lat = next;
                break;
            }
            lat = next;
        }
        let n = WGS84_A / (1.0 - E2 * lat.sin().powi(2)).sqrt();
        let h = xy * lat.cos() + p.z * lat.sin() - n * (1.0 - E2 * lat.sin().powi(2));
        Self::radians(p.y.atan2(p.x), lat, h)
    }
    pub fn enu_to_ecef(self) -> DMat4 {
        let (s, c) = self.longitude.sin_cos();
        let (sl, cl) = self.latitude.sin_cos();
        DMat4::from_cols(
            DVec3::new(-s, c, 0.0).extend(0.0),
            DVec3::new(-sl * c, -sl * s, cl).extend(0.0),
            DVec3::new(cl * c, cl * s, sl).extend(0.0),
            self.to_ecef().extend(1.0),
        )
    }
}

pub fn ray_ellipsoid(origin: DVec3, direction: DVec3) -> Option<f64> {
    if !origin.is_finite() || !direction.is_finite() || direction.length_squared() < 1e-30 {
        return None;
    }
    let axes = DVec3::new(WGS84_A, WGS84_A, WGS84_B);
    let o = origin / axes;
    let d = direction / axes;
    let a = d.dot(d);
    let b = 2.0 * o.dot(d);
    let c = o.dot(o) - 1.0;
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 {
        return None;
    }
    let q = -0.5 * (b + disc.sqrt().copysign(b));
    let (a_t, b_t) = if q == 0.0 { (0.0, 0.0) } else { (q / a, c / q) };
    [a_t, b_t]
        .into_iter()
        .filter(|t| *t >= 0.0 && t.is_finite())
        .min_by(f64::total_cmp)
}

pub fn linear_part(m: DMat4) -> DMat3 {
    DMat3::from_cols(
        m.x_axis.truncate(),
        m.y_axis.truncate(),
        m.z_axis.truncate(),
    )
}
/// sqrt(norm1 * normInfinity), a conservative spectral norm bound (including shear).
pub fn scale_bound(a: DMat3) -> f64 {
    let c = [a.x_axis.abs(), a.y_axis.abs(), a.z_axis.abs()];
    let n1 = c.iter().map(|v| v.element_sum()).fold(0.0, f64::max);
    let ni = (c[0] + c[1] + c[2]).max_element();
    (n1 * ni).sqrt()
}
pub fn relative_model_view(model: DMat4, eye: DVec3, camera_to_ecef: DMat3) -> Mat4 {
    let r = camera_to_ecef.transpose();
    let a = r * linear_part(model);
    let t = r * (model.w_axis.truncate() - eye);
    DMat4::from_cols(
        a.x_axis.extend(0.0),
        a.y_axis.extend(0.0),
        a.z_axis.extend(0.0),
        t.extend(1.0),
    )
    .as_mat4()
}
pub fn reverse_z(fovy: f64, aspect: f64, near: f64) -> Result<Mat4, MathError> {
    if fovy <= 0.0
        || !fovy.is_finite()
        || !(0.0..std::f64::consts::PI).contains(&fovy)
        || !aspect.is_finite()
        || aspect <= 0.0
        || !near.is_finite()
        || near <= 0.0
    {
        return Err(MathError);
    }
    let f = (1.0 / (fovy * 0.5).tan()) as f32;
    Ok(Mat4::from_cols(
        Vec4::new(f / aspect as f32, 0.0, 0.0, 0.0),
        Vec4::new(0.0, f, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 0.0, -1.0),
        Vec4::new(0.0, 0.0, near as f32, 0.0),
    ))
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
}
impl Sphere {
    pub fn transformed(self, m: DMat4) -> Self {
        Self {
            center: m.transform_point3(self.center),
            radius: self.radius * scale_bound(linear_part(m)),
        }
    }
    pub fn distance(self, p: DVec3) -> f64 {
        (p.distance(self.center) - self.radius).max(0.0)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub eye: DVec3,
    pub camera_to_ecef: DMat3,
    pub fovy: f64,
    pub near: f64,
    pub width: u32,
    pub height: u32,
}
impl Camera {
    pub fn look_at(
        eye: DVec3,
        target: DVec3,
        up: DVec3,
        width: u32,
        height: u32,
    ) -> Result<Self, MathError> {
        let z = (eye - target).try_normalize().ok_or(MathError)?;
        let x = up.cross(z).try_normalize().ok_or(MathError)?;
        Ok(Self {
            eye,
            camera_to_ecef: DMat3::from_cols(x, z.cross(x), z),
            fovy: 60f64.to_radians(),
            near: 0.1,
            width,
            height,
        })
    }
    pub fn projection(self) -> Result<Mat4, MathError> {
        if self.width == 0 || self.height == 0 {
            return Err(MathError);
        }
        reverse_z(self.fovy, self.width as f64 / self.height as f64, self.near)
    }
    pub fn sse(self, sphere: Sphere, error: f64) -> f64 {
        error * self.height as f64
            / (2.0 * (self.fovy * 0.5).tan() * sphere.distance(self.eye).max(1e-6))
    }
    pub fn visible(self, s: Sphere) -> bool {
        let p = self.camera_to_ecef.transpose() * (s.center - self.eye);
        let ty = (self.fovy * 0.5).tan();
        let tx = ty * self.width as f64 / self.height.max(1) as f64;
        // Exact sphere/side-plane distances, conservative for all orientations.
        -p.z + s.radius >= self.near
            && p.x.abs() + p.z * tx <= s.radius * (1.0 + tx * tx).sqrt()
            && p.y.abs() + p.z * ty <= s.radius * (1.0 + ty * ty).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrips() {
        for lat in [-90.0, -89.999, -60.0, 0.0, 60.0, 89.999, 90.0] {
            for lon in [-180.0, -170.0, 0.0, 179.999] {
                for h in [-500.0, 0.0, 10000.0, 36_000_000.0] {
                    let a = Cartographic::degrees(lon, lat, h).unwrap();
                    let b = Cartographic::from_ecef(a.to_ecef()).unwrap();
                    assert!(a.to_ecef().distance(b.to_ecef()) < 1e-6);
                    assert!((a.height - b.height).abs() < 1e-6);
                }
            }
        }
    }
    #[test]
    fn enu_is_orthonormal() {
        let m = Cartographic::degrees(121.0, 31.0, 0.0)
            .unwrap()
            .enu_to_ecef();
        let a = linear_part(m);
        assert!((a.determinant() - 1.0).abs() < 1e-14);
        assert!((a.transpose() * a - DMat3::IDENTITY)
            .to_cols_array()
            .iter()
            .all(|v| v.abs() < 1e-14));
    }
    #[test]
    fn projection_and_relative_translation() {
        let p = reverse_z(1.0, 1.0, 0.1).unwrap();
        for d in [0.1, 1.0, 1e6] {
            let q = p * Vec4::new(0.0, 0.0, -d, 1.0);
            assert!((q.z / q.w - 0.1 / d).abs() < 1e-6);
        }
        let eye = DVec3::new(WGS84_A, 2.0, 3.0);
        let m = relative_model_view(
            DMat4::from_translation(eye + DVec3::X * 0.01),
            eye,
            DMat3::IDENTITY,
        );
        assert!((m.w_axis.x - 0.01).abs() < 1e-7);
        assert!(reverse_z(0.0, 1.0, 0.1).is_err());
    }
    #[test]
    fn ray_and_shear() {
        let t = ray_ellipsoid(DVec3::X * (WGS84_A + 100.0), -DVec3::X).unwrap();
        assert!((t - 100.0).abs() < 1e-7);
        assert!(ray_ellipsoid(DVec3::X * (WGS84_A + 100.0), DVec3::X).is_none());
        assert!(Cartographic::from_ecef(DVec3::ZERO).is_err());
        let a = DMat3::from_cols(DVec3::X, DVec3::new(2.0, 1.0, 0.0), DVec3::Z);
        assert!(scale_bound(a) >= a.mul_vec3(DVec3::new(1.0, 1.0, 0.0).normalize()).length());
    }
}
