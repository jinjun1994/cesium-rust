use std::collections::HashMap;

use cesium_math::{
    Cartesian3, Cartographic, Color, ComponentDatatype, DeveloperError, Ellipsoid, GeoRectangle,
    GeometryInstance, GeometryInstanceAttribute, IntersectionTests, MapProjection, Matrix4,
    OrientedBoundingBox, Plane, PlaneIntersection, Ray, RectangleOutlineGeometry,
    RectangleOutlineGeometryOptions, SceneMode, Sphere,
};

use crate::scene_primitive::{Primitive, PrimitiveAppearance, PrimitiveOptions};

/// Options for creating a `TileBoundingRegion`.
#[derive(Clone, Debug)]
pub struct TileBoundingRegionOptions {
    pub rectangle: GeoRectangle,
    pub minimum_height: Option<f64>,
    pub maximum_height: Option<f64>,
    pub ellipsoid: Option<Ellipsoid>,
    pub compute_bounding_volumes: Option<bool>,
}

/// Camera state used in `distance_to_camera`.
#[derive(Clone, Debug)]
pub struct TileBoundingRegionCamera {
    pub position_wc: Cartesian3,
    pub position_cartographic: Cartographic,
}

impl TileBoundingRegionCamera {
    pub fn new(position_wc: Cartesian3, position_cartographic: Cartographic) -> Self {
        Self {
            position_wc,
            position_cartographic,
        }
    }

    pub fn from_cartographic(cartographic: Cartographic, ellipsoid: &Ellipsoid) -> Self {
        let position_wc = to_cartesian3(ellipsoid, cartographic);
        Self {
            position_wc,
            position_cartographic: cartographic,
        }
    }
}

/// Frame state passed to `TileBoundingRegion::distance_to_camera`.
pub struct TileBoundingRegionFrameState<'a> {
    pub camera: TileBoundingRegionCamera,
    pub mode: SceneMode,
    pub map_projection: Option<&'a dyn MapProjection>,
}

/// A tile bounding volume specified as a longitude/latitude/height region.
#[derive(Clone, Debug)]
pub struct TileBoundingRegion {
    pub rectangle: GeoRectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub southwest_corner_cartesian: Cartesian3,
    pub northeast_corner_cartesian: Cartesian3,
    pub west_normal: Cartesian3,
    pub south_normal: Cartesian3,
    pub east_normal: Cartesian3,
    pub north_normal: Cartesian3,
    oriented_bounding_box: Option<OrientedBoundingBox>,
    bounding_sphere: Option<Sphere>,
}

fn to_cartesian3(ellipsoid: &Ellipsoid, c: Cartographic) -> Cartesian3 {
    ellipsoid
        .cartographic_to_cartesian(c)
        .unwrap_or(glam::DVec3::ZERO)
        .into()
}

impl TileBoundingRegion {
    pub fn new(options: TileBoundingRegionOptions) -> Result<Self, DeveloperError> {
        let minimum_height = options.minimum_height.unwrap_or(0.0);
        let maximum_height = options.maximum_height.unwrap_or(0.0);
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let mut region = Self {
            rectangle: options.rectangle,
            minimum_height,
            maximum_height,
            southwest_corner_cartesian: Cartesian3::ZERO,
            northeast_corner_cartesian: Cartesian3::ZERO,
            west_normal: Cartesian3::ZERO,
            south_normal: Cartesian3::ZERO,
            east_normal: Cartesian3::ZERO,
            north_normal: Cartesian3::ZERO,
            oriented_bounding_box: None,
            bounding_sphere: None,
        };

        region.compute_box(&ellipsoid);

        if options.compute_bounding_volumes.unwrap_or(true) {
            region.compute_bounding_volumes(Some(&ellipsoid));
        }

        Ok(region)
    }

    pub fn bounding_volume(&self) -> Option<&OrientedBoundingBox> {
        self.oriented_bounding_box.as_ref()
    }

    pub fn bounding_sphere(&self) -> Option<&Sphere> {
        self.bounding_sphere.as_ref()
    }

    pub fn compute_bounding_volumes(&mut self, ellipsoid: Option<&Ellipsoid>) {
        let ellipsoid_ref = ellipsoid.cloned().unwrap_or(Ellipsoid::WGS84);
        let obb = OrientedBoundingBox::from_rectangle(
            self.rectangle,
            self.minimum_height,
            self.maximum_height,
            ellipsoid_ref,
        )
        .ok();
        self.bounding_sphere = obb
            .as_ref()
            .and_then(|o| Sphere::from_oriented_bounding_box(o).ok());
        self.oriented_bounding_box = obb;
    }

    fn compute_box(&mut self, ellipsoid: &Ellipsoid) {
        let sw_cartographic = Cartographic::new(self.rectangle.west, self.rectangle.south, 0.0);
        let ne_cartographic = Cartographic::new(self.rectangle.east, self.rectangle.north, 0.0);

        self.southwest_corner_cartesian = to_cartesian3(ellipsoid, sw_cartographic);
        self.northeast_corner_cartesian = to_cartesian3(ellipsoid, ne_cartographic);

        let western_midpoint_cartographic = Cartographic::new(
            self.rectangle.west,
            (self.rectangle.south + self.rectangle.north) * 0.5,
            0.0,
        );
        let western_midpoint_cartesian =
            to_cartesian3(ellipsoid, western_midpoint_cartographic);

        let west_normal = western_midpoint_cartesian.cross(&Cartesian3::UNIT_Z);
        self.west_normal = west_normal.normalize();

        let eastern_midpoint_cartographic = Cartographic::new(
            self.rectangle.east,
            (self.rectangle.south + self.rectangle.north) * 0.5,
            0.0,
        );
        let eastern_midpoint_cartesian =
            to_cartesian3(ellipsoid, eastern_midpoint_cartographic);

        let east_normal = Cartesian3::UNIT_Z.cross(&eastern_midpoint_cartesian);
        self.east_normal = east_normal.normalize();

        let mut west_vector = western_midpoint_cartesian - eastern_midpoint_cartesian;
        if west_vector.magnitude() == 0.0 {
            west_vector = self.west_normal;
        }

        let east_west_normal = west_vector.normalize();

        // Compute south normal
        let south = self.rectangle.south;
        let south_surface_normal: Cartesian3 = if south > 0.0 {
            let south_center = Cartographic::new(
                (self.rectangle.west + self.rectangle.east) * 0.5,
                south,
                0.0,
            );
            let south_center_cartesian = to_cartesian3(ellipsoid, south_center);
            if let Ok(ray) = Ray::new(south_center_cartesian.into(), east_west_normal.into()) {
                if let Ok(west_plane) = Plane::from_point_normal(
                    self.southwest_corner_cartesian.into(),
                    self.west_normal.into(),
                ) {
                    if let Some(intersection) = IntersectionTests::ray_plane(&ray, &west_plane) {
                        self.southwest_corner_cartesian = intersection.into();
                    }
                }
            }
            ellipsoid
                .geodetic_surface_normal(south_center_cartesian.into())
                .unwrap_or(glam::DVec3::Z)
                .into()
        } else {
            let se = Cartographic::new(self.rectangle.east, self.rectangle.south, 0.0);
            Ellipsoid::geodetic_surface_normal_cartographic(se).into()
        };

        let south_normal = south_surface_normal.cross(&west_vector);
        self.south_normal = south_normal.normalize();

        // Compute north normal
        let north = self.rectangle.north;
        let north_surface_normal: Cartesian3 = if north < 0.0 {
            let north_center = Cartographic::new(
                (self.rectangle.west + self.rectangle.east) * 0.5,
                north,
                0.0,
            );
            let north_center_cartesian = to_cartesian3(ellipsoid, north_center);
            if let Ok(ray) = Ray::new(north_center_cartesian.into(), (-east_west_normal).into()) {
                if let Ok(east_plane) = Plane::from_point_normal(
                    self.northeast_corner_cartesian.into(),
                    self.east_normal.into(),
                ) {
                    if let Some(intersection) = IntersectionTests::ray_plane(&ray, &east_plane) {
                        self.northeast_corner_cartesian = intersection.into();
                    }
                }
            }
            ellipsoid
                .geodetic_surface_normal(north_center_cartesian.into())
                .unwrap_or(glam::DVec3::Z)
                .into()
        } else {
            let nw = Cartographic::new(self.rectangle.west, self.rectangle.north, 0.0);
            Ellipsoid::geodetic_surface_normal_cartographic(nw).into()
        };

        let north_normal = west_vector.cross(&north_surface_normal);
        self.north_normal = north_normal.normalize();
    }

    /// Gets the distance from the camera to the closest point on the tile, in meters.
    pub fn distance_to_camera(&self, frame_state: &TileBoundingRegionFrameState) -> f64 {
        let region_result = self.distance_to_camera_region(frame_state);
        if frame_state.mode == SceneMode::Scene3D {
            if let Some(obb) = &self.oriented_bounding_box {
                if let Some(dist_sq) =
                    obb.distance_squared_to(frame_state.camera.position_wc.into())
                {
                    return region_result.max(dist_sq.sqrt());
                }
            }
        }
        region_result
    }

    fn distance_to_camera_region(&self, frame_state: &TileBoundingRegionFrameState) -> f64 {
        let camera = &frame_state.camera;
        let camera_cartesian = camera.position_wc;
        let camera_cartographic = camera.position_cartographic;

        let mut result = 0.0;
        if !self.rectangle.contains(camera_cartographic) {
            let mut southwest_corner_cartesian = self.southwest_corner_cartesian;
            let mut northeast_corner_cartesian = self.northeast_corner_cartesian;
            let mut west_normal = self.west_normal;
            let mut south_normal = self.south_normal;
            let mut east_normal = self.east_normal;
            let mut north_normal = self.north_normal;

            if frame_state.mode != SceneMode::Scene3D {
                if let Some(proj) = frame_state.map_projection {
                    let sw_proj = proj
                        .project(Cartographic::new(
                            self.rectangle.west,
                            self.rectangle.south,
                            0.0,
                        ))
                        .unwrap_or(glam::DVec3::ZERO);
                    southwest_corner_cartesian = Cartesian3::new(0.0, sw_proj.x, sw_proj.y);

                    let ne_proj = proj
                        .project(Cartographic::new(
                            self.rectangle.east,
                            self.rectangle.north,
                            0.0,
                        ))
                        .unwrap_or(glam::DVec3::ZERO);
                    northeast_corner_cartesian = Cartesian3::new(0.0, ne_proj.x, ne_proj.y);

                    west_normal = Cartesian3::new(0.0, -1.0, 0.0);
                    east_normal = Cartesian3::UNIT_Y;
                    south_normal = Cartesian3::new(0.0, 0.0, -1.0);
                    north_normal = Cartesian3::UNIT_Z;
                }
            }

            let vector_from_southwest = camera_cartesian - southwest_corner_cartesian;
            let distance_to_west_plane = vector_from_southwest.dot(&west_normal);
            let distance_to_south_plane = vector_from_southwest.dot(&south_normal);

            let vector_from_northeast = camera_cartesian - northeast_corner_cartesian;
            let distance_to_east_plane = vector_from_northeast.dot(&east_normal);
            let distance_to_north_plane = vector_from_northeast.dot(&north_normal);

            if distance_to_west_plane > 0.0 {
                result += distance_to_west_plane * distance_to_west_plane;
            } else if distance_to_east_plane > 0.0 {
                result += distance_to_east_plane * distance_to_east_plane;
            }

            if distance_to_south_plane > 0.0 {
                result += distance_to_south_plane * distance_to_south_plane;
            } else if distance_to_north_plane > 0.0 {
                result += distance_to_north_plane * distance_to_north_plane;
            }
        }

        let (camera_height, min_h, max_h) = if frame_state.mode == SceneMode::Scene3D {
            (
                camera_cartographic.height,
                self.minimum_height,
                self.maximum_height,
            )
        } else {
            (camera_cartesian.x, 0.0, 0.0)
        };

        if camera_height > max_h {
            let distance_above = camera_height - max_h;
            result += distance_above * distance_above;
        } else if camera_height < min_h {
            let distance_below = min_h - camera_height;
            result += distance_below * distance_below;
        }

        result.sqrt()
    }

    /// Determines which side of a plane this box is located.
    pub fn intersect_plane(&self, plane: Plane) -> Result<PlaneIntersection, DeveloperError> {
        self.oriented_bounding_box
            .map(|obb| obb.intersect_plane(plane))
            .ok_or_else(|| DeveloperError::new("Oriented bounding box is undefined"))
    }

    /// Creates a debug primitive that shows the outline of the tile bounding region.
    pub fn create_debug_volume(&self, color: Color) -> Result<Primitive, DeveloperError> {
        let outline_options = RectangleOutlineGeometryOptions {
            rectangle: self.rectangle,
            height: Some(self.minimum_height),
            extruded_height: Some(self.maximum_height),
            ellipsoid: Some(Ellipsoid::WGS84),
            granularity: None,
            rotation: None,
            offset_attribute: None,
        };
        let outline_geom = RectangleOutlineGeometry::new(outline_options)?;
        let geometry = RectangleOutlineGeometry::create_geometry(&outline_geom)
            .map_err(|e| DeveloperError::new(format!("Failed to create geometry: {e:?}")))?
            .ok_or_else(|| DeveloperError::new("Geometry creation returned None"))?;

        let mut attributes = HashMap::new();
        attributes.insert(
            "color".to_string(),
            GeometryInstanceAttribute {
                component_datatype: ComponentDatatype::Float,
                components_per_attribute: 4,
                normalize: false,
                value: vec![
                    color.red as f64,
                    color.green as f64,
                    color.blue as f64,
                    color.alpha as f64,
                ],
            },
        );

        let instance = GeometryInstance::new(
            geometry,
            Some(Matrix4::IDENTITY),
            Some("outline".to_string()),
            Some(attributes),
        );

        Ok(Primitive::new(PrimitiveOptions {
            geometry_instances: Some(vec![instance]),
            appearance: Some(PrimitiveAppearance {
                closed: false,
                translucent: false,
            }),
            asynchronous: Some(false),
            ..Default::default()
        }))
    }
}
