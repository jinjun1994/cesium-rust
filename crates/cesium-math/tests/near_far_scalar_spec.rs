use cesium_math::NearFarScalar;

#[test]
fn test_near_far_scalar_upstream() {
    let s = NearFarScalar::new(10.0, 1.0, 100.0, 0.0);
    assert_eq!(s.near, 10.0);
    assert_eq!(s.near_value, 1.0);
    assert_eq!(s.far, 100.0);
    assert_eq!(s.far_value, 0.0);

    let mut packed = [0.0; 4];
    s.pack(&mut packed, 0).unwrap();
    assert_eq!(packed, [10.0, 1.0, 100.0, 0.0]);

    let unpacked = NearFarScalar::unpack(&[0.0, 10.0, 1.0, 100.0, 0.0], 1).unwrap();
    assert_eq!(unpacked, s);
    assert!(unpacked.equals(&s));
    assert!(unpacked.equals_epsilon(&s, 1e-12));
}
