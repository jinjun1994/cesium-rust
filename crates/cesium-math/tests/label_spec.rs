use cesium_math::{
    Cartesian2, Cartesian3, Color, HorizontalOrigin, Label, LabelStyle, NearFarScalar,
    VerticalOrigin,
};

#[test]
fn equals_returns_true_for_equivalent_label() {
    let mut label = Label {
        position: Cartesian3::new(1.0, 2.0, 3.0),
        ..Default::default()
    };
    label.set_text("equals");

    let mut other = Label {
        position: Cartesian3::new(1.0, 2.0, 3.0),
        ..Default::default()
    };
    other.set_text("equals");

    assert!(label.equals(Some(&other)));
}

#[test]
fn equals_returns_false_for_inequivalent_label_and_none() {
    let label = Label {
        position: Cartesian3::new(1.0, 2.0, 3.0),
        ..Default::default()
    };

    let other = Label {
        position: Cartesian3::new(4.0, 5.0, 6.0),
        ..Default::default()
    };

    assert!(!label.equals(Some(&other)));
    assert!(!label.equals(None));
}

#[test]
fn can_set_properties() {
    let mut label = Label::default();

    let show = false;
    let position = Cartesian3::new(1.0, 2.0, 3.0);
    let text = "abc";
    let font = "24px \"Open Sans\"";
    let fill_color = Color::new(2.0, 3.0, 4.0, 1.0);
    let outline_color = Color::new(3.0, 4.0, 2.0, 1.0);
    let outline_width = 2.0;
    let style = LabelStyle::FillAndOutline;
    let pixel_offset = Cartesian2::new(4.0, 5.0);
    let eye_offset = Cartesian3::new(6.0, 7.0, 8.0);
    let horizontal_origin = HorizontalOrigin::Left;
    let vertical_origin = VerticalOrigin::Bottom;
    let scale = 2.0;
    let translucency = NearFarScalar::new(1.0e4, 1.0, 1.0e6, 0.0);
    let pixel_offset_scale = NearFarScalar::new(1.0e4, 1.0, 1.0e6, 0.0);
    let scale_by_distance = NearFarScalar::new(1.0e4, 1.0, 1.0e6, 0.0);

    label.show = show;
    label.position = position;
    label.set_text(text);
    label.font = font.to_string();
    label.fill_color = fill_color;
    label.outline_color = outline_color;
    label.outline_width = outline_width;
    label.style = style;
    label.pixel_offset = pixel_offset;
    label.eye_offset = eye_offset;
    label.horizontal_origin = horizontal_origin;
    label.vertical_origin = vertical_origin;
    label.scale = scale;
    label.translucency_by_distance = Some(translucency);
    label.pixel_offset_scale_by_distance = Some(pixel_offset_scale);
    label.scale_by_distance = Some(scale_by_distance);

    assert_eq!(label.show, show);
    assert_eq!(label.position, position);
    assert_eq!(label.text, text);
    assert_eq!(label.font, font);
    assert_eq!(label.fill_color, fill_color);
    assert_eq!(label.outline_color, outline_color);
    assert_eq!(label.outline_width, outline_width);
    assert_eq!(label.style, style);
    assert_eq!(label.pixel_offset, pixel_offset);
    assert_eq!(label.eye_offset, eye_offset);
    assert_eq!(label.horizontal_origin, horizontal_origin);
    assert_eq!(label.vertical_origin, vertical_origin);
    assert_eq!(label.scale, scale);
    assert_eq!(label.translucency_by_distance, Some(translucency));
    assert_eq!(label.pixel_offset_scale_by_distance, Some(pixel_offset_scale));
    assert_eq!(label.scale_by_distance, Some(scale_by_distance));
}

#[test]
fn filter_unsupported_characters_preserves_emojis_and_filters_control_chars() {
    assert_eq!(Label::filter_unsupported_characters("a😀b"), "a😀b");
    assert_eq!(Label::filter_unsupported_characters("awe2!$34f❤️b"), "awe2!$34f❤️b");
    assert_eq!(Label::filter_unsupported_characters("lakneklf\u{200F}sldknfklf"), "lakneklfsldknfklf");
    assert_eq!(Label::filter_unsupported_characters("test \u{200F} - with Right-to-Left Mark (RLM)"), "test  - with Right-to-Left Mark (RLM)");
    assert_eq!(Label::filter_unsupported_characters("test \u{200B} - with Zero-Width Space"), "test  - with Zero-Width Space");
    assert_eq!(Label::filter_unsupported_characters("test \u{202A} - with Left-to-Right Embedding"), "test  - with Left-to-Right Embedding");
    assert_eq!(Label::filter_unsupported_characters("test 😀 - Emoji"), "test 😀 - Emoji");
    assert_eq!(Label::filter_unsupported_characters("test ❤️ - Emoji"), "test ❤️ - Emoji");
    assert_eq!(Label::filter_unsupported_characters("test 🌍 - Emoji"), "test 🌍 - Emoji");
    assert_eq!(Label::filter_unsupported_characters("test \u{2060} - with Word Joiner"), "test  - with Word Joiner");
    assert_eq!(Label::filter_unsupported_characters("test \u{0000} - with Null Character"), "test  - with Null Character");
    assert_eq!(Label::filter_unsupported_characters("test \u{000E} - with Shift Out (SO)"), "test  - with Shift Out (SO)");
    assert_eq!(Label::filter_unsupported_characters("test \u{001F} - with Unit Separator (US)"), "test  - with Unit Separator (US)");
}

#[test]
fn right_to_left_detection_and_reversal() {
    Label::set_enable_right_to_left_detection(true);

    let mut label = Label::default();

    // No RTL characters
    label.set_text("bla bla bla");
    assert_eq!(label.rendered_text, "bla bla bla");

    // Hebrew only
    label.set_text("שלום");
    assert_eq!(label.rendered_text, "םולש");

    // Arabic only
    label.set_text("مرحبا");
    assert_eq!(label.rendered_text, "ابحرم");

    // Brackets and non-alphanumeric only
    label.set_text("([{- -}])");
    assert_eq!(label.rendered_text, "([{- -}])");

    // Range \u05D0-\u05EA
    label.set_text("\u{05D1}\u{05D2}");
    assert_eq!(label.rendered_text, "\u{05D2}\u{05D1}");

    // Range \u0600-\u06FF
    label.set_text("\u{0601}\u{0602}");
    assert_eq!(label.rendered_text, "\u{0602}\u{0601}");

    // Range \u0750-\u077F
    label.set_text("\u{0751}\u{0752}");
    assert_eq!(label.rendered_text, "\u{0752}\u{0751}");

    // Range \u08A0-\u08FF
    label.set_text("\u{08A1}\u{08A2}");
    assert_eq!(label.rendered_text, "\u{08A2}\u{08A1}");

    Label::set_enable_right_to_left_detection(false);
}
