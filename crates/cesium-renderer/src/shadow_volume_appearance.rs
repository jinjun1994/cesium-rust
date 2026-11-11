//! ShadowVolumeAppearance matching CesiumJS `Scene/ShadowVolumeAppearance.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{
    Cartesian2, Cartesian3, Cartographic, ComponentDatatype, DeveloperError, Ellipsoid,
    EncodedCartesian3, FixedFrameTransforms, GeoRectangle, GeometryInstanceAttribute, MapProjection,
    EPSILON5,
};

use crate::shader_source::ShaderSource;

/// Appearance target kind for `ShadowVolumeAppearance`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShadowAppearanceSource {
    PerInstanceColor {
        flat: bool,
    },
    Material {
        material_shader_source: String,
        fragment_shader_source: String,
        flat: bool,
    },
}

impl ShadowAppearanceSource {
    pub fn is_flat(&self) -> bool {
        match self {
            Self::PerInstanceColor { flat } => *flat,
            Self::Material { flat, .. } => *flat,
        }
    }

    pub fn is_per_instance_color(&self) -> bool {
        matches!(self, Self::PerInstanceColor { .. })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ShaderDependencies {
    pub requires_ec: bool,
    pub requires_wc: bool,
    pub requires_normal_ec: bool,
    pub requires_texture_coordinates: bool,
    pub normal_ec: bool,
    pub position_to_eye_ec: bool,
    pub tangent_to_eye_matrix: bool,
    pub st: bool,
}

/// Creates shaders and instance attributes for a ClassificationPrimitive to use a given Appearance, as well as for picking.
#[derive(Clone, Debug)]
pub struct ShadowVolumeAppearance {
    pub extents_culling: bool,
    pub planar_extents: bool,
    pub appearance: ShadowAppearanceSource,
    color_shader_dependencies: ShaderDependencies,
    pick_shader_dependencies: ShaderDependencies,
}

impl ShadowVolumeAppearance {
    /// Texture coordinates for ground primitives are computed either using spherical coordinates
    /// for large areas or using distance from planes for small areas.
    pub const MAX_WIDTH_FOR_PLANAR_EXTENTS: f64 = 1.0f64.to_radians();

    pub fn new(
        extents_culling: bool,
        planar_extents: bool,
        appearance: ShadowAppearanceSource,
    ) -> Self {
        let is_flat = appearance.is_flat();
        let is_per_instance_color = appearance.is_per_instance_color();

        let mut color_deps = ShaderDependencies {
            requires_texture_coordinates: extents_culling,
            requires_wc: extents_culling,
            requires_ec: extents_culling || !is_flat,
            ..Default::default()
        };

        let pick_deps = ShaderDependencies {
            requires_texture_coordinates: extents_culling,
            requires_wc: extents_culling,
            requires_ec: extents_culling,
            ..Default::default()
        };

        if is_per_instance_color {
            color_deps.requires_normal_ec = !is_flat;
        } else if let ShadowAppearanceSource::Material {
            ref material_shader_source,
            ref fragment_shader_source,
            ..
        } = appearance
        {
            let combined = format!("{material_shader_source}\n{fragment_shader_source}");
            let normal_ec = combined.contains("materialInput.normalEC")
                || combined.contains("czm_getDefaultMaterial");
            let pos_to_eye = combined.contains("materialInput.positionToEyeEC");
            let tangent = combined.contains("materialInput.tangentToEyeMatrix");
            let st = combined.contains("materialInput.st");

            color_deps.normal_ec = normal_ec;
            color_deps.position_to_eye_ec = pos_to_eye;
            color_deps.tangent_to_eye_matrix = tangent;
            color_deps.st = st;

            if normal_ec {
                color_deps.requires_normal_ec = true;
                color_deps.requires_ec = true;
            }
            if pos_to_eye {
                color_deps.requires_ec = true;
            }
            if tangent {
                color_deps.requires_wc = true;
                color_deps.requires_normal_ec = true;
                color_deps.requires_ec = true;
            }
            if st {
                color_deps.requires_texture_coordinates = true;
                color_deps.requires_wc = true;
                color_deps.requires_ec = true;
            }
        }

        Self {
            extents_culling,
            planar_extents,
            appearance,
            color_shader_dependencies: color_deps,
            pick_shader_dependencies: pick_deps,
        }
    }

    /// Fast approximation of `atan(x)` over `[-1, 1]`.
    pub fn fast_approximate_atan(x: f64) -> f64 {
        x * (-0.1784 * x.abs() - 0.0663 * x * x + 1.0301)
    }

    /// Fast approximation of `atan2(x, y)` matching CesiumJS.
    pub fn fast_approximate_atan2(x: f64, y: f64) -> f64 {
        let t_abs = x.abs();
        let opp_abs = y.abs();
        let adjacent = t_abs.max(opp_abs);
        let opposite = t_abs.min(opp_abs);

        let opposite_over_adjacent = opposite / adjacent;
        let mut t = Self::fast_approximate_atan(opposite_over_adjacent);

        if opp_abs > t_abs {
            t = std::f64::consts::FRAC_PI_2 - t;
        }
        if x < 0.0 {
            t = std::f64::consts::PI - t;
        }
        if y < 0.0 {
            t = -t;
        }
        t
    }

    pub fn lat_long_to_spherical(
        latitude: f64,
        longitude: f64,
        ellipsoid: &Ellipsoid,
    ) -> Cartesian2 {
        let carto = Cartographic::new(longitude, latitude, 0.0);
        let p = ellipsoid
            .cartographic_to_cartesian(carto)
            .unwrap_or(glam::DVec3::ZERO);
        let mag_xy = (p.x * p.x + p.y * p.y).sqrt();

        let sphere_lat = Self::fast_approximate_atan2(mag_xy, p.z);
        let sphere_lon = Self::fast_approximate_atan2(p.x, p.y);

        Cartesian2::new(sphere_lat, sphere_lon)
    }

    pub fn should_use_spherical_coordinates(rectangle: &GeoRectangle) -> bool {
        rectangle.width().max(rectangle.height()) > Self::MAX_WIDTH_FOR_PLANAR_EXTENTS
    }

    fn point_line_distance(point1: Cartesian2, point2: Cartesian2, point: Cartesian2) -> f64 {
        let dist = point2.distance(&point1);
        if dist == 0.0 {
            return 0.0;
        }
        ((point2.y - point1.y) * point.x - (point2.x - point1.x) * point.y + point2.x * point1.y
            - point2.y * point1.x)
            .abs()
            / dist
    }

    fn add_texture_coordinate_rotation_attributes(
        attributes: &mut HashMap<String, GeometryInstanceAttribute>,
        rotation_points: &[f64],
    ) {
        let min_xy = Cartesian2::new(rotation_points[0], rotation_points[1]);
        let max_y = Cartesian2::new(rotation_points[2], rotation_points[3]);
        let max_x = Cartesian2::new(rotation_points[4], rotation_points[5]);

        attributes.insert(
            "uMaxVmax".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                4,
                false,
                vec![max_y.x, max_y.y, max_x.x, max_x.y],
            )
            .unwrap(),
        );

        let inv_x = 1.0 / Self::point_line_distance(min_xy, max_y, max_x);
        let inv_y = 1.0 / Self::point_line_distance(min_xy, max_x, max_y);

        attributes.insert(
            "uvMinAndExtents".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                4,
                false,
                vec![min_xy.x, min_xy.y, inv_x, inv_y],
            )
            .unwrap(),
        );
    }

    fn add_2d_texture_coordinate_attributes(
        rectangle: &GeoRectangle,
        projection: &dyn MapProjection,
        attributes: &mut HashMap<String, GeometryInstanceAttribute>,
    ) {
        let sw = projection
            .project(Cartographic::new(rectangle.west, rectangle.south, 0.0))
            .unwrap_or(glam::DVec3::ZERO);
        let nw = projection
            .project(Cartographic::new(rectangle.west, rectangle.north, 0.0))
            .unwrap_or(glam::DVec3::ZERO);
        let se = projection
            .project(Cartographic::new(rectangle.east, rectangle.south, 0.0))
            .unwrap_or(glam::DVec3::ZERO);

        let (sw_x_h, sw_x_l) = EncodedCartesian3::encode(sw.x);
        let (sw_y_h, sw_y_l) = EncodedCartesian3::encode(sw.y);
        let (nw_y_h, nw_y_l) = EncodedCartesian3::encode(nw.y);
        let (se_x_h, se_x_l) = EncodedCartesian3::encode(se.x);

        attributes.insert(
            "planes2D_HIGH".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                4,
                false,
                vec![sw_x_h, sw_y_h, nw_y_h, se_x_h],
            )
            .unwrap(),
        );
        attributes.insert(
            "planes2D_LOW".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                4,
                false,
                vec![sw_x_l, sw_y_l, nw_y_l, se_x_l],
            )
            .unwrap(),
        );
    }

    /// Computes planar texture coordinate attributes for small area rectangles.
    pub fn get_planar_texture_coordinate_attributes(
        bounding_rectangle: &GeoRectangle,
        texture_coordinate_rotation_points: &[f64],
        ellipsoid: &Ellipsoid,
        projection: &dyn MapProjection,
        height: Option<f64>,
    ) -> Result<HashMap<String, GeometryInstanceAttribute>, DeveloperError> {
        let h = height.unwrap_or(0.0);
        let center = bounding_rectangle.center();
        let center_carto = Cartographic::new(center.longitude, center.latitude, h);
        let center_cartesian = ellipsoid
            .cartographic_to_cartesian(center_carto)
            .unwrap_or(glam::DVec3::ZERO);

        let enu_matrix = FixedFrameTransforms::east_north_up_to_fixed_frame(
            Cartesian3::new(center_cartesian.x, center_cartesian.y, center_cartesian.z),
            Some(*ellipsoid),
        )?;
        let inverse_enu = enu_matrix
            .inverse()
            .map_err(|_| DeveloperError::new("Failed to invert ENU matrix."))?;

        let west = bounding_rectangle.west;
        let east = bounding_rectangle.east;
        let north = bounding_rectangle.north;
        let south = bounding_rectangle.south;
        let lon_center = (west + east) * 0.5;
        let lat_center = (north + south) * 0.5;

        let cartographics = [
            Cartographic::new(west, south, h),
            Cartographic::new(west, north, h),
            Cartographic::new(east, north, h),
            Cartographic::new(east, south, h),
            Cartographic::new(lon_center, south, h),
            Cartographic::new(lon_center, north, h),
            Cartographic::new(west, lat_center, h),
            Cartographic::new(east, lat_center, h),
        ];

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for carto in cartographics {
            let p_world = ellipsoid
                .cartographic_to_cartesian(carto)
                .unwrap_or(glam::DVec3::ZERO);
            let mut p_enu = inverse_enu.multiply_by_point(Cartesian3::new(p_world.x, p_world.y, p_world.z));
            p_enu.z = 0.0;
            min_x = min_x.min(p_enu.x);
            max_x = max_x.max(p_enu.x);
            min_y = min_y.min(p_enu.y);
            max_y = max_y.max(p_enu.y);
        }

        let sw_world = enu_matrix.multiply_by_point(Cartesian3::new(min_x, min_y, 0.0));
        let se_world = enu_matrix.multiply_by_point(Cartesian3::new(max_x, min_y, 0.0));
        let nw_world = enu_matrix.multiply_by_point(Cartesian3::new(min_x, max_y, 0.0));

        let eastward = se_world - sw_world;
        let northward = nw_world - sw_world;

        let mut attributes = HashMap::new();
        Self::add_texture_coordinate_rotation_attributes(
            &mut attributes,
            texture_coordinate_rotation_points,
        );

        let (sw_h_x, sw_l_x) = EncodedCartesian3::encode(sw_world.x);
        let (sw_h_y, sw_l_y) = EncodedCartesian3::encode(sw_world.y);
        let (sw_h_z, sw_l_z) = EncodedCartesian3::encode(sw_world.z);

        attributes.insert(
            "southWest_HIGH".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                3,
                false,
                vec![sw_h_x, sw_h_y, sw_h_z],
            )
            .unwrap(),
        );
        attributes.insert(
            "southWest_LOW".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                3,
                false,
                vec![sw_l_x, sw_l_y, sw_l_z],
            )
            .unwrap(),
        );
        attributes.insert(
            "eastward".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                3,
                false,
                vec![eastward.x, eastward.y, eastward.z],
            )
            .unwrap(),
        );
        attributes.insert(
            "northward".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                3,
                false,
                vec![northward.x, northward.y, northward.z],
            )
            .unwrap(),
        );

        Self::add_2d_texture_coordinate_attributes(bounding_rectangle, projection, &mut attributes);

        Ok(attributes)
    }

    /// Computes spherical extent attributes for large area rectangles.
    pub fn get_spherical_extent_geometry_instance_attributes(
        bounding_rectangle: &GeoRectangle,
        texture_coordinate_rotation_points: &[f64],
        ellipsoid: &Ellipsoid,
        projection: &dyn MapProjection,
    ) -> HashMap<String, GeometryInstanceAttribute> {
        let sw =
            Self::lat_long_to_spherical(bounding_rectangle.south, bounding_rectangle.west, ellipsoid);
        let mut south = sw.x;
        let mut west = sw.y;

        let ne =
            Self::lat_long_to_spherical(bounding_rectangle.north, bounding_rectangle.east, ellipsoid);
        let mut north = ne.x;
        let mut east = ne.y;

        let mut rotation_radians = 0.0;
        if west > east {
            rotation_radians = std::f64::consts::PI - west;
            west = -std::f64::consts::PI;
            east += rotation_radians;
        }

        south -= EPSILON5;
        west -= EPSILON5;
        north += EPSILON5;
        east += EPSILON5;

        let lon_range_inv = 1.0 / (east - west);
        let lat_range_inv = 1.0 / (north - south);

        let mut attributes = HashMap::new();
        attributes.insert(
            "sphericalExtents".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                4,
                false,
                vec![south, west, lat_range_inv, lon_range_inv],
            )
            .unwrap(),
        );
        attributes.insert(
            "longitudeRotation".to_string(),
            GeometryInstanceAttribute::new(
                ComponentDatatype::Float,
                1,
                false,
                vec![rotation_radians],
            )
            .unwrap(),
        );

        Self::add_texture_coordinate_rotation_attributes(
            &mut attributes,
            texture_coordinate_rotation_points,
        );
        Self::add_2d_texture_coordinate_attributes(bounding_rectangle, projection, &mut attributes);

        attributes
    }

    pub fn has_attributes_for_spherical_extents(
        attributes: &HashMap<String, GeometryInstanceAttribute>,
    ) -> bool {
        attributes.contains_key("sphericalExtents")
            && attributes.contains_key("longitudeRotation")
            && attributes.contains_key("planes2D_HIGH")
            && attributes.contains_key("planes2D_LOW")
            && attributes.contains_key("uMaxVmax")
            && attributes.contains_key("uvMinAndExtents")
    }

    pub fn has_attributes_for_texture_coordinate_planes(
        attributes: &HashMap<String, GeometryInstanceAttribute>,
    ) -> bool {
        attributes.contains_key("southWest_HIGH")
            && attributes.contains_key("southWest_LOW")
            && attributes.contains_key("northward")
            && attributes.contains_key("eastward")
            && attributes.contains_key("planes2D_HIGH")
            && attributes.contains_key("planes2D_LOW")
            && attributes.contains_key("uMaxVmax")
            && attributes.contains_key("uvMinAndExtents")
    }

    fn format_extent_define(name: &str, val: f64) -> String {
        let val_str = if val.fract() == 0.0 && !val.to_string().contains('.') {
            format!("{val:.0}")
        } else {
            val.to_string()
        };
        let precision = val_str.len() + 1;
        format!("{name} {val:.precision$}")
    }

    pub fn compute_projection_extent_defines(
        projection: &dyn MapProjection,
    ) -> (String, String, String, String) {
        let east_carto = Cartographic::new(std::f64::consts::PI, 0.0, 0.0);
        let east_proj = projection.project(east_carto).unwrap_or(glam::DVec3::ZERO);
        let (eh, el) = EncodedCartesian3::encode(east_proj.x);

        let west_carto = Cartographic::new(-std::f64::consts::PI, 0.0, 0.0);
        let west_proj = projection.project(west_carto).unwrap_or(glam::DVec3::ZERO);
        let (wh, wl) = EncodedCartesian3::encode(west_proj.x);

        (
            Self::format_extent_define("EAST_MOST_X_HIGH", eh),
            Self::format_extent_define("EAST_MOST_X_LOW", el),
            Self::format_extent_define("WEST_MOST_X_HIGH", wh),
            Self::format_extent_define("WEST_MOST_X_LOW", wl),
        )
    }

    pub fn create_fragment_shader(&self, columbus_view_2d: bool) -> ShaderSource {
        let mut defines = Vec::new();
        if !columbus_view_2d && !self.planar_extents {
            defines.push("SPHERICAL".to_string());
        }
        if self.color_shader_dependencies.requires_ec {
            defines.push("REQUIRES_EC".to_string());
        }
        if self.color_shader_dependencies.requires_wc {
            defines.push("REQUIRES_WC".to_string());
        }
        if self.color_shader_dependencies.requires_texture_coordinates {
            defines.push("TEXTURE_COORDINATES".to_string());
        }
        if self.extents_culling {
            defines.push("CULL_FRAGMENTS".to_string());
        }
        if self.color_shader_dependencies.requires_normal_ec {
            defines.push("NORMAL_EC".to_string());
        }
        if self.appearance.is_per_instance_color() {
            defines.push("PER_INSTANCE_COLOR".to_string());
        }
        if self.color_shader_dependencies.normal_ec {
            defines.push("USES_NORMAL_EC".to_string());
        }
        if self.color_shader_dependencies.position_to_eye_ec {
            defines.push("USES_POSITION_TO_EYE_EC".to_string());
        }
        if self.color_shader_dependencies.tangent_to_eye_matrix {
            defines.push("USES_TANGENT_TO_EYE".to_string());
        }
        if self.color_shader_dependencies.st {
            defines.push("USES_ST".to_string());
        }
        if self.appearance.is_flat() {
            defines.push("FLAT".to_string());
        }

        ShaderSource {
            defines,
            sources: vec!["// ShadowVolumeAppearanceFS\n".to_string()],
            pick_color_qualifier: None,
            include_built_ins: true,
        }
    }

    pub fn create_pick_fragment_shader(&self, columbus_view_2d: bool) -> ShaderSource {
        let mut defines = vec!["PICK".to_string()];
        if !columbus_view_2d && !self.planar_extents {
            defines.push("SPHERICAL".to_string());
        }
        if self.pick_shader_dependencies.requires_ec {
            defines.push("REQUIRES_EC".to_string());
        }
        if self.pick_shader_dependencies.requires_wc {
            defines.push("REQUIRES_WC".to_string());
        }
        if self.pick_shader_dependencies.requires_texture_coordinates {
            defines.push("TEXTURE_COORDINATES".to_string());
        }
        if self.extents_culling {
            defines.push("CULL_FRAGMENTS".to_string());
        }

        ShaderSource {
            defines,
            sources: vec!["// ShadowVolumeAppearanceFS Pick\n".to_string()],
            pick_color_qualifier: Some("in".to_string()),
            include_built_ins: true,
        }
    }

    pub fn create_vertex_shader(
        &self,
        defines: &[String],
        vs_source: &str,
        columbus_view_2d: bool,
        projection: &dyn MapProjection,
    ) -> ShaderSource {
        let mut all_defines = defines.to_vec();

        if columbus_view_2d {
            let (eh, el, wh, wl) = Self::compute_projection_extent_defines(projection);
            all_defines.push(eh);
            all_defines.push(el);
            all_defines.push(wh);
            all_defines.push(wl);
        }

        if self.appearance.is_per_instance_color() {
            all_defines.push("PER_INSTANCE_COLOR".to_string());
        }
        if self.color_shader_dependencies.requires_texture_coordinates {
            all_defines.push("TEXTURE_COORDINATES".to_string());
            if !(self.planar_extents || columbus_view_2d) {
                all_defines.push("SPHERICAL".to_string());
            }
            if columbus_view_2d {
                all_defines.push("COLUMBUS_VIEW_2D".to_string());
            }
        }

        ShaderSource {
            defines: all_defines,
            sources: vec![vs_source.to_string()],
            pick_color_qualifier: None,
            include_built_ins: true,
        }
    }

    pub fn create_pick_vertex_shader(
        &self,
        defines: &[String],
        vs_source: &str,
        columbus_view_2d: bool,
        projection: &dyn MapProjection,
    ) -> ShaderSource {
        let mut all_defines = defines.to_vec();

        if columbus_view_2d {
            let (eh, el, wh, wl) = Self::compute_projection_extent_defines(projection);
            all_defines.push(eh);
            all_defines.push(el);
            all_defines.push(wh);
            all_defines.push(wl);
        }

        if self.pick_shader_dependencies.requires_texture_coordinates {
            all_defines.push("TEXTURE_COORDINATES".to_string());
            if !(self.planar_extents || columbus_view_2d) {
                all_defines.push("SPHERICAL".to_string());
            }
            if columbus_view_2d {
                all_defines.push("COLUMBUS_VIEW_2D".to_string());
            }
        }

        ShaderSource {
            defines: all_defines,
            sources: vec![vs_source.to_string()],
            pick_color_qualifier: None,
            include_built_ins: true,
        }
    }
}
