use cesium_renderer::{PixelDatatype, PixelFormat, Texture3D, Texture3DOptions};

#[test]
fn test_has_expected_default_values_for_pixel_format_and_datatype() {
    let texture = Texture3D::new(Texture3DOptions {
        width: 2,
        height: 2,
        depth: 2,
        pixel_format: None,
        pixel_datatype: None,
        sampler: None,
    })
    .unwrap();

    assert!(texture.id > 0);
    assert_eq!(texture.pixel_format, PixelFormat::Rgba);
    assert_eq!(texture.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(texture.width, 2);
    assert_eq!(texture.height, 2);
    assert_eq!(texture.depth, 2);
    assert_eq!(texture.size_in_bytes(), 2 * 2 * 2 * 4);
}

#[test]
fn test_can_get_the_size_in_bytes_of_a_texture() {
    fn check_byte_size(
        width: u32,
        height: u32,
        depth: u32,
        pixel_format: PixelFormat,
        pixel_datatype: PixelDatatype,
        expected_size: usize,
    ) {
        let tex = Texture3D::new(Texture3DOptions {
            width,
            height,
            depth,
            pixel_format: Some(pixel_format),
            pixel_datatype: Some(pixel_datatype),
            sampler: None,
        })
        .unwrap();
        assert_eq!(tex.size_in_bytes(), expected_size);
    }

    // Depth textures
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::DepthComponent,
        PixelDatatype::UnsignedShort,
        16 * 16 * 16 * 2,
    );
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::DepthComponent,
        PixelDatatype::UnsignedInt,
        16 * 16 * 16 * 4,
    );
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::DepthStencil,
        PixelDatatype::UnsignedInt24_8,
        16 * 16 * 16 * 4,
    );

    // Uncompressed formats
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::Alpha,
        PixelDatatype::UnsignedByte,
        16 * 16 * 16,
    );
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::Rgb,
        PixelDatatype::UnsignedByte,
        16 * 16 * 16 * 3,
    );
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::Rgba,
        PixelDatatype::UnsignedByte,
        16 * 16 * 16 * 4,
    );
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::Luminance,
        PixelDatatype::UnsignedByte,
        16 * 16 * 16,
    );
    check_byte_size(
        16,
        16,
        16,
        PixelFormat::LuminanceAlpha,
        PixelDatatype::UnsignedByte,
        16 * 16 * 16 * 2,
    );
}

#[test]
fn test_can_be_destroyed() {
    let mut tex = Texture3D::new(Texture3DOptions {
        width: 4,
        height: 4,
        depth: 4,
        pixel_format: Some(PixelFormat::Rgba),
        pixel_datatype: None,
        sampler: None,
    })
    .unwrap();

    assert!(!tex.is_destroyed());
    tex.destroy();
    assert!(tex.is_destroyed());
}

#[test]
fn test_throws_when_creating_without_valid_dimensions() {
    assert!(Texture3D::new(Texture3DOptions {
        width: 0,
        height: 4,
        depth: 4,
        pixel_format: None,
        pixel_datatype: None,
        sampler: None,
    })
    .is_err());
}
