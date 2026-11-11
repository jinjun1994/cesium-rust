use cesium_renderer::supported_image_formats::{
    SupportedImageFormats, SupportedImageFormatsOptions,
};

#[test]
fn test_constructs_with_options() {
    let supported_image_formats = SupportedImageFormats::new(Some(SupportedImageFormatsOptions {
        webp: Some(true),
        basis: Some(false),
    }));

    assert!(supported_image_formats.webp);
    assert!(!supported_image_formats.basis);
}

#[test]
fn test_constructs_with_default_values() {
    let supported_image_formats =
        SupportedImageFormats::new(Some(SupportedImageFormatsOptions::default()));

    assert!(!supported_image_formats.webp);
    assert!(!supported_image_formats.basis);

    let default_constructed = SupportedImageFormats::new(None);
    assert!(!default_constructed.webp);
    assert!(!default_constructed.basis);

    let std_default = SupportedImageFormats::default();
    assert_eq!(supported_image_formats, std_default);
}

#[test]
fn test_with_flags_convenience_constructor() {
    let formats = SupportedImageFormats::with_flags(true, false);
    assert!(formats.basis);
    assert!(!formats.webp);

    let formats2 = SupportedImageFormats::from_flags(true, true);
    assert!(formats2.webp);
    assert!(formats2.basis);
}
