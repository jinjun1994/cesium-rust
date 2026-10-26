//! Bounded geographical and screen entities and a deliberately strict CZML subset.
use cesium_formats::{Material, Primitive, Vertex};
use cesium_math::Cartographic;
use glam::{DMat4, DQuat, DVec3};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CornerType {
    #[default]
    Rounded,
    Mitered,
    Beveled,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Shape {
    Corridor {
        #[serde(rename = "positionsDegrees")]
        positions: Vec<[f64; 3]>,
        width: f64,
        #[serde(default)]
        height: f64,
        #[serde(default, rename = "extrudedHeight")]
        extruded_height: Option<f64>,
        #[serde(default, rename = "cornerType")]
        corner_type: CornerType,
    },
    Point {
        #[serde(default = "point_size", rename = "pixelSize")]
        pixel_size: f64,
        #[serde(default, rename = "outlineWidth")]
        outline_width: f64,
        #[serde(default = "outline_color", rename = "outlineColor")]
        outline_color: [f32; 4],
    },
    Billboard {
        image: String,
        #[serde(default = "billboard_size")]
        width: f64,
        #[serde(default = "billboard_size")]
        height: f64,
        #[serde(default = "unit_scale")]
        scale: f64,
        #[serde(default, rename = "pixelOffset")]
        pixel_offset: [f64; 2],
    },
    Label {
        #[serde(default)]
        appearance: crate::label_appearance::Appearance,
        text: String,
        #[serde(default = "label_size", rename = "fontSize")]
        font_size: f32,
        #[serde(default = "unit_scale")]
        scale: f64,
        #[serde(default, rename = "pixelOffset")]
        pixel_offset: [f64; 2],
    },
    Box {
        dimensions: [f64; 3],
    },
    Ellipsoid {
        radii: [f64; 3],
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "innerRadii"
        )]
        inner_radii: Option<[f64; 3]>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "minimumClock"
        )]
        minimum_clock: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "maximumClock"
        )]
        maximum_clock: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "minimumCone"
        )]
        minimum_cone: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "maximumCone"
        )]
        maximum_cone: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "stackPartitions"
        )]
        stacks: Option<u32>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "slicePartitions"
        )]
        slices: Option<u32>,
    },
    Cylinder {
        length: f64,
        #[serde(rename = "topRadius")]
        top_radius: f64,
        #[serde(rename = "bottomRadius")]
        bottom_radius: f64,
    },
    Plane {
        dimensions: [f64; 2],
        #[serde(default = "plane_normal")]
        normal: [f64; 3],
        #[serde(default)]
        distance: f64,
    },
    LineSet {
        paths: Vec<Vec<[f64; 3]>>,
        #[serde(default)]
        colors: Vec<Vec<[f32; 4]>>,
        #[serde(default)]
        geodesic: bool,
        #[serde(default, rename = "colorsPerVertex")]
        colors_per_vertex: bool,
    },
    WidePolyline {
        data: crate::wide_polyline::Polyline,
    },
    Polyline {
        #[serde(rename = "positionsDegrees")]
        positions: Vec<[f64; 3]>,
        #[serde(rename = "widthMetres")]
        width: f64,
    },
    PolygonSurface {
        data: crate::polygon_surface::PolygonSurface,
    },
    Polygon {
        #[serde(rename = "positionsDegrees")]
        positions: Vec<[f64; 3]>,
        #[serde(default)]
        holes: Vec<Vec<[f64; 3]>>,
        #[serde(default, rename = "extrudedHeight")]
        extruded_height: Option<f64>,
    },
    Rectangle {
        #[serde(rename = "coordinatesDegrees")]
        coordinates: [f64; 4],
        #[serde(default)]
        height: f64,
        #[serde(default, rename = "extrudedHeight")]
        extruded_height: Option<f64>,
        #[serde(default)]
        rotation: f64,
    },
    Ellipse {
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "numberOfVerticalLines"
        )]
        vertical_lines: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<f64>,
        #[serde(rename = "semiMajorAxis")]
        semi_major: f64,
        #[serde(rename = "semiMinorAxis")]
        semi_minor: f64,
        #[serde(default)]
        rotation: f64,
        #[serde(default, rename = "extrudedHeight")]
        extruded_height: Option<f64>,
    },
    Wall {
        #[serde(rename = "positionsDegrees")]
        positions: Vec<[f64; 3]>,
        #[serde(default, rename = "minimumHeights")]
        minimum_heights: Vec<f64>,
        #[serde(default, rename = "maximumHeights")]
        maximum_heights: Vec<f64>,
    },
    PolylineVolume {
        #[serde(rename = "positionsDegrees")]
        positions: Vec<[f64; 3]>,
        #[serde(rename = "shapePositions", alias = "shape")]
        shape_positions: Vec<[f64; 2]>,
        #[serde(default, rename = "cornerType")]
        corner_type: CornerType,
    },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Sample {
    pub time: f64,
    pub cartesian: [f64; 3],
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SurfaceAppearance {
    pub flat: bool,
    pub closed: bool,
    pub translucent: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    #[serde(default = "shown")]
    pub show: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub position_degrees: Option<[f64; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_matrix: Option<[f64; 16]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<[f64; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_appearance: Option<SurfaceAppearance>,
    #[serde(default)]
    pub samples: Vec<Sample>,
    pub shape: Shape,
    #[serde(default)]
    pub screen: crate::screen_options::ScreenOptions,
    #[serde(default = "shown")]
    pub fill: bool,
    #[serde(default)]
    pub outline: bool,
    #[serde(default = "outline_color", rename = "outlineColor")]
    pub outline_color: [f32; 4],
    #[serde(default = "outline_width", rename = "outlineWidth")]
    pub outline_width: f64,
    #[serde(default = "color")]
    pub color: [f32; 4],
    #[serde(default)]
    pub material: Option<crate::entity_material::SurfaceMaterial>,
    #[serde(default)]
    pub availability: Option<[f64; 2]>,
    #[serde(default)]
    pub properties: Value,
}
fn plane_normal() -> [f64; 3] {
    [0., 0., 1.]
}
fn label_size() -> f32 {
    30.
}
fn point_size() -> f64 {
    1.
}
fn billboard_size() -> f64 {
    64.
}
fn unit_scale() -> f64 {
    1.
}
fn outline_color() -> [f32; 4] {
    [0., 0., 0., 1.]
}
fn outline_width() -> f64 {
    1.
}
fn shown() -> bool {
    true
}
fn color() -> [f32; 4] {
    [1., 0.65, 0.1, 1.]
}
impl Entity {
    pub(crate) fn apply_surface_appearance(&self, p: &mut Primitive) {
        if let Some(a) = self.surface_appearance {
            p.material.unlit = a.flat;
            p.material.lambert = !a.flat;
            p.material.double_sided = !a.closed;
            p.material.face_forward = !a.closed;
            p.material.alpha_blend = a.translucent;
            p.material.two_pass = a.closed && a.translucent;
        }
    }
    pub fn position(&self, time: f64) -> Option<DVec3> {
        if !self.show || !time.is_finite() {
            return None;
        }
        if self.availability.is_some_and(|[a, b]| time < a || time > b) {
            return None;
        }
        if let Some(matrix) = self.model_matrix {
            return Some(DMat4::from_cols_array(&matrix).w_axis.truncate());
        }
        if !self.samples.is_empty() {
            if time < self.samples[0].time || time > self.samples.last()?.time {
                return None;
            }
            let i = self.samples.partition_point(|s| s.time < time);
            if i == 0 {
                return Some(DVec3::from_array(self.samples[0].cartesian));
            }
            if i == self.samples.len() {
                return Some(DVec3::from_array(self.samples[i - 1].cartesian));
            }
            let a = &self.samples[i - 1];
            let b = &self.samples[i];
            return Some(DVec3::from_array(a.cartesian).lerp(
                DVec3::from_array(b.cartesian),
                (time - a.time) / (b.time - a.time),
            ));
        }
        if self.shape.standalone() {
            return crate::geographic::anchor(&self.shape).ok();
        }
        let p = self.position_degrees?;
        Some(Cartographic::degrees(p[0], p[1], p[2]).ok()?.to_ecef())
    }
    pub fn model(&self, time: f64) -> Option<DMat4> {
        let p = self.position(time)?;
        if let Some(matrix) = self.model_matrix {
            return Some(DMat4::from_cols_array(&matrix));
        }
        if self.shape.geographic() {
            return Some(DMat4::from_translation(p));
        }
        if let Some(q_arr) = self.orientation {
            let q = DQuat::from_array(q_arr);
            if q.is_finite() && (q.length_squared() - 1.0).abs() <= 1e-3 {
                let mut m = DMat4::from_quat(q);
                m.w_axis = p.extend(1.);
                return Some(m);
            }
        }
        let c = Cartographic::from_ecef(p).ok()?;
        let mut m = c.enu_to_ecef();
        m.w_axis = p.extend(1.);
        Some(m)
    }
    pub fn metadata(&self) -> Value {
        json!({"id":self.id,"name":self.name,"properties":self.properties})
    }
    fn validate(&self) -> Result<(), String> {
        self.screen.validate()?;
        if self.surface_appearance.is_some()
            && (!self.fill || self.shape.screen() || self.material.is_some())
        {
            return Err("surfaceAppearance requires filled non-screen solid geometry".into());
        }
        if !self.shape.screen() && self.screen != Default::default() {
            return Err("screen properties require screen graphics".into());
        }
        if self.shape.screen() {
            crate::screen::validate(self)?;
        }
        if self.id.is_empty() || self.id.len() > 256 || self.name.len() > 1024 {
            return Err("entity id/name limit".into());
        }
        if self
            .color
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
        {
            return Err("entity color must be finite RGBA in 0..1".into());
        }
        if self
            .outline_color
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
            || !self.outline_width.is_finite()
            || !(1.0..=10.0).contains(&self.outline_width)
        {
            return Err("finite RGBA outline color and outlineWidth 1..10 required".into());
        }
        if let Some(m) = &self.material {
            m.parameters()?;
        }
        if let Some(matrix) = self.model_matrix {
            let m = DMat4::from_cols_array(&matrix);
            let det = glam::DMat3::from_mat4(m).determinant();
            if self.shape.geographic()
                || self.shape.screen()
                || !m.is_finite()
                || [matrix[3], matrix[7], matrix[11], matrix[15]] != [0., 0., 0., 1.]
                || !det.is_finite()
                || det.abs() < 1e-12
                || [m.x_axis, m.y_axis, m.z_axis]
                    .iter()
                    .any(|v| !(1e-6..=100.).contains(&v.truncate().length()))
                || !(1.0..=1e8).contains(&m.w_axis.truncate().length())
            {
                return Err(
                    "bounded nonsingular affine local-geometry modelMatrix required".into(),
                );
            }
        }
        if self.shape.standalone() {
            if self.model_matrix.is_some()
                || self.position_degrees.is_some()
                || !self.samples.is_empty()
            {
                return Err("geographic geometry supplies its own positions; entity position/samples are not allowed".into());
            }
        } else if usize::from(self.position_degrees.is_some())
            + usize::from(!self.samples.is_empty())
            + usize::from(self.model_matrix.is_some())
            != 1
        {
            return Err("exactly one of positionDegrees, samples or modelMatrix required".into());
        }
        if self.shape.geographic() && !self.samples.is_empty() {
            return Err("sampled geographic geometry is not supported".into());
        }
        if let Some(p) = self.position_degrees {
            Cartographic::degrees(p[0], p[1], p[2]).map_err(|e| e.to_string())?;
            if p[2].abs() > 1e8 {
                return Err("entity height limit".into());
            }
        }
        if self.samples.len() > 10000 {
            return Err("entity sample limit".into());
        }
        for (i, s) in self.samples.iter().enumerate() {
            let p = DVec3::from_array(s.cartesian);
            if !s.time.is_finite()
                || s.time.abs() > 1e12
                || !p.is_finite()
                || p.length() < 1.
                || p.length() > 1e8
                || i > 0 && self.samples[i - 1].time >= s.time
            {
                return Err(
                    "strictly increasing finite samples with bounded ECEF coordinates required"
                        .into(),
                );
            }
        }
        if self
            .availability
            .is_some_and(|[a, b]| !a.is_finite() || !b.is_finite() || a > b)
        {
            return Err("availability interval".into());
        }
        let positive = |x: f64| x.is_finite() && (0.0..=1e6).contains(&x) && x > 0.;
        let valid = match self.shape {
            Shape::Box { dimensions } => dimensions.into_iter().all(positive),
            Shape::Ellipsoid {
                radii,
                inner_radii,
                minimum_clock,
                maximum_clock,
                minimum_cone,
                maximum_cone,
                stacks,
                slices,
            } => {
                radii.into_iter().all(positive)
                    && inner_radii.as_ref().is_none_or(|in_r| {
                        in_r.iter().copied().all(positive)
                            && in_r[0] < radii[0]
                            && in_r[1] < radii[1]
                            && in_r[2] < radii[2]
                    })
                    && minimum_clock.is_none_or(|c| c.is_finite())
                    && maximum_clock.is_none_or(|c| c.is_finite())
                    && minimum_cone
                        .is_none_or(|c| c.is_finite() && (0.0..=std::f64::consts::PI).contains(&c))
                    && maximum_cone
                        .is_none_or(|c| c.is_finite() && (0.0..=std::f64::consts::PI).contains(&c))
                    && match (minimum_cone, maximum_cone) {
                        (Some(min_c), Some(max_c)) => min_c <= max_c,
                        _ => true,
                    }
                    && match (minimum_clock, maximum_clock) {
                        (Some(min_c), Some(max_c)) => min_c <= max_c,
                        _ => true,
                    }
                    && stacks.is_none_or(|n| (3..=128).contains(&n))
                    && slices.is_none_or(|n| (3..=128).contains(&n))
            }
            Shape::Rectangle { rotation, .. } => rotation.is_finite(),
            Shape::Plane {
                dimensions,
                normal,
                distance,
            } => {
                dimensions.into_iter().all(positive)
                    && DVec3::from_array(normal).is_finite()
                    && (DVec3::from_array(normal).length_squared() - 1.).abs() <= 1e-8
                    && distance.is_finite()
                    && distance.abs() <= 1e6
            }
            Shape::Cylinder {
                length,
                top_radius,
                bottom_radius,
            } => {
                positive(length)
                    && top_radius.is_finite()
                    && bottom_radius.is_finite()
                    && (0.0..=1e6).contains(&top_radius)
                    && (0.0..=1e6).contains(&bottom_radius)
                    && (top_radius > 0. || bottom_radius > 0.)
            }
            Shape::PolylineVolume {
                ref positions,
                ref shape_positions,
                ..
            } => {
                (2..=64).contains(&positions.len())
                    && (3..=1024).contains(&shape_positions.len())
                    && positions.iter().all(|p| {
                        p.iter().all(|v| v.is_finite())
                            && p[0].abs() <= 180.
                            && p[1].abs() <= 85.
                            && p[2].abs() <= 1e7
                    })
                    && shape_positions.iter().all(|p| {
                        p.iter().all(|v| v.is_finite()) && p[0].abs() <= 1e7 && p[1].abs() <= 1e7
                    })
            }
            _ => true,
        };
        if !valid {
            return Err(
                "geometry dimensions must be finite, positive and <= 1,000,000 metres".into(),
            );
        }
        Ok(())
    }
    pub fn primitive(&self) -> Result<Primitive, String> {
        self.validate()?;
        if let Shape::WidePolyline { data } = &self.shape {
            return crate::wide_polyline::primitive(self, data);
        }
        if let Shape::PolylineVolume {
            positions,
            shape_positions,
            corner_type,
        } = &self.shape
        {
            return crate::polyline_volume::primitive(
                self,
                positions,
                shape_positions,
                *corner_type,
            );
        }
        if matches!(self.shape, Shape::LineSet { .. }) {
            return crate::line_set::primitive(self);
        }
        if self.shape.screen() {
            return crate::screen::primitive(self);
        }
        if self.shape.geographic() {
            let mut p = crate::geographic::primitive(self)?;
            crate::entity_material::apply(self, &mut p)?;
            self.apply_surface_appearance(&mut p);
            return Ok(p);
        }
        let mut p = Primitive {
            vertices: vec![],
            indices: vec![],
            transform: DMat4::IDENTITY,
            material: Material {
                base_color: self.color,
                metallic: 0.,
                double_sided: true,
                ..Default::default()
            },
        };
        let mut triangle = |a: DVec3, b: DVec3, c: DVec3| {
            let cross = (b - a).cross(c - a);
            let Some(n) = cross.try_normalize() else {
                return;
            };
            let first = p.vertices.len() as u32;
            for v in [a, b, c] {
                p.vertices.push(Vertex {
                    position: v.as_vec3().to_array(),
                    normal: n.as_vec3().to_array(),
                    uv: [0.; 2],
                    uv1: [0.; 2],
                    color: [1.; 4],
                    tangent: [1., 0., 0., 1.],
                    feature_id: 0,
                });
            }
            p.indices.extend([first, first + 1, first + 2]);
        };
        match self.shape {
            Shape::Box { dimensions } => {
                let d = DVec3::from_array(dimensions) * 0.5;
                let v = [
                    DVec3::new(-d.x, -d.y, -d.z),
                    DVec3::new(d.x, -d.y, -d.z),
                    DVec3::new(d.x, d.y, -d.z),
                    DVec3::new(-d.x, d.y, -d.z),
                    DVec3::new(-d.x, -d.y, d.z),
                    DVec3::new(d.x, -d.y, d.z),
                    DVec3::new(d.x, d.y, d.z),
                    DVec3::new(-d.x, d.y, d.z),
                ];
                for [a, b, c, d] in [
                    [0, 3, 2, 1],
                    [4, 5, 6, 7],
                    [0, 1, 5, 4],
                    [1, 2, 6, 5],
                    [2, 3, 7, 6],
                    [3, 0, 4, 7],
                ] {
                    triangle(v[a], v[b], v[c]);
                    triangle(v[a], v[c], v[d]);
                }
            }
            Shape::Plane {
                dimensions,
                normal,
                distance,
            } => {
                let [a, b, c, d] = crate::plane::corners(dimensions, normal, distance);
                triangle(a, b, c);
                triangle(a, c, d);
            }
            Shape::Ellipsoid {
                radii,
                inner_radii,
                minimum_clock,
                maximum_clock,
                minimum_cone,
                maximum_cone,
                stacks,
                slices,
            } => {
                let stacks = stacks.unwrap_or(64);
                let slices = slices.unwrap_or(64);
                let r_outer = DVec3::from_array(radii);
                let in_r = inner_radii.map(DVec3::from_array);

                let min_clock = minimum_clock.unwrap_or(0.0);
                let max_clock = maximum_clock.unwrap_or(std::f64::consts::TAU);
                let min_cone = minimum_cone.unwrap_or(0.0);
                let max_cone = maximum_cone.unwrap_or(std::f64::consts::PI);

                let clock_span = max_clock - min_clock;
                let cone_span = max_cone - min_cone;
                let is_closed_clock = (clock_span - std::f64::consts::TAU).abs() <= 1e-4;

                let eval_pt = |r: DVec3, u: f64, v: f64| {
                    let sin_v = if v.sin().abs() < 1e-14 { 0. } else { v.sin() };
                    DVec3::new(sin_v * u.cos(), sin_v * u.sin(), v.cos()) * r
                };

                // 1. Outer shell
                for j in 0..stacks {
                    let v = min_cone + (j as f64 * cone_span / stacks as f64);
                    let vn = min_cone + ((j + 1) as f64 * cone_span / stacks as f64);
                    for i in 0..slices {
                        let u = min_clock + (i as f64 * clock_span / slices as f64);
                        let un = min_clock + ((i + 1) as f64 * clock_span / slices as f64);
                        let p00 = eval_pt(r_outer, u, v);
                        let p01 = eval_pt(r_outer, u, vn);
                        let p11 = eval_pt(r_outer, un, vn);
                        let p10 = eval_pt(r_outer, un, v);
                        triangle(p00, p01, p11);
                        triangle(p00, p11, p10);
                    }
                }

                // 2. Inner shell (if inner_radii is present)
                if let Some(r_inner) = in_r {
                    for j in 0..stacks {
                        let v = min_cone + (j as f64 * cone_span / stacks as f64);
                        let vn = min_cone + ((j + 1) as f64 * cone_span / stacks as f64);
                        for i in 0..slices {
                            let u = min_clock + (i as f64 * clock_span / slices as f64);
                            let un = min_clock + ((i + 1) as f64 * clock_span / slices as f64);
                            let p00 = eval_pt(r_inner, u, v);
                            let p01 = eval_pt(r_inner, u, vn);
                            let p11 = eval_pt(r_inner, un, vn);
                            let p10 = eval_pt(r_inner, un, v);
                            // Reverse winding for inward facing normals
                            triangle(p00, p11, p01);
                            triangle(p00, p10, p11);
                        }
                    }
                }

                // 3. Top cone cap (if min_cone > 0)
                if min_cone > 1e-4 {
                    let v = min_cone;
                    for i in 0..slices {
                        let u = min_clock + (i as f64 * clock_span / slices as f64);
                        let un = min_clock + ((i + 1) as f64 * clock_span / slices as f64);
                        let p_out0 = eval_pt(r_outer, u, v);
                        let p_out1 = eval_pt(r_outer, un, v);
                        if let Some(r_inner) = in_r {
                            let p_in0 = eval_pt(r_inner, u, v);
                            let p_in1 = eval_pt(r_inner, un, v);
                            triangle(p_out0, p_out1, p_in1);
                            triangle(p_out0, p_in1, p_in0);
                        } else {
                            let top_center = DVec3::new(0., 0., v.cos() * r_outer.z);
                            triangle(p_out0, p_out1, top_center);
                        }
                    }
                }

                // 4. Bottom cone cap (if max_cone < PI)
                if max_cone < std::f64::consts::PI - 1e-4 {
                    let v = max_cone;
                    for i in 0..slices {
                        let u = min_clock + (i as f64 * clock_span / slices as f64);
                        let un = min_clock + ((i + 1) as f64 * clock_span / slices as f64);
                        let p_out0 = eval_pt(r_outer, u, v);
                        let p_out1 = eval_pt(r_outer, un, v);
                        if let Some(r_inner) = in_r {
                            let p_in0 = eval_pt(r_inner, u, v);
                            let p_in1 = eval_pt(r_inner, un, v);
                            triangle(p_out0, p_in1, p_out1);
                            triangle(p_out0, p_in0, p_in1);
                        } else {
                            let bottom_center = DVec3::new(0., 0., v.cos() * r_outer.z);
                            triangle(p_out0, bottom_center, p_out1);
                        }
                    }
                }

                // 5. Clock cut caps (if not full 360 circle)
                if !is_closed_clock {
                    // Start clock cap at u = min_clock
                    let u_start = min_clock;
                    for j in 0..stacks {
                        let v = min_cone + (j as f64 * cone_span / stacks as f64);
                        let vn = min_cone + ((j + 1) as f64 * cone_span / stacks as f64);
                        let p_out0 = eval_pt(r_outer, u_start, v);
                        let p_out1 = eval_pt(r_outer, u_start, vn);
                        let p_in0 = in_r.map_or(DVec3::ZERO, |r| eval_pt(r, u_start, v));
                        let p_in1 = in_r.map_or(DVec3::ZERO, |r| eval_pt(r, u_start, vn));
                        triangle(p_out0, p_in0, p_in1);
                        triangle(p_out0, p_in1, p_out1);
                    }
                    // End clock cap at u = max_clock
                    let u_end = max_clock;
                    for j in 0..stacks {
                        let v = min_cone + (j as f64 * cone_span / stacks as f64);
                        let vn = min_cone + ((j + 1) as f64 * cone_span / stacks as f64);
                        let p_out0 = eval_pt(r_outer, u_end, v);
                        let p_out1 = eval_pt(r_outer, u_end, vn);
                        let p_in0 = in_r.map_or(DVec3::ZERO, |r| eval_pt(r, u_end, v));
                        let p_in1 = in_r.map_or(DVec3::ZERO, |r| eval_pt(r, u_end, vn));
                        triangle(p_out0, p_in1, p_in0);
                        triangle(p_out0, p_out1, p_in1);
                    }
                }
            }
            Shape::Cylinder {
                length,
                top_radius,
                bottom_radius,
            } => {
                for i in 0..128 {
                    let u = i as f64 * std::f64::consts::TAU / 128.;
                    let v = (i + 1) as f64 * std::f64::consts::TAU / 128.;
                    let a = DVec3::new(
                        u.cos() * bottom_radius,
                        u.sin() * bottom_radius,
                        -length / 2.,
                    );
                    let b = DVec3::new(
                        v.cos() * bottom_radius,
                        v.sin() * bottom_radius,
                        -length / 2.,
                    );
                    let c = DVec3::new(v.cos() * top_radius, v.sin() * top_radius, length / 2.);
                    let d = DVec3::new(u.cos() * top_radius, u.sin() * top_radius, length / 2.);
                    triangle(a, b, c);
                    triangle(a, c, d);
                    triangle(DVec3::new(0., 0., -length / 2.), b, a);
                    triangle(DVec3::new(0., 0., length / 2.), d, c);
                }
            }
            _ => unreachable!("geographic geometry handled above"),
        }
        if let Shape::Ellipsoid {
            radii,
            inner_radii,
            minimum_cone,
            maximum_cone,
            minimum_clock,
            maximum_clock,
            ..
        } = self.shape
        {
            let is_full = inner_radii.is_none()
                && minimum_cone.unwrap_or(0.0) <= 1e-4
                && maximum_cone.unwrap_or(std::f64::consts::PI) >= std::f64::consts::PI - 1e-4
                && (maximum_clock.unwrap_or(std::f64::consts::TAU)
                    - minimum_clock.unwrap_or(0.0)
                    - std::f64::consts::TAU)
                    .abs()
                    <= 1e-4;
            if is_full {
                let r2 = DVec3::from_array(radii).powf(2.);
                for v in &mut p.vertices {
                    v.normal = (DVec3::from_array(v.position.map(f64::from)) / r2)
                        .normalize()
                        .as_vec3()
                        .to_array();
                }
            }
        }
        if self.surface_appearance.is_some() {
            if let Shape::Cylinder {
                length,
                top_radius,
                bottom_radius,
            } = self.shape
            {
                for v in &mut p.vertices {
                    if v.normal[2].abs() < 0.999 {
                        let point = DVec3::from_array(v.position.map(f64::from));
                        let radius = point.x.hypot(point.y);
                        if radius > 1e-6 {
                            v.normal = DVec3::new(
                                length * point.x / radius,
                                length * point.y / radius,
                                bottom_radius - top_radius,
                            )
                            .normalize()
                            .as_vec3()
                            .to_array();
                        }
                    }
                }
            }
        }
        crate::entity_material::apply(self, &mut p)?;
        self.apply_surface_appearance(&mut p);
        Ok(p)
    }
}
impl Shape {
    pub fn screen(&self) -> bool {
        matches!(
            self,
            Self::Point { .. } | Self::Billboard { .. } | Self::Label { .. }
        )
    }
    pub fn standalone(&self) -> bool {
        matches!(
            self,
            Self::WidePolyline { .. }
                | Self::PolylineVolume { .. }
                | Self::Corridor { .. }
                | Self::LineSet { .. }
                | Self::Polyline { .. }
                | Self::Polygon { .. }
                | Self::PolygonSurface { .. }
                | Self::Rectangle { .. }
                | Self::Wall { .. }
        )
    }
    pub fn geographic(&self) -> bool {
        self.standalone() || matches!(self, Self::Ellipse { .. })
    }
}
pub fn decode(bytes: &[u8]) -> Result<Vec<Entity>, String> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("entity input limit".into());
    }
    let entities: Vec<Entity> = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    validate_collection(&entities)?;
    Ok(entities)
}
fn validate_collection(entities: &[Entity]) -> Result<(), String> {
    if entities.len() > 256 {
        return Err("entity count limit (256)".into());
    }
    let mut ids = std::collections::HashSet::new();
    let mut samples = 0;
    let mut gpu = 0;
    for e in entities {
        e.validate()?;
        if !ids.insert(&e.id) {
            return Err("duplicate entity id".into());
        }
        samples += e.samples.len();
        let prim_b = e.primitive()?.gpu_bytes();
        let out_b = crate::outline::primitive(e)?.map_or(0, |p| p.gpu_bytes());
        gpu += prim_b + out_b;
        if samples > 100000 || gpu > 64 * 1024 * 1024 {
            return Err("entity sample/geometry budget".into());
        }
    }
    Ok(())
}
fn utc(v: &str) -> Result<f64, String> {
    let t = chrono::DateTime::parse_from_rfc3339(v).map_err(|e| e.to_string())?;
    if chrono::Timelike::nanosecond(&t) >= 1_000_000_000 {
        return Err("leap-second dates not supported by UTC clock".into());
    }
    Ok(t.timestamp() as f64 + t.timestamp_subsec_nanos() as f64 / 1e9)
}
fn interval(v: &str) -> Result<[f64; 2], String> {
    let (a, b) = v.split_once('/').ok_or("ISO interval required")?;
    let a = utc(a)?;
    let b = utc(b)?;
    if a > b {
        return Err("reversed interval".into());
    }
    Ok([a, b])
}
pub(crate) fn known(v: &Value, keys: &[&str]) -> Result<(), String> {
    let o = v.as_object().ok_or("CZML object required")?;
    for k in o.keys() {
        if !keys.contains(&k.as_str()) {
            return Err(format!("unsupported CZML property: {k}"));
        }
    }
    Ok(())
}
pub(crate) fn number(v: &Value) -> Result<f64, String> {
    if v.is_object() {
        known(v, &["number"])?;
        return number(&v["number"]);
    }
    v.as_f64()
        .filter(|x| x.is_finite())
        .ok_or("finite number required".into())
}
fn partitions(v: &Value) -> Result<u32, String> {
    let n = number(v)?;
    if n.fract() != 0. || !(3.0..=128.).contains(&n) {
        return Err("ellipsoid partitions integer 3..128 required".into());
    }
    Ok(n as u32)
}
fn triple(v: &Value) -> Result<[f64; 3], String> {
    let a = v.as_array().ok_or("3-vector required")?;
    if a.len() != 3 {
        return Err("3-vector required".into());
    }
    Ok([number(&a[0])?, number(&a[1])?, number(&a[2])?])
}
pub(crate) fn boolean(v: &Value) -> Result<bool, String> {
    if v.is_object() {
        known(v, &["boolean"])?;
        return boolean(&v["boolean"]);
    }
    v.as_bool().ok_or("constant boolean required".into())
}
fn graphics_show(g: &Value) -> Result<bool, String> {
    g.get("show")
        .map(boolean)
        .transpose()
        .map(|v| v.unwrap_or(true))
}
fn material_color(g: &Value) -> Result<[f32; 4], String> {
    crate::entity_material::czml(g).map(|(c, _)| c)
}
pub(crate) fn position_list(v: &Value) -> Result<Vec<[f64; 3]>, String> {
    known(v, &["cartographicDegrees", "cartesian"])?;
    let geographic = v.get("cartographicDegrees");
    let cartesian = v.get("cartesian");
    if geographic.is_some() == cartesian.is_some() {
        return Err("exactly one static position list encoding required".into());
    }
    let a = geographic
        .or(cartesian)
        .unwrap()
        .as_array()
        .ok_or("position list array required")?;
    if a.is_empty() || !a.len().is_multiple_of(3) || a.len() > 3072 {
        return Err("bounded lon,lat,height position tuples required".into());
    }
    a.as_chunks::<3>()
        .0
        .iter()
        .map(|p| {
            let p = [number(&p[0])?, number(&p[1])?, number(&p[2])?];
            if geographic.is_some() {
                Ok(p)
            } else {
                let c = Cartographic::from_ecef(DVec3::from_array(p)).map_err(|e| e.to_string())?;
                Ok([c.longitude.to_degrees(), c.latitude.to_degrees(), c.height])
            }
        })
        .collect()
}
fn height_list(v: Option<&Value>) -> Result<Vec<f64>, String> {
    let Some(v) = v else {
        return Ok(vec![]);
    };
    let a = if v.is_object() {
        known(v, &["array"])?;
        v["array"].as_array()
    } else {
        v.as_array()
    }
    .ok_or("height array required")?;
    if a.len() > 1024 {
        return Err("height list limit".into());
    }
    a.iter().map(number).collect()
}
fn geographic_packet(packet: &Value, key: &str) -> Result<Entity, String> {
    let g = &packet[key];
    let mut allowed = vec![
        "material",
        "show",
        "fill",
        "outline",
        "outlineColor",
        "outlineWidth",
    ];
    allowed.extend(match key {
        "polygon" => vec![
            "positions",
            "holes",
            "height",
            "perPositionHeight",
            "extrudedHeight",
            "closeTop",
            "closeBottom",
        ],
        "rectangle" => vec!["coordinates", "height", "extrudedHeight", "rotation"],
        "corridor" => vec![
            "positions",
            "width",
            "height",
            "extrudedHeight",
            "cornerType",
        ],
        "polylineVolume" => vec!["positions", "shape", "cornerType"],
        _ => vec!["positions", "minimumHeights", "maximumHeights"],
    });
    known(g, &allowed)?;
    let shape = match key {
        "polylineVolume" => {
            let positions = position_list(&g["positions"])?;
            let shape_val = g.get("shape").ok_or("polylineVolume shape required")?;
            known(shape_val, &["cartesian2"])?;
            let c2 = shape_val
                .get("cartesian2")
                .ok_or("cartesian2 shape array required")?
                .as_array()
                .ok_or("cartesian2 array required")?;
            if c2.len() < 6 || !c2.len().is_multiple_of(2) {
                return Err("cartesian2 requires at least 3 [x, y] points".into());
            }
            let mut shape_positions = Vec::with_capacity(c2.len() / 2);
            for chunk in c2.as_chunks::<2>().0 {
                shape_positions.push([number(&chunk[0])?, number(&chunk[1])?]);
            }
            let corner_type = if let Some(ct) = g.get("cornerType") {
                match ct.as_str().ok_or("cornerType string required")? {
                    "ROUNDED" => CornerType::Rounded,
                    "MITERED" => CornerType::Mitered,
                    "BEVELED" => CornerType::Beveled,
                    _ => return Err("unsupported cornerType".into()),
                }
            } else {
                CornerType::Rounded
            };
            Shape::PolylineVolume {
                positions,
                shape_positions,
                corner_type,
            }
        }
        "polygon" => {
            let per_position = g
                .get("perPositionHeight")
                .map(boolean)
                .transpose()?
                .unwrap_or(false);
            let height = g.get("height").map(number).transpose()?.unwrap_or(0.);
            let mut p = position_list(&g["positions"])?;
            let mut holes = vec![];
            if let Some(h) = g.get("holes") {
                known(h, &["cartographicDegrees", "cartesian"])?;
                let geo = h.get("cartographicDegrees");
                let cart = h.get("cartesian");
                if geo.is_some() == cart.is_some() {
                    return Err("one hole position encoding required".into());
                }
                let a = geo
                    .or(cart)
                    .unwrap()
                    .as_array()
                    .ok_or("hole arrays required")?;
                if a.len() > 32 {
                    return Err("hole count limit".into());
                }
                for a in a {
                    let list = if geo.is_some() {
                        json!({"cartographicDegrees":a})
                    } else {
                        json!({"cartesian":a})
                    };
                    holes.push(position_list(&list)?);
                }
            }
            if !per_position {
                for v in p.iter_mut().chain(holes.iter_mut().flatten()) {
                    v[2] = height;
                }
            } else if g.get("height").is_some() {
                return Err("height combined with perPositionHeight unsupported".into());
            }
            if per_position || g.get("closeTop").is_some() || g.get("closeBottom").is_some() {
                Shape::PolygonSurface {
                    data: crate::polygon_surface::PolygonSurface {
                        hierarchy: crate::polygon_surface::Hierarchy {
                            positions: p,
                            holes: holes
                                .into_iter()
                                .map(|p| crate::polygon_surface::Hierarchy {
                                    positions: p,
                                    holes: vec![],
                                })
                                .collect(),
                        },
                        texture_coordinates: None,
                        height,
                        per_position_height: per_position,
                        extruded_height: g.get("extrudedHeight").map(number).transpose()?,
                        close_top: g.get("closeTop").map(boolean).transpose()?.unwrap_or(true),
                        close_bottom: g
                            .get("closeBottom")
                            .map(boolean)
                            .transpose()?
                            .unwrap_or(true),
                        coplanar: false,
                        arc_type: "GEODESIC".into(),
                        st_rotation: 0.,
                    },
                }
            } else {
                Shape::Polygon {
                    positions: p,
                    holes,
                    extruded_height: g.get("extrudedHeight").map(number).transpose()?,
                }
            }
        }
        "rectangle" => {
            let c = &g["coordinates"];
            known(c, &["wsenDegrees", "wsen"])?;
            let deg = c.get("wsenDegrees");
            let rad = c.get("wsen");
            if deg.is_some() == rad.is_some() {
                return Err("one rectangle coordinate encoding required".into());
            }
            let a = deg
                .or(rad)
                .unwrap()
                .as_array()
                .ok_or("rectangle coordinates array")?;
            if a.len() != 4 {
                return Err("rectangle w,s,e,n required".into());
            }
            let mut p = [0.; 4];
            for (i, v) in a.iter().enumerate() {
                p[i] = number(v)?;
                if rad.is_some() {
                    p[i] = p[i].to_degrees();
                }
            }
            Shape::Rectangle {
                coordinates: p,
                height: g.get("height").map(number).transpose()?.unwrap_or(0.),
                extruded_height: g.get("extrudedHeight").map(number).transpose()?,
                rotation: g.get("rotation").map(number).transpose()?.unwrap_or(0.),
            }
        }
        "corridor" => Shape::Corridor {
            positions: position_list(&g["positions"])?,
            width: number(&g["width"])?,
            height: g.get("height").map(number).transpose()?.unwrap_or(0.),
            extruded_height: g.get("extrudedHeight").map(number).transpose()?,
            corner_type: g
                .get("cornerType")
                .map(|v| serde_json::from_value(v.clone()).map_err(|e| e.to_string()))
                .transpose()?
                .unwrap_or_default(),
        },
        _ => Shape::Wall {
            positions: position_list(&g["positions"])?,
            minimum_heights: height_list(g.get("minimumHeights"))?,
            maximum_heights: height_list(g.get("maximumHeights"))?,
        },
    };
    Ok(Entity {
        id: packet["id"].as_str().ok_or("entity id required")?.into(),
        show: graphics_show(g)?,
        name: packet
            .get("name")
            .map(|v| v.as_str().ok_or("entity name string"))
            .transpose()?
            .unwrap_or("")
            .into(),
        model_matrix: None,
        orientation: None,
        surface_appearance: None,
        position_degrees: None,
        samples: vec![],
        shape,
        screen: Default::default(),
        fill: g.get("fill").map(boolean).transpose()?.unwrap_or(true),
        outline: g.get("outline").map(boolean).transpose()?.unwrap_or(false),
        outline_color: g
            .get("outlineColor")
            .map(crate::entity_material::rgba)
            .transpose()?
            .unwrap_or(outline_color()),
        outline_width: if let Some(w) = g.get("outlineWidth") {
            let n = number(w)?;
            if !(1.0..=10.0).contains(&n) {
                return Err("outlineWidth between 1 and 10 required".into());
            }
            1.
        } else {
            1.
        },
        color: material_color(g)?,
        material: crate::entity_material::czml(g)?.1,
        availability: packet
            .get("availability")
            .map(|v| interval(v.as_str().ok_or("availability ISO interval")?))
            .transpose()?,
        properties: packet.get("properties").cloned().unwrap_or(Value::Null),
    })
}
fn screen_offset(g: &Value) -> Result<[f64; 2], String> {
    let Some(o) = g.get("pixelOffset") else {
        return Ok([0.; 2]);
    };
    known(o, &["cartesian2"])?;
    let a = o["cartesian2"]
        .as_array()
        .ok_or("pixelOffset cartesian2 required")?;
    if a.len() != 2 {
        return Err("pixelOffset length".into());
    }
    Ok([number(&a[0])?, number(&a[1])?])
}
pub fn czml(bytes: &[u8]) -> Result<(Vec<Entity>, crate::clock::Clock), String> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("CZML input limit".into());
    }
    let packets: Vec<Value> = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if packets.is_empty() || packets.len() > 257 {
        return Err("CZML document and <=256 entities required".into());
    }
    let doc = &packets[0];
    known(doc, &["id", "name", "version", "clock"])?;
    if doc["id"] != "document" || doc["version"] != "1.0" {
        return Err("CZML document version 1.0 required".into());
    }
    let mut clock = crate::clock::Clock::default();
    if let Some(c) = doc.get("clock") {
        known(
            c,
            &["interval", "currentTime", "multiplier", "range", "step"],
        )?;
        let [a, b] = interval(c["interval"].as_str().ok_or("clock interval required")?)?;
        clock.start_time = a;
        clock.stop_time = b;
        clock.current_time = c
            .get("currentTime")
            .map(|v| utc(v.as_str().ok_or("currentTime ISO date")?))
            .transpose()?
            .unwrap_or(a);
        if let Some(m) = c.get("multiplier") {
            clock.multiplier = number(m)?;
        }
        if let Some(r) = c.get("range") {
            clock.range = serde_json::from_value(r.clone()).map_err(|e| e.to_string())?;
        }
        if c.get("step")
            .is_some_and(|s| s != "SYSTEM_CLOCK_MULTIPLIER")
        {
            return Err("only SYSTEM_CLOCK_MULTIPLIER supported".into());
        }
        clock.validate()?;
    }
    let mut entities = vec![];
    for packet in &packets[1..] {
        known(
            packet,
            &[
                "id",
                "name",
                "position",
                "availability",
                "box",
                "ellipsoid",
                "cylinder",
                "properties",
                "polygon",
                "polyline",
                "corridor",
                "rectangle",
                "ellipse",
                "wall",
                "point",
                "billboard",
                "label",
                "polylineVolume",
            ],
        )?;
        let shapes: Vec<_> = [
            "box",
            "ellipsoid",
            "cylinder",
            "polygon",
            "polyline",
            "corridor",
            "rectangle",
            "ellipse",
            "wall",
            "point",
            "billboard",
            "label",
            "polylineVolume",
        ]
        .into_iter()
        .filter(|s| packet.get(*s).is_some())
        .collect();
        if shapes.len() != 1 {
            return Err("one supported geometry required".into());
        }
        let key = shapes[0];
        if key == "polyline" {
            entities.push(crate::wide_polyline::czml(packet)?);
            continue;
        }
        if matches!(
            key,
            "polygon" | "rectangle" | "wall" | "corridor" | "polylineVolume"
        ) {
            if packet.get("position").is_some() {
                return Err("geographic CZML geometry uses graphics.positions/coordinates; extra entity position is unsupported".into());
            }
            entities.push(geographic_packet(packet, key)?);
            continue;
        }
        let pos = packet.get("position").ok_or("position required")?;
        known(
            pos,
            &[
                "cartographicDegrees",
                "cartographicRadians",
                "cartesian",
                "epoch",
                "referenceFrame",
                "interpolationAlgorithm",
                "interpolationDegree",
            ],
        )?;
        if pos.get("referenceFrame").is_some_and(|v| v != "FIXED")
            || pos
                .get("interpolationAlgorithm")
                .is_some_and(|v| v != "LINEAR")
            || pos.get("interpolationDegree").is_some_and(|v| v != 1)
        {
            return Err("only FIXED LINEAR position interpolation supported".into());
        }
        let radians = pos.get("cartographicRadians");
        if radians.is_some() && pos.get("cartographicDegrees").is_some() {
            return Err("one geographic position encoding required".into());
        }
        let geographic = pos.get("cartographicDegrees").or(radians);
        let cartesian = pos.get("cartesian");
        if geographic.is_some() == cartesian.is_some() {
            return Err("exactly one CZML position encoding required".into());
        }
        let a = geographic
            .or(cartesian)
            .unwrap()
            .as_array()
            .ok_or("position array")?;
        let mut samples = vec![];
        let mut position_degrees = None;
        if a.len() == 3 && pos.get("epoch").is_none() {
            let mut p = triple(&Value::Array(a.clone()))?;
            if radians.is_some() {
                p = radian_position(p)?;
            }
            if geographic.is_some() {
                position_degrees = Some(p);
            } else {
                let c = Cartographic::from_ecef(DVec3::from_array(p)).map_err(|e| e.to_string())?;
                position_degrees =
                    Some([c.longitude.to_degrees(), c.latitude.to_degrees(), c.height]);
            }
        } else {
            if a.is_empty() || !a.len().is_multiple_of(4) || a.len() > 40000 {
                return Err("sampled position time,x,y,z tuples required".into());
            }
            let epoch = pos
                .get("epoch")
                .map(|v| utc(v.as_str().ok_or("epoch ISO date")?))
                .transpose()?;
            for v in a.as_chunks::<4>().0 {
                let time = if let Some(epoch) = epoch {
                    epoch + number(&v[0])?
                } else {
                    utc(v[0].as_str().ok_or("sample ISO time or epoch required")?)?
                };
                let mut p = [number(&v[1])?, number(&v[2])?, number(&v[3])?];
                if radians.is_some() {
                    p = radian_position(p)?;
                }
                let p = if geographic.is_some() {
                    Cartographic::degrees(p[0], p[1], p[2])
                        .map_err(|e| e.to_string())?
                        .to_ecef()
                        .to_array()
                } else {
                    p
                };
                samples.push(Sample { time, cartesian: p });
            }
        }
        let g = &packet[key];
        let mut allowed = vec![
            "material",
            "show",
            "fill",
            "outline",
            "outlineColor",
            "outlineWidth",
        ];
        if matches!(key, "point" | "billboard" | "label") {
            allowed = vec![
                "show",
                "color",
                "distanceDisplayCondition",
                "scaleByDistance",
                "translucencyByDistance",
                "pixelOffsetScaleByDistance",
                "disableDepthTestDistance",
            ];
            if key != "point" {
                allowed.extend(["horizontalOrigin", "verticalOrigin"]);
                if key == "billboard" {
                    allowed.push("rotation");
                }
            }
            if key == "label" {
                allowed.retain(|k| *k != "color");
            }
        }
        allowed.extend(match key {
            "point" => vec!["pixelSize", "outlineWidth", "outlineColor"],
            "label" => vec![
                "text",
                "font",
                "fillColor",
                "scale",
                "pixelOffset",
                "style",
                "showBackground",
                "backgroundColor",
                "backgroundPadding",
                "outlineColor",
                "outlineWidth",
            ],
            "billboard" => vec!["image", "width", "height", "scale", "pixelOffset"],
            "box" => vec!["dimensions"],
            "ellipsoid" => vec![
                "radii",
                "innerRadii",
                "minimumClock",
                "maximumClock",
                "minimumCone",
                "maximumCone",
                "stackPartitions",
                "slicePartitions",
            ],
            "ellipse" => vec![
                "semiMajorAxis",
                "semiMinorAxis",
                "rotation",
                "height",
                "extrudedHeight",
            ],
            _ => vec!["length", "topRadius", "bottomRadius"],
        });
        known(g, &allowed)?;
        let shape = match key {
            "label" => {
                let raw = &g["text"];
                let text = if raw.is_object() {
                    known(raw, &["string"])?;
                    raw["string"].as_str()
                } else {
                    raw.as_str()
                }
                .ok_or("label text string required")?
                .to_owned();
                Shape::Label {
                    appearance: crate::label_appearance::czml(g)?,
                    text,
                    font_size: g
                        .get("font")
                        .map(crate::label::css_font)
                        .transpose()?
                        .unwrap_or(30.),
                    scale: g.get("scale").map(number).transpose()?.unwrap_or(1.),
                    pixel_offset: screen_offset(g)?,
                }
            }
            "point" => Shape::Point {
                pixel_size: g.get("pixelSize").map(number).transpose()?.unwrap_or(1.),
                outline_width: g.get("outlineWidth").map(number).transpose()?.unwrap_or(0.),
                outline_color: g
                    .get("outlineColor")
                    .map(crate::entity_material::rgba)
                    .transpose()?
                    .unwrap_or(outline_color()),
            },
            "billboard" => {
                let raw = &g["image"];
                let image = if raw.is_object() {
                    known(raw, &["uri"])?;
                    raw["uri"].as_str()
                } else {
                    raw.as_str()
                }
                .ok_or("billboard image URI required")?
                .to_owned();
                let offset = screen_offset(g)?;
                Shape::Billboard {
                    image,
                    width: g.get("width").map(number).transpose()?.unwrap_or(64.),
                    height: g.get("height").map(number).transpose()?.unwrap_or(64.),
                    scale: g.get("scale").map(number).transpose()?.unwrap_or(1.),
                    pixel_offset: offset,
                }
            }
            "box" => {
                known(&g["dimensions"], &["cartesian"])?;
                Shape::Box {
                    dimensions: triple(&g["dimensions"]["cartesian"])?,
                }
            }
            "ellipsoid" => {
                known(&g["radii"], &["cartesian"])?;
                let in_r = if let Some(ir) = g.get("innerRadii") {
                    known(ir, &["cartesian"])?;
                    Some(triple(&ir["cartesian"])?)
                } else {
                    None
                };
                Shape::Ellipsoid {
                    radii: triple(&g["radii"]["cartesian"])?,
                    inner_radii: in_r,
                    minimum_clock: g.get("minimumClock").map(number).transpose()?,
                    maximum_clock: g.get("maximumClock").map(number).transpose()?,
                    minimum_cone: g.get("minimumCone").map(number).transpose()?,
                    maximum_cone: g.get("maximumCone").map(number).transpose()?,
                    stacks: g.get("stackPartitions").map(partitions).transpose()?,
                    slices: g.get("slicePartitions").map(partitions).transpose()?,
                }
            }
            "ellipse" => {
                if !samples.is_empty() {
                    return Err("sampled ellipse center unsupported".into());
                }
                Shape::Ellipse {
                    vertical_lines: None,
                    height: Some(g.get("height").map(number).transpose()?.unwrap_or(0.)),
                    semi_major: number(&g["semiMajorAxis"])?,
                    semi_minor: number(&g["semiMinorAxis"])?,
                    rotation: g.get("rotation").map(number).transpose()?.unwrap_or(0.),
                    extruded_height: g.get("extrudedHeight").map(number).transpose()?,
                }
            }
            _ => Shape::Cylinder {
                length: number(&g["length"])?,
                top_radius: number(&g["topRadius"])?,
                bottom_radius: number(&g["bottomRadius"])?,
            },
        };
        let screen = shape.screen();
        let rgba = if screen {
            g.get(if key == "label" { "fillColor" } else { "color" })
                .map(crate::entity_material::rgba)
                .transpose()?
                .unwrap_or([1.; 4])
        } else {
            material_color(g)?
        };
        let orientation = if let Some(o) = packet.get("orientation") {
            if let Some(uq) = o.get("unitQuaternion") {
                let a = uq.as_array().ok_or("unitQuaternion array required")?;
                if a.len() == 4 {
                    Some([
                        number(&a[0])?,
                        number(&a[1])?,
                        number(&a[2])?,
                        number(&a[3])?,
                    ])
                } else {
                    None
                }
            } else if let Some(a) = o.as_array() {
                if a.len() == 4 {
                    Some([
                        number(&a[0])?,
                        number(&a[1])?,
                        number(&a[2])?,
                        number(&a[3])?,
                    ])
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        let e = Entity {
            id: packet["id"].as_str().ok_or("entity id required")?.into(),
            show: graphics_show(g)?,
            name: packet
                .get("name")
                .map(|v| v.as_str().ok_or("entity name string"))
                .transpose()?
                .unwrap_or("")
                .into(),
            model_matrix: None,
            orientation,
            surface_appearance: None,
            position_degrees,
            samples,
            shape,
            screen: if screen {
                crate::screen_options::czml(g)?
            } else {
                Default::default()
            },
            fill: g.get("fill").map(boolean).transpose()?.unwrap_or(true),
            outline: g.get("outline").map(boolean).transpose()?.unwrap_or(false),
            outline_color: g
                .get("outlineColor")
                .map(crate::entity_material::rgba)
                .transpose()?
                .unwrap_or(outline_color()),
            outline_width: if screen {
                1.
            } else if let Some(w) = g.get("outlineWidth") {
                let n = number(w)?;
                if !(1.0..=10.0).contains(&n) {
                    return Err("outlineWidth between 1 and 10 required".into());
                }
                1.
            } else {
                1.
            },
            color: rgba,
            material: if screen {
                None
            } else {
                crate::entity_material::czml(g)?.1
            },
            availability: packet
                .get("availability")
                .map(|v| interval(v.as_str().ok_or("availability ISO interval")?))
                .transpose()?,
            properties: packet.get("properties").cloned().unwrap_or(Value::Null),
        };
        entities.push(e);
    }
    validate_collection(&entities)?;
    if doc.get("clock").is_none() {
        let ranges: Vec<_> = entities.iter().filter_map(|e| e.availability).collect();
        if !ranges.is_empty() {
            clock.start_time = ranges.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min);
            clock.stop_time = ranges
                .iter()
                .map(|v| v[1])
                .fold(f64::NEG_INFINITY, f64::max);
            clock.current_time = clock.start_time;
            clock.multiplier = (clock.stop_time - clock.start_time) / 120.;
            clock.validate()?;
        }
    }
    Ok((entities, clock))
}
fn radian_position(p: [f64; 3]) -> Result<[f64; 3], String> {
    if p.iter().any(|v| !v.is_finite())
        || p[0].abs() > 10000.
        || p[1].abs() > 10000.
        || p[2].abs() > 1e8
    {
        return Err("bounded cartographic radians required".into());
    }
    let (sl, cl) = p[0].sin_cos();
    let (sf, cf) = p[1].sin_cos();
    let e2 = (1. / 298.257223563) * (2. - 1. / 298.257223563);
    let n = 6378137. / (1. - e2 * sf * sf).sqrt();
    let c = Cartographic::from_ecef(DVec3::new(
        (n + p[2]) * cf * cl,
        (n + p[2]) * cf * sl,
        (n * (1. - e2) + p[2]) * sf,
    ))
    .map_err(|e| e.to_string())?;
    Ok([c.longitude.to_degrees(), c.latitude.to_degrees(), c.height])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn motion_geometry_and_validation() {
        let e = Entity {
            id: "x".into(),
            show: true,
            name: "".into(),
            model_matrix: None,
            orientation: None,
            surface_appearance: None,
            position_degrees: None,
            samples: vec![
                Sample {
                    time: 0.,
                    cartesian: [6378137., 0., 100.],
                },
                Sample {
                    time: 10.,
                    cartesian: [6378137., 100., 100.],
                },
            ],
            fill: true,
            outline: false,
            outline_color: outline_color(),
            outline_width: 1.,
            screen: Default::default(),
            shape: Shape::Box {
                dimensions: [10., 20., 30.],
            },
            color: color(),
            material: None,
            availability: None,
            properties: Value::Null,
        };
        assert_eq!(e.position(5.).unwrap().y, 50.);
        assert!(e.position(-1.).is_none());
        let p = e.primitive().unwrap();
        assert_eq!(p.indices.len(), 36);
        assert!(p
            .vertices
            .iter()
            .all(|v| DVec3::from_array(v.normal.map(f64::from)).length() > 0.99));
        let mut invalid = e;
        invalid.samples[1].time = 0.;
        assert!(invalid.primitive().is_err());
    }
    #[test]
    fn czml_subset_and_rejection() {
        let v = json!([{"id":"document","version":"1.0","clock":{"interval":"2026-01-01T00:00:00Z/2026-01-01T00:01:00Z"}},{"id":"box","position":{"epoch":"2026-01-01T00:00:00Z","cartographicDegrees":[0,179,0,1000,60,-179,0,1000]},"box":{"dimensions":{"cartesian":[10,10,10]}}}]);
        let (e, c) = czml(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(e[0].position(c.current_time + 30.).unwrap().x < -6e6);
        let mut bad = v;
        bad[1]["label"] = json!({"text":"a second geometry"});
        assert!(czml(&serde_json::to_vec(&bad).unwrap())
            .unwrap_err()
            .contains("one supported geometry"));
    }
    #[test]
    fn invalid_alpha_and_leap_seconds() {
        assert!(utc("2016-12-31T23:59:60Z").is_err());
        assert!(decode(br#"[{"id":"x","positionDegrees":[0,0,0],"shape":{"type":"box","dimensions":[1,1,1]},"color":[1,1,1,1.5]}]"#).is_err());
    }
}
