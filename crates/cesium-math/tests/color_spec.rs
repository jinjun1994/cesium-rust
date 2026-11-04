use cesium_math::Color;

#[test]
fn test_color_constants_and_constructors() {
    let white = Color::WHITE;
    assert_eq!(white.red, 1.0);
    assert_eq!(white.green, 1.0);
    assert_eq!(white.blue, 1.0);
    assert_eq!(white.alpha, 1.0);

    let red = Color::RED;
    assert_eq!(red.red, 1.0);
    assert_eq!(red.green, 0.0);
    assert_eq!(red.blue, 0.0);

    let from_bytes = Color::from_bytes(255, 128, 0, 255);
    assert_eq!(from_bytes.red, 1.0);
    assert!((from_bytes.green - 128.0 / 255.0).abs() < 1e-6);
    assert_eq!(from_bytes.blue, 0.0);
    assert_eq!(from_bytes.alpha, 1.0);

    let bytes = from_bytes.to_bytes();
    assert_eq!(bytes, [255, 128, 0, 255]);
}

#[test]
fn test_color_rgba_u32() {
    let color = Color::from_bytes(0x12, 0x34, 0x56, 0x78);
    let rgba_num = color.to_rgba();
    assert_eq!(rgba_num, 0x12345678);

    let recovered = Color::from_rgba(0x12345678);
    assert_eq!(color.to_bytes(), recovered.to_bytes());
}

#[test]
fn test_color_from_css_strings() {
    let c1 = Color::from_css_color_string("#fff").unwrap();
    assert_eq!(c1, Color::WHITE);

    let c2 = Color::from_css_color_string("#FF0000").unwrap();
    assert_eq!(c2.red, 1.0);
    assert_eq!(c2.green, 0.0);
    assert_eq!(c2.blue, 0.0);

    let c3 = Color::from_css_color_string("rgb(255, 0, 0)").unwrap();
    assert_eq!(c3.red, 1.0);
    assert_eq!(c3.green, 0.0);
    assert_eq!(c3.blue, 0.0);
    assert_eq!(c3.alpha, 1.0);

    let c4 = Color::from_css_color_string("rgba(0, 255, 0, 0.5)").unwrap();
    assert_eq!(c4.red, 0.0);
    assert_eq!(c4.green, 1.0);
    assert_eq!(c4.blue, 0.0);
    assert!((c4.alpha - 0.5).abs() < 1e-4);
}

#[test]
fn test_brighten_darken_and_pack() {
    let red = Color::new(0.5, 0.0, 0.0, 1.0);
    let brightened = red.brighten(0.5);
    assert!((brightened.red - 0.75).abs() < 1e-6);

    let darkened = red.darken(0.5);
    assert!((darkened.red - 0.25).abs() < 1e-6);

    let mut packed = [0.0; 4];
    red.pack(&mut packed, 0).unwrap();
    assert_eq!(packed, [0.5, 0.0, 0.0, 1.0]);
    let unpacked = Color::unpack(&[1.0, 0.5, 0.0, 0.0, 1.0], 1).unwrap();
    assert_eq!(unpacked, red);
}
