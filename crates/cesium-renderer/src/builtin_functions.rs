//! Builtin GLSL functions matching CesiumJS `Shaders/Builtin/Functions/*.glsl` (Apache-2.0).

use cesium_math::{Cartesian2, Cartesian3, Cartesian4, Matrix2, Matrix3, Matrix4};

/// Ray representation used by `czm_pointAlongRay`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Cartesian3,
    pub direction: Cartesian3,
}

impl Ray {
    pub fn new(origin: Cartesian3, direction: Cartesian3) -> Self {
        Self { origin, direction }
    }
}

/// Cesium built-in shader mathematical algorithms evaluated on CPU/GPU.
pub struct BuiltinFunctions;

impl BuiltinFunctions {
    /// Transposes a 2x2 matrix.
    pub fn transpose_mat2(m: &Matrix2) -> Matrix2 {
        m.transpose()
    }

    /// Transposes a 3x3 matrix.
    pub fn transpose_mat3(m: &Matrix3) -> Matrix3 {
        m.transpose()
    }

    /// Transposes a 4x4 matrix.
    pub fn transpose_mat4(m: &Matrix4) -> Matrix4 {
        m.transpose()
    }

    /// Signed not zero: returns 1.0 if v >= 0.0, else -1.0.
    #[inline]
    pub fn sign_not_zero_f32(v: f32) -> f32 {
        if v >= 0.0 { 1.0 } else { -1.0 }
    }

    pub fn sign_not_zero_vec2(v: [f32; 2]) -> [f32; 2] {
        [Self::sign_not_zero_f32(v[0]), Self::sign_not_zero_f32(v[1])]
    }

    pub fn sign_not_zero_vec3(v: [f32; 3]) -> [f32; 3] {
        [
            Self::sign_not_zero_f32(v[0]),
            Self::sign_not_zero_f32(v[1]),
            Self::sign_not_zero_f32(v[2]),
        ]
    }

    pub fn sign_not_zero_vec4(v: [f32; 4]) -> [f32; 4] {
        [
            Self::sign_not_zero_f32(v[0]),
            Self::sign_not_zero_f32(v[1]),
            Self::sign_not_zero_f32(v[2]),
            Self::sign_not_zero_f32(v[3]),
        ]
    }

    /// Signed distance from a plane to a point: `dot(plane.xyz, point) + plane.w`.
    pub fn plane_distance(plane: Cartesian4, point: Cartesian3) -> f64 {
        plane.x * point.x + plane.y * point.y + plane.z * point.z + plane.w
    }

    /// Perpendicular distance from a point to a 2D line segment between point1 and point2.
    pub fn line_distance(point1: Cartesian2, point2: Cartesian2, point: Cartesian2) -> f64 {
        let v1 = point2 - point1;
        let v2 = point - point1;
        let cross = (v1.x * v2.y - v1.y * v2.x).abs();
        let len = v1.magnitude();
        if len > 0.0 { cross / len } else { 0.0 }
    }

    /// Computes tangent-to-eye space rotation matrix from normal, tangent, and bitangent vectors.
    pub fn tangent_to_eye_space_matrix(
        normal: Cartesian3,
        tangent: Cartesian3,
        bitangent: Cartesian3,
    ) -> Matrix3 {
        Matrix3::new(
            tangent.x, bitangent.x, normal.x,
            tangent.y, bitangent.y, normal.y,
            tangent.z, bitangent.z, normal.z,
        )
    }

    /// Translates encoded high-low coordinates relative to eye position.
    pub fn translate_relative_to_eye(
        high: Cartesian3,
        low: Cartesian3,
        camera_pos_high: Cartesian3,
        camera_pos_low: Cartesian3,
    ) -> Cartesian4 {
        let diff_high = high - camera_pos_high;
        let diff_low = low - camera_pos_low;
        let p = diff_high + diff_low;
        Cartesian4::new(p.x, p.y, p.z, 1.0)
    }

    /// Point along ray: `origin + direction * t`.
    pub fn point_along_ray(ray: &Ray, t: f64) -> Cartesian3 {
        ray.origin + ray.direction.multiply_by_scalar(t)
    }

    /// Fast branch-free ternary selector.
    #[inline]
    pub fn branch_free_ternary<T: Copy>(cond: bool, a: T, b: T) -> T {
        if cond { a } else { b }
    }

    /// Decodes an oct-encoded 2D normal vector into a unit-length 3D vector.
    pub fn oct_decode(encoded: [f32; 2], range: f32) -> [f32; 3] {
        if encoded[0] == 0.0 && encoded[1] == 0.0 {
            return [0.0, 0.0, 0.0];
        }

        let ex = encoded[0] / range * 2.0 - 1.0;
        let ey = encoded[1] / range * 2.0 - 1.0;
        let mut v = [ex, ey, 1.0 - ex.abs() - ey.abs()];

        if v[2] < 0.0 {
            let old_x = v[0];
            let old_y = v[1];
            v[0] = (1.0 - old_y.abs()) * Self::sign_not_zero_f32(old_x);
            v[1] = (1.0 - old_x.abs()) * Self::sign_not_zero_f32(old_y);
        }

        let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if mag > 0.0 {
            [v[0] / mag, v[1] / mag, v[2] / mag]
        } else {
            [0.0, 0.0, 0.0]
        }
    }

    /// Decodes an oct-encoded normal packed into a single float (snorm 8-bit components).
    pub fn oct_decode_float(encoded: f32) -> [f32; 3] {
        let temp = encoded / 256.0;
        let x = temp.floor();
        let y = encoded - x * 256.0;
        Self::oct_decode([x, y], 255.0)
    }

    /// Decodes texture coordinates compressed into a single float (4096-based encoding).
    pub fn decompress_texture_coordinates(encoded: f32) -> [f32; 2] {
        let temp = encoded / 4096.0;
        let x_zero_to_4095 = temp.floor();
        let stx = x_zero_to_4095 / 4095.0;
        let sty = (encoded - x_zero_to_4095 * 4096.0) / 4095.0;
        [stx, sty]
    }

    /// Evaluates cosine and sine simultaneously: `(cos(angle), sin(angle))`.
    pub fn cosine_and_sine(angle: f32) -> [f32; 2] {
        [angle.cos(), angle.sin()]
    }

    /// Evaluates near-far scalar clamping and smooth linear interpolation.
    pub fn near_far_scalar(params: [f32; 4], dist_sq: f32) -> f32 {
        let near_dist = params[0];
        let near_val = params[1];
        let far_dist = params[2];
        let far_val = params[3];

        let dist = dist_sq.sqrt();
        if dist <= near_dist {
            near_val
        } else if dist >= far_dist {
            far_val
        } else {
            let t = (dist - near_dist) / (far_dist - near_dist);
            near_val + t * (far_val - near_val)
        }
    }

    /// Unpacks an IEEE-754 float from little-endian normalized RGBA vec4.
    pub fn unpack_float(packed: [f32; 4]) -> f32 {
        let p0 = (packed[0] * 255.0 + 0.5).floor();
        let p1 = (packed[1] * 255.0 + 0.5).floor();
        let p2 = (packed[2] * 255.0 + 0.5).floor();
        let p3 = (packed[3] * 255.0 + 0.5).floor();

        let sign = if p3 >= 128.0 { -1.0 } else { 1.0 };
        let exp_high = p3 % 128.0;
        let exp_low = if p2 >= 128.0 { 1.0 } else { 0.0 };
        let exponent = 2.0 * exp_high + exp_low - 127.0;

        if exponent == -127.0 {
            return 0.0;
        }

        let mantissa = (p2 % 128.0) * 65536.0 + p1 * 256.0 + p0 + 8388608.0; // 0x800000
        sign * (2.0f32).powf(exponent - 23.0) * mantissa
    }

    /// Fast approximate arctangent.
    pub fn fast_approximate_atan(x: f32) -> f32 {
        // High-order polynomial approximation matching CesiumJS fastApproximateAtan
        let a = 0.99997726;
        let b = -0.33262347;
        let c = 0.19354346;
        let d = -0.11643287;
        let e = 0.05265332;
        let f = -0.011_721_2;

        let x2 = x * x;
        x * (a + x2 * (b + x2 * (c + x2 * (d + x2 * (e + x2 * f)))))
    }

    /// Transforms a plane by a 4x4 matrix.
    pub fn transform_plane(plane: Cartesian4, transform: &Matrix4) -> Cartesian4 {
        let normal = Cartesian3::new(plane.x, plane.y, plane.z);
        let dist = plane.w;
        let p = normal.multiply_by_scalar(-dist);
        let transformed_p = transform.multiply_by_point(p);
        let inv_t = transform.inverse_transpose().unwrap_or(Matrix4::IDENTITY);
        let transformed_normal = inv_t.multiply_by_point_as_vector(normal).normalize();
        let new_dist = -transformed_normal.dot(&transformed_p);
        Cartesian4::new(
            transformed_normal.x,
            transformed_normal.y,
            transformed_normal.z,
            new_dist,
        )
    }

    /// Antialiasing color interpolation.
    pub fn antialias(color0: [f32; 4], color1: [f32; 4], _target: [f32; 4], factor: f32) -> [f32; 4] {
        [
            color0[0] + factor * (color1[0] - color0[0]),
            color0[1] + factor * (color1[1] - color0[1]),
            color0[2] + factor * (color1[2] - color0[2]),
            color0[3] + factor * (color1[3] - color0[3]),
        ]
    }
}
