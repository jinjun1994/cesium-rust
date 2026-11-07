use cesium_renderer::{PixelDatatype, PixelFormat, Texture, TextureOptions};

#[test]
fn test_has_expected_default_values_for_pixel_format_and_datatype() {
    let texture = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    });

    assert!(texture.id > 0);
    assert_eq!(texture.pixel_format, PixelFormat::Rgba);
    assert_eq!(texture.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(texture.width, 1);
    assert_eq!(texture.height, 1);
    assert_eq!(texture.size_in_bytes(), 4);
}

#[test]
fn test_texture_size_in_bytes_for_formats() {
    let rgb_tex = Texture::new(TextureOptions {
        width: 10,
        height: 20,
        pixel_format: Some(PixelFormat::Rgb),
        pixel_datatype: Some(PixelDatatype::UnsignedByte),
        ..Default::default()
    });
    assert_eq!(rgb_tex.size_in_bytes(), 10 * 20 * 3);

    let depth_tex = Texture::new(TextureOptions {
        width: 16,
        height: 16,
        pixel_format: Some(PixelFormat::DepthComponent),
        pixel_datatype: Some(PixelDatatype::UnsignedInt),
        ..Default::default()
    });
    assert_eq!(depth_tex.size_in_bytes(), 256);
}

#[test]
fn test_destroys() {
    let mut texture = Texture::new(TextureOptions {
        width: 4,
        height: 4,
        ..Default::default()
    });
    assert!(!texture.is_destroyed());
    texture.destroy();
    assert!(texture.is_destroyed());
}
