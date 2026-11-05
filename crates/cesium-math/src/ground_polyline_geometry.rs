use std::collections::HashMap;

use glam::DVec3;

use crate::{
    ApproximateTerrainHeights, ArcType, BoundingSphere, Cartesian3, Cartographic, ComponentDatatype,
    DeveloperError, Ellipsoid, EllipsoidGeodesic, EllipsoidRhumbLine, EncodedCartesian3,
    GeoRectangle, Geometry, GeometryAttribute, GeometryAttributes, PrimitiveType,
};

const MITER_BREAK_SMALL: f64 = 0.8660254037844387; // cos(30 deg)
const MITER_BREAK_LARGE: f64 = -0.8660254037844387; // cos(150 deg)

const WALL_INITIAL_MAX_HEIGHT: f64 = 1000.0;

// Indices for inside-out shadow volume of each segment (6 faces * 2 triangles * 3 vertices = 36 indices)
const REFERENCE_INDICES: [u16; 36] = [
    0, 2, 1, 0, 3, 2, // right
    0, 7, 3, 0, 4, 7, // start
    0, 5, 4, 0, 1, 5, // bottom
    5, 7, 4, 5, 6, 7, // left
    5, 2, 6, 5, 1, 2, // end
    3, 6, 2, 3, 7, 6, // top
];

/// Options for constructing GroundPolylineGeometry.
#[derive(Debug, Clone)]
pub struct GroundPolylineGeometryOptions {
    pub positions: Vec<Cartesian3>,
    pub width: Option<f64>,
    pub granularity: Option<f64>,
    pub loop_: Option<bool>,
    pub arc_type: Option<ArcType>,
    pub ellipsoid: Option<Ellipsoid>,
}

/// A description of a polyline on terrain or 3D Tiles.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundPolylineGeometry {
    positions: Vec<Cartesian3>,
    pub width: f64,
    pub granularity: f64,
    pub loop_: bool,
    pub arc_type: ArcType,
    pub ellipsoid: Ellipsoid,
    pub projection_index: usize, // 0 = Geographic, 1 = WebMercator
    pub scene_3d_only: bool,
}

impl GroundPolylineGeometry {
    /// Creates a new GroundPolylineGeometry instance.
    pub fn new(options: GroundPolylineGeometryOptions) -> Result<Self, DeveloperError> {
        if options.positions.len() < 2 {
            return Err(DeveloperError::new(
                "At least two positions are required.",
            ));
        }

        let arc_type = options.arc_type.unwrap_or(ArcType::Geodesic);
        if arc_type != ArcType::Geodesic && arc_type != ArcType::Rhumb {
            return Err(DeveloperError::new(
                "Valid options for arcType are ArcType.GEODESIC and ArcType.RHUMB.",
            ));
        }

        Ok(Self {
            positions: options.positions,
            width: options.width.unwrap_or(1.0),
            granularity: options.granularity.unwrap_or(9999.0),
            loop_: options.loop_.unwrap_or(false),
            arc_type,
            ellipsoid: options.ellipsoid.unwrap_or(Ellipsoid::WGS84),
            projection_index: 0,
            scene_3d_only: false,
        })
    }

    /// The polyline positions.
    pub fn positions(&self) -> &[Cartesian3] {
        &self.positions
    }

    /// Sets projection and ellipsoid.
    pub fn set_projection_and_ellipsoid(&mut self, projection_index: usize, ellipsoid: Ellipsoid) {
        self.projection_index = projection_index;
        self.ellipsoid = ellipsoid;
    }

    /// Packed length in f64 array.
    pub fn packed_length(&self) -> usize {
        1 + self.positions.len() * 3 + 1 + 1 + 1 + 3 + 1 + 1
    }

    /// Stores the provided instance into the provided array.
    pub fn pack(&self, array: &mut Vec<f64>) {
        array.push(self.positions.len() as f64);
        for p in &self.positions {
            array.push(p.x);
            array.push(p.y);
            array.push(p.z);
        }
        array.push(self.width);
        array.push(self.granularity);
        array.push(if self.loop_ { 1.0 } else { 0.0 });
        array.push(self.ellipsoid.radii().x);
        array.push(self.ellipsoid.radii().y);
        array.push(self.ellipsoid.radii().z);
        array.push(self.arc_type as u8 as f64);
        array.push(self.projection_index as f64);
    }

    /// Retrieves an instance from a packed array.
    pub fn unpack(
        array: &[f64],
        starting_index: usize,
    ) -> Result<Self, DeveloperError> {
        let mut idx = starting_index;
        let pos_len = array[idx] as usize;
        idx += 1;

        let mut positions = Vec::with_capacity(pos_len);
        for _ in 0..pos_len {
            positions.push(Cartesian3::new(array[idx], array[idx + 1], array[idx + 2]));
            idx += 3;
        }

        let width = array[idx];
        idx += 1;
        let granularity = array[idx];
        idx += 1;
        let loop_ = array[idx] > 0.0;
        idx += 1;

        let radii = DVec3::new(array[idx], array[idx + 1], array[idx + 2]);
        let ellipsoid = Ellipsoid::new(radii).map_err(|e| DeveloperError::new(e.to_string()))?;
        idx += 3;

        let arc_type = if (array[idx] - ArcType::Rhumb as u8 as f64).abs() < 1e-4 {
            ArcType::Rhumb
        } else {
            ArcType::Geodesic
        };
        idx += 1;

        let projection_index = array[idx] as usize;

        Ok(Self {
            positions,
            width,
            granularity,
            loop_,
            arc_type,
            ellipsoid,
            projection_index,
            scene_3d_only: false,
        })
    }

    /// Computes the geometry for a ground polyline.
    pub fn create_geometry(geometry: &Self) -> Option<Geometry> {
        let ellipsoid = geometry.ellipsoid;

        // Convert positions to cartographics with 0 height and remove adjacent duplicates
        let mut cartographics = Vec::new();
        for p in &geometry.positions {
            if let Some(mut c) = ellipsoid.cartesian_to_cartographic(DVec3::from(*p)) {
                c.height = 0.0;
                cartographics.push(c);
            }
        }

        // Deduplicate adjacent cartographics with epsilon comparison
        cartographics.dedup_by(|a, b| {
            (a.longitude - b.longitude).abs() < 1e-11
                && (a.latitude - b.latitude).abs() < 1e-11
        });

        if cartographics.len() < 2 {
            return None;
        }

        // Handle loop by appending start point if specified
        let is_loop = geometry.loop_ && cartographics.len() >= 3;

        let mut normals = Vec::new();
        let mut bottom_positions = Vec::new();
        let mut top_positions = Vec::new();
        let mut intermediate_cartographics = Vec::new();

        let num_pts = cartographics.len();
        for i in 0..num_pts {
            let curr = cartographics[i];
            let next = if i + 1 < num_pts {
                cartographics[i + 1]
            } else if is_loop {
                cartographics[0]
            } else {
                break;
            };

            let curr_bottom = Cartesian3::from(ellipsoid.cartographic_to_cartesian(curr)?);
            let curr_top = Cartesian3::from(ellipsoid.cartographic_to_cartesian(Cartographic::new(
                curr.longitude,
                curr.latitude,
                WALL_INITIAL_MAX_HEIGHT,
            ))?);

            let normal = compute_right_normal(&curr, &next, WALL_INITIAL_MAX_HEIGHT, ellipsoid)?;

            normals.push(normal);
            bottom_positions.push(curr_bottom);
            top_positions.push(curr_top);
            intermediate_cartographics.push(curr);

            // Interpolate segment if granularity > 0
            if geometry.granularity > 0.0 {
                interpolate_segment(
                    &curr,
                    &next,
                    geometry.granularity,
                    geometry.arc_type,
                    ellipsoid,
                    &mut normals,
                    &mut bottom_positions,
                    &mut top_positions,
                    &mut intermediate_cartographics,
                );
            }
        }

        // Add last point
        let last = if is_loop {
            cartographics[0]
        } else {
            *cartographics.last()?
        };
        let last_bottom = Cartesian3::from(ellipsoid.cartographic_to_cartesian(last)?);
        let last_top = Cartesian3::from(ellipsoid.cartographic_to_cartesian(Cartographic::new(
            last.longitude,
            last.latitude,
            WALL_INITIAL_MAX_HEIGHT,
        ))?);
        let pre_last = cartographics[cartographics.len() - 2];
        let last_normal = compute_right_normal(&pre_last, &last, WALL_INITIAL_MAX_HEIGHT, ellipsoid)?;

        normals.push(last_normal);
        bottom_positions.push(last_bottom);
        top_positions.push(last_top);
        intermediate_cartographics.push(last);

        if bottom_positions.len() < 2 {
            return None;
        }

        generate_geometry_attributes(
            geometry,
            &bottom_positions,
            &top_positions,
            &normals,
            &intermediate_cartographics,
        )
    }
}

fn compute_right_normal(
    start: &Cartographic,
    end: &Cartographic,
    max_height: f64,
    ellipsoid: Ellipsoid,
) -> Option<Cartesian3> {
    let start_bottom = Cartesian3::from(ellipsoid.cartographic_to_cartesian(*start)?);
    let start_top = Cartesian3::from(ellipsoid.cartographic_to_cartesian(Cartographic::new(
        start.longitude,
        start.latitude,
        max_height,
    ))?);
    let end_bottom = Cartesian3::from(ellipsoid.cartographic_to_cartesian(*end)?);

    let up = (start_top - start_bottom).normalize();
    let forward = (end_bottom - start_bottom).normalize();
    Some(forward.cross(&up).normalize())
}

#[allow(clippy::too_many_arguments)]
fn interpolate_segment(
    start: &Cartographic,
    end: &Cartographic,
    granularity: f64,
    arc_type: ArcType,
    ellipsoid: Ellipsoid,
    normals: &mut Vec<Cartesian3>,
    bottom_positions: &mut Vec<Cartesian3>,
    top_positions: &mut Vec<Cartesian3>,
    cartographics: &mut Vec<Cartographic>,
) {
    let normal = match compute_right_normal(start, end, WALL_INITIAL_MAX_HEIGHT, ellipsoid) {
        Some(n) => n,
        None => return,
    };

    if arc_type == ArcType::Geodesic {
        let geodesic = EllipsoidGeodesic::new(Some(*start), Some(*end), Some(ellipsoid));
        if let Ok(dist) = geodesic.surface_distance() {
            if dist > granularity {
                let segments = (dist / granularity).ceil() as usize;
                let step = dist / segments as f64;
                for i in 1..segments {
                    let d = step * (i as f64);
                    if let Ok(interp) = geodesic.interpolate_using_surface_distance(d) {
                        if let (Some(b), Some(t)) = (
                            ellipsoid.cartographic_to_cartesian(interp),
                            ellipsoid.cartographic_to_cartesian(Cartographic::new(
                                interp.longitude,
                                interp.latitude,
                                WALL_INITIAL_MAX_HEIGHT,
                            )),
                        ) {
                            normals.push(normal);
                            bottom_positions.push(Cartesian3::from(b));
                            top_positions.push(Cartesian3::from(t));
                            cartographics.push(interp);
                        }
                    }
                }
            }
        }
    } else {
        let rhumb = EllipsoidRhumbLine::new(Some(*start), Some(*end), Some(ellipsoid));
        if let Ok(dist) = rhumb.surface_distance() {
            if dist > granularity {
                let segments = (dist / granularity).ceil() as usize;
                let step = dist / segments as f64;
                for i in 1..segments {
                    let d = step * (i as f64);
                    if let Ok(interp) = rhumb.interpolate_using_surface_distance(d) {
                        if let (Some(b), Some(t)) = (
                            ellipsoid.cartographic_to_cartesian(interp),
                            ellipsoid.cartographic_to_cartesian(Cartographic::new(
                                interp.longitude,
                                interp.latitude,
                                WALL_INITIAL_MAX_HEIGHT,
                            )),
                        ) {
                            normals.push(normal);
                            bottom_positions.push(Cartesian3::from(b));
                            top_positions.push(Cartesian3::from(t));
                            cartographics.push(interp);
                        }
                    }
                }
            }
        }
    }
}

fn generate_geometry_attributes(
    geometry: &GroundPolylineGeometry,
    bottom_positions: &[Cartesian3],
    _top_positions: &[Cartesian3],
    normals: &[Cartesian3],
    cartographics: &[Cartographic],
) -> Option<Geometry> {
    let segment_count = bottom_positions.len() - 1;
    let vertex_count = segment_count * 8;
    let index_count = segment_count * 36;

    let mut indices: Vec<u32> = Vec::with_capacity(index_count);
    let mut positions: Vec<f64> = Vec::with_capacity(vertex_count * 3);
    let mut start_hi_and_forward_offset_x = Vec::with_capacity(vertex_count * 4);
    let mut start_lo_and_forward_offset_y = Vec::with_capacity(vertex_count * 4);
    let mut start_normal_and_forward_offset_z = Vec::with_capacity(vertex_count * 4);
    let mut end_normal_and_texcoord_x = Vec::with_capacity(vertex_count * 4);
    let mut right_normal_and_texcoord_y = Vec::with_capacity(vertex_count * 4);

    let ellipsoid = geometry.ellipsoid;

    for i in 0..segment_count {
        let start_cart = cartographics[i];
        let end_cart = cartographics[i + 1];

        // Query terrain height range for this segment
        let rect = GeoRectangle::from_cartographics(&[start_cart, end_cart]);
        let heights = if let Some(ref r) = rect {
            ApproximateTerrainHeights::get_minimum_maximum_heights(r, Some(ellipsoid))
        } else {
            crate::TerrainHeights {
                minimum_terrain_height: ApproximateTerrainHeights::DEFAULT_MIN_TERRAIN_HEIGHT,
                maximum_terrain_height: ApproximateTerrainHeights::DEFAULT_MAX_TERRAIN_HEIGHT,
            }
        };
        let min_height = heights.minimum_terrain_height;
        let max_height = heights.maximum_terrain_height.max(min_height + 1.0);

        let start_bottom = Cartesian3::from(ellipsoid.cartographic_to_cartesian(Cartographic::new(
            start_cart.longitude,
            start_cart.latitude,
            min_height,
        ))?);
        let start_top = Cartesian3::from(ellipsoid.cartographic_to_cartesian(Cartographic::new(
            start_cart.longitude,
            start_cart.latitude,
            max_height,
        ))?);
        let end_bottom = Cartesian3::from(ellipsoid.cartographic_to_cartesian(Cartographic::new(
            end_cart.longitude,
            end_cart.latitude,
            min_height,
        ))?);
        let end_top = Cartesian3::from(ellipsoid.cartographic_to_cartesian(Cartographic::new(
            end_cart.longitude,
            end_cart.latitude,
            max_height,
        ))?);

        let mut start_norm = normals[i];
        let mut end_norm = normals[i + 1];

        // Break miter check
        let line_dir = (end_bottom - start_bottom).normalize();
        let dot_end = line_dir.dot(&end_norm);
        if dot_end > MITER_BREAK_SMALL || dot_end < MITER_BREAK_LARGE {
            let up = (end_top - end_bottom).normalize();
            end_norm = line_dir.cross(&up).normalize();
        }
        let dot_start = line_dir.dot(&start_norm);
        if dot_start > MITER_BREAK_SMALL || dot_start < MITER_BREAK_LARGE {
            let up = (start_top - start_bottom).normalize();
            start_norm = line_dir.cross(&up).normalize();
        }

        let right_norm = (start_norm + end_norm).normalize();

        // 8 vertices for the segment's volume
        // 0-3 at start, 4-7 at end
        let v_corners = [
            start_bottom - start_norm * 5.0,
            start_bottom + start_norm * 5.0,
            start_top + start_norm * 5.0,
            start_top - start_norm * 5.0,
            end_bottom - end_norm * 5.0,
            end_bottom + end_norm * 5.0,
            end_top + end_norm * 5.0,
            end_top - end_norm * 5.0,
        ];

        let forward_offset = end_bottom - start_bottom;
        let encoded = EncodedCartesian3::from_cartesian(start_bottom);
        let start_hi = encoded.high;
        let start_lo = encoded.low;

        let v_base = (i * 8) as u32;
        for &idx in &REFERENCE_INDICES {
            indices.push(v_base + idx as u32);
        }

        for v in &v_corners {
            positions.push(v.x);
            positions.push(v.y);
            positions.push(v.z);

            start_hi_and_forward_offset_x.push(start_hi.x as f32);
            start_hi_and_forward_offset_x.push(start_hi.y as f32);
            start_hi_and_forward_offset_x.push(start_hi.z as f32);
            start_hi_and_forward_offset_x.push(forward_offset.x as f32);

            start_lo_and_forward_offset_y.push(start_lo.x as f32);
            start_lo_and_forward_offset_y.push(start_lo.y as f32);
            start_lo_and_forward_offset_y.push(start_lo.z as f32);
            start_lo_and_forward_offset_y.push(forward_offset.y as f32);

            start_normal_and_forward_offset_z.push(start_norm.x as f32);
            start_normal_and_forward_offset_z.push(start_norm.y as f32);
            start_normal_and_forward_offset_z.push(start_norm.z as f32);
            start_normal_and_forward_offset_z.push(forward_offset.z as f32);

            end_normal_and_texcoord_x.push(end_norm.x as f32);
            end_normal_and_texcoord_x.push(end_norm.y as f32);
            end_normal_and_texcoord_x.push(end_norm.z as f32);
            end_normal_and_texcoord_x.push(1.0);

            right_normal_and_texcoord_y.push(right_norm.x as f32);
            right_normal_and_texcoord_y.push(right_norm.y as f32);
            right_normal_and_texcoord_y.push(right_norm.z as f32);
            right_normal_and_texcoord_y.push(1.0);
        }
    }

    let bounding_points: Vec<DVec3> = positions
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| DVec3::new(c[0], c[1], c[2]))
        .collect();
    let bounding_sphere = BoundingSphere::from_points(&bounding_points).ok();

    let mut custom = HashMap::new();

    custom.insert(
        "startHiAndForwardOffsetX".to_string(),
        GeometryAttribute {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 4,
            normalize: false,
            values: start_hi_and_forward_offset_x.into_iter().map(|v| v as f64).collect(),
        },
    );

    custom.insert(
        "startLoAndForwardOffsetY".to_string(),
        GeometryAttribute {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 4,
            normalize: false,
            values: start_lo_and_forward_offset_y.into_iter().map(|v| v as f64).collect(),
        },
    );

    custom.insert(
        "startNormalAndForwardOffsetZ".to_string(),
        GeometryAttribute {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 4,
            normalize: false,
            values: start_normal_and_forward_offset_z.into_iter().map(|v| v as f64).collect(),
        },
    );

    custom.insert(
        "endNormalAndTextureCoordinateNormalizationX".to_string(),
        GeometryAttribute {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 4,
            normalize: false,
            values: end_normal_and_texcoord_x.into_iter().map(|v| v as f64).collect(),
        },
    );

    custom.insert(
        "rightNormalAndTextureCoordinateNormalizationY".to_string(),
        GeometryAttribute {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 4,
            normalize: false,
            values: right_normal_and_texcoord_y.into_iter().map(|v| v as f64).collect(),
        },
    );

    let attributes = GeometryAttributes {
        position: Some(GeometryAttribute {
            component_datatype: ComponentDatatype::Double,
            components_per_attribute: 3,
            normalize: false,
            values: positions,
        }),
        custom,
        ..Default::default()
    };

    Some(Geometry::new(
        attributes,
        indices,
        PrimitiveType::Triangles,
        bounding_sphere,
    ))
}
