use cesium_math::{Color, JulianDate};
use cesium_renderer::{Cesium3DTilesetHeatmap, HeatmapTile};

fn verify_color(tile_color: Color, expected_color: Color) {
    let threshold = 0.11;
    assert!((expected_color.red - tile_color.red).abs() < threshold);
    assert!((expected_color.green - tile_color.green).abs() < threshold);
    assert!((expected_color.blue - tile_color.blue).abs() < threshold);
}

#[test]
fn reset_minimum_maximum() {
    let mut heatmap = Cesium3DTilesetHeatmap::new("_centerZDepth");
    heatmap.minimum = -1.0;
    heatmap.maximum = 1.0;
    heatmap.reset_minimum_maximum();

    assert_eq!(heatmap.minimum, f64::MAX);
    assert_eq!(heatmap.maximum, -f64::MAX);
    assert_eq!(heatmap.previous_minimum, -1.0);
    assert_eq!(heatmap.previous_maximum, 1.0);
}

#[test]
fn uses_reference_minimum_maximum() {
    let tile_property_name = "_loadTimestamp";
    let mut heatmap = Cesium3DTilesetHeatmap::new(tile_property_name);

    let ref_min_jd = JulianDate::now();
    let ref_max_jd = ref_min_jd.add_seconds(10.0);

    heatmap.set_reference_minimum_maximum(ref_min_jd, ref_max_jd, tile_property_name);
    let ref_min = heatmap.reference_minimum[tile_property_name];
    let ref_max = heatmap.reference_maximum[tile_property_name];

    heatmap.minimum = -1.0;
    heatmap.maximum = 1.0;
    heatmap.reset_minimum_maximum();

    assert_eq!(heatmap.minimum, f64::MAX);
    assert_eq!(heatmap.maximum, -f64::MAX);
    assert_eq!(heatmap.previous_minimum, ref_min);
    assert_eq!(heatmap.previous_maximum, ref_max);
}

#[test]
fn expected_color() {
    let mut heatmap = Cesium3DTilesetHeatmap::new("_centerZDepth");
    let frame_number = 1;

    let mut tile = HeatmapTile {
        content_available: true,
        selected_frame: frame_number,
        debug_color: Color::WHITE,
        center_z_depth: Some(1.0),
        ..Default::default()
    };
    let original_color = tile.debug_color;

    // First frame: previous_minimum/maximum are uninitialized so no coloring occurs
    heatmap.colorize(&mut tile, frame_number);
    tile.center_z_depth = Some(-1.0);
    heatmap.colorize(&mut tile, frame_number);

    assert_eq!(heatmap.minimum, -1.0);
    assert_eq!(heatmap.maximum, 1.0);
    verify_color(tile.debug_color, original_color);

    // Next frame: previous_minimum/maximum take current frame's values
    heatmap.reset_minimum_maximum();
    tile.center_z_depth = Some(-1.0);
    heatmap.colorize(&mut tile, frame_number);

    verify_color(tile.debug_color, Color::BLACK);
}
