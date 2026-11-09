use cesium_math::{Cartesian3, DeveloperError, GeoRectangle};
use cesium_renderer::{
    ClippingPolygon, ClippingPolygonCollection, ClippingPolygonCollectionOptions,
    ClippingPolygonOptions, Intersect,
};

fn sample_positions() -> Vec<Cartesian3> {
    Cartesian3::from_radians_array(
        &[
            -1.3194369277314022,
            0.6988062530900625,
            -1.31941,
            0.69879,
            -1.3193955980204217,
            0.6988091578771254,
            -1.3193931220959367,
            0.698743632490865,
            -1.3194358224045408,
            0.6987471965556998,
        ],
        None,
    )
    .unwrap()
}

fn sample_hole() -> Vec<Cartesian3> {
    Cartesian3::from_radians_array(
        &[
            -1.31942, 0.69879, -1.319405, 0.69879, -1.319412, 0.698763,
        ],
        None,
    )
    .unwrap()
}

#[test]
fn test_default_constructor() {
    let polygons = ClippingPolygonCollection::default();
    assert_eq!(polygons.length(), 0);
    assert!(polygons.enabled);
    assert!(!polygons.inverse);
    assert_eq!(polygons.quality, 1.0);
}

#[test]
fn test_gets_length_of_list_of_polygons() {
    let mut polygons = ClippingPolygonCollection::default();
    assert_eq!(polygons.length(), 0);

    let poly1 = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(sample_positions()),
        ..Default::default()
    })
    .unwrap();
    let poly2 = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(sample_positions()),
        ..Default::default()
    })
    .unwrap();

    polygons.add(poly1.clone());
    polygons.add(poly2);
    assert_eq!(polygons.length(), 2);

    polygons.remove(&poly1);
    assert_eq!(polygons.length(), 1);
}

#[test]
fn test_clipping_polygons_state() {
    let mut polygons = ClippingPolygonCollection::default();
    assert_eq!(polygons.clipping_polygons_state(), 0);

    polygons.add(
        ClippingPolygon::new(ClippingPolygonOptions {
            positions: Some(sample_positions()),
            ..Default::default()
        })
        .unwrap(),
    );
    assert_eq!(polygons.clipping_polygons_state(), 1);

    polygons.inverse = true;
    assert_eq!(polygons.clipping_polygons_state(), -1);

    polygons.inverse = false;
    polygons.enabled = false;
    assert_eq!(polygons.clipping_polygons_state(), 0);
}

#[test]
fn test_packs_holes() {
    let poly = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(sample_positions()),
        holes: Some(vec![sample_hole()]),
        ..Default::default()
    })
    .unwrap();

    let polygons = ClippingPolygonCollection::new(ClippingPolygonCollectionOptions {
        polygons: Some(vec![poly]),
        ..Default::default()
    });

    assert_eq!(polygons.hole_count(), 1);
    assert_eq!(
        polygons.vertex_count(),
        sample_positions().len() + sample_hole().len()
    );
}

#[test]
fn test_contains_and_remove() {
    let poly1 = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(sample_positions()),
        ..Default::default()
    })
    .unwrap();
    let poly2 = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(sample_positions()),
        ..Default::default()
    })
    .unwrap();

    let mut collection = ClippingPolygonCollection::default();
    collection.add(poly1.clone());

    assert!(collection.contains(&poly1));
    assert!(!collection.contains(&poly2));

    assert!(collection.remove(&poly1));
    assert_eq!(collection.length(), 0);
}

#[test]
fn test_remove_all() {
    let mut collection = ClippingPolygonCollection::default();
    collection.add(
        ClippingPolygon::new(ClippingPolygonOptions {
            positions: Some(sample_positions()),
            ..Default::default()
        })
        .unwrap(),
    );
    collection.add(
        ClippingPolygon::new(ClippingPolygonOptions {
            positions: Some(sample_positions()),
            ..Default::default()
        })
        .unwrap(),
    );
    assert_eq!(collection.length(), 2);

    collection.remove_all();
    assert_eq!(collection.length(), 0);
}

#[test]
fn test_computes_intersection_with_bounding_volume() {
    let poly = ClippingPolygon::new(ClippingPolygonOptions {
        positions: Some(sample_positions()),
        ..Default::default()
    })
    .unwrap();

    let collection = ClippingPolygonCollection::new(ClippingPolygonCollectionOptions {
        polygons: Some(vec![poly.clone()]),
        ..Default::default()
    });

    let rect_overlapping = poly.rectangle();
    assert_eq!(
        collection.compute_intersection_with_rectangle(&rect_overlapping),
        Intersect::Intersecting
    );

    let far_rect = GeoRectangle {
        west: 2.0,
        south: -1.0,
        east: 2.5,
        north: -0.5,
    };
    assert_eq!(
        collection.compute_intersection_with_rectangle(&far_rect),
        Intersect::Outside
    );
}

#[test]
fn test_request_rectangle_data_throws_without_rectangle() {
    let collection = ClippingPolygonCollection::default();
    let err = collection
        .validate_request_data_params(None)
        .expect_err("should throw without rectangle");
    assert_eq!(err.name(), DeveloperError::NAME);
}
