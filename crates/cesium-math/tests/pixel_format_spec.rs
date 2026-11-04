use cesium_math::{PixelDatatype, PixelFormat};

#[test]
fn test_pixel_format_flip_y() {
    let width = 1;
    let height = 2;
    let values = [255u8, 0, 0, 0, 255, 0];
    let expected = [0u8, 255, 0, 255, 0, 0];

    let flipped = PixelFormat::flip_y(
        &values,
        PixelFormat::RGB,
        PixelDatatype::UNSIGNED_BYTE,
        width,
        height,
    );
    assert_eq!(flipped, expected);
}

#[test]
fn test_pixel_format_flip_y_early_return_height_1() {
    let width = 1;
    let height = 1;
    let values = [255u8, 255, 255];

    let flipped = PixelFormat::flip_y(
        &values,
        PixelFormat::RGB,
        PixelDatatype::UNSIGNED_BYTE,
        width,
        height,
    );
    assert_eq!(flipped, values);
}

#[test]
fn test_pixel_format_to_internal_format_webgl1() {
    let webgl2 = false;

    assert_eq!(
        PixelFormat::RED.to_internal_format(PixelDatatype::FLOAT, webgl2),
        PixelFormat::RED.as_gl_enum()
    );
    assert_eq!(
        PixelFormat::RG.to_internal_format(PixelDatatype::FLOAT, webgl2),
        PixelFormat::RG.as_gl_enum()
    );
    assert_eq!(
        PixelFormat::RGB.to_internal_format(PixelDatatype::FLOAT, webgl2),
        PixelFormat::RGB.as_gl_enum()
    );
    assert_eq!(
        PixelFormat::RGBA.to_internal_format(PixelDatatype::FLOAT, webgl2),
        PixelFormat::RGBA.as_gl_enum()
    );
}

#[test]
fn test_pixel_format_to_internal_format_webgl2_float() {
    let webgl2 = true;

    // R32F = 0x822E
    assert_eq!(
        PixelFormat::RED.to_internal_format(PixelDatatype::FLOAT, webgl2),
        0x822E
    );
    // RG32F = 0x8230
    assert_eq!(
        PixelFormat::RG.to_internal_format(PixelDatatype::FLOAT, webgl2),
        0x8230
    );
    // RGB32F = 0x8815
    assert_eq!(
        PixelFormat::RGB.to_internal_format(PixelDatatype::FLOAT, webgl2),
        0x8815
    );
    // RGBA32F = 0x8814
    assert_eq!(
        PixelFormat::RGBA.to_internal_format(PixelDatatype::FLOAT, webgl2),
        0x8814
    );
}

#[test]
fn test_pixel_format_to_internal_format_webgl2_half_float() {
    let webgl2 = true;

    // R16F = 0x822D
    assert_eq!(
        PixelFormat::RED.to_internal_format(PixelDatatype::HALF_FLOAT, webgl2),
        0x822D
    );
    // RG16F = 0x822F
    assert_eq!(
        PixelFormat::RG.to_internal_format(PixelDatatype::HALF_FLOAT, webgl2),
        0x822F
    );
    // RGB16F = 0x881B
    assert_eq!(
        PixelFormat::RGB.to_internal_format(PixelDatatype::HALF_FLOAT, webgl2),
        0x881B
    );
    // RGBA16F = 0x881A
    assert_eq!(
        PixelFormat::RGBA.to_internal_format(PixelDatatype::HALF_FLOAT, webgl2),
        0x881A
    );
}

#[test]
fn test_pixel_format_to_internal_format_webgl2_unsigned_byte() {
    let webgl2 = true;

    // R8 = 0x8229
    assert_eq!(
        PixelFormat::RED.to_internal_format(PixelDatatype::UNSIGNED_BYTE, webgl2),
        0x8229
    );
    // RG8 = 0x822B
    assert_eq!(
        PixelFormat::RG.to_internal_format(PixelDatatype::UNSIGNED_BYTE, webgl2),
        0x822B
    );
    // RGB8 = 0x8051
    assert_eq!(
        PixelFormat::RGB.to_internal_format(PixelDatatype::UNSIGNED_BYTE, webgl2),
        0x8051
    );
    // RGBA8 = 0x8058
    assert_eq!(
        PixelFormat::RGBA.to_internal_format(PixelDatatype::UNSIGNED_BYTE, webgl2),
        0x8058
    );
}
