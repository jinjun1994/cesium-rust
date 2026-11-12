use cesium_math::Color;
use cesium_renderer::{BatchTexture, BatchTextureOptions};

#[test]
fn test_validation() {
    assert!(BatchTexture::new(BatchTextureOptions {
        features_length: 0,
        owner_id: "owner".to_string(),
        color_table: None,
    })
    .is_err());

    assert!(BatchTexture::new(BatchTextureOptions {
        features_length: 1,
        owner_id: "".to_string(),
        color_table: None,
    })
    .is_err());
}

#[test]
fn test_show_lazy_allocation_and_out_of_range() {
    let mut bt = BatchTexture::new(BatchTextureOptions {
        features_length: 2,
        owner_id: "owner".to_string(),
        color_table: None,
    })
    .unwrap();

    assert!(bt.get_show(0).unwrap());
    assert!(bt.get_show(2).is_err());
    assert!(bt.set_show(2, false).is_err());

    assert!(!bt.has_batch_values());
    assert!(!bt.has_batch_texture());

    // Setting default true does not allocate
    bt.set_show(0, true).unwrap();
    bt.update();
    assert!(!bt.has_batch_values());
    assert!(!bt.has_batch_texture());

    // Setting false allocates and dirties
    bt.set_show(0, false).unwrap();
    assert!(bt.is_batch_values_dirty());
    bt.update();
    assert!(bt.has_batch_values());
    assert!(bt.has_batch_texture());
    assert!(!bt.is_batch_values_dirty());
    assert!(!bt.get_show(0).unwrap());

    // Setting same value keeps dirty false
    bt.set_show(0, false).unwrap();
    assert!(!bt.is_batch_values_dirty());

    bt.set_all_show(true);
    assert!(bt.get_show(0).unwrap());
    assert!(bt.get_show(1).unwrap());
}

#[test]
fn test_color_lazy_allocation_and_set_all() {
    let mut bt = BatchTexture::new(BatchTextureOptions {
        features_length: 2,
        owner_id: "owner".to_string(),
        color_table: None,
    })
    .unwrap();

    assert_eq!(bt.get_color(0).unwrap(), Color::WHITE);
    assert!(bt.get_color(2).is_err());
    assert!(bt.set_color(2, Color::RED).is_err());

    // Default white does not allocate
    bt.set_color(0, Color::WHITE).unwrap();
    bt.update();
    assert!(!bt.has_batch_values());

    // Non-white allocates and tracks translucency
    let semi = Color::new(1.0, 0.0, 0.0, 0.5);
    bt.set_color(0, semi).unwrap();
    assert_eq!(bt.translucent_features_length(), 1);
    assert!(bt.is_batch_values_dirty());
    bt.update();
    assert!(!bt.is_batch_values_dirty());
    let got = bt.get_color(0).unwrap();
    assert!((got.red - 1.0).abs() < 1e-3);
    assert!((got.alpha - 0.5).abs() < 0.01);

    bt.set_all_color(Color::BLUE);
    assert_eq!(bt.translucent_features_length(), 0);
    assert_eq!(bt.get_color(0).unwrap(), Color::BLUE);
    assert_eq!(bt.get_color(1).unwrap(), Color::BLUE);

    bt.destroy();
    assert!(bt.is_destroyed());
}
