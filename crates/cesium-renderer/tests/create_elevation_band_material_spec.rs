use cesium_math::Color;
use cesium_renderer::{
    create_elevation_band_material, ElevationBandEntry, ElevationBandLayer,
    ELEVATION_BAND_MAXIMUM_HEIGHT, ELEVATION_BAND_MINIMUM_HEIGHT,
};

#[test]
fn test_throws_without_layers() {
    assert!(create_elevation_band_material(&[]).is_err());
}

#[test]
fn test_throws_with_no_entries() {
    let layers = vec![ElevationBandLayer {
        entries: vec![],
        ..Default::default()
    }];
    assert!(create_elevation_band_material(&layers).is_err());
}

#[test]
fn test_throws_with_no_height() {
    let layers = vec![ElevationBandLayer {
        entries: vec![ElevationBandEntry {
            height: None,
            color: Some(Color::RED),
        }],
        ..Default::default()
    }];
    assert!(create_elevation_band_material(&layers).is_err());
}

#[test]
fn test_throws_with_no_color() {
    let layers = vec![ElevationBandLayer {
        entries: vec![ElevationBandEntry {
            height: Some(0.0),
            color: None,
        }],
        ..Default::default()
    }];
    assert!(create_elevation_band_material(&layers).is_err());
}

#[test]
fn test_creates_material_with_one_entry() {
    let layers = vec![ElevationBandLayer {
        entries: vec![ElevationBandEntry::new(0.0, Color::RED)],
        ..Default::default()
    }];

    let res = create_elevation_band_material(&layers).expect("valid material");
    assert_eq!(res.height_data.len(), 2);
    assert_eq!(res.height_data[0], ELEVATION_BAND_MINIMUM_HEIGHT);
    assert_eq!(res.height_data[1], ELEVATION_BAND_MAXIMUM_HEIGHT);

    // Color is premultiplied RED: [255, 0, 0, 255]
    assert_eq!(&res.color_data[0..4], &[255, 0, 0, 255]);
    assert_eq!(&res.color_data[4..8], &[255, 0, 0, 255]);
}

#[test]
fn test_creates_material_with_one_entry_that_extends_upwards() {
    let layers = vec![ElevationBandLayer {
        entries: vec![ElevationBandEntry::new(0.0, Color::RED)],
        extend_upwards: Some(true),
        ..Default::default()
    }];

    let res = create_elevation_band_material(&layers).expect("valid material");
    assert_eq!(res.height_data.len(), 2);
    assert_eq!(res.height_data[0], 0.0);
    assert_eq!(res.height_data[1], ELEVATION_BAND_MAXIMUM_HEIGHT);
}

#[test]
fn test_creates_material_with_one_entry_that_extends_downwards() {
    let layers = vec![ElevationBandLayer {
        entries: vec![ElevationBandEntry::new(0.0, Color::RED)],
        extend_downwards: Some(true),
        ..Default::default()
    }];

    let res = create_elevation_band_material(&layers).expect("valid material");
    assert_eq!(res.height_data.len(), 2);
    assert_eq!(res.height_data[0], ELEVATION_BAND_MINIMUM_HEIGHT);
    assert_eq!(res.height_data[1], 0.0);
}

#[test]
fn test_creates_material_with_one_entry_that_extends_upwards_and_downwards() {
    let layers = vec![ElevationBandLayer {
        entries: vec![ElevationBandEntry::new(0.0, Color::RED)],
        extend_downwards: Some(true),
        extend_upwards: Some(true),
    }];

    let res = create_elevation_band_material(&layers).expect("valid material");
    assert_eq!(res.height_data.len(), 2);
    assert_eq!(res.height_data[0], ELEVATION_BAND_MINIMUM_HEIGHT);
    assert_eq!(res.height_data[1], ELEVATION_BAND_MAXIMUM_HEIGHT);
}

#[test]
fn test_removes_unused_entries() {
    let layers = vec![ElevationBandLayer {
        entries: vec![
            ElevationBandEntry::new(3.0, Color::new(1.0, 0.0, 0.0, 1.0)),
            ElevationBandEntry::new(3.0, Color::new(0.0, 1.0, 0.0, 1.0)),
            ElevationBandEntry::new(2.0, Color::new(0.0, 1.0, 0.0, 1.0)),
            ElevationBandEntry::new(1.0, Color::new(0.0, 1.0, 0.0, 1.0)),
        ],
        extend_downwards: Some(false),
        extend_upwards: Some(false),
    }];

    let res = create_elevation_band_material(&layers).expect("valid material");
    // Duplicate height/color combinations cleaned up
    assert!(res.height_data.len() < 4);
}
