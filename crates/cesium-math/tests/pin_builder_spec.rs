use std::sync::Arc;

use cesium_math::{Color, PinBuilder, PinImage};

fn get_pin_color(image: &PinImage) -> Color {
    let px = image.get_pixel(64, 5);
    Color::from_bytes(
        (px.red * 255.0).round() as u8,
        (px.green * 255.0).round() as u8,
        (px.blue * 255.0).round() as u8,
        255,
    )
}

fn get_icon_color(image: &PinImage) -> Color {
    let px = image.get_pixel(64, 32);
    Color::from_bytes(
        (px.red * 255.0).round() as u8,
        (px.green * 255.0).round() as u8,
        (px.blue * 255.0).round() as u8,
        255,
    )
}

#[test]
fn test_from_color_creates_icon_of_correct_color() {
    let mut builder = PinBuilder::new();
    let image = builder.from_color(Some(Color::GREEN), Some(128)).unwrap();
    assert_eq!(get_pin_color(&image), Color::GREEN);
}

#[test]
fn test_from_text_creates_icon_of_correct_color_with_symbol() {
    let mut builder = PinBuilder::new();
    let image = builder
        .from_text(Some("\u{25A0}"), Some(Color::BLUE), Some(128))
        .unwrap();
    assert_eq!(get_pin_color(&image), Color::BLUE);
    assert_eq!(get_icon_color(&image), Color::WHITE);
}

#[test]
fn test_from_url_creates_icon_of_correct_color_with_symbol() {
    let mut builder = PinBuilder::new();
    let image = builder
        .from_url(Some("Assets/Textures/maki/square.png"), Some(Color::RED), Some(128))
        .unwrap();
    assert_eq!(get_pin_color(&image), Color::RED);
    assert_eq!(get_icon_color(&image), Color::WHITE);
}

#[test]
fn test_from_maki_icon_id_creates_icon_of_correct_color() {
    let mut builder = PinBuilder::new();
    let image = builder
        .from_maki_icon_id(Some("square"), Some(Color::YELLOW), Some(128))
        .unwrap();
    assert_eq!(get_pin_color(&image), Color::YELLOW);
    assert_eq!(get_icon_color(&image), Color::WHITE);
}

#[test]
fn test_caches_and_returns_existing_image() {
    let mut builder = PinBuilder::new();
    let img1 = builder.from_color(Some(Color::GREEN), Some(128)).unwrap();
    let img2 = builder.from_color(Some(Color::GREEN), Some(128)).unwrap();
    assert!(Arc::ptr_eq(&img1, &img2));
}

#[test]
fn test_error_handling() {
    let mut builder = PinBuilder::new();

    // from_color
    assert!(builder.from_color(None, Some(128)).is_err());
    assert!(builder.from_color(Some(Color::RED), None).is_err());

    // from_text
    assert!(builder.from_text(None, Some(Color::RED), Some(128)).is_err());
    assert!(builder.from_text(Some("text"), None, Some(128)).is_err());
    assert!(builder.from_text(Some("text"), Some(Color::RED), None).is_err());

    // from_url
    assert!(builder.from_url(None, Some(Color::RED), Some(128)).is_err());
    assert!(builder.from_url(Some("url"), None, Some(128)).is_err());
    assert!(builder.from_url(Some("url"), Some(Color::RED), None).is_err());

    // from_maki_icon_id
    assert!(builder.from_maki_icon_id(None, Some(Color::RED), Some(128)).is_err());
    assert!(builder.from_maki_icon_id(Some("hospital"), None, Some(128)).is_err());
    assert!(builder.from_maki_icon_id(Some("hospital"), Some(Color::RED), None).is_err());
}
