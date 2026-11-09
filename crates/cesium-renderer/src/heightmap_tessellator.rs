//! HeightmapTessellator matching CesiumJS `Core/HeightmapTessellator.js` (Apache-2.0).

use cesium_math::{
    AxisAlignedBoundingBox, BoundingSphere, Cartesian2, Cartesian3, DeveloperError, Ellipsoid,
    EllipsoidalOccluder, FixedFrameTransforms, GeoRectangle, Matrix4, OrientedBoundingBox,
    TerrainEncoding, WebMercatorProjection,
};
use glam::DVec3;
use std::f64::consts::FRAC_PI_2;

/// Native bounds for heightmap projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeightmapNativeRectangle {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

impl HeightmapNativeRectangle {
    pub const fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self {
            west,
            south,
            east,
            north,
        }
    }

    pub fn width(&self) -> f64 {
        self.east - self.west
    }

    pub fn height(&self) -> f64 {
        self.north - self.south
    }
}

/// Description of the binary structure of a heightmap buffer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeightmapStructure {
    pub height_scale: f64,
    pub height_offset: f64,
    pub elements_per_height: usize,
    pub stride: usize,
    pub element_multiplier: f64,
    pub is_big_endian: bool,
    pub lowest_encoded_height: Option<f64>,
    pub highest_encoded_height: Option<f64>,
}

impl Default for HeightmapStructure {
    fn default() -> Self {
        Self {
            height_scale: 1.0,
            height_offset: 0.0,
            elements_per_height: 1,
            stride: 1,
            element_multiplier: 256.0,
            is_big_endian: false,
            lowest_encoded_height: None,
            highest_encoded_height: None,
        }
    }
}

/// Options passed to [`HeightmapTessellator::compute_vertices`].
#[derive(Clone, Debug)]
pub struct ComputeVerticesOptions<'a> {
    pub heightmap: Option<&'a [f64]>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub skirt_height: Option<f64>,
    pub native_rectangle: Option<HeightmapNativeRectangle>,
    pub rectangle: Option<GeoRectangle>,
    pub is_geographic: Option<bool>,
    pub relative_to_center: Option<Cartesian3>,
    pub ellipsoid: Option<Ellipsoid>,
    pub structure: Option<HeightmapStructure>,
    pub exaggeration: Option<f64>,
    pub exaggeration_relative_height: Option<f64>,
    pub include_web_mercator_t: Option<bool>,
}

/// Results computed by [`HeightmapTessellator::compute_vertices`].
#[derive(Clone, Debug)]
pub struct ComputeVerticesResults {
    pub vertices: Vec<f32>,
    pub maximum_height: f64,
    pub minimum_height: f64,
    pub encoding: TerrainEncoding,
    pub bounding_sphere_3d: BoundingSphere,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub occludee_point_in_scaled_space: Option<Cartesian3>,
}

/// Functions to create a 3D mesh from a heightmap image/buffer.
pub struct HeightmapTessellator;

impl HeightmapTessellator {
    /// Fills an array of vertices from a heightmap image.
    pub fn compute_vertices(
        options: ComputeVerticesOptions,
    ) -> Result<ComputeVerticesResults, DeveloperError> {
        let heightmap = options
            .heightmap
            .ok_or_else(|| DeveloperError::new("options.heightmap is required."))?;
        let width = options
            .width
            .ok_or_else(|| DeveloperError::new("options.width and options.height are required."))?;
        let height = options
            .height
            .ok_or_else(|| DeveloperError::new("options.width and options.height are required."))?;
        if width == 0 || height == 0 {
            return Err(DeveloperError::new(
                "options.width and options.height must be greater than zero.",
            ));
        }
        let native_rectangle = options
            .native_rectangle
            .ok_or_else(|| DeveloperError::new("options.nativeRectangle is required."))?;
        let skirt_height = options
            .skirt_height
            .ok_or_else(|| DeveloperError::new("options.skirtHeight is required."))?;

        let has_skirts = skirt_height > 0.0;
        let is_geographic = options.is_geographic.unwrap_or(true);
        let ellipsoid = options.ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let one_over_globe_semimajor_axis = 1.0 / ellipsoid.maximum_radius();

        let (geo_west, geo_south, geo_east, geo_north) = if let Some(rect) = options.rectangle {
            (rect.west, rect.south, rect.east, rect.north)
        } else if is_geographic {
            (
                native_rectangle.west.to_radians(),
                native_rectangle.south.to_radians(),
                native_rectangle.east.to_radians(),
                native_rectangle.north.to_radians(),
            )
        } else {
            let w = native_rectangle.west * one_over_globe_semimajor_axis;
            let s = FRAC_PI_2 - 2.0 * (-native_rectangle.south * one_over_globe_semimajor_axis).exp().atan();
            let e = native_rectangle.east * one_over_globe_semimajor_axis;
            let n = FRAC_PI_2 - 2.0 * (-native_rectangle.north * one_over_globe_semimajor_axis).exp().atan();
            (w, s, e, n)
        };

        let relative_to_center = options.relative_to_center.unwrap_or(Cartesian3::ZERO);
        let has_relative_to_center = options.relative_to_center.is_some();
        let include_web_mercator_t = options.include_web_mercator_t.unwrap_or(false);

        let exaggeration = options.exaggeration.unwrap_or(1.0);
        let exaggeration_relative_height = options.exaggeration_relative_height.unwrap_or(0.0);
        let has_exaggeration = (exaggeration - 1.0).abs() > 1e-9;
        let include_geodetic_surface_normals = has_exaggeration;

        let structure = options.structure.unwrap_or_default();
        let height_scale = structure.height_scale;
        let height_offset = structure.height_offset;
        let elements_per_height = structure.elements_per_height;
        let stride = structure.stride;
        let element_multiplier = structure.element_multiplier;
        let is_big_endian = structure.is_big_endian;

        let mut rectangle_width = native_rectangle.width();
        let mut rectangle_height = native_rectangle.height();

        let granularity_x = rectangle_width / (width - 1) as f64;
        let granularity_y = rectangle_height / (height - 1) as f64;

        if !is_geographic {
            rectangle_width *= one_over_globe_semimajor_axis;
            rectangle_height *= one_over_globe_semimajor_axis;
        }

        let radii_squared = ellipsoid.radii_squared();
        let radii_squared_x = radii_squared.x;
        let radii_squared_y = radii_squared.y;
        let radii_squared_z = radii_squared.z;

        let mut minimum_height = 65536.0f64;
        let mut maximum_height = -65536.0f64;

        let from_enu = FixedFrameTransforms::east_north_up_to_fixed_frame(
            relative_to_center,
            Some(ellipsoid),
        )
        .map_err(|e| DeveloperError::new(format!("Failed eastNorthUpToFixedFrame: {e:?}")))?;
        let to_enu = from_enu.inverse().unwrap_or(Matrix4::IDENTITY);

        let (south_mercator_y, one_over_mercator_height) = if include_web_mercator_t {
            let sy = WebMercatorProjection::geodetic_latitude_to_mercator_angle(geo_south);
            let ny = WebMercatorProjection::geodetic_latitude_to_mercator_angle(geo_north);
            (sy, 1.0 / (ny - sy))
        } else {
            (0.0, 0.0)
        };

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut min_z = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut max_z = f64::NEG_INFINITY;
        let mut h_min = f64::INFINITY;

        let grid_vertex_count = width * height;
        let edge_vertex_count = if has_skirts {
            width * 2 + height * 2
        } else {
            0
        };
        let vertex_count = grid_vertex_count + edge_vertex_count;

        let mut positions = vec![Cartesian3::ZERO; vertex_count];
        let mut heights = vec![0.0f64; vertex_count];
        let mut uvs = vec![Cartesian2::ZERO; vertex_count];
        let mut web_mercator_ts = if include_web_mercator_t {
            vec![0.0f64; vertex_count]
        } else {
            Vec::new()
        };
        let mut geodetic_surface_normals = if include_geodetic_surface_normals {
            vec![Cartesian3::ZERO; vertex_count]
        } else {
            Vec::new()
        };

        let mut start_row: isize = 0;
        let mut end_row: isize = height as isize;
        let mut start_col: isize = 0;
        let mut end_col: isize = width as isize;

        if has_skirts {
            start_row -= 1;
            end_row += 1;
            start_col -= 1;
            end_col += 1;
        }

        let skirt_offset_percentage = 0.00001;

        for row_index in start_row..end_row {
            let row = row_index.clamp(0, height as isize - 1) as usize;

            let mut latitude = native_rectangle.north - granularity_y * row as f64;
            if !is_geographic {
                latitude = FRAC_PI_2 - 2.0 * (-latitude * one_over_globe_semimajor_axis).exp().atan();
            } else {
                latitude = latitude.to_radians();
            }

            let mut v = (latitude - geo_south) / (geo_north - geo_south);
            v = v.clamp(0.0, 1.0);

            let is_north_edge = row_index == start_row;
            let is_south_edge = row_index == end_row - 1;
            if has_skirts {
                if is_north_edge {
                    latitude += skirt_offset_percentage * rectangle_height;
                } else if is_south_edge {
                    latitude -= skirt_offset_percentage * rectangle_height;
                }
            }

            let cos_latitude = latitude.cos();
            let n_z = latitude.sin();
            let k_z = radii_squared_z * n_z;

            let web_mercator_t = if include_web_mercator_t {
                (WebMercatorProjection::geodetic_latitude_to_mercator_angle(latitude) - south_mercator_y)
                    * one_over_mercator_height
            } else {
                0.0
            };

            for col_index in start_col..end_col {
                let col = col_index.clamp(0, width as isize - 1) as usize;

                let terrain_offset = row * (width * stride) + col * stride;

                let mut height_sample = if elements_per_height == 1 {
                    heightmap[terrain_offset]
                } else if is_big_endian {
                    let mut s = 0.0;
                    for offset in 0..elements_per_height {
                        s = s * element_multiplier + heightmap[terrain_offset + offset];
                    }
                    s
                } else {
                    let mut s = 0.0;
                    for offset in (0..elements_per_height).rev() {
                        s = s * element_multiplier + heightmap[terrain_offset + offset];
                    }
                    s
                };

                height_sample = height_sample * height_scale + height_offset;

                maximum_height = maximum_height.max(height_sample);
                minimum_height = minimum_height.min(height_sample);

                let mut longitude = native_rectangle.west + granularity_x * col as f64;
                if !is_geographic {
                    longitude *= one_over_globe_semimajor_axis;
                } else {
                    longitude = longitude.to_radians();
                }

                let mut u = (longitude - geo_west) / (geo_east - geo_west);
                u = u.clamp(0.0, 1.0);

                let mut index = row * width + col;

                if has_skirts {
                    let is_west_edge = col_index == start_col;
                    let is_east_edge = col_index == end_col - 1;
                    let is_edge = is_north_edge || is_south_edge || is_west_edge || is_east_edge;
                    let is_corner = (is_north_edge || is_south_edge) && (is_west_edge || is_east_edge);

                    if is_corner {
                        continue;
                    } else if is_edge {
                        height_sample -= skirt_height;

                        if is_west_edge {
                            index = grid_vertex_count + (height - row - 1);
                            longitude -= skirt_offset_percentage * rectangle_width;
                        } else if is_south_edge {
                            index = grid_vertex_count + height + (width - col - 1);
                        } else if is_east_edge {
                            index = grid_vertex_count + height + width + row;
                            longitude += skirt_offset_percentage * rectangle_width;
                        } else if is_north_edge {
                            index = grid_vertex_count + height + width + height + col;
                        }
                    }
                }

                let n_x = cos_latitude * longitude.cos();
                let n_y = cos_latitude * longitude.sin();

                let k_x = radii_squared_x * n_x;
                let k_y = radii_squared_y * n_y;

                let gamma = (k_x * n_x + k_y * n_y + k_z * n_z).sqrt();
                let one_over_gamma = 1.0 / gamma;

                let r_surface_x = k_x * one_over_gamma;
                let r_surface_y = k_y * one_over_gamma;
                let r_surface_z = k_z * one_over_gamma;

                let position = Cartesian3::new(
                    r_surface_x + n_x * height_sample,
                    r_surface_y + n_y * height_sample,
                    r_surface_z + n_z * height_sample,
                );

                let enu_pos = to_enu.multiply_by_point(position);
                min_x = min_x.min(enu_pos.x);
                min_y = min_y.min(enu_pos.y);
                min_z = min_z.min(enu_pos.z);
                max_x = max_x.max(enu_pos.x);
                max_y = max_y.max(enu_pos.y);
                max_z = max_z.max(enu_pos.z);
                h_min = h_min.min(height_sample);

                positions[index] = position;
                uvs[index] = Cartesian2::new(u, v);
                heights[index] = height_sample;

                if include_web_mercator_t {
                    web_mercator_ts[index] = web_mercator_t;
                }

                if include_geodetic_surface_normals {
                    let dpos = DVec3::new(position.x, position.y, position.z);
                    let norm = ellipsoid
                        .geodetic_surface_normal(dpos)
                        .map(|n| Cartesian3::new(n.x, n.y, n.z))
                        .unwrap_or(Cartesian3::ZERO);
                    geodetic_surface_normals[index] = norm;
                }
            }
        }

        let dvec_positions: Vec<DVec3> = positions
            .iter()
            .map(|p| DVec3::new(p.x, p.y, p.z))
            .collect();
        let bounding_sphere_3d = BoundingSphere::from_points(&dvec_positions).unwrap_or_default();
        let oriented_bounding_box = options.rectangle.and_then(|rect| {
            OrientedBoundingBox::from_rectangle(
                rect,
                minimum_height,
                maximum_height,
                ellipsoid,
            )
            .ok()
        });

        let occludee_point_in_scaled_space = if has_relative_to_center {
            let occ = EllipsoidalOccluder::new(ellipsoid, DVec3::ZERO).ok();
            occ.and_then(|occluder| {
                let dir = DVec3::new(relative_to_center.x, relative_to_center.y, relative_to_center.z);
                occluder
                    .compute_horizon_culling_point_under_ellipsoid(
                        dir,
                        &dvec_positions,
                        minimum_height,
                    )
                    .map(|p| Cartesian3::new(p.x, p.y, p.z))
            })
        } else {
            None
        };

        let aa_box = AxisAlignedBoundingBox::new(
            DVec3::new(min_x, min_y, min_z),
            DVec3::new(max_x, max_y, max_z),
            Some(DVec3::new(relative_to_center.x, relative_to_center.y, relative_to_center.z)),
        );
        let encoding = TerrainEncoding::new(
            relative_to_center,
            Some(&aa_box),
            Some(h_min),
            Some(maximum_height),
            Some(from_enu),
            false,
            Some(include_web_mercator_t),
            Some(include_geodetic_surface_normals),
            Some(exaggeration),
            Some(exaggeration_relative_height),
        );

        let mut vertices = Vec::with_capacity(vertex_count * encoding.stride);
        let mut buffer_index = 0;
        for j in 0..vertex_count {
            buffer_index = encoding.encode(
                &mut vertices,
                buffer_index,
                positions[j],
                uvs[j],
                heights[j],
                None,
                if include_web_mercator_t {
                    Some(web_mercator_ts[j])
                } else {
                    None
                },
                if include_geodetic_surface_normals {
                    Some(geodetic_surface_normals[j])
                } else {
                    None
                },
            );
        }

        Ok(ComputeVerticesResults {
            vertices,
            maximum_height,
            minimum_height,
            encoding,
            bounding_sphere_3d,
            oriented_bounding_box,
            occludee_point_in_scaled_space,
        })
    }
}
