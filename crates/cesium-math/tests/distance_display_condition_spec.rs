use cesium_math::DistanceDisplayCondition;

#[test]
fn test_distance_display_condition_default_constructs() {
    let dc = DistanceDisplayCondition::default();
    assert_eq!(dc.near, 0.0);
    assert_eq!(dc.far, f64::MAX);
}

#[test]
fn test_distance_display_condition_constructs_with_params() {
    let dc = DistanceDisplayCondition::new(10.0, 100.0);
    assert_eq!(dc.near, 10.0);
    assert_eq!(dc.far, 100.0);
}

#[test]
fn test_distance_display_condition_equals() {
    let dc = DistanceDisplayCondition::new(10.0, 100.0);
    assert!(dc.equals(&DistanceDisplayCondition::new(10.0, 100.0)));
    assert!(!dc.equals(&DistanceDisplayCondition::new(11.0, 100.0)));
    assert!(!dc.equals(&DistanceDisplayCondition::new(10.0, 101.0)));
}

#[test]
fn test_distance_display_condition_pack_unpack() {
    let dc = DistanceDisplayCondition::new(15.0, 250.0);
    let mut packed = [0.0; 2];
    dc.pack(&mut packed, 0).expect("pack failed");
    assert_eq!(packed, [15.0, 250.0]);

    let unpacked = DistanceDisplayCondition::unpack(&packed, 0).expect("unpack failed");
    assert_eq!(dc, unpacked);
}
