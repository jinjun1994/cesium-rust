//! Direct port and verification against CesiumJS `EntityClusterSpec.js` (Apache-2.0).

use cesium_math::{BoundingRectangle, EntityCluster, EntityClusterOptions};

#[test]
fn test_constructor_sets_default_properties() {
    let mut cluster = EntityCluster::default();
    assert!(!cluster.enabled);
    assert!(cluster.show);
    assert_eq!(cluster.pixel_range, 80.0);
    assert_eq!(cluster.minimum_cluster_size, 2);
    assert!(cluster.cluster_billboards);
    assert!(cluster.cluster_labels);
    assert!(cluster.cluster_points);

    cluster.enabled = true;
    assert!(cluster.enabled);

    cluster.pixel_range = 30.0;
    assert_eq!(cluster.pixel_range, 30.0);

    cluster.minimum_cluster_size = 5;
    assert_eq!(cluster.minimum_cluster_size, 5);
}

#[test]
fn test_constructor_sets_expected_properties() {
    let options = EntityClusterOptions {
        enabled: true,
        show: false,
        pixel_range: 30.0,
        minimum_cluster_size: 5,
        cluster_billboards: false,
        cluster_labels: false,
        cluster_points: false,
    };
    let cluster = EntityCluster::new(options);
    assert!(cluster.enabled);
    assert!(!cluster.show);
    assert_eq!(cluster.pixel_range, 30.0);
    assert_eq!(cluster.minimum_cluster_size, 5);
    assert!(!cluster.cluster_billboards);
    assert!(!cluster.cluster_labels);
    assert!(!cluster.cluster_points);
}

#[test]
fn test_expand_bounding_box() {
    let mut bbox = BoundingRectangle::new(10.0, 20.0, 30.0, 40.0);
    EntityCluster::expand_bounding_box(&mut bbox, 10.0);
    assert_eq!(bbox.x, 0.0);
    assert_eq!(bbox.y, 10.0);
    assert_eq!(bbox.width, 50.0);
    assert_eq!(bbox.height, 60.0);
}
