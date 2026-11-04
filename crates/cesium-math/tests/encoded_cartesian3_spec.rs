use cesium_math::{Cartesian3, EncodedCartesian3};

#[test]
fn test_encoded_cartesian3_defaults() {
    let encoded = EncodedCartesian3::default();
    assert_eq!(encoded.high, Cartesian3::ZERO);
    assert_eq!(encoded.low, Cartesian3::ZERO);
}

#[test]
fn test_encoded_cartesian3_encode() {
    let (h_pos, l_pos) = EncodedCartesian3::encode(10000000.0);
    assert_eq!(h_pos + l_pos, 10000000.0);

    let (h_neg, l_neg) = EncodedCartesian3::encode(-10000000.0);
    assert_eq!(h_neg + l_neg, -10000000.0);

    let (h_zero, l_zero) = EncodedCartesian3::encode(0.0);
    assert_eq!(h_zero, 0.0);
    assert_eq!(l_zero, 0.0);
}

#[test]
fn test_encoded_cartesian3_from_cartesian() {
    let c = Cartesian3::new(-10000000.0, 0.0, 10000000.0);
    let encoded = EncodedCartesian3::from_cartesian(c);

    assert_eq!(encoded.high.x + encoded.low.x, -10000000.0);
    assert_eq!(encoded.high.y + encoded.low.y, 0.0);
    assert_eq!(encoded.high.z + encoded.low.z, 10000000.0);
}

#[test]
fn test_encoded_cartesian3_write_elements() {
    let p = Cartesian3::new(-10000000.0, 0.0, 10000000.0);
    let mut positions = [0.0f32; 6];
    EncodedCartesian3::write_elements(p, &mut positions, 0);

    let encoded = EncodedCartesian3::from_cartesian(p);
    assert_eq!(positions[0], encoded.high.x as f32);
    assert_eq!(positions[1], encoded.high.y as f32);
    assert_eq!(positions[2], encoded.high.z as f32);
    assert_eq!(positions[3], encoded.low.x as f32);
    assert_eq!(positions[4], encoded.low.y as f32);
    assert_eq!(positions[5], encoded.low.z as f32);
}
