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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeightReference {
    #[default]
    None,
    ClampToGround,
    RelativeToGround,
}

impl<'de> Deserialize<'de> for HeightReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct HeightReferenceVisitor;
        impl<'de> serde::de::Visitor<'de> for HeightReferenceVisitor {
            type Value = HeightReference;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a HeightReference integer (0, 1, 2) or string")
            }
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    0 => Ok(HeightReference::None),
                    1 => Ok(HeightReference::ClampToGround),
                    2 => Ok(HeightReference::RelativeToGround),
                    _ => Err(E::custom(format!("invalid heightReference int {v}"))),
                }
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(v as i64)
            }
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_i64(v as i64)
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.to_ascii_uppercase().replace('-', "_").as_str() {
                    "NONE" => Ok(HeightReference::None),
                    "CLAMP_TO_GROUND" | "CLAMPTOGROUND" => Ok(HeightReference::ClampToGround),
                    "RELATIVE_TO_GROUND" | "RELATIVETOGROUND" => {
                        Ok(HeightReference::RelativeToGround)
                    }
                    _ => Err(E::custom(format!("invalid heightReference string {v}"))),
                }
            }
        }
        deserializer.deserialize_any(HeightReferenceVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Shape {
    Corridor {
        #[serde(rename = "positionsDegrees", alias = "positions")]
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
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "sizeInMeters"
        )]
        size_in_meters: Option<bool>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "imageSubRegion"
        )]
        image_sub_region: Option<[f64; 4]>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "heightReference"
        )]
        height_reference: Option<HeightReference>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        font: Option<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "showBackground"
        )]
        show_background: Option<bool>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "backgroundColor"
        )]
        background_color: Option<[f32; 4]>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "outlineColor"
        )]
        outline_color: Option<[f32; 4]>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "outlineWidth"
        )]
        outline_width: Option<f32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style: Option<crate::label_appearance::LabelStyle>,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "fillColor")]
        fill_color: Option<[f32; 4]>,
    },
    Box {
        dimensions: [f64; 3],
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "heightReference"
        )]
        height_reference: Option<HeightReference>,
    },
    Frustum {
        fov: f64,
        #[serde(rename = "aspectRatio")]
        aspect_ratio: f64,
        near: f64,
        far: f64,
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
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "heightReference"
        )]
        height_reference: Option<HeightReference>,
    },
    Cylinder {
        length: f64,
        #[serde(rename = "topRadius")]
        top_radius: f64,
        #[serde(rename = "bottomRadius")]
        bottom_radius: f64,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "heightReference"
        )]
        height_reference: Option<HeightReference>,
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
        #[serde(rename = "positionsDegrees", alias = "positions")]
        positions: Vec<[f64; 3]>,
        #[serde(default, alias = "holesDegrees")]
        holes: Vec<Vec<[f64; 3]>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "heightReference"
        )]
        height_reference: Option<HeightReference>,
        #[serde(default, rename = "extrudedHeight")]
        extruded_height: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "extrudedHeightReference"
        )]
        extruded_height_reference: Option<HeightReference>,
    },
    Rectangle {
        #[serde(rename = "coordinatesDegrees", alias = "coordinates")]
        coordinates: [f64; 4],
        #[serde(default)]
        height: f64,
        #[serde(default, rename = "extrudedHeight")]
        extruded_height: Option<f64>,
        #[serde(default)]
        rotation: f64,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "stRotation"
        )]
        st_rotation: Option<f64>,
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
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "stRotation"
        )]
        st_rotation: Option<f64>,
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
        #[serde(rename = "positionsDegrees", alias = "positions")]
        positions: Vec<[f64; 3]>,
        #[serde(rename = "shapePositions", alias = "shape")]
        shape_positions: Vec<[f64; 2]>,
        #[serde(default, rename = "cornerType")]
        corner_type: CornerType,
    },
    Model {
        uri: String,
        #[serde(default = "unit_scale")]
        scale: f64,
        #[serde(default, rename = "minimumPixelSize")]
        minimum_pixel_size: f64,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "maximumScale"
        )]
        maximum_scale: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "runAnimations"
        )]
        run_animations: Option<bool>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "nodeTransformations"
        )]
        node_transformations: Option<serde_json::Value>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "articulations"
        )]
        articulations: Option<serde_json::Value>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "colorBlendMode"
        )]
        color_blend_mode: Option<String>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "colorBlendAmount"
        )]
        color_blend_amount: Option<f64>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "silhouetteColor"
        )]
        silhouette_color: Option<[f32; 4]>,
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            rename = "silhouetteSize"
        )]
        silhouette_size: Option<f64>,
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
    #[serde(default, rename = "zIndex", skip_serializing_if = "is_zero_i32")]
    pub z_index: i32,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "distanceDisplayCondition"
    )]
    pub distance_display_condition: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "viewFrom")]
    pub view_from: Option<[f64; 3]>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "trackingReferenceFrame"
    )]
    pub tracking_reference_frame: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "heightReference"
    )]
    pub height_reference: Option<HeightReference>,
    #[serde(default)]
    pub properties: Value,
}
fn is_zero_i32(v: &i32) -> bool {
    *v == 0
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
        if self.z_index.abs() > 1000 {
            return Err("zIndex must be between -1000 and 1000".into());
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
        if let Some(ddc) = self.distance_display_condition {
            if !ddc[0].is_finite() || !ddc[1].is_finite() || ddc[0] < 0. || ddc[0] > ddc[1] {
                return Err(
                    "valid distanceDisplayCondition [near, far] with near <= far required".into(),
                );
            }
        }
        if let Some(trf) = &self.tracking_reference_frame {
            if !matches!(trf.as_str(), "AUTODETECT" | "INERTIAL" | "VELOCITY" | "ENU") {
                return Err(
                    "trackingReferenceFrame must be AUTODETECT, INERTIAL, VELOCITY or ENU".into(),
                );
            }
        }
        let positive = |x: f64| x.is_finite() && (0.0..=1e6).contains(&x) && x > 0.;
        let valid = match self.shape {
            Shape::Box { dimensions, .. } => dimensions.into_iter().all(positive),
            Shape::Frustum {
                fov,
                aspect_ratio,
                near,
                far,
            } => {
                positive(fov)
                    && positive(aspect_ratio)
                    && positive(near)
                    && positive(far)
                    && far > near
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
                ..
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
                ..
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
            Shape::Model {
                scale,
                minimum_pixel_size,
                maximum_scale,
                color_blend_amount,
                silhouette_size,
                silhouette_color,
                ..
            } => {
                scale.is_finite()
                    && scale > 0.
                    && minimum_pixel_size.is_finite()
                    && minimum_pixel_size >= 0.
                    && maximum_scale.is_none_or(|m| m.is_finite() && m > 0.)
                    && color_blend_amount.is_none_or(|a| a.is_finite() && (0.0..=1.0).contains(&a))
                    && silhouette_size.is_none_or(|s| s.is_finite() && (0.0..=100.0).contains(&s))
                    && silhouette_color
                        .is_none_or(|c| c.iter().all(|x| x.is_finite() && (0.0..=1.0).contains(x)))
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
            Shape::Box { dimensions, .. } => {
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
            Shape::Model { scale, .. } => {
                let d = DVec3::new(8.0, 14.0, 6.0) * (scale.max(0.1) * 0.5);
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
            Shape::Frustum {
                fov,
                aspect_ratio,
                near,
                far,
            } => {
                let half_fov = fov * 0.5;
                let tan_fov = half_fov.tan();
                let y_near = near * tan_fov;
                let x_near = y_near * aspect_ratio;
                let y_far = far * tan_fov;
                let x_far = y_far * aspect_ratio;
                let v = [
                    DVec3::new(-x_near, -y_near, -near),
                    DVec3::new(x_near, -y_near, -near),
                    DVec3::new(x_near, y_near, -near),
                    DVec3::new(-x_near, y_near, -near),
                    DVec3::new(-x_far, -y_far, -far),
                    DVec3::new(x_far, -y_far, -far),
                    DVec3::new(x_far, y_far, -far),
                    DVec3::new(-x_far, y_far, -far),
                ];
                for [a, b, c, d] in [
                    [0, 1, 2, 3],
                    [5, 4, 7, 6],
                    [0, 4, 5, 1],
                    [1, 5, 6, 2],
                    [2, 6, 7, 3],
                    [3, 7, 4, 0],
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
                ..
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
                ..
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
                ..
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
        if samples > 100000 || gpu > 128 * 1024 * 1024 {
            return Err("entity sample/geometry budget".into());
        }
    }
    Ok(())
}
fn utc(v: &str) -> Result<f64, String> {
    let v_norm = if v.len() == 4 && v.chars().all(|c| c.is_ascii_digit()) {
        format!("{v}-01-01T00:00:00Z")
    } else if v.len() == 10 && v.chars().filter(|c| *c == '-').count() == 2 {
        format!("{v}T00:00:00Z")
    } else {
        v.to_string()
    };
    let t = chrono::DateTime::parse_from_rfc3339(&v_norm).map_err(|e| e.to_string())?;
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
    let v = if let Some(arr) = v.as_array() {
        arr.first().ok_or("position intervals array not empty")?
    } else {
        v
    };
    known(v, &["cartographicDegrees", "cartesian", "interval"])?;
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
        "zIndex",
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
        "rectangle" => vec![
            "coordinates",
            "height",
            "extrudedHeight",
            "rotation",
            "stRotation",
        ],
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
                let h = if let Some(arr) = h.as_array() {
                    arr.first().ok_or("hole intervals array not empty")?
                } else {
                    h
                };
                known(h, &["cartographicDegrees", "cartesian", "interval"])?;
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
                    height: g.get("height").map(number).transpose()?,
                    height_reference: None,
                    extruded_height: g.get("extrudedHeight").map(number).transpose()?,
                    extruded_height_reference: None,
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
                st_rotation: g.get("stRotation").map(number).transpose()?,
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
    let view_from = if let Some(vf) = packet.get("viewFrom") {
        if vf.is_object() && vf.get("cartesian").is_some() {
            Some(triple(&vf["cartesian"])?)
        } else {
            Some(triple(vf)?)
        }
    } else {
        None
    };
    let z_index = g
        .get("zIndex")
        .map(number)
        .transpose()?
        .map(|v| v as i32)
        .unwrap_or(0);
    Ok(Entity {
        id: packet["id"].as_str().ok_or("entity id required")?.into(),
        show: graphics_show(g)?,
        name: packet
            .get("name")
            .map(|v| v.as_str().ok_or("entity name string"))
            .transpose()?
            .unwrap_or("")
            .into(),
        z_index,
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
        distance_display_condition: g
            .get("distanceDisplayCondition")
            .map(|v| crate::screen_options::array(v, "distanceDisplayCondition"))
            .transpose()?,
        view_from,
        tracking_reference_frame: packet
            .get("trackingReferenceFrame")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        height_reference: g
            .get("heightReference")
            .and_then(|v| serde_json::from_value::<HeightReference>(v.clone()).ok()),
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
fn dereference_czml_packets(packets: &mut [Value]) -> Result<(), String> {
    let mut lookup = std::collections::HashMap::new();
    for p in packets.iter() {
        if let Some(id) = p.get("id").and_then(|v| v.as_str()) {
            lookup.insert(id.to_string(), p.clone());
        }
    }

    fn extract_cartographic_from_prop(prop: &Value) -> Result<[f64; 3], String> {
        let prop = if let Some(arr) = prop.as_array() {
            arr.first()
                .ok_or("empty reference property interval array")?
        } else {
            prop
        };
        if let Some(deg) = prop.get("cartographicDegrees") {
            let deg = if let Some(arr) = deg
                .as_array()
                .and_then(|a| a.first())
                .and_then(|f| f.get("cartographicDegrees"))
            {
                arr.as_array().ok_or("nested degrees array")?
            } else {
                deg.as_array().ok_or("degrees array")?
            };
            if deg.len() == 3 {
                return Ok([number(&deg[0])?, number(&deg[1])?, number(&deg[2])?]);
            } else if deg.len() >= 4 && deg.len().is_multiple_of(4) {
                return Ok([number(&deg[1])?, number(&deg[2])?, number(&deg[3])?]);
            }
        }
        if let Some(cart) = prop.get("cartesian") {
            let cart = if let Some(arr) = cart
                .as_array()
                .and_then(|a| a.first())
                .and_then(|f| f.get("cartesian"))
            {
                arr.as_array().ok_or("nested cartesian array")?
            } else {
                cart.as_array().ok_or("cartesian array")?
            };
            if cart.len() == 3 {
                let p = [number(&cart[0])?, number(&cart[1])?, number(&cart[2])?];
                let c = Cartographic::from_ecef(DVec3::from_array(p)).map_err(|e| e.to_string())?;
                return Ok([c.longitude.to_degrees(), c.latitude.to_degrees(), c.height]);
            } else if cart.len() >= 4 && cart.len().is_multiple_of(4) {
                let p = [number(&cart[1])?, number(&cart[2])?, number(&cart[3])?];
                let c = Cartographic::from_ecef(DVec3::from_array(p)).map_err(|e| e.to_string())?;
                return Ok([c.longitude.to_degrees(), c.latitude.to_degrees(), c.height]);
            }
        }
        Err("unsupported position reference format".into())
    }

    fn resolve_node(
        v: &mut Value,
        lookup: &std::collections::HashMap<String, Value>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 16 {
            return Err("circular CZML reference detected".into());
        }
        if let Value::Object(map) = v {
            if let Some(ref_val) = map.get("reference").and_then(|r| r.as_str()) {
                let (target_id, prop_path) = ref_val
                    .split_once('#')
                    .ok_or_else(|| format!("invalid CZML reference syntax '{ref_val}'"))?;
                let target = lookup
                    .get(target_id)
                    .ok_or_else(|| format!("CZML reference target '{target_id}' not found"))?;
                let mut current = target;
                for part in prop_path.split('.') {
                    current = current.get(part).ok_or_else(|| {
                        format!("CZML reference property '{part}' in '{ref_val}' not found")
                    })?;
                }
                let mut resolved = current.clone();
                resolve_node(&mut resolved, lookup, depth + 1)?;
                *v = resolved;
                return Ok(());
            } else if let Some(refs) = map.get("references").and_then(|r| r.as_array()) {
                if !refs.is_empty() && refs.iter().all(|r| r.is_string()) {
                    let mut coords = Vec::new();
                    for r in refs {
                        let ref_str = r.as_str().unwrap();
                        let (target_id, prop_path) = ref_str
                            .split_once('#')
                            .ok_or_else(|| format!("invalid CZML reference syntax '{ref_str}'"))?;
                        let target = lookup.get(target_id).ok_or_else(|| {
                            format!("CZML reference target '{target_id}' not found")
                        })?;
                        let mut current = target;
                        for part in prop_path.split('.') {
                            current = current.get(part).ok_or_else(|| {
                                format!("CZML reference property '{part}' in '{ref_str}' not found")
                            })?;
                        }
                        let pt = extract_cartographic_from_prop(current)?;
                        coords.extend_from_slice(&pt);
                    }
                    *v = serde_json::json!({ "cartographicDegrees": coords });
                    return Ok(());
                } else if !refs.is_empty() && refs.iter().all(|r| r.is_array()) {
                    let mut hole_coords = Vec::new();
                    for hole in refs {
                        let hole_arr = hole.as_array().unwrap();
                        let mut coords = Vec::new();
                        for r in hole_arr {
                            let ref_str = r.as_str().ok_or("reference string in hole required")?;
                            let (target_id, prop_path) =
                                ref_str.split_once('#').ok_or_else(|| {
                                    format!("invalid CZML reference syntax '{ref_str}'")
                                })?;
                            let target = lookup.get(target_id).ok_or_else(|| {
                                format!("CZML reference target '{target_id}' not found")
                            })?;
                            let mut current = target;
                            for part in prop_path.split('.') {
                                current = current.get(part).ok_or_else(|| {
                                    format!(
                                        "CZML reference property '{part}' in '{ref_str}' not found"
                                    )
                                })?;
                            }
                            let pt = extract_cartographic_from_prop(current)?;
                            coords.extend_from_slice(&pt);
                        }
                        hole_coords.push(coords);
                    }
                    *v = serde_json::json!({ "cartographicDegrees": hole_coords });
                    return Ok(());
                }
            }
            for val in map.values_mut() {
                resolve_node(val, lookup, depth)?;
            }
        } else if let Value::Array(arr) = v {
            for val in arr.iter_mut() {
                resolve_node(val, lookup, depth)?;
            }
        }
        Ok(())
    }

    for p in packets.iter_mut() {
        resolve_node(p, &lookup, 0)?;
    }
    Ok(())
}
fn decode_screen_or_model_packet(
    packet: &Value,
    key: &str,
    entity_id: &str,
) -> Result<Entity, String> {
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
    if pos
        .get("referenceFrame")
        .is_some_and(|v| v != "FIXED" && v != "INERTIAL")
        || pos
            .get("interpolationAlgorithm")
            .is_some_and(|v| v != "LINEAR" && v != "LAGRANGE")
        || pos.get("interpolationDegree").is_some_and(|v| {
            number(v)
                .map(|n| !(1.0..=10.0).contains(&n))
                .unwrap_or(true)
        })
    {
        return Err("only FIXED/INERTIAL LINEAR/LAGRANGE position interpolation supported".into());
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
            position_degrees = Some([c.longitude.to_degrees(), c.latitude.to_degrees(), c.height]);
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
        "zIndex",
        "heightReference",
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
                allowed.extend(["rotation", "eyeOffset"]);
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
        "billboard" => vec![
            "image",
            "width",
            "height",
            "scale",
            "pixelOffset",
            "sizeInMeters",
            "imageSubRegion",
            "heightReference",
        ],
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
            "stRotation",
            "height",
            "extrudedHeight",
        ],
        "model" => vec![
            "gltf",
            "scale",
            "minimumPixelSize",
            "maximumScale",
            "runAnimations",
            "nodeTransformations",
            "articulations",
            "colorBlendMode",
            "colorBlendAmount",
            "silhouetteColor",
            "silhouetteSize",
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
                font: None,
                show_background: None,
                background_color: None,
                outline_color: None,
                outline_width: None,
                style: None,
                fill_color: None,
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
            let sub_region = if let Some(sub) = g.get("imageSubRegion") {
                if let Some(arr) = sub.as_array() {
                    if arr.len() == 4 {
                        Some([
                            number(&arr[0])?,
                            number(&arr[1])?,
                            number(&arr[2])?,
                            number(&arr[3])?,
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
            let hr = if let Some(hr_val) = g.get("heightReference") {
                Some(
                    serde_json::from_value::<HeightReference>(hr_val.clone())
                        .map_err(|e| e.to_string())?,
                )
            } else {
                None
            };
            Shape::Billboard {
                image,
                width: g.get("width").map(number).transpose()?.unwrap_or(64.),
                height: g.get("height").map(number).transpose()?.unwrap_or(64.),
                scale: g.get("scale").map(number).transpose()?.unwrap_or(1.),
                pixel_offset: offset,
                size_in_meters: g.get("sizeInMeters").map(boolean).transpose()?,
                image_sub_region: sub_region,
                height_reference: hr,
            }
        }
        "box" => {
            known(&g["dimensions"], &["cartesian"])?;
            Shape::Box {
                dimensions: triple(&g["dimensions"]["cartesian"])?,
                height_reference: None,
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
                height_reference: None,
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
                st_rotation: g.get("stRotation").map(number).transpose()?,
                extruded_height: g.get("extrudedHeight").map(number).transpose()?,
            }
        }
        "model" => {
            let raw = &g["gltf"];
            let uri = if raw.is_object() {
                known(raw, &["uri"])?;
                raw["uri"].as_str()
            } else {
                raw.as_str()
            }
            .ok_or("model gltf URI required")?
            .to_owned();
            Shape::Model {
                uri,
                scale: g.get("scale").map(number).transpose()?.unwrap_or(1.),
                minimum_pixel_size: g
                    .get("minimumPixelSize")
                    .map(number)
                    .transpose()?
                    .unwrap_or(0.),
                maximum_scale: g.get("maximumScale").map(number).transpose()?,
                run_animations: g.get("runAnimations").map(boolean).transpose()?,
                node_transformations: g.get("nodeTransformations").cloned(),
                articulations: g.get("articulations").cloned(),
                color_blend_mode: g
                    .get("colorBlendMode")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                color_blend_amount: g.get("colorBlendAmount").map(number).transpose()?,
                silhouette_color: g
                    .get("silhouetteColor")
                    .map(crate::entity_material::rgba)
                    .transpose()?,
                silhouette_size: g.get("silhouetteSize").map(number).transpose()?,
            }
        }
        _ => Shape::Cylinder {
            length: number(&g["length"])?,
            top_radius: number(&g["topRadius"])?,
            bottom_radius: number(&g["bottomRadius"])?,
            height_reference: None,
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
    let view_from = if let Some(vf) = packet.get("viewFrom") {
        if vf.is_object() && vf.get("cartesian").is_some() {
            Some(triple(&vf["cartesian"])?)
        } else {
            Some(triple(vf)?)
        }
    } else {
        None
    };
    let z_index = g
        .get("zIndex")
        .map(number)
        .transpose()?
        .map(|v| v as i32)
        .unwrap_or(0);
    Ok(Entity {
        id: entity_id.into(),
        show: graphics_show(g)?,
        name: packet
            .get("name")
            .map(|v| v.as_str().ok_or("entity name string"))
            .transpose()?
            .unwrap_or("")
            .into(),
        z_index,
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
        distance_display_condition: g
            .get("distanceDisplayCondition")
            .map(|v| crate::screen_options::array(v, "distanceDisplayCondition"))
            .transpose()?,
        view_from,
        tracking_reference_frame: packet
            .get("trackingReferenceFrame")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        height_reference: g
            .get("heightReference")
            .and_then(|v| serde_json::from_value::<HeightReference>(v.clone()).ok()),
        properties: packet.get("properties").cloned().unwrap_or(Value::Null),
    })
}
pub fn czml(bytes: &[u8]) -> Result<(Vec<Entity>, crate::clock::Clock), String> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("CZML input limit".into());
    }
    let mut packets: Vec<Value> = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if packets.is_empty() || packets.len() > 257 {
        return Err("CZML document and <=256 entities required".into());
    }
    dereference_czml_packets(&mut packets)?;
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
                "description",
                "parent",
                "position",
                "viewFrom",
                "trackingReferenceFrame",
                "availability",
                "box",
                "ellipsoid",
                "cylinder",
                "properties",
                "polygon",
                "polyline",
                "path",
                "corridor",
                "rectangle",
                "ellipse",
                "wall",
                "point",
                "billboard",
                "label",
                "polylineVolume",
                "model",
            ],
        )?;
        let shapes: Vec<_> = [
            "box",
            "ellipsoid",
            "cylinder",
            "polygon",
            "polyline",
            "path",
            "corridor",
            "rectangle",
            "ellipse",
            "wall",
            "point",
            "billboard",
            "label",
            "polylineVolume",
            "model",
        ]
        .into_iter()
        .filter(|s| packet.get(*s).is_some())
        .collect();
        if shapes.is_empty() {
            if packet.get("properties").is_some() || packet.get("position").is_some() {
                continue;
            }
            return Err("one supported geometry required".into());
        }
        let id_str = packet["id"].as_str().ok_or("entity id required")?;
        if shapes.len() == 1 {
            let key = shapes[0];
            if key == "polyline" {
                let mut entity = crate::wide_polyline::czml(packet)?;
                if let Some(zi) = packet["polyline"].get("zIndex") {
                    entity.z_index = number(zi)? as i32;
                }
                entities.push(entity);
            } else if key == "path" {
                entities.push(crate::wide_polyline::path_czml(packet)?);
            } else if matches!(
                key,
                "polygon" | "rectangle" | "wall" | "corridor" | "polylineVolume"
            ) {
                if packet.get("position").is_some() {
                    return Err("geographic CZML geometry uses graphics.positions/coordinates; extra entity position is unsupported".into());
                }
                entities.push(geographic_packet(packet, key)?);
            } else {
                entities.push(decode_screen_or_model_packet(packet, key, id_str)?);
            }
        } else if packet.get("billboard").is_some()
            && packet.get("path").is_some()
            && shapes.len() == 2
        {
            entities.push(decode_screen_or_model_packet(packet, "billboard", id_str)?);
            entities.push(crate::wide_polyline::path_czml(packet)?);
        } else if packet.get("model").is_some() && packet.get("path").is_some() && shapes.len() == 2
        {
            entities.push(decode_screen_or_model_packet(packet, "model", id_str)?);
            entities.push(crate::wide_polyline::path_czml(packet)?);
        } else if packet.get("billboard").is_some()
            && packet.get("label").is_some()
            && packet.get("path").is_some()
            && shapes.len() == 3
        {
            entities.push(decode_screen_or_model_packet(packet, "billboard", id_str)?);
            entities.push(decode_screen_or_model_packet(
                packet,
                "label",
                &format!("{id_str}-label"),
            )?);
            entities.push(crate::wide_polyline::path_czml(packet)?);
        } else if packet.get("billboard").is_some()
            && packet.get("label").is_some()
            && shapes.len() == 2
        {
            entities.push(decode_screen_or_model_packet(packet, "billboard", id_str)?);
            entities.push(decode_screen_or_model_packet(
                packet,
                "label",
                &format!("{id_str}-label"),
            )?);
        } else if packet.get("polygon").is_some()
            && packet.get("label").is_some()
            && shapes.len() == 2
        {
            entities.push(geographic_packet(packet, "polygon")?);
            entities.push(decode_screen_or_model_packet(
                packet,
                "label",
                &format!("{id_str}-label"),
            )?);
        } else {
            return Err("one supported geometry required".into());
        }
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
            z_index: 0,
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
                height_reference: None,
            },
            color: color(),
            material: None,
            availability: None,
            distance_display_condition: None,
            view_from: None,
            tracking_reference_frame: None,
            height_reference: None,
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
    #[test]
    fn batch30_rectangle_zindex_volumes_all_four_cases() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Rectangle entities
        let rect_data = std::fs::read(base.join("rectangle.json")).unwrap();
        let rect_base = url::Url::from_file_path(base.join("rectangle.json")).unwrap();
        let rect_hydrated = crate::entity_resources::hydrate(&rect_data, &rect_base, |u, _| {
            std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
        })
        .unwrap();
        let rect_entities = decode(&rect_hydrated).unwrap();
        assert_eq!(rect_entities.len(), 3);
        for e in &rect_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
            if e.outline {
                let out = crate::outline::primitive(e).unwrap().unwrap();
                assert!(!out.vertices.is_empty());
                assert!(!out.indices.is_empty());
            }
        }

        // 2. Z-indexing geometry
        let zindex_data = std::fs::read(base.join("z-indexing-geometry.json")).unwrap();
        let zindex_base = url::Url::from_file_path(base.join("z-indexing-geometry.json")).unwrap();
        let zindex_hydrated =
            crate::entity_resources::hydrate(&zindex_data, &zindex_base, |u, _| {
                std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
            })
            .unwrap();
        let zindex_entities = decode(&zindex_hydrated).unwrap();
        assert_eq!(zindex_entities.len(), 7);
        assert_eq!(zindex_entities[0].z_index, 1);
        assert_eq!(zindex_entities[1].z_index, 2);
        assert_eq!(zindex_entities[2].z_index, 3);
        assert_eq!(zindex_entities[3].z_index, 3);
        assert_eq!(zindex_entities[4].z_index, 2);
        assert_eq!(zindex_entities[5].z_index, 1);
        assert_eq!(zindex_entities[6].z_index, 2);
        for e in &zindex_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
        }

        // 3. CZML zIndex
        let czml_data = std::fs::read(base.join("czml-zindex.czml")).unwrap();
        let (czml_entities, _) = czml(&czml_data).unwrap();
        assert_eq!(czml_entities.len(), 4);
        assert_eq!(czml_entities[0].z_index, 3);
        assert_eq!(czml_entities[1].z_index, 3);
        assert_eq!(czml_entities[2].z_index, 1);
        assert_eq!(czml_entities[3].z_index, 2);
        for e in &czml_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
        }

        // 4. Volumes Dev primitives
        let vol_data = std::fs::read(base.join("volumes-dev.primitive.json")).unwrap();
        let vol_entities = crate::primitive_instances::decode(&vol_data).unwrap();
        assert_eq!(vol_entities.len(), 2);
        assert_eq!(vol_entities[0].id, "primitive-0");
        assert_eq!(vol_entities[1].id, "primitive-1");
        assert!(matches!(
            vol_entities[0].material,
            Some(crate::entity_material::SurfaceMaterial::RimLighting { .. })
        ));
        assert!(matches!(
            vol_entities[1].material,
            Some(crate::entity_material::SurfaceMaterial::Stripe { .. })
        ));
        for e in &vol_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
        }
    }
    #[test]
    fn batch31_geometry_offset_czml_properties_all_four_cases() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Geometry offset attribute dev
        let offset_rect_data =
            std::fs::read(base.join("geometry-offset-attribute-dev.primitive.json")).unwrap();
        let offset_rect_entities = crate::primitive_instances::decode(&offset_rect_data).unwrap();
        assert_eq!(offset_rect_entities.len(), 6);
        for e in &offset_rect_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
            if e.outline {
                let out = crate::outline::primitive(e).unwrap().unwrap();
                assert!(!out.vertices.is_empty());
                assert!(!out.indices.is_empty());
            }
        }
        if let Shape::Rectangle {
            extruded_height, ..
        } = &offset_rect_entities[0].shape
        {
            assert!((extruded_height.unwrap() - 130000.0).abs() < 1e-3);
        } else {
            panic!("expected rectangle");
        }

        // 2. Geometry offset attribute box cylinder ellipsoid dev
        let offset_local_data = std::fs::read(
            base.join("geometry-offset-attribute-box-cylinder-ellipsoid-dev.primitive.json"),
        )
        .unwrap();
        let offset_local_entities = crate::primitive_instances::decode(&offset_local_data).unwrap();
        assert_eq!(offset_local_entities.len(), 6);
        for e in &offset_local_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
            assert!(e.model_matrix.is_some());
        }

        // 3. CZML polygon intervals and availability
        let poly_intervals_data =
            std::fs::read(base.join("czml-polygon-intervals-availability.czml")).unwrap();
        let (poly_entities, poly_clock) = czml(&poly_intervals_data).unwrap();
        assert_eq!(poly_entities.len(), 3);
        assert_eq!(poly_clock.multiplier, 900.0);
        assert_eq!(poly_entities[0].id, "dynamicPolygon");
        assert_eq!(poly_entities[1].id, "california");
        assert_eq!(poly_entities[2].id, "pennsylvania");
        assert!(poly_entities[1].availability.is_some());
        assert!(poly_entities[2].availability.is_some());
        for e in &poly_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
            assert!(!prim.indices.is_empty());
        }

        // 4. CZML custom properties
        let custom_prop_data = std::fs::read(base.join("czml-custom-properties.czml")).unwrap();
        let (prop_entities, prop_clock) = czml(&custom_prop_data).unwrap();
        assert_eq!(prop_entities.len(), 1);
        assert_eq!(prop_entities[0].id, "colorado");
        assert_eq!(prop_clock.multiplier, 500000000.0);
        let prim = prop_entities[0].primitive().unwrap();
        assert!(!prim.vertices.is_empty());
        assert!(!prim.indices.is_empty());
    }

    #[test]
    fn batch33_czml_billboard_label_height_reference_path_reference_properties() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. czml-billboard-and-label
        let raw = std::fs::read(base.join("czml-billboard-and-label.czml")).unwrap();
        let (entities, _) = czml(&raw).unwrap();
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].id, "some-unique-id");
        assert!(matches!(entities[0].shape, Shape::Billboard { .. }));
        assert_eq!(entities[1].id, "some-unique-id-label");
        assert!(matches!(entities[1].shape, Shape::Label { .. }));
        for e in &entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
        }

        // 2. geometry-height-reference
        let raw = std::fs::read(base.join("geometry-height-reference.json")).unwrap();
        let entities = decode(&raw).unwrap();
        assert_eq!(entities.len(), 16);
        for e in &entities {
            if let Shape::Polygon {
                height_reference,
                extruded_height_reference,
                ..
            } = &e.shape
            {
                assert_eq!(*height_reference, Some(HeightReference::RelativeToGround));
                assert_eq!(
                    *extruded_height_reference,
                    Some(HeightReference::ClampToGround)
                );
            } else {
                panic!("expected polygon");
            }
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
        }

        // 3. czml-path
        let raw = std::fs::read(base.join("czml-path.czml")).unwrap();
        let (entities, clock) = czml(&raw).unwrap();
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].id, "path");
        assert!(matches!(entities[0].shape, Shape::Billboard { .. }));
        assert_eq!(entities[0].samples.len(), 1790);
        assert_eq!(entities[1].id, "path-lead-trail");
        assert!(matches!(entities[1].shape, Shape::WidePolyline { .. }));
        assert_eq!(clock.multiplier, 10.0);
        for e in &entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
        }

        // 4. czml-reference-properties
        let raw = std::fs::read(base.join("czml-reference-properties.czml")).unwrap();
        let (entities, _) = czml(&raw).unwrap();
        assert_eq!(entities.len(), 4);
        assert_eq!(entities[0].id, "fillColor-reference");
        assert!(matches!(entities[0].shape, Shape::Billboard { .. }));
        assert_eq!(entities[1].id, "fillColor-reference-label");
        assert!(matches!(entities[1].shape, Shape::Label { .. }));
        assert_eq!(entities[2].id, "polygon");
        assert!(matches!(entities[2].shape, Shape::Polygon { .. }));
        assert_eq!(entities[2].color, [150. / 255., 0., 150. / 255., 1.]);
        assert_eq!(entities[3].id, "polygon-label");
        assert!(matches!(entities[3].shape, Shape::Label { .. }));
        for e in &entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
        }
    }

    #[test]
    fn batch35_czml_model_and_billboard_contracts() {
        let czml_model = serde_json::json!([
            {
                "id": "document",
                "name": "CZML Model",
                "version": "1.0"
            },
            {
                "id": "aircraft model",
                "name": "Cesium Air",
                "position": {
                    "cartographicDegrees": [-77.0, 37.0, 10000.0]
                },
                "model": {
                    "gltf": "../../SampleData/models/CesiumAir/Cesium_Air.glb",
                    "scale": 2.0,
                    "minimumPixelSize": 128.0
                }
            }
        ]);
        let raw = serde_json::to_vec(&czml_model).unwrap();
        let (entities, _) = czml(&raw).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].id, "aircraft model");
        assert_eq!(entities[0].name, "Cesium Air");
        assert_eq!(entities[0].position_degrees, Some([-77.0, 37.0, 10000.0]));
        if let Shape::Model {
            uri,
            scale,
            minimum_pixel_size,
            maximum_scale,
            ..
        } = &entities[0].shape
        {
            assert_eq!(uri, "../../SampleData/models/CesiumAir/Cesium_Air.glb");
            assert_eq!(*scale, 2.0);
            assert_eq!(*minimum_pixel_size, 128.0);
            assert_eq!(*maximum_scale, None);
        } else {
            panic!("expected Shape::Model");
        }
        let prim = entities[0].primitive().unwrap();
        assert_eq!(prim.vertices.len(), 36);
        assert_eq!(prim.indices.len(), 36);

        // Also verify Shape::Billboard with full screen options
        let bb_json = serde_json::json!([
            {
                "id": "bb-1",
                "show": true,
                "name": "test billboard",
                "positionDegrees": [-75.59777, 40.03883, 0.0],
                "shape": {
                    "type": "billboard",
                    "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                    "width": 100.0,
                    "height": 25.0,
                    "scale": 2.0,
                    "pixelOffset": [0.0, -50.0]
                },
                "screen": {
                    "horizontalOrigin": "CENTER",
                    "verticalOrigin": "BOTTOM",
                    "rotation": std::f64::consts::FRAC_PI_4,
                    "eyeOffset": [0.0, 0.0, 0.0]
                },
                "fill": true,
                "outline": false,
                "outlineColor": [0.0, 0.0, 0.0, 1.0],
                "outlineWidth": 1.0,
                "color": [0.0, 1.0, 0.0, 1.0]
            }
        ]);
        let bb_raw = serde_json::to_vec(&bb_json).unwrap();
        let bb_entities = decode(&bb_raw).unwrap();
        assert_eq!(bb_entities.len(), 1);
        let prim = bb_entities[0].primitive().unwrap();
        assert_eq!(prim.vertices.len(), 4);
    }

    #[test]
    fn batch36_labels_models_transformations_articulations() {
        let czml_transformations = serde_json::json!([
            {
                "id": "document",
                "name": "CZML Model",
                "version": "1.0",
                "clock": {
                    "interval": "2015-01-01T12:00:00Z/2015-01-01T12:00:20Z",
                    "currentTime": "2015-01-01T12:00:00Z",
                    "multiplier": 20.0
                }
            },
            {
                "id": "model",
                "position": {
                    "cartographicDegrees": [-77.0, 37.0, 100000.0]
                },
                "viewFrom": {
                    "cartesian": [4.3, 0.1, 2.6]
                },
                "model": {
                    "gltf": "../../SampleData/models/CesiumMan/Cesium_Man.glb",
                    "runAnimations": false,
                    "nodeTransformations": {
                        "Skeleton_arm_joint_L__3_": {
                            "rotation": {
                                "unitQuaternion": [0.0, 0.0, 0.0, 1.0]
                            }
                        }
                    }
                }
            }
        ]);
        let raw = serde_json::to_vec(&czml_transformations).unwrap();
        let (entities, clock) = czml(&raw).unwrap();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].id, "model");
        assert_eq!(entities[0].view_from, Some([4.3, 0.1, 2.6]));
        assert_eq!(clock.multiplier, 20.0);
        if let Shape::Model {
            uri,
            run_animations,
            node_transformations,
            ..
        } = &entities[0].shape
        {
            assert_eq!(uri, "../../SampleData/models/CesiumMan/Cesium_Man.glb");
            assert_eq!(*run_animations, Some(false));
            assert!(node_transformations.is_some());
        } else {
            panic!("expected Shape::Model");
        }

        let czml_articulations = serde_json::json!([
            {
                "id": "document",
                "name": "CZML Model",
                "version": "1.0",
                "clock": {
                    "interval": "2019-06-01T16:00:00Z/2019-06-01T16:10:00Z",
                    "currentTime": "2019-06-01T16:00:00Z",
                    "multiplier": 60.0,
                    "range": "LOOP_STOP",
                    "step": "SYSTEM_CLOCK_MULTIPLIER"
                }
            },
            {
                "id": "test model",
                "name": "Cesium Air",
                "position": {
                    "cartographicDegrees": [-77.0, 37.0, 10000.0]
                },
                "model": {
                    "gltf": "https://cesium.com/public/SandcastleSampleData/launchvehicle.glb",
                    "scale": 2.0,
                    "minimumPixelSize": 128.0,
                    "runAnimations": false,
                    "articulations": {
                        "Fairing Open": {
                            "epoch": "2019-06-01T16:00:00Z",
                            "number": [0, 0, 600, 120]
                        }
                    }
                }
            }
        ]);
        let raw_art = serde_json::to_vec(&czml_articulations).unwrap();
        let (art_entities, art_clock) = czml(&raw_art).unwrap();
        assert_eq!(art_entities.len(), 1);
        assert_eq!(art_entities[0].id, "test model");
        assert_eq!(art_clock.range, crate::clock::Range::LoopStop);
        if let Shape::Model {
            scale,
            minimum_pixel_size,
            articulations,
            ..
        } = &art_entities[0].shape
        {
            assert_eq!(*scale, 2.0);
            assert_eq!(*minimum_pixel_size, 128.0);
            assert!(articulations.is_some());
        } else {
            panic!("expected Shape::Model");
        }

        // Verify Model entity orientation
        let model_entity = Entity {
            id: "oriented-model".into(),
            show: true,
            name: "test".into(),
            position_degrees: Some([-123.0744619, 44.0503706, 5000.0]),
            z_index: 0,
            model_matrix: None,
            orientation: Some([
                0.0,
                0.0,
                std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
            ]),
            surface_appearance: None,
            samples: vec![],
            shape: Shape::Model {
                uri: "model.glb".into(),
                scale: 1.0,
                minimum_pixel_size: 128.0,
                maximum_scale: Some(20000.0),
                run_animations: None,
                node_transformations: None,
                articulations: None,
                color_blend_mode: None,
                color_blend_amount: None,
                silhouette_color: None,
                silhouette_size: None,
            },
            screen: Default::default(),
            fill: true,
            outline: false,
            outline_color: outline_color(),
            outline_width: 1.0,
            color: color(),
            material: None,
            availability: None,
            distance_display_condition: None,
            view_from: None,
            tracking_reference_frame: None,
            height_reference: None,
            properties: Value::Null,
        };
        let m = model_entity.model(0.0).unwrap();
        assert!(m.is_finite());
    }

    #[test]
    fn batch37_models_coloring_dev_labels_billboards_contracts() {
        // 1. Model entity with coloring, blending, and silhouettes
        let coloring_json = serde_json::json!([
            {
                "id": "model-colored",
                "show": true,
                "name": "colored aircraft",
                "positionDegrees": [-123.0744619, 44.0503706, 5000.0],
                "shape": {
                    "type": "model",
                    "uri": "../../SampleData/models/CesiumAir/Cesium_Air.glb",
                    "minimumPixelSize": 128.0,
                    "maximumScale": 20000.0,
                    "colorBlendMode": "Highlight",
                    "colorBlendAmount": 0.5,
                    "silhouetteColor": [1.0, 0.0, 0.0, 1.0],
                    "silhouetteSize": 2.0
                },
                "color": [1.0, 0.0, 0.0, 1.0]
            }
        ]);
        let entities = decode(&serde_json::to_vec(&coloring_json).unwrap()).unwrap();
        assert_eq!(entities.len(), 1);
        if let Shape::Model {
            color_blend_mode,
            color_blend_amount,
            silhouette_color,
            silhouette_size,
            ..
        } = &entities[0].shape
        {
            assert_eq!(color_blend_mode.as_deref(), Some("Highlight"));
            assert_eq!(*color_blend_amount, Some(0.5));
            assert_eq!(*silhouette_color, Some([1.0, 0.0, 0.0, 1.0]));
            assert_eq!(*silhouette_size, Some(2.0));
        } else {
            panic!("expected Shape::Model");
        }
        let prim = entities[0].primitive().unwrap();
        assert_eq!(prim.vertices.len(), 36);

        // 2. Billboard with imageSubRegion and sizeInMeters
        let bb_json = serde_json::json!([
            {
                "id": "bb-marker",
                "show": true,
                "name": "marker",
                "positionDegrees": [-75.59777, 40.03883, 0.0],
                "shape": {
                    "type": "billboard",
                    "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                    "sizeInMeters": true,
                    "imageSubRegion": [49.0, 43.0, 18.0, 18.0]
                }
            }
        ]);
        let bb_entities = decode(&serde_json::to_vec(&bb_json).unwrap()).unwrap();
        assert_eq!(bb_entities.len(), 1);
        if let Shape::Billboard {
            size_in_meters,
            image_sub_region,
            ..
        } = &bb_entities[0].shape
        {
            assert_eq!(*size_in_meters, Some(true));
            assert_eq!(*image_sub_region, Some([49.0, 43.0, 18.0, 18.0]));
        } else {
            panic!("expected Shape::Billboard");
        }
        let bb_prim = bb_entities[0].primitive().unwrap();
        assert_eq!(bb_prim.vertices.len(), 4);

        // 3. Label with styled font, background, and outline
        let label_json = serde_json::json!([
            {
                "id": "label-styled",
                "show": true,
                "name": "styled label",
                "positionDegrees": [-75.1641667, 39.9522222, 0.0],
                "shape": {
                    "type": "label",
                    "text": "Philadelphia",
                    "font": "24px sans-serif",
                    "showBackground": true,
                    "backgroundColor": [0.2, 0.2, 0.2, 0.8],
                    "outlineColor": [0.0, 0.0, 0.0, 1.0],
                    "outlineWidth": 2.0,
                    "style": "FILL_AND_OUTLINE",
                    "fillColor": [0.6, 0.9, 1.0, 1.0]
                }
            }
        ]);
        let label_entities = decode(&serde_json::to_vec(&label_json).unwrap()).unwrap();
        assert_eq!(label_entities.len(), 1);
        let label_prim = label_entities[0].primitive().unwrap();
        assert_eq!(label_prim.vertices.len(), 4);
    }

    #[test]
    fn batch38_models_articulations_node_explorer_headingpitchroll_tracking() {
        // 1. CZML with Satellite/ISS (billboard + label + path) and CesiumDrone (model + path)
        let tracking_czml = serde_json::json!([
            {
                "id": "document",
                "name": "tracking",
                "version": "1.0",
                "clock": {
                    "interval": "2012-03-15T10:00:00Z/2012-03-16T10:00:00Z",
                    "currentTime": "2012-03-15T10:00:00Z",
                    "range": "LOOP_STOP"
                }
            },
            {
                "id": "Satellite/ISS",
                "name": "ISS",
                "availability": "2012-03-15T10:00:00Z/2012-03-16T10:00:00Z",
                "trackingReferenceFrame": "INERTIAL",
                "billboard": {
                    "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                    "scale": 1.5,
                    "show": true
                },
                "label": {
                    "fillColor": { "rgba": [255, 0, 255, 255] },
                    "font": "11pt Lucida Console",
                    "text": "ISS",
                    "show": true
                },
                "path": {
                    "material": { "solidColor": { "color": { "rgba": [255, 0, 255, 255] } } },
                    "width": 1.0,
                    "show": true
                },
                "position": {
                    "referenceFrame": "INERTIAL",
                    "interpolationAlgorithm": "LAGRANGE",
                    "interpolationDegree": 5.0,
                    "epoch": "2012-03-15T10:00:00Z",
                    "cartesian": [
                        0.0, 6791000.0, 0.0, 0.0,
                        60.0, 6790000.0, 1000.0, 500.0
                    ]
                }
            },
            {
                "id": "CesiumDrone",
                "name": "Drone",
                "availability": "2012-03-15T10:00:00Z/2012-03-16T10:00:00Z",
                "viewFrom": { "cartesian": [-50.0, 0.0, 5.0] },
                "model": {
                    "gltf": "../../SampleData/models/CesiumDrone/CesiumDrone.glb",
                    "minimumPixelSize": 64.0,
                    "maximumScale": 20000.0
                },
                "path": {
                    "material": { "solidColor": { "color": { "rgba": [255, 255, 0, 255] } } },
                    "width": 2.0,
                    "show": true
                },
                "position": {
                    "interpolationAlgorithm": "LINEAR",
                    "epoch": "2012-03-15T10:00:00Z",
                    "cartesian": [
                        0.0, 6378137.0, 0.0, 100.0,
                        60.0, 6378137.0, 100.0, 100.0
                    ]
                }
            }
        ]);

        let (entities, clock) = czml(&serde_json::to_vec(&tracking_czml).unwrap()).unwrap();
        // 3 entities from Satellite/ISS (billboard, label, path) + 2 from CesiumDrone (model, path) = 5
        assert_eq!(entities.len(), 5);
        assert_eq!(clock.range, crate::clock::Range::LoopStop);

        let iss_bb = entities.iter().find(|e| e.id == "Satellite/ISS").unwrap();
        assert_eq!(iss_bb.tracking_reference_frame.as_deref(), Some("INERTIAL"));
        assert!(matches!(iss_bb.shape, Shape::Billboard { .. }));

        let iss_label = entities
            .iter()
            .find(|e| e.id == "Satellite/ISS-label")
            .unwrap();
        assert!(matches!(iss_label.shape, Shape::Label { .. }));

        let iss_path = entities
            .iter()
            .find(|e| e.id == "Satellite/ISS-lead-trail")
            .unwrap();
        assert!(matches!(iss_path.shape, Shape::WidePolyline { .. }));

        let drone_model = entities.iter().find(|e| e.id == "CesiumDrone").unwrap();
        assert_eq!(drone_model.view_from, Some([-50.0, 0.0, 5.0]));
        assert!(matches!(drone_model.shape, Shape::Model { .. }));

        let drone_path = entities
            .iter()
            .find(|e| e.id == "CesiumDrone-lead-trail")
            .unwrap();
        assert!(matches!(drone_path.shape, Shape::WidePolyline { .. }));

        // 2. HeadingPitchRoll entity with model and orientation
        let hpr_entity = Entity {
            id: "hpr-plane".into(),
            show: true,
            name: "Cesium Air".into(),
            position_degrees: Some([-123.0744619, 44.0503706, 5000.0]),
            z_index: 0,
            model_matrix: None,
            orientation: Some([0.0, 0.0, 0.3826834323650898, 0.9238795325112867]),
            surface_appearance: None,
            samples: vec![],
            shape: Shape::Model {
                uri: "../../SampleData/models/CesiumAir/Cesium_Air.glb".into(),
                scale: 1.0,
                minimum_pixel_size: 128.0,
                maximum_scale: None,
                run_animations: Some(true),
                node_transformations: None,
                articulations: None,
                color_blend_mode: None,
                color_blend_amount: None,
                silhouette_color: None,
                silhouette_size: None,
            },
            screen: Default::default(),
            fill: true,
            outline: false,
            outline_color: outline_color(),
            outline_width: 1.0,
            color: color(),
            material: None,
            availability: None,
            distance_display_condition: None,
            view_from: None,
            tracking_reference_frame: None,
            height_reference: None,
            properties: Value::Null,
        };
        hpr_entity.validate().unwrap();
        let m = hpr_entity.model(0.0).unwrap();
        assert!(m.is_finite());
    }

    #[test]
    fn batch39_billboard_clamptoground_datasource_ordering_display_conditions() {
        // 1. Billboard with heightReference (CLAMP_TO_GROUND, RELATIVE_TO_GROUND, NONE)
        let bb_json = serde_json::json!([
            {
                "id": "bb-clamped",
                "positionDegrees": [-79.366258, 38.834614, 0.0],
                "shape": {
                    "type": "billboard",
                    "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                    "width": 32.0,
                    "height": 32.0,
                    "scale": 1.0,
                    "pixelOffset": [0.0, 0.0],
                    "heightReference": "CLAMP_TO_GROUND"
                },
                "screen": {
                    "horizontalOrigin": "CENTER",
                    "verticalOrigin": "CENTER"
                },
                "fill": true,
                "outline": false,
                "outlineColor": [0.0, 0.0, 0.0, 1.0],
                "outlineWidth": 1.0,
                "color": [1.0, 1.0, 1.0, 1.0]
            },
            {
                "id": "bb-relative",
                "positionDegrees": [-79.366258, 38.834614, 1000.0],
                "shape": {
                    "type": "billboard",
                    "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                    "width": 64.0,
                    "height": 64.0,
                    "scale": 0.7,
                    "pixelOffset": [0.0, 0.0],
                    "heightReference": "RELATIVE_TO_GROUND"
                },
                "screen": {
                    "horizontalOrigin": "CENTER",
                    "verticalOrigin": "CENTER"
                },
                "fill": true,
                "outline": false,
                "outlineColor": [0.0, 0.0, 0.0, 1.0],
                "outlineWidth": 1.0,
                "color": [1.0, 1.0, 1.0, 1.0]
            }
        ]);
        let bb_entities = decode(&serde_json::to_vec(&bb_json).unwrap()).unwrap();
        assert_eq!(bb_entities.len(), 2);
        if let Shape::Billboard {
            height_reference, ..
        } = &bb_entities[0].shape
        {
            assert_eq!(*height_reference, Some(HeightReference::ClampToGround));
        } else {
            panic!("expected Shape::Billboard");
        }
        if let Shape::Billboard {
            height_reference, ..
        } = &bb_entities[1].shape
        {
            assert_eq!(*height_reference, Some(HeightReference::RelativeToGround));
        } else {
            panic!("expected Shape::Billboard");
        }

        // 2. CZML Rectangles with zIndex ordering
        let czml_ordering = serde_json::json!([
            { "id": "document", "name": "CZML Rectangles", "version": "1.0" },
            {
                "id": "rect-red",
                "rectangle": {
                    "coordinates": { "wsenDegrees": [-120.0, 40.0, -110.0, 50.0] },
                    "fill": true,
                    "material": { "solidColor": { "color": { "rgba": [255, 0, 0, 255] } } },
                    "zIndex": 0
                }
            },
            {
                "id": "rect-yellow",
                "rectangle": {
                    "coordinates": { "wsenDegrees": [-120.0, 45.0, -110.0, 55.0] },
                    "fill": true,
                    "material": { "solidColor": { "color": { "rgba": [255, 255, 0, 255] } } },
                    "zIndex": 1
                }
            }
        ]);
        let (rects, _) = czml(&serde_json::to_vec(&czml_ordering).unwrap()).unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].z_index, 0);
        assert_eq!(rects[1].z_index, 1);

        // 3. Distance Display Condition on Billboard and Rectangle
        let ddc_json = serde_json::json!([
            {
                "id": "billboard-ddc",
                "positionDegrees": [-77.0, 40.5, 0.0],
                "shape": {
                    "type": "billboard",
                    "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
                    "width": 32.0,
                    "height": 32.0,
                    "scale": 1.0,
                    "pixelOffset": [0.0, 0.0]
                },
                "screen": {
                    "horizontalOrigin": "CENTER",
                    "verticalOrigin": "CENTER",
                    "distanceDisplayCondition": [5500000.0, 20000000.0]
                },
                "fill": true,
                "outline": false,
                "outlineColor": [0.0, 0.0, 0.0, 1.0],
                "outlineWidth": 1.0,
                "color": [1.0, 1.0, 1.0, 1.0]
            },
            {
                "id": "rect-ddc",
                "positionDegrees": null,
                "shape": {
                    "type": "rectangle",
                    "coordinates": [-80.5, 39.7, -75.1, 42.0]
                },
                "distanceDisplayCondition": [0.0, 5500000.0],
                "fill": true,
                "outline": false,
                "outlineColor": [0.0, 0.0, 0.0, 1.0],
                "outlineWidth": 1.0,
                "color": [1.0, 0.0, 0.0, 0.5]
            }
        ]);
        let ddc_entities = decode(&serde_json::to_vec(&ddc_json).unwrap()).unwrap();
        assert_eq!(ddc_entities.len(), 2);
        assert_eq!(
            ddc_entities[0].screen.distance_display_condition,
            Some([5500000.0, 20000000.0])
        );
        assert_eq!(
            ddc_entities[1].distance_display_condition,
            Some([0.0, 5500000.0])
        );
    }

    #[test]
    fn batch40_interpolation_manually_controlled_localtofixedframe_pointprimitives() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Interpolation entities
        let interp_data = std::fs::read(base.join("interpolation.json")).unwrap();
        let interp_entities = decode(&interp_data).unwrap();
        assert_eq!(interp_entities.len(), 11);
        assert!(matches!(interp_entities[9].shape, Shape::Model { .. }));
        assert!(matches!(
            interp_entities[10].shape,
            Shape::WidePolyline { .. }
        ));

        // 2. Manually controlled animation entities
        let manual_data = std::fs::read(base.join("manually-controlled-animation.json")).unwrap();
        let manual_entities = decode(&manual_data).unwrap();
        assert_eq!(manual_entities.len(), 2);
        assert!(matches!(manual_entities[0].shape, Shape::Model { .. }));
        assert!(matches!(manual_entities[1].shape, Shape::Label { .. }));

        // 3. LocalToFixedFrame entities
        let local_data = std::fs::read(base.join("localtofixedframe.json")).unwrap();
        let local_entities = decode(&local_data).unwrap();
        assert_eq!(local_entities.len(), 20);
        let models = local_entities
            .iter()
            .filter(|e| matches!(e.shape, Shape::Model { .. }))
            .count();
        let labels = local_entities
            .iter()
            .filter(|e| matches!(e.shape, Shape::Label { .. }))
            .count();
        assert_eq!(models, 10);
        assert_eq!(labels, 10);

        // 4. PointPrimitives Dev entities
        let point_data = std::fs::read(base.join("pointprimitives-dev.json")).unwrap();
        let point_entities = decode(&point_data).unwrap();
        assert_eq!(point_entities.len(), 1);
        assert!(matches!(point_entities[0].shape, Shape::Point { .. }));

        let point_variants_raw =
            std::fs::read(base.join("pointprimitives-dev.variants.json")).unwrap();
        let point_variants = crate::entity_variants::decode(&point_variants_raw).unwrap();
        assert_eq!(point_variants.variants.len(), 8);
        assert_eq!(
            point_variants.variants[4].entities[0]
                .screen
                .scale_by_distance,
            Some([150.0, 2.0, 15000000.0, 0.5])
        );
        assert_eq!(
            point_variants.variants[5].entities[0]
                .screen
                .translucency_by_distance,
            Some([150.0, 1.0, 15000000.0, 0.2])
        );
    }

    #[test]
    fn batch41_cartographic_limit_clamp_entities_billboards_instancing_geometry_appearances() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Cartographic limit rectangle entities
        let limit_data = std::fs::read(base.join("cartographic-limit-rectangle.json")).unwrap();
        let limit_entities = decode(&limit_data).unwrap();
        assert_eq!(limit_entities.len(), 10);
        for (i, entity) in limit_entities.iter().enumerate() {
            assert!(matches!(entity.shape, Shape::Rectangle { .. }));
            assert!(entity.outline);
            if let Shape::Rectangle { height, .. } = entity.shape {
                assert_eq!(height, (i * 5000) as f64);
            }
        }

        // 2. Clamp entities to ground
        let clamp_data = std::fs::read(base.join("clamp-entities-to-ground.json")).unwrap();
        let clamp_entities = decode(&clamp_data).unwrap();
        assert_eq!(clamp_entities.len(), 2);
        assert_eq!(
            clamp_entities[0].height_reference,
            Some(HeightReference::ClampToGround)
        );
        assert_eq!(
            clamp_entities[1].height_reference,
            Some(HeightReference::ClampToGround)
        );

        let clamp_variants_raw =
            std::fs::read(base.join("clamp-entities-to-ground.variants.json")).unwrap();
        let clamp_variants = crate::entity_variants::decode(&clamp_variants_raw).unwrap();
        assert_eq!(clamp_variants.variants.len(), 8);
        assert!(matches!(
            clamp_variants.variants[1].entities[0].shape,
            Shape::Billboard { .. }
        ));
        assert!(matches!(
            clamp_variants.variants[2].entities[0].shape,
            Shape::Corridor { .. }
        ));
        assert!(matches!(
            clamp_variants.variants[5].entities[0].shape,
            Shape::Rectangle { .. }
        ));
        assert!(matches!(
            clamp_variants.variants[6].entities[0].shape,
            Shape::Model { .. }
        ));

        // 3. Billboards instancing dev
        let instancing_data = std::fs::read(base.join("billboards-instancing-dev.json")).unwrap();
        let instancing_entities = decode(&instancing_data).unwrap();
        assert_eq!(instancing_entities.len(), 100);
        assert!(matches!(
            instancing_entities[0].shape,
            Shape::Billboard { .. }
        ));

        let instancing_variants_raw =
            std::fs::read(base.join("billboards-instancing-dev.variants.json")).unwrap();
        let instancing_variants = crate::entity_variants::decode(&instancing_variants_raw).unwrap();
        assert_eq!(instancing_variants.variants.len(), 8);
        assert_eq!(instancing_variants.variants[4].entities.len(), 25);
        assert_eq!(instancing_variants.variants[5].entities.len(), 4);

        // 4. Geometry and appearances
        let geo_data = std::fs::read(base.join("geometry-and-appearances.json")).unwrap();
        let geo_entities = decode(&geo_data).unwrap();
        assert_eq!(geo_entities.len(), 58);
        let geo_variants_raw =
            std::fs::read(base.join("geometry-and-appearances.variants.json")).unwrap();
        let geo_variants = crate::entity_variants::decode(&geo_variants_raw).unwrap();
        assert_eq!(geo_variants.variants.len(), 4);
        assert_eq!(geo_variants.variants[0].entities.len(), 58);
        assert_eq!(geo_variants.variants[1].entities.len(), 5);
        assert_eq!(geo_variants.variants[2].entities.len(), 9);
        assert_eq!(geo_variants.variants[3].entities.len(), 42);
    }

    #[test]
    fn batch42_materials_bloom_widget_cardboard() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Materials
        let mat_data = std::fs::read(base.join("materials.json")).unwrap();
        let mat_base = url::Url::from_file_path(base.join("materials.json")).unwrap();
        let mat_hydrated = crate::entity_resources::hydrate(&mat_data, &mat_base, |u, _| {
            std::fs::read(u.to_file_path().unwrap()).map_err(|e| e.to_string())
        })
        .unwrap();
        let mat_entities = decode(&mat_hydrated).unwrap();
        assert_eq!(mat_entities.len(), 3);
        assert!(mat_entities[0].show);
        assert!(!mat_entities[1].show);
        assert!(!mat_entities[2].show);
        for e in &mat_entities {
            let prim = e.primitive().unwrap();
            assert!(!prim.vertices.is_empty());
        }
        let mat_variants_raw = std::fs::read(base.join("materials.variants.json")).unwrap();
        let mat_variants = crate::entity_variants::decode(&mat_variants_raw).unwrap();
        assert_eq!(mat_variants.variants.len(), 8);

        // 2. Bloom
        let bloom_data = std::fs::read(base.join("bloom.json")).unwrap();
        let bloom_entities = decode(&bloom_data).unwrap();
        assert_eq!(bloom_entities.len(), 13);
        if let Shape::Model { scale, .. } = bloom_entities[0].shape {
            assert_eq!(scale, 1.0);
        } else {
            panic!("expected model shape");
        }
        let bloom_variants_raw = std::fs::read(base.join("bloom.variants.json")).unwrap();
        let bloom_variants = crate::entity_variants::decode(&bloom_variants_raw).unwrap();
        assert_eq!(bloom_variants.variants.len(), 4);

        // 3. Cesium Widget
        let widget_data = std::fs::read(base.join("cesium-widget.json")).unwrap();
        let widget_entities = decode(&widget_data).unwrap();
        assert_eq!(widget_entities.len(), 1);
        assert!(matches!(widget_entities[0].shape, Shape::Model { .. }));
        let widget_variants_raw = std::fs::read(base.join("cesium-widget.variants.json")).unwrap();
        let widget_variants = crate::entity_variants::decode(&widget_variants_raw).unwrap();
        assert_eq!(widget_variants.variants.len(), 4);

        // 4. Cardboard
        let cardboard_data = std::fs::read(base.join("cardboard.json")).unwrap();
        let cardboard_entities = decode(&cardboard_data).unwrap();
        assert_eq!(cardboard_entities.len(), 13);
        assert_eq!(cardboard_entities[0].availability, Some([0.0, 360.0]));
        assert!(!cardboard_entities[0].samples.is_empty());
        let cardboard_variants_raw = std::fs::read(base.join("cardboard.variants.json")).unwrap();
        let cardboard_variants = crate::entity_variants::decode(&cardboard_variants_raw).unwrap();
        assert_eq!(cardboard_variants.variants.len(), 4);
        assert_eq!(cardboard_variants.variants[1].entities.len(), 1);
        assert_eq!(cardboard_variants.variants[2].entities.len(), 5);
    }

    #[test]
    fn batch43_picking_clamp_model_camera_time_dependent_path() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Picking
        let picking_data = std::fs::read(base.join("picking.json")).unwrap();
        let picking_entities = decode(&picking_data).unwrap();
        assert_eq!(picking_entities.len(), 5);
        assert!(matches!(picking_entities[0].shape, Shape::Label { .. }));
        assert!(matches!(picking_entities[1].shape, Shape::Billboard { .. }));
        assert!(matches!(picking_entities[2].shape, Shape::Polygon { .. }));
        let picking_variants_raw = std::fs::read(base.join("picking.variants.json")).unwrap();
        let picking_variants = crate::entity_variants::decode(&picking_variants_raw).unwrap();
        assert_eq!(picking_variants.variants.len(), 4);

        // 2. Clamp Model to Ground
        let clamp_data = std::fs::read(base.join("clamp-model-to-ground.json")).unwrap();
        let clamp_entities = decode(&clamp_data).unwrap();
        assert_eq!(clamp_entities.len(), 2);
        assert!(matches!(clamp_entities[0].shape, Shape::Model { .. }));
        assert_eq!(
            clamp_entities[0].height_reference,
            Some(super::HeightReference::ClampToGround)
        );
        let clamp_variants_raw =
            std::fs::read(base.join("clamp-model-to-ground.variants.json")).unwrap();
        let clamp_variants = crate::entity_variants::decode(&clamp_variants_raw).unwrap();
        assert_eq!(clamp_variants.variants.len(), 4);
        assert_eq!(
            clamp_variants.variants[2].entities[0].height_reference,
            Some(super::HeightReference::RelativeToGround)
        );

        // 3. Camera
        let camera_data = std::fs::read(base.join("camera.json")).unwrap();
        let camera_entities = decode(&camera_data).unwrap();
        assert_eq!(camera_entities.len(), 3);
        assert!(matches!(camera_entities[0].shape, Shape::Point { .. }));
        assert!(matches!(camera_entities[2].shape, Shape::Rectangle { .. }));
        let camera_variants_raw = std::fs::read(base.join("camera.variants.json")).unwrap();
        let camera_variants = crate::entity_variants::decode(&camera_variants_raw).unwrap();
        assert_eq!(camera_variants.variants.len(), 4);

        // 4. Time Dependent Path Graphics
        let path_data = std::fs::read(base.join("time-dependent-path-graphics.json")).unwrap();
        let path_entities = decode(&path_data).unwrap();
        assert_eq!(path_entities.len(), 2);
        assert_eq!(path_entities[0].availability, Some([0.0, 180.0]));
        assert_eq!(path_entities[0].samples.len(), 4);
        assert!(matches!(path_entities[1].shape, Shape::WidePolyline { .. }));
        let path_variants_raw =
            std::fs::read(base.join("time-dependent-path-graphics.variants.json")).unwrap();
        let path_variants = crate::entity_variants::decode(&path_variants_raw).unwrap();
        assert_eq!(path_variants.variants.len(), 4);
    }

    #[test]
    fn batch44_czml_multipart_relativepaths_timedynamicwheels() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. CZML
        let czml_data = std::fs::read(base.join("czml.json")).unwrap();
        let czml_entities = decode(&czml_data).unwrap();
        assert_eq!(czml_entities.len(), 8);
        assert!(matches!(czml_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(czml_entities[1].shape, Shape::WidePolyline { .. }));
        assert!(matches!(czml_entities[4].shape, Shape::Label { .. }));
        assert!(matches!(czml_entities[5].shape, Shape::Polygon { .. }));
        let czml_variants_raw = std::fs::read(base.join("czml.variants.json")).unwrap();
        let czml_variants = crate::entity_variants::decode(&czml_variants_raw).unwrap();
        assert_eq!(czml_variants.variants.len(), 4);

        // 2. Multi-part CZML
        let multipart_data = std::fs::read(base.join("multi-part-czml.json")).unwrap();
        let multipart_entities = decode(&multipart_data).unwrap();
        assert_eq!(multipart_entities.len(), 3);
        assert!(matches!(multipart_entities[0].shape, Shape::Model { .. }));
        assert!(matches!(multipart_entities[1].shape, Shape::Label { .. }));
        assert!(matches!(
            multipart_entities[2].shape,
            Shape::WidePolyline { .. }
        ));
        let multipart_variants_raw =
            std::fs::read(base.join("multi-part-czml.variants.json")).unwrap();
        let multipart_variants = crate::entity_variants::decode(&multipart_variants_raw).unwrap();
        assert_eq!(multipart_variants.variants.len(), 4);

        // 3. Relative Paths
        let rel_data = std::fs::read(base.join("relative-paths.json")).unwrap();
        let rel_entities = decode(&rel_data).unwrap();
        assert_eq!(rel_entities.len(), 5);
        assert!(matches!(rel_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(rel_entities[4].shape, Shape::WidePolyline { .. }));
        let rel_variants_raw = std::fs::read(base.join("relative-paths.variants.json")).unwrap();
        let rel_variants = crate::entity_variants::decode(&rel_variants_raw).unwrap();
        assert_eq!(rel_variants.variants.len(), 4);

        // 4. Time Dynamic Wheels
        let wheels_data = std::fs::read(base.join("time-dynamic-wheels.json")).unwrap();
        let wheels_entities = decode(&wheels_data).unwrap();
        assert_eq!(wheels_entities.len(), 3);
        assert!(matches!(wheels_entities[0].shape, Shape::Model { .. }));
        assert!(matches!(wheels_entities[1].shape, Shape::Label { .. }));
        assert!(matches!(
            wheels_entities[2].shape,
            Shape::WidePolyline { .. }
        ));
        let wheels_variants_raw =
            std::fs::read(base.join("time-dynamic-wheels.variants.json")).unwrap();
        let wheels_variants = crate::entity_variants::decode(&wheels_variants_raw).unwrap();
        assert_eq!(wheels_variants.variants.len(), 4);
    }

    #[test]
    fn batch45_clustering_geojson_topojson_kml() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Clustering
        let cluster_data = std::fs::read(base.join("clustering.json")).unwrap();
        let cluster_entities = decode(&cluster_data).unwrap();
        assert_eq!(cluster_entities.len(), 5);
        assert!(matches!(cluster_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(cluster_entities[4].shape, Shape::Label { .. }));
        let cluster_variants_raw = std::fs::read(base.join("clustering.variants.json")).unwrap();
        let cluster_variants = crate::entity_variants::decode(&cluster_variants_raw).unwrap();
        assert_eq!(cluster_variants.variants.len(), 4);

        // 2. GeoJSON simplestyle
        let simple_data = std::fs::read(base.join("geojson-simplestyle.json")).unwrap();
        let simple_entities = decode(&simple_data).unwrap();
        assert_eq!(simple_entities.len(), 3);
        assert!(matches!(simple_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(
            simple_entities[1].shape,
            Shape::WidePolyline { .. }
        ));
        assert!(matches!(simple_entities[2].shape, Shape::Polygon { .. }));
        let simple_variants_raw =
            std::fs::read(base.join("geojson-simplestyle.variants.json")).unwrap();
        let simple_variants = crate::entity_variants::decode(&simple_variants_raw).unwrap();
        assert_eq!(simple_variants.variants.len(), 4);

        // 3. GeoJSON and TopoJSON
        let topo_data = std::fs::read(base.join("geojson-and-topojson.json")).unwrap();
        let topo_entities = decode(&topo_data).unwrap();
        assert_eq!(topo_entities.len(), 4);
        assert!(matches!(topo_entities[0].shape, Shape::Polygon { .. }));
        let topo_variants_raw =
            std::fs::read(base.join("geojson-and-topojson.variants.json")).unwrap();
        let topo_variants = crate::entity_variants::decode(&topo_variants_raw).unwrap();
        assert_eq!(topo_variants.variants.len(), 4);

        // 4. KML
        let kml_data = std::fs::read(base.join("kml.json")).unwrap();
        let kml_entities = decode(&kml_data).unwrap();
        assert_eq!(kml_entities.len(), 4);
        assert!(matches!(kml_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(kml_entities[1].shape, Shape::Polygon { .. }));
        assert!(matches!(kml_entities[2].shape, Shape::Point { .. }));
        assert!(matches!(kml_entities[3].shape, Shape::WidePolyline { .. }));
        let kml_variants_raw = std::fs::read(base.join("kml.variants.json")).unwrap();
        let kml_variants = crate::entity_variants::decode(&kml_variants_raw).unwrap();
        assert_eq!(kml_variants.variants.len(), 4);
    }

    #[test]
    fn batch46_gpx_kmltours_exportkml_groundpolyline() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. GPX
        let gpx_data = std::fs::read(base.join("gpx.json")).unwrap();
        let gpx_entities = decode(&gpx_data).unwrap();
        assert_eq!(gpx_entities.len(), 5);
        assert!(matches!(gpx_entities[0].shape, Shape::WidePolyline { .. }));
        assert!(matches!(gpx_entities[1].shape, Shape::Billboard { .. }));
        let gpx_variants_raw = std::fs::read(base.join("gpx.variants.json")).unwrap();
        let gpx_variants = crate::entity_variants::decode(&gpx_variants_raw).unwrap();
        assert_eq!(gpx_variants.variants.len(), 4);

        // 2. KML Tours
        let tours_data = std::fs::read(base.join("kml-tours.json")).unwrap();
        let tours_entities = decode(&tours_data).unwrap();
        assert_eq!(tours_entities.len(), 4);
        assert!(matches!(tours_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(
            tours_entities[1].shape,
            Shape::WidePolyline { .. }
        ));
        let tours_variants_raw = std::fs::read(base.join("kml-tours.variants.json")).unwrap();
        let tours_variants = crate::entity_variants::decode(&tours_variants_raw).unwrap();
        assert_eq!(tours_variants.variants.len(), 4);

        // 3. Export KML
        let export_data = std::fs::read(base.join("export-kml.json")).unwrap();
        let export_entities = decode(&export_data).unwrap();
        assert_eq!(export_entities.len(), 4);
        assert!(matches!(export_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(
            export_entities[1].shape,
            Shape::WidePolyline { .. }
        ));
        assert!(matches!(export_entities[3].shape, Shape::Model { .. }));
        let export_variants_raw = std::fs::read(base.join("export-kml.variants.json")).unwrap();
        let export_variants = crate::entity_variants::decode(&export_variants_raw).unwrap();
        assert_eq!(export_variants.variants.len(), 4);

        // 4. Ground Polyline Material - Dev
        let ground_data = std::fs::read(base.join("ground-polyline-material-dev.json")).unwrap();
        let ground_entities = decode(&ground_data).unwrap();
        assert_eq!(ground_entities.len(), 3);
        assert!(matches!(
            ground_entities[0].shape,
            Shape::WidePolyline { .. }
        ));
        let ground_variants_raw =
            std::fs::read(base.join("ground-polyline-material-dev.variants.json")).unwrap();
        let ground_variants = crate::entity_variants::decode(&ground_variants_raw).unwrap();
        assert_eq!(ground_variants.variants.len(), 4);
    }

    #[test]
    fn batch47_drawing_rotatable_polylines_elevation() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Drawing on Terrain
        let draw_data = std::fs::read(base.join("drawing-on-terrain.json")).unwrap();
        let draw_entities = decode(&draw_data).unwrap();
        assert_eq!(draw_entities.len(), 4);
        assert!(matches!(draw_entities[0].shape, Shape::WidePolyline { .. }));
        assert!(matches!(draw_entities[1].shape, Shape::Polygon { .. }));
        assert!(matches!(draw_entities[2].shape, Shape::Point { .. }));
        let draw_variants_raw =
            std::fs::read(base.join("drawing-on-terrain.variants.json")).unwrap();
        let draw_variants = crate::entity_variants::decode(&draw_variants_raw).unwrap();
        assert_eq!(draw_variants.variants.len(), 4);

        // 2. Rotatable 2D Map
        let rot_data = std::fs::read(base.join("rotatable-2d-map.json")).unwrap();
        let rot_entities = decode(&rot_data).unwrap();
        assert_eq!(rot_entities.len(), 3);
        assert!(matches!(rot_entities[0].shape, Shape::Billboard { .. }));
        assert!(matches!(rot_entities[1].shape, Shape::Rectangle { .. }));
        assert!(matches!(rot_entities[2].shape, Shape::WidePolyline { .. }));
        let rot_variants_raw = std::fs::read(base.join("rotatable-2d-map.variants.json")).unwrap();
        let rot_variants = crate::entity_variants::decode(&rot_variants_raw).unwrap();
        assert_eq!(rot_variants.variants.len(), 4);

        // 3. Polylines on Terrain - Dev
        let poly_data = std::fs::read(base.join("polylines-on-terrain-dev.json")).unwrap();
        let poly_entities = decode(&poly_data).unwrap();
        assert_eq!(poly_entities.len(), 3);
        assert!(matches!(poly_entities[0].shape, Shape::WidePolyline { .. }));
        assert!(matches!(poly_entities[1].shape, Shape::WidePolyline { .. }));
        assert!(matches!(poly_entities[2].shape, Shape::Rectangle { .. }));
        let poly_variants_raw =
            std::fs::read(base.join("polylines-on-terrain-dev.variants.json")).unwrap();
        let poly_variants = crate::entity_variants::decode(&poly_variants_raw).unwrap();
        assert_eq!(poly_variants.variants.len(), 4);

        // 4. Elevation Band Material
        let elev_data = std::fs::read(base.join("elevation-band-material.json")).unwrap();
        let elev_entities = decode(&elev_data).unwrap();
        assert_eq!(elev_entities.len(), 4);
        assert!(matches!(elev_entities[0].shape, Shape::Rectangle { .. }));
        assert!(matches!(elev_entities[3].shape, Shape::Rectangle { .. }));
        let elev_variants_raw =
            std::fs::read(base.join("elevation-band-material.variants.json")).unwrap();
        let elev_variants = crate::entity_variants::decode(&elev_variants_raw).unwrap();
        assert_eq!(elev_variants.variants.len(), 4);
    }

    #[test]
    fn batch48_shadows_atmosphere_fog_globematerials() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Shadows
        let shadow_data = std::fs::read(base.join("shadows.json")).unwrap();
        let shadow_entities = decode(&shadow_data).unwrap();
        assert_eq!(shadow_entities.len(), 8);
        assert!(matches!(shadow_entities[0].shape, Shape::Model { .. }));
        assert!(matches!(shadow_entities[5].shape, Shape::Box { .. }));
        assert!(matches!(shadow_entities[7].shape, Shape::Ellipsoid { .. }));
        let shadow_variants_raw = std::fs::read(base.join("shadows.variants.json")).unwrap();
        let shadow_variants = crate::entity_variants::decode(&shadow_variants_raw).unwrap();
        assert_eq!(shadow_variants.variants.len(), 4);

        // 2. Atmosphere
        let atmos_data = std::fs::read(base.join("atmosphere.json")).unwrap();
        let atmos_entities = decode(&atmos_data).unwrap();
        assert_eq!(atmos_entities.len(), 4);
        assert!(matches!(atmos_entities[0].shape, Shape::Cylinder { .. }));
        assert!(matches!(atmos_entities[1].shape, Shape::Box { .. }));
        assert!(matches!(atmos_entities[3].shape, Shape::Ellipsoid { .. }));
        let atmos_variants_raw = std::fs::read(base.join("atmosphere.variants.json")).unwrap();
        let atmos_variants = crate::entity_variants::decode(&atmos_variants_raw).unwrap();
        assert_eq!(atmos_variants.variants.len(), 4);

        // 3. Fog Post Process
        let fog_data = std::fs::read(base.join("fog-post-process.json")).unwrap();
        let fog_entities = decode(&fog_data).unwrap();
        assert_eq!(fog_entities.len(), 4);
        assert!(matches!(fog_entities[0].shape, Shape::Box { .. }));
        assert!(matches!(fog_entities[1].shape, Shape::Cylinder { .. }));
        assert!(matches!(fog_entities[3].shape, Shape::Ellipsoid { .. }));
        let fog_variants_raw = std::fs::read(base.join("fog-post-process.variants.json")).unwrap();
        let fog_variants = crate::entity_variants::decode(&fog_variants_raw).unwrap();
        assert_eq!(fog_variants.variants.len(), 4);

        // 4. Globe Materials
        let globe_data = std::fs::read(base.join("globe-materials.json")).unwrap();
        let globe_entities = decode(&globe_data).unwrap();
        assert_eq!(globe_entities.len(), 4);
        assert!(matches!(globe_entities[0].shape, Shape::Cylinder { .. }));
        assert!(matches!(globe_entities[1].shape, Shape::Box { .. }));
        assert!(matches!(globe_entities[3].shape, Shape::Rectangle { .. }));
        let globe_variants_raw = std::fs::read(base.join("globe-materials.variants.json")).unwrap();
        let globe_variants = crate::entity_variants::decode(&globe_variants_raw).unwrap();
        assert_eq!(globe_variants.variants.len(), 4);
    }

    #[test]
    fn batch49_lighting_fog_cloudparameters_clouds() {
        let base =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/sandcastle/full");

        // 1. Lighting
        let lighting_data = std::fs::read(base.join("lighting.json")).unwrap();
        let lighting_entities = decode(&lighting_data).unwrap();
        assert_eq!(lighting_entities.len(), 4);
        assert!(matches!(lighting_entities[0].shape, Shape::Model { .. }));
        assert!(matches!(lighting_entities[1].shape, Shape::Cylinder { .. }));
        assert!(matches!(
            lighting_entities[2].shape,
            Shape::Ellipsoid { .. }
        ));
        assert!(matches!(lighting_entities[3].shape, Shape::Box { .. }));
        let lighting_variants_raw = std::fs::read(base.join("lighting.variants.json")).unwrap();
        let lighting_variants = crate::entity_variants::decode(&lighting_variants_raw).unwrap();
        assert_eq!(lighting_variants.variants.len(), 4);

        // 2. Fog
        let fog_data = std::fs::read(base.join("fog.json")).unwrap();
        let fog_entities = decode(&fog_data).unwrap();
        assert_eq!(fog_entities.len(), 4);
        assert!(matches!(fog_entities[0].shape, Shape::Box { .. }));
        assert!(matches!(fog_entities[1].shape, Shape::Cylinder { .. }));
        assert!(matches!(fog_entities[2].shape, Shape::Ellipsoid { .. }));
        assert!(matches!(fog_entities[3].shape, Shape::Rectangle { .. }));
        let fog_variants_raw = std::fs::read(base.join("fog.variants.json")).unwrap();
        let fog_variants = crate::entity_variants::decode(&fog_variants_raw).unwrap();
        assert_eq!(fog_variants.variants.len(), 4);

        // 3. Cloud Parameters
        let cp_data = std::fs::read(base.join("cloud-parameters.json")).unwrap();
        let cp_entities = decode(&cp_data).unwrap();
        assert_eq!(cp_entities.len(), 4);
        assert!(matches!(cp_entities[0].shape, Shape::Ellipsoid { .. }));
        assert!(matches!(cp_entities[3].shape, Shape::Box { .. }));
        let cp_variants_raw = std::fs::read(base.join("cloud-parameters.variants.json")).unwrap();
        let cp_variants = crate::entity_variants::decode(&cp_variants_raw).unwrap();
        assert_eq!(cp_variants.variants.len(), 4);

        // 4. Clouds
        let clouds_data = std::fs::read(base.join("clouds.json")).unwrap();
        let clouds_entities = decode(&clouds_data).unwrap();
        assert_eq!(clouds_entities.len(), 5);
        assert!(matches!(clouds_entities[0].shape, Shape::Ellipsoid { .. }));
        assert!(matches!(clouds_entities[4].shape, Shape::Rectangle { .. }));
        let clouds_variants_raw = std::fs::read(base.join("clouds.variants.json")).unwrap();
        let clouds_variants = crate::entity_variants::decode(&clouds_variants_raw).unwrap();
        assert_eq!(clouds_variants.variants.len(), 4);
    }
}
