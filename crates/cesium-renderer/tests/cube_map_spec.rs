use cesium_renderer::{
    CubeMap, CubeMapOptions, PixelDatatype, PixelFormat, Sampler, SamplerOptions,
    TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};

#[test]
fn test_gets_pixel_format_and_datatype() {
    let cube_map = CubeMap::new(CubeMapOptions {
        width: 16,
        height: 16,
        ..Default::default()
    })
    .unwrap();

    assert_eq!(cube_map.pixel_format, PixelFormat::Rgba);
    assert_eq!(cube_map.positive_x.pixel_format, PixelFormat::Rgba);
    assert_eq!(cube_map.negative_x.pixel_format, PixelFormat::Rgba);
    assert_eq!(cube_map.positive_y.pixel_format, PixelFormat::Rgba);
    assert_eq!(cube_map.negative_y.pixel_format, PixelFormat::Rgba);
    assert_eq!(cube_map.positive_z.pixel_format, PixelFormat::Rgba);
    assert_eq!(cube_map.negative_z.pixel_format, PixelFormat::Rgba);

    assert_eq!(cube_map.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(cube_map.positive_x.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(cube_map.negative_x.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(cube_map.positive_y.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(cube_map.negative_y.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(cube_map.positive_z.pixel_datatype, PixelDatatype::UnsignedByte);
    assert_eq!(cube_map.negative_z.pixel_datatype, PixelDatatype::UnsignedByte);
}

#[test]
fn test_sets_a_sampler() {
    let sampler = Sampler::new(SamplerOptions {
        wrap_s: Some(TextureWrap::Repeat),
        wrap_t: Some(TextureWrap::MirroredRepeat),
        minification_filter: Some(TextureMinificationFilter::Nearest),
        magnification_filter: Some(TextureMagnificationFilter::Nearest),
        ..Default::default()
    })
    .unwrap();

    let cube_map = CubeMap::new(CubeMapOptions {
        width: 16,
        height: 16,
        sampler: Some(sampler.clone()),
        ..Default::default()
    })
    .unwrap();

    assert_eq!(cube_map.sampler.wrap_s, TextureWrap::Repeat);
    assert_eq!(cube_map.sampler.wrap_t, TextureWrap::MirroredRepeat);
    assert_eq!(
        cube_map.sampler.minification_filter,
        TextureMinificationFilter::Nearest
    );
    assert_eq!(
        cube_map.sampler.magnification_filter,
        TextureMagnificationFilter::Nearest
    );
}

#[test]
fn test_gets_width_height_and_size_in_bytes() {
    let cube_map = CubeMap::new(CubeMapOptions {
        width: 16,
        height: 16,
        ..Default::default()
    })
    .unwrap();

    assert_eq!(cube_map.width, 16);
    assert_eq!(cube_map.height, 16);
    assert_eq!(cube_map.size_in_bytes(), 256 * 4 * 6);
}

#[test]
fn test_flip_y() {
    let cube_map = CubeMap::new(CubeMapOptions {
        width: 16,
        height: 16,
        flip_y: Some(true),
        ..Default::default()
    })
    .unwrap();

    assert!(cube_map.flip_y);
}

#[test]
fn test_destroys() {
    let mut cube_map = CubeMap::new(CubeMapOptions {
        width: 8,
        height: 8,
        ..Default::default()
    })
    .unwrap();

    assert!(!cube_map.is_destroyed());
    cube_map.destroy();
    assert!(cube_map.is_destroyed());
}
