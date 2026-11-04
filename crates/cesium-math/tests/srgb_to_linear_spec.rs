use cesium_math::srgb_to_linear;

#[test]
fn test_converts_0_to_0() {
    assert_eq!(srgb_to_linear(0.0), 0.0);
}

#[test]
fn test_converts_value_less_than_0_04045() {
    let result = srgb_to_linear(0.0386);
    assert_eq!(format!("{:.3}", result), "0.003");
}

#[test]
fn test_converts_value_greater_than_0_04045() {
    let result = srgb_to_linear(0.5);
    assert_eq!(format!("{:.3}", result), "0.214");
}

#[test]
fn test_converts_1_to_1() {
    assert_eq!(srgb_to_linear(1.0), 1.0);
}
