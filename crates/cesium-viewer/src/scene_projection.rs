//! Camera-relative ECEF projection, in physical framebuffer pixels; SDK converts to CSS pixels.
use crate::Scene;
use glam::DVec3;
impl Scene {
    pub fn cartesian_to_canvas(&self, p: DVec3, w: u32, h: u32) -> Option<[f64; 2]> {
        if w == 0 || h == 0 || !p.is_finite() || !(1.0..=1e8).contains(&p.length()) {
            return None;
        }
        let camera = self.orbit.camera(w, h);
        let v = camera.camera_to_ecef.transpose() * (p - camera.eye);
        if v.z >= -1e-8 {
            return None;
        }
        let ty = (camera.fovy / 2.).tan();
        Some([
            (v.x / (-v.z * ty * w as f64 / h as f64) + 1.) * w as f64 / 2.,
            (1. - v.y / (-v.z * ty)) * h as f64 / 2.,
        ])
    }
}
