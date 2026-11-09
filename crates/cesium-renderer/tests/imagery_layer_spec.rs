use std::sync::Arc;
use cesium_math::{Color, GeoRectangle};
use cesium_renderer::imagery_layer::{
    ImageryLayer, ImageryLayerOptions, ImageryProvider, SplitDirection,
    TextureMagnificationFilter, TextureMinificationFilter,
};

struct CustomProvider {
    rectangle: GeoRectangle,
}

impl ImageryProvider for CustomProvider {
    fn rectangle(&self) -> GeoRectangle {
        self.rectangle
    }
}

#[test]
fn constructs_with_default_options() {
    let layer = ImageryLayer::new(None, None);
    assert!(layer.show());
    assert!(layer.is_ready());
    assert!(!layer.is_base_layer());
    assert_eq!(layer.layer_index(), 0);
    assert_eq!(layer.alpha(), 1.0);
    assert_eq!(layer.brightness(), 1.0);
    assert_eq!(layer.contrast(), 1.0);
    assert_eq!(layer.hue(), 0.0);
    assert_eq!(layer.saturation(), 1.0);
    assert_eq!(layer.gamma(), 1.0);
    assert_eq!(layer.split_direction(), SplitDirection::None);
    assert_eq!(layer.minification_filter(), TextureMinificationFilter::Linear);
    assert_eq!(layer.magnification_filter(), TextureMagnificationFilter::Linear);
    assert!(!layer.is_destroyed());
}

#[test]
fn allows_setting_texture_filters_and_color_properties() {
    let mut layer = ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            minification_filter: Some(TextureMinificationFilter::Nearest),
            magnification_filter: Some(TextureMagnificationFilter::Nearest),
            split_direction: Some(SplitDirection::Left),
            color_to_alpha: Some(Color::from_bytes(255, 0, 0, 255)),
            color_to_alpha_threshold: Some(0.1),
            ..Default::default()
        }),
    );

    assert_eq!(layer.minification_filter(), TextureMinificationFilter::Nearest);
    assert_eq!(layer.magnification_filter(), TextureMagnificationFilter::Nearest);
    assert_eq!(layer.split_direction(), SplitDirection::Left);
    assert_eq!(layer.color_to_alpha(), Some(Color::from_bytes(255, 0, 0, 255)));
    assert!((layer.color_to_alpha_threshold() - 0.1).abs() < 1e-6);

    layer.set_minification_filter(TextureMinificationFilter::LinearMipmapLinear);
    layer.set_magnification_filter(TextureMagnificationFilter::Linear);
    layer.set_split_direction(SplitDirection::Right);
    layer.set_show(false);

    assert_eq!(
        layer.minification_filter(),
        TextureMinificationFilter::LinearMipmapLinear
    );
    assert_eq!(layer.magnification_filter(), TextureMagnificationFilter::Linear);
    assert_eq!(layer.split_direction(), SplitDirection::Right);
    assert!(!layer.show());
}

#[test]
fn get_imagery_rectangle_computes_intersection() {
    let provider_rect = GeoRectangle::from_degrees(8.2, 61.09, 8.5, 61.7);
    let layer_rect = GeoRectangle::from_degrees(7.2, 60.9, 9.0, 61.7);

    let provider = Arc::new(CustomProvider {
        rectangle: provider_rect,
    });

    let layer = ImageryLayer::new(
        Some(provider),
        Some(ImageryLayerOptions {
            rectangle: Some(layer_rect),
            ..Default::default()
        }),
    );

    let expected = layer_rect.intersection(provider_rect);
    assert_eq!(layer.get_imagery_rectangle(), expected);
}

#[test]
fn honors_terrain_level_properties() {
    let mut layer = ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            minimum_terrain_level: Some(2),
            maximum_terrain_level: Some(4),
            ..Default::default()
        }),
    );

    assert!(!layer.is_tile_level_supported(0));
    assert!(!layer.is_tile_level_supported(1));
    assert!(layer.is_tile_level_supported(2));
    assert!(layer.is_tile_level_supported(3));
    assert!(layer.is_tile_level_supported(4));
    assert!(!layer.is_tile_level_supported(5));

    layer.set_minimum_terrain_level(None);
    layer.set_maximum_terrain_level(None);
    assert!(layer.is_tile_level_supported(0));
    assert!(layer.is_tile_level_supported(100));
}

#[test]
fn basic_lifecycle_and_destruction() {
    let mut layer = ImageryLayer::new(None, None);
    assert!(!layer.is_destroyed());
    layer.destroy();
    assert!(layer.is_destroyed());
}
