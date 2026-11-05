use std::sync::Arc;

use cesium_math::{
    BoundingSphere, BufferPolygon, BufferPolygonCollection, BufferPolygonCollectionOptions,
    BufferPolygonMaterial, BufferPolyline, BufferPolylineCollection,
    BufferPolylineCollectionOptions, BufferPolylineMaterial, Cartesian3, Cartographic, Color,
    Ellipsoid, GeoRectangle, GeographicTilingScheme, HeightReference,
    VectorPipeline, VectorProvider, VectorProviderOptions, WidthUnits,
    MAXIMUM_GROUND_WIDTH_ANGLE,
};

fn c3(lon: f64, lat: f64) -> Cartesian3 {
    Cartesian3::from_degrees(lon, lat, Some(0.0), None).unwrap()
}

fn line_midpoint() -> Cartographic {
    Cartographic::from_degrees(-95.0, 40.0, Some(0.0))
}

fn far_point() -> Cartographic {
    Cartographic::from_degrees(100.0, -40.0, Some(0.0))
}

fn polyline_positions(lon: f64, lat: f64) -> Vec<Cartesian3> {
    vec![
        c3(lon - 5.0, lat),
        c3(lon, lat),
        c3(lon + 5.0, lat),
    ]
}

fn create_polyline_collection(
    lon: Option<f64>,
    lat: Option<f64>,
    height_reference: Option<HeightReference>,
    width_units: Option<WidthUnits>,
    allow_picking: Option<bool>,
) -> BufferPolylineCollection {
    let coll = BufferPolylineCollection::new(BufferPolylineCollectionOptions {
        height_reference,
        width_units,
        allow_picking,
        ..Default::default()
    });
    coll.add(BufferPolyline::new(
        polyline_positions(lon.unwrap_or(-95.0), lat.unwrap_or(40.0)),
        BufferPolylineMaterial::default(),
    ));
    coll
}

fn min_packed_v(width: f64, width_units: WidthUnits) -> f64 {
    let coll = BufferPolylineCollection::new(BufferPolylineCollectionOptions {
        height_reference: Some(HeightReference::ClampToTerrain),
        width_units: Some(width_units),
        ..Default::default()
    });
    let positions = vec![
        c3(-95.0, 40.0),
        c3(-95.0, 33.1875),
        c3(-90.0, 33.1875),
    ];
    coll.add(BufferPolyline::new(
        positions,
        BufferPolylineMaterial::new(width, Color::WHITE),
    ));

    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    let mut min_v = f64::INFINITY;
    for i in (1..data.polyline_segment_texels.len()).step_by(2) {
        let v = data.polyline_segment_texels[i] as f64;
        if v > -0.5 {
            min_v = min_v.min(v);
        }
    }
    min_v
}

fn create_wide_short_collection(width: f64, width_units: WidthUnits) -> BufferPolylineCollection {
    let coll = BufferPolylineCollection::new(BufferPolylineCollectionOptions {
        height_reference: Some(HeightReference::ClampToTerrain),
        width_units: Some(width_units),
        ..Default::default()
    });
    let positions = vec![
        c3(-95.02, 40.0),
        c3(-94.98, 40.0),
    ];
    coll.add(BufferPolyline::new(
        positions,
        BufferPolylineMaterial::new(width, Color::WHITE),
    ));
    coll
}

fn create_polygon_collection(with_hole: bool) -> BufferPolygonCollection {
    let coll = BufferPolygonCollection::new(BufferPolygonCollectionOptions {
        height_reference: Some(HeightReference::ClampToTerrain),
        ..Default::default()
    });
    let mut positions = vec![
        c3(-100.0, 35.0),
        c3(-90.0, 35.0),
        c3(-90.0, 45.0),
        c3(-100.0, 45.0),
    ];
    let holes = if with_hole {
        positions.extend_from_slice(&[
            c3(-97.0, 38.0),
            c3(-93.0, 38.0),
            c3(-93.0, 42.0),
            c3(-97.0, 42.0),
        ]);
        vec![4]
    } else {
        Vec::new()
    };
    coll.add(BufferPolygon::new(
        positions,
        holes,
        BufferPolygonMaterial::default(),
    ));
    coll
}

fn count_ray_crossings(data: &cesium_math::VectorTileData, uv_x: f64, uv_y: f64) -> usize {
    let grid_width = data.polygon_grid_cell_indices[0] as usize;
    let grid_height = data.polygon_grid_cell_indices[1] as usize;
    let cell_x = ((uv_x * grid_width as f64).floor() as usize).min(grid_width - 1);
    let cell_y = ((uv_y * grid_height as f64).floor() as usize).min(grid_height - 1);
    let cell = cell_x + cell_y * grid_width;

    let start = if cell == 0 {
        0
    } else {
        data.polygon_grid_cell_indices[cell + 1] as usize
    };
    let end = data.polygon_grid_cell_indices[cell + 2] as usize;

    let mut crossings = 0;
    for e in start..end {
        let ax = data.polygon_edge_texels[e * 4] as f64;
        let ay = data.polygon_edge_texels[e * 4 + 1] as f64;
        let bx = data.polygon_edge_texels[e * 4 + 2] as f64;
        let by = data.polygon_edge_texels[e * 4 + 3] as f64;

        if (ay > uv_y) != (by > uv_y) {
            let t = (uv_y - ay) / (by - ay);
            if uv_x < ax + t * (bx - ax) {
                crossings += 1;
            }
        }
    }
    crossings
}

#[test]
fn returns_hidden_vector_data_with_no_collections() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);
    assert!(!data.show);
}

#[test]
fn returns_packed_lookup_data_for_a_tile_overlapping_a_polyline() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    assert!(data.show);
    assert!(!data.widths.is_empty());
    assert!(!data.colors.is_empty());

    let grid_w = data.polyline_grid_cell_indices[0];
    let grid_h = data.polyline_grid_cell_indices[1];
    assert!(grid_w > 0);
    assert!(grid_h > 0);
    assert_eq!(
        data.polyline_grid_cell_indices.len(),
        (grid_w * grid_h + 2) as usize
    );

    let mut packed_count = 0;
    for &t in &data.polyline_segment_texels {
        if t >= 0.0 {
            packed_count += 1;
        }
    }
    assert!(packed_count > 0);
}

#[test]
fn bakes_a_pick_color_for_each_primitive_of_a_pickable_collection() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, Some(true));
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    let pick_id = coll.get(0).unwrap().pick_id;
    let expected = Color::from_rgba(pick_id).to_bytes();
    assert_eq!(data.pick_colors[0], expected);
    assert!(data.pick_color_texture_defined);
}

#[test]
fn clips_segments_to_the_tile_uv_domain_plus_a_small_margin() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    let max_margin = 0.01_f32;
    for &val in &data.polyline_segment_texels {
        if val > -0.5 {
            assert!(val >= -max_margin - 1.0e-5);
            assert!(val <= 1.0 + max_margin + 1.0e-5);
        }
    }
}

#[test]
fn widens_the_clip_margin_so_a_wide_line_just_outside_a_tile_is_kept() {
    let outside_v = -0.05;
    assert!(min_packed_v(1.0, WidthUnits::Pixels) > outside_v);
    let min_v60 = min_packed_v(60.0, WidthUnits::Pixels);
    assert!((min_v60 - outside_v).abs() < 0.01);
}

#[test]
fn measures_the_clip_margin_in_ground_meters_when_widths_are_in_meters() {
    let outside_v = -0.05;
    assert!(min_packed_v(60.0, WidthUnits::Meters) > outside_v);
    let min_v_wide = min_packed_v(200_000.0, WidthUnits::Meters);
    assert!((min_v_wide - outside_v).abs() < 0.01);
}

#[test]
fn packs_a_line_wide_in_meters_into_a_tile_its_centerline_misses() {
    let detail_level = 10;
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_wide_short_collection(50_000.0, WidthUnits::Meters);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme
        .position_to_tile(line_midpoint(), detail_level)
        .unwrap();
    let data = provider.request_tile_data(x, y - 1, detail_level, HeightReference::ClampToTerrain);
    assert!(data.show);
}

#[test]
fn packs_a_line_wide_in_pixels_into_a_tile_its_centerline_misses() {
    let detail_level = 10;
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_wide_short_collection(2000.0, WidthUnits::Pixels);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme
        .position_to_tile(line_midpoint(), detail_level)
        .unwrap();
    let data = provider.request_tile_data(x, y - 1, detail_level, HeightReference::ClampToTerrain);
    assert!(data.show);
}

#[test]
fn clamps_a_width_in_meters_too_wide_for_a_tiles_tangent_plane() {
    let coll = create_wide_short_collection(5_000_000.0, WidthUnits::Meters);
    let data = VectorPipeline::pack_polyline_collection_data(&coll, &Ellipsoid::WGS84);

    let max_allowed = MAXIMUM_GROUND_WIDTH_ANGLE * Ellipsoid::WGS84.maximum_radius();
    assert!(-data.widths[0] > 0.0);
    assert!((-data.widths[0] as f64 - max_allowed).abs() < 1.0);
    assert!(-data.widths[0] < 5_000_000.0);
}

#[test]
fn reports_the_tiles_ground_size_for_world_space_widths() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, Some(WidthUnits::Meters), None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    assert!(data.has_meter_widths);
    assert!(!data.has_pixel_widths);
    assert!(data.meters_per_uv.x > 0.0);
    assert!(data.meters_per_uv.x < data.meters_per_uv.y);
}

#[test]
fn reports_both_units_for_a_tile_mixing_pixel_and_meter_widths() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let pixel_coll = create_polyline_collection(None, None, None, Some(WidthUnits::Pixels), None);
    let meter_coll = create_polyline_collection(None, None, None, Some(WidthUnits::Meters), None);

    provider.mark_for_frame_polyline(&pixel_coll, 0, HeightReference::ClampToTerrain);
    provider.mark_for_frame_polyline(&meter_coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    assert!(data.has_pixel_widths);
    assert!(data.has_meter_widths);
}

#[test]
fn returns_hidden_vector_data_for_a_tile_not_overlapping_any_polyline() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(far_point(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);
    assert!(!data.show);
}

#[test]
fn stops_returning_data_after_a_collection_is_removed() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);
    provider.remove_polyline(&coll);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);
    assert!(!data.show);
}

#[test]
fn reports_whether_a_collection_is_being_draped() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    assert!(!provider.has_polyline(&coll));

    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);
    assert!(provider.has_polyline(&coll));

    provider.remove_polyline(&coll);
    assert!(!provider.has_polyline(&coll));
}

#[test]
fn stops_reporting_a_collection_once_it_is_pruned() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    provider.update(Some(1));
    assert!(provider.has_polyline(&coll));

    provider.update(Some(2));
    assert!(!provider.has_polyline(&coll));
}

#[test]
fn leaves_a_tile_clean_when_no_dirty_regions_are_recorded() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);
    provider.make_clean();

    provider.update(None);
    assert!(!provider.is_tile_dirty(x, y, 4));
}

#[test]
fn marks_overlapping_tiles_dirty_after_a_collections_content_changes() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);
    provider.make_clean();

    coll.set_polyline_positions(0, polyline_positions(-95.0, 41.0));
    provider.update(None);
    assert!(provider.is_tile_dirty(x, y, 4));
}

#[test]
fn marks_a_tile_only_a_widened_stroke_reaches_dirty() {
    let detail_level = 10;
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_wide_short_collection(1000.0, WidthUnits::Meters);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme
        .position_to_tile(line_midpoint(), detail_level)
        .unwrap();
    let data = provider.request_tile_data(x, y - 1, detail_level, HeightReference::ClampToTerrain);
    provider.make_clean();
    assert!(!data.show);

    coll.set_polyline_material(
        0,
        BufferPolylineMaterial::new(50_000.0, Color::WHITE),
    );
    provider.update(None);
    assert!(provider.is_tile_dirty(x, y - 1, detail_level));

    let updated =
        provider.request_tile_data(x, y - 1, detail_level, HeightReference::ClampToTerrain);
    assert!(updated.show);
}

#[test]
fn records_and_clears_a_dirty_rectangle_for_a_collection_with_a_local_region() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = BufferPolylineCollection::new(BufferPolylineCollectionOptions {
        height_reference: Some(HeightReference::ClampToTerrain),
        bounding_volume: Some(BoundingSphere::new(
            c3(-95.0, 40.0),
            100_000.0,
        )),
        ..Default::default()
    });

    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);
    assert_eq!(provider.dirty_regions().len(), 1);

    provider.make_clean();
    assert_eq!(provider.dirty_regions().len(), 0);

    provider.remove_polyline(&coll);
    assert_eq!(provider.dirty_regions().len(), 1);
}

#[test]
fn returns_packed_polygon_lookup_data_for_a_tile_overlapping_a_polygon() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polygon_collection(false);
    provider.mark_for_frame_polygon(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    assert!(data.show);
    assert!(!data.colors.is_empty());

    let grid_w = data.polygon_grid_cell_indices[0] as usize;
    let grid_h = data.polygon_grid_cell_indices[1] as usize;
    assert!(grid_w > 0);
    assert!(grid_h > 0);
    assert_eq!(data.polygon_grid_cell_indices.len(), grid_w * grid_h + 2);

    let mut packed_count = 0;
    for &val in &data.polygon_edge_texels {
        if val >= 0.0 {
            packed_count += 1;
        }
    }
    assert!(packed_count > 0);
    assert!(data.polyline_segment_texels.is_empty());

    let cell_count = grid_w * grid_h;
    for cell in 0..cell_count {
        let start = if cell == 0 {
            0
        } else {
            data.polygon_grid_cell_indices[cell + 1] as usize
        };
        let end = data.polygon_grid_cell_indices[cell + 2] as usize;
        let cell_y = cell / grid_w;
        let probe_y = (cell_y as f64 + 0.5) / grid_h as f64;

        let mut crossings = 0;
        for e in start..end {
            let ay = data.polygon_edge_texels[e * 4 + 1] as f64;
            let by = data.polygon_edge_texels[e * 4 + 3] as f64;
            if (ay > probe_y) != (by > probe_y) {
                crossings += 1;
            }
        }
        assert_eq!(crossings % 2, 0, "Cell {} must have even parity", cell);
    }
}

#[test]
fn packs_hole_rings_so_interior_fragments_resolve_to_even_parity() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polygon_collection(true);
    provider.mark_for_frame_polygon(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);
    assert!(data.show);

    let tile_rect = tiling_scheme.tile_rectangle(x, y, 4).unwrap();
    let to_uv = |lon_deg: f64, lat_deg: f64| {
        (
            (lon_deg.to_radians() - tile_rect.west) / tile_rect.width(),
            (lat_deg.to_radians() - tile_rect.south) / tile_rect.height(),
        )
    };

    let (hx, hy) = to_uv(-95.0, 40.0);
    assert_eq!(count_ray_crossings(&data, hx, hy) % 2, 0);

    let (fx, fy) = to_uv(-98.5, 40.0);
    assert_eq!(count_ray_crossings(&data, fx, fy) % 2, 1);
}

#[test]
fn returns_hidden_vector_data_for_a_tile_not_overlapping_any_polygon() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polygon_collection(false);
    provider.mark_for_frame_polygon(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(far_point(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);
    assert!(!data.show);
}

#[test]
fn packs_polylines_and_polygons_into_a_shared_primitive_index_space() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let polylines = create_polyline_collection(None, None, None, None, None);
    let polygons = create_polygon_collection(false);
    provider.mark_for_frame_polyline(&polylines, 0, HeightReference::ClampToTerrain);
    provider.mark_for_frame_polygon(&polygons, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    let data = provider.request_tile_data(x, y, 4, HeightReference::ClampToTerrain);

    assert!(data.show);
    assert!(!data.polyline_segment_texels.is_empty());
    assert!(!data.polygon_edge_texels.is_empty());
    assert_eq!(data.primitive_count, 2);

    let mut max_polygon_primitive = -1.0_f32;
    for &idx in &data.polygon_edge_primitive_indices_texels {
        max_polygon_primitive = max_polygon_primitive.max(idx);
    }
    assert_eq!(max_polygon_primitive as i32, 1);
}

#[test]
fn bakes_a_terrain_clamped_collection_only_for_terrain_targets() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(
        None,
        None,
        Some(HeightReference::ClampToTerrain),
        None,
        None,
    );
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    assert!(
        provider
            .request_tile_data(x, y, 4, HeightReference::ClampToTerrain)
            .show
    );
    assert!(
        !provider
            .request_tile_data(x, y, 4, HeightReference::ClampTo3DTiles)
            .show
    );
}

#[test]
fn bakes_a_3d_tiles_clamped_collection_only_for_model_targets() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(
        None,
        None,
        Some(HeightReference::ClampTo3DTiles),
        None,
        None,
    );
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampTo3DTiles);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    assert!(
        !provider
            .request_tile_data(x, y, 4, HeightReference::ClampToTerrain)
            .show
    );
    assert!(
        provider
            .request_tile_data(x, y, 4, HeightReference::ClampTo3DTiles)
            .show
    );
}

#[test]
fn bakes_a_ground_clamped_collection_for_both_terrain_and_model_targets() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(
        None,
        None,
        Some(HeightReference::ClampToGround),
        None,
        None,
    );
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToGround);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    assert!(
        provider
            .request_tile_data(x, y, 4, HeightReference::ClampToTerrain)
            .show
    );
    assert!(
        provider
            .request_tile_data(x, y, 4, HeightReference::ClampTo3DTiles)
            .show
    );
}

#[test]
fn prunes_a_collection_once_a_frame_passes_without_it_being_marked() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let coll = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&coll, 0, HeightReference::ClampToTerrain);

    let (x, y) = tiling_scheme.position_to_tile(line_midpoint(), 4).unwrap();
    assert!(
        provider
            .request_tile_data(x, y, 4, HeightReference::ClampToTerrain)
            .show
    );

    provider.update(Some(1));
    assert!(
        provider
            .request_tile_data(x, y, 4, HeightReference::ClampToTerrain)
            .show
    );

    provider.update(Some(2));
    assert!(
        !provider
            .request_tile_data(x, y, 4, HeightReference::ClampToTerrain)
            .show
    );
}

#[test]
fn keeps_baked_rectangle_data_when_only_a_non_overlapping_collection_changes() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let near = create_polyline_collection(None, None, None, None, None);
    let far = create_polyline_collection(Some(100.0), Some(-40.0), None, None, None);
    provider.mark_for_frame_polyline(&near, 0, HeightReference::ClampToTerrain);
    provider.mark_for_frame_polyline(&far, 0, HeightReference::ClampToTerrain);

    let near_rect = GeoRectangle::from_degrees(-105.0, 35.0, -85.0, 45.0);
    let far_rect = GeoRectangle::from_degrees(90.0, -45.0, 110.0, -35.0);

    let data = provider.request_data_for_rectangle(&near_rect, HeightReference::ClampToTerrain);
    provider.request_data_for_rectangle(&far_rect, HeightReference::ClampToTerrain);
    assert!(data.show);

    far.set_polyline_positions(0, polyline_positions(100.0, -41.0));
    provider.update(None);

    let updated = provider.update_data_for_rectangle(
        &near_rect,
        &data,
        HeightReference::ClampToTerrain,
    );
    assert!(Arc::ptr_eq(&updated, &data));
}

#[test]
fn re_bakes_rectangle_data_when_an_overlapping_collection_changes() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let near = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&near, 0, HeightReference::ClampToTerrain);

    let near_rect = GeoRectangle::from_degrees(-105.0, 35.0, -85.0, 45.0);
    let data = provider.request_data_for_rectangle(&near_rect, HeightReference::ClampToTerrain);

    near.set_polyline_positions(0, polyline_positions(-95.0, 41.0));
    provider.update(None);

    let updated = provider.update_data_for_rectangle(
        &near_rect,
        &data,
        HeightReference::ClampToTerrain,
    );
    assert!(!Arc::ptr_eq(&updated, &data));
    assert!(updated.show);
}

#[test]
fn re_bakes_rectangle_data_when_the_rectangle_changes() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let near = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&near, 0, HeightReference::ClampToTerrain);

    let near_rect = GeoRectangle::from_degrees(-105.0, 35.0, -85.0, 45.0);
    let data = provider.request_data_for_rectangle(&near_rect, HeightReference::ClampToTerrain);
    assert!(data.show);

    let shifted_rect = GeoRectangle::from_degrees(-106.0, 35.0, -86.0, 45.0);
    let updated = provider.update_data_for_rectangle(
        &shifted_rect,
        &data,
        HeightReference::ClampToTerrain,
    );
    assert!(!Arc::ptr_eq(&updated, &data));
    assert_eq!(updated.rectangle, shifted_rect);
}

#[test]
fn re_bakes_rectangle_data_when_a_collection_it_was_baked_from_is_removed() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let near = create_polyline_collection(None, None, None, None, None);
    provider.mark_for_frame_polyline(&near, 0, HeightReference::ClampToTerrain);

    let near_rect = GeoRectangle::from_degrees(-105.0, 35.0, -85.0, 45.0);
    let data = provider.request_data_for_rectangle(&near_rect, HeightReference::ClampToTerrain);
    provider.remove_polyline(&near);

    let updated = provider.update_data_for_rectangle(
        &near_rect,
        &data,
        HeightReference::ClampToTerrain,
    );
    assert!(!Arc::ptr_eq(&updated, &data));
    assert!(!updated.show);
}

#[test]
fn re_bakes_rectangle_data_when_a_collection_moves_into_the_rectangle() {
    let tiling_scheme = GeographicTilingScheme::default();
    let mut provider = VectorProvider::new(VectorProviderOptions {
        tiling_scheme,
        antialias: None,
        minimum_tile_screen_pixels: None,
    });
    let far = create_polyline_collection(Some(100.0), Some(-40.0), None, None, None);
    provider.mark_for_frame_polyline(&far, 0, HeightReference::ClampToTerrain);

    let near_rect = GeoRectangle::from_degrees(-105.0, 35.0, -85.0, 45.0);
    let data = provider.request_data_for_rectangle(&near_rect, HeightReference::ClampToTerrain);
    assert!(!data.show);

    far.set_polyline_positions(0, polyline_positions(-95.0, 40.0));

    let updated = provider.update_data_for_rectangle(
        &near_rect,
        &data,
        HeightReference::ClampToTerrain,
    );
    assert!(!Arc::ptr_eq(&updated, &data));
    assert!(updated.show);
}
