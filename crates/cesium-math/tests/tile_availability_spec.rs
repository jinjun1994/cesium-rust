use std::f64::consts::PI;

use cesium_math::{
    Cartographic, GeoRectangle, GeographicTilingScheme, TileAvailability, TileScheme,
    WebMercatorTilingScheme,
};

fn create_availability(scheme: TileScheme, max_level: u32) -> TileAvailability {
    let mut availability = TileAvailability::new(scheme, max_level);
    let nx = scheme.number_of_x_tiles_at_level(0);
    let ny = scheme.number_of_y_tiles_at_level(0);
    availability.add_available_tile_range(0, 0, 0, nx - 1, ny - 1);
    availability
}

fn from_degrees_carto(lon: f64, lat: f64) -> Cartographic {
    Cartographic::degrees(lon, lat, 0.0).expect("valid degrees")
}

fn from_degrees_rect(w: f64, s: f64, e: f64, n: f64) -> GeoRectangle {
    GeoRectangle::from_degrees(w, s, e, n)
}

#[test]
fn test_returns_neg_1_if_position_outside_tiling_scheme() {
    let web_mercator = TileScheme::WebMercator(WebMercatorTilingScheme::default());
    let availability = create_availability(web_mercator, 15);
    // 88 degrees latitude is outside Web Mercator bounds (~85.05 degrees)
    let pos = from_degrees_carto(25.0, 88.0);
    assert_eq!(availability.compute_maximum_level_at_position(pos), -1);
}

#[test]
fn test_returns_0_if_there_are_no_rectangles() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let availability = create_availability(geo, 15);
    let pos = from_degrees_carto(25.0, 88.0);
    assert_eq!(availability.compute_maximum_level_at_position(pos), 0);
}

#[test]
fn test_returns_higher_level_when_on_boundary_at_level_0() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = create_availability(geo, 15);
    availability.add_available_tile_range(0, 0, 0, 0, 0);
    availability.add_available_tile_range(1, 1, 0, 1, 0);
    let center = Cartographic::from_radians(0.0, 0.0);
    assert_eq!(availability.compute_maximum_level_at_position(center), 1);

    // Make sure it isn't dependent on the order we add the rectangles.
    let mut availability2 = create_availability(geo, 15);
    availability2.add_available_tile_range(1, 1, 0, 1, 0);
    availability2.add_available_tile_range(0, 0, 0, 0, 0);
    assert_eq!(availability2.compute_maximum_level_at_position(center), 1);
}

#[test]
fn test_returns_higher_level_when_on_boundary_at_level_1() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = create_availability(geo, 15);
    availability.add_available_tile_range(0, 0, 0, 1, 0);
    availability.add_available_tile_range(1, 1, 1, 1, 1);
    let pos = Cartographic::from_radians(-PI / 2.0, 0.0);
    assert_eq!(availability.compute_maximum_level_at_position(pos), 1);
}

#[test]
fn test_compute_best_available_level_over_rectangle_no_rectangles() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let availability = create_availability(geo, 15);
    let rect = from_degrees_rect(1.0, 2.0, 3.0, 4.0);
    assert_eq!(availability.compute_best_available_level_over_rectangle(rect), 0);
}

#[test]
fn test_reports_correct_level_when_entirely_inside_worldwide_rectangle() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = create_availability(geo, 15);
    let nx5 = geo.number_of_x_tiles_at_level(5);
    let ny5 = geo.number_of_y_tiles_at_level(5);
    availability.add_available_tile_range(5, 0, 0, nx5 - 1, ny5 - 1);
    availability.add_available_tile_range(6, 7, 8, 9, 10);

    let rect = from_degrees_rect(1.0, 2.0, 3.0, 4.0);
    assert_eq!(availability.compute_best_available_level_over_rectangle(rect), 5);
}

#[test]
fn test_reports_correct_level_when_entirely_inside_smaller_rectangle() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = create_availability(geo, 15);
    let nx5 = geo.number_of_x_tiles_at_level(5);
    let ny5 = geo.number_of_y_tiles_at_level(5);
    availability.add_available_tile_range(5, 0, 0, nx5 - 1, ny5 - 1);
    availability.add_available_tile_range(6, 7, 8, 9, 10);

    let rect = geo.tile_rectangle(8, 9, 6).expect("tile rectangle");
    assert_eq!(availability.compute_best_available_level_over_rectangle(rect), 6);
}

#[test]
fn test_reports_correct_level_when_partially_overlapping_smaller_rectangle() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = create_availability(geo, 15);
    let nx5 = geo.number_of_x_tiles_at_level(5);
    let ny5 = geo.number_of_y_tiles_at_level(5);
    availability.add_available_tile_range(5, 0, 0, nx5 - 1, ny5 - 1);
    availability.add_available_tile_range(6, 7, 8, 7, 8);

    let mut rect = geo.tile_rectangle(7, 8, 6).expect("tile rectangle");
    rect.west -= 0.01;
    rect.east += 0.01;
    rect.south -= 0.01;
    rect.north += 0.01;
    assert_eq!(availability.compute_best_available_level_over_rectangle(rect), 5);
}

#[test]
fn test_works_with_rectangle_crossing_180_degrees_longitude() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = create_availability(geo, 15);
    let nx5 = geo.number_of_x_tiles_at_level(5);
    let ny5 = geo.number_of_y_tiles_at_level(5);
    availability.add_available_tile_range(5, 0, 0, nx5 - 1, ny5 - 1);

    let nx6 = geo.number_of_x_tiles_at_level(6);
    let ny6 = geo.number_of_y_tiles_at_level(6);
    availability.add_available_tile_range(6, 0, 0, 10, ny6 - 1);
    availability.add_available_tile_range(6, nx6 - 11, 0, nx6 - 1, ny6 - 1);

    let rect1 = from_degrees_rect(179.0, 45.0, -179.0, 50.0);
    assert_eq!(availability.compute_best_available_level_over_rectangle(rect1), 6);

    let rect2 = from_degrees_rect(45.0, 45.0, -45.0, 50.0);
    assert_eq!(availability.compute_best_available_level_over_rectangle(rect2), 5);
}

#[test]
fn test_works_when_four_rectangles_combine_to_cover_the_area() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = create_availability(geo, 15);
    let nx5 = geo.number_of_x_tiles_at_level(5);
    let ny5 = geo.number_of_y_tiles_at_level(5);
    availability.add_available_tile_range(5, 0, 0, nx5 - 1, ny5 - 1);
    availability.add_available_tile_range(6, 0, 2, 1, 3);
    availability.add_available_tile_range(6, 2, 0, 3, 1);
    availability.add_available_tile_range(6, 0, 0, 1, 1);
    availability.add_available_tile_range(6, 2, 2, 3, 3);

    let rect = geo.tile_rectangle(0, 0, 4).expect("tile rectangle");
    assert_eq!(availability.compute_best_available_level_over_rectangle(rect), 6);
}

#[test]
fn test_ensure_boundary_rectangles_are_sorted_properly() {
    let geo = TileScheme::Geographic(GeographicTilingScheme::default());
    let mut availability = TileAvailability::new(geo, 6);
    availability.add_available_tile_range(0, 0, 0, 1, 0);
    availability.add_available_tile_range(1, 0, 0, 2, 0);
    availability.add_available_tile_range(2, 0, 0, 4, 0);
    availability.add_available_tile_range(3, 0, 0, 8, 0);
    availability.add_available_tile_range(0, 0, 0, 1, 0);

    for node in &availability.nodes {
        let rects = &node.rectangles;
        for i in 0..rects.len() {
            for j in i..rects.len() {
                assert!(rects[i].level <= rects[j].level);
            }
        }
    }
}
