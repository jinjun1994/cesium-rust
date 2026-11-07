use cesium_math::{BoundingRectangle, Cartesian2};
use cesium_renderer::{PixelFormat, TextureAtlas, TextureAtlasOptions};

#[test]
fn test_throws_with_negative_border_width_in_pixels() {
    let res = TextureAtlas::new(TextureAtlasOptions {
        border_width_in_pixels: Some(-1),
        ..Default::default()
    });
    assert!(res.is_err());
}

#[test]
fn test_throws_with_initial_size_less_than_one() {
    let res1 = TextureAtlas::new(TextureAtlasOptions {
        initial_size: Some(Cartesian2::new(0.0, 1.0)),
        ..Default::default()
    });
    assert!(res1.is_err());

    let res2 = TextureAtlas::new(TextureAtlasOptions {
        initial_size: Some(Cartesian2::new(1.0, 0.0)),
        ..Default::default()
    });
    assert!(res2.is_err());
}

#[test]
fn test_constructs_with_default_and_custom_parameters() {
    let atlas_default = TextureAtlas::default();
    assert_eq!(atlas_default.border_width_in_pixels, 1);
    assert_eq!(atlas_default.pixel_format, PixelFormat::Rgba);

    let atlas_custom = TextureAtlas::new(TextureAtlasOptions {
        pixel_format: Some(PixelFormat::Rgb),
        border_width_in_pixels: Some(0),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(atlas_custom.border_width_in_pixels, 0);
    assert_eq!(atlas_custom.pixel_format, PixelFormat::Rgb);
}

#[test]
fn test_add_image_and_sub_region() {
    let mut atlas = TextureAtlas::default();

    assert!(atlas.add_image(None, 10, 10).is_err());
    assert!(atlas.add_image(Some("img1"), 0, 10).is_err());

    let idx1 = atlas.add_image(Some("img1"), 10, 10).unwrap();
    assert_eq!(idx1, 0);
    assert_eq!(atlas.number_of_images(), 1);

    // Adding same id returns existing index
    let idx1_again = atlas.add_image(Some("img1"), 10, 10).unwrap();
    assert_eq!(idx1_again, 0);
    assert_eq!(atlas.number_of_images(), 1);

    // Subregion errors
    assert!(atlas.add_image_sub_region(None, Some(BoundingRectangle::default())).is_err());
    assert!(atlas.add_image_sub_region(Some("img1"), None).is_err());
    assert!(atlas.add_image_sub_region(Some("not_found"), Some(BoundingRectangle::default())).is_err());

    // Successful subregion
    let sub_idx = atlas
        .add_image_sub_region(Some("img1"), Some(BoundingRectangle::new(0.0, 0.0, 0.5, 0.5)))
        .unwrap();
    assert_eq!(sub_idx, 1);
}

#[test]
fn test_destroys() {
    let mut atlas = TextureAtlas::default();
    assert!(!atlas.is_destroyed());
    atlas.destroy();
    assert!(atlas.is_destroyed());
}
