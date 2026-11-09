use cesium_math::ComponentDatatype;
use cesium_renderer::Cesium3DTileFeatureTable;
use serde_json::json;

#[test]
fn test_loads_from_json() {
    let mut feature_table = Cesium3DTileFeatureTable::new(
        json!({
            "TEST": [0, 1, 2, 3, 4, 5]
        }),
        None,
    );
    feature_table.features_length = 3;

    assert!(feature_table.has_property("TEST"));

    let all = feature_table
        .get_global_property("TEST", Some(ComponentDatatype::UnsignedByte), None)
        .expect("should get global property");
    assert_eq!(all, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);

    let mut feature = vec![0.0; 2];
    feature_table
        .get_property(
            "TEST",
            Some(ComponentDatatype::UnsignedByte),
            2,
            1,
            &mut feature,
        )
        .expect("should get property for feature 1");
    assert_eq!(feature, vec![2.0, 3.0]);

    let properties = feature_table
        .get_property_array("TEST", Some(ComponentDatatype::UnsignedByte), 2)
        .expect("should get property array");
    assert_eq!(properties, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn test_loads_from_binary() {
    let mut feature_table = Cesium3DTileFeatureTable::new(
        json!({
            "TEST": {
                "byteOffset": 4
            }
        }),
        Some(vec![0, 0, 0, 0, 0, 1, 2, 3, 4, 5]),
    );
    feature_table.features_length = 3;

    assert!(feature_table.has_property("TEST"));

    let all = feature_table
        .get_global_property("TEST", Some(ComponentDatatype::UnsignedByte), Some(6))
        .expect("should get global property");
    assert_eq!(all, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);

    let mut feature = vec![0.0; 2];
    feature_table
        .get_property(
            "TEST",
            Some(ComponentDatatype::UnsignedByte),
            2,
            1,
            &mut feature,
        )
        .expect("should get property for feature 1");
    assert_eq!(feature, vec![2.0, 3.0]);

    let properties = feature_table
        .get_property_array("TEST", Some(ComponentDatatype::UnsignedByte), 2)
        .expect("should get property array");
    assert_eq!(properties, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
}
