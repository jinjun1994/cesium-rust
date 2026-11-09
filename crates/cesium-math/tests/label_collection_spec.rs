use cesium_math::{
    Cartesian2, Cartesian3, Color, DistanceDisplayCondition, HeightReference, HorizontalOrigin,
    Label, LabelCollection, LabelStyle, NearFarScalar, VerticalOrigin,
};

#[test]
fn has_default_values_when_adding_default_label() {
    let mut labels = LabelCollection::new();
    let label = labels.add_default();

    assert!(label.show);
    assert_eq!(label.position, Cartesian3::ZERO);
    assert_eq!(label.text, "");
    assert_eq!(label.font, "30px sans-serif");
    assert_eq!(label.fill_color, Color::WHITE);
    assert_eq!(label.outline_color, Color::BLACK);
    assert_eq!(label.outline_width, 1.0);
    assert!(!label.show_background);
    assert_eq!(label.background_color, Color::new(0.165, 0.165, 0.165, 0.8));
    assert_eq!(label.background_padding, Cartesian2::new(7.0, 5.0));
    assert_eq!(label.style, LabelStyle::Fill);
    assert_eq!(label.pixel_offset, Cartesian2::ZERO);
    assert_eq!(label.eye_offset, Cartesian3::ZERO);
    assert_eq!(label.height_reference, HeightReference::None);
    assert_eq!(label.horizontal_origin, HorizontalOrigin::Left);
    assert_eq!(label.vertical_origin, VerticalOrigin::Baseline);
    assert_eq!(label.scale, 1.0);
    assert!(label.id.is_none());
    assert!(label.translucency_by_distance.is_none());
    assert!(label.pixel_offset_scale_by_distance.is_none());
    assert!(label.scale_by_distance.is_none());
    assert!(label.distance_display_condition.is_none());
    assert!(label.disable_depth_test_distance.is_none());
}

#[test]
fn can_add_label_with_specified_values() {
    let mut labels = LabelCollection::new();

    let mut custom = Label {
        id: Some("id".to_string()),
        show: false,
        position: Cartesian3::new(1.0, 2.0, 3.0),
        font: "24px \"Open Sans\"".to_string(),
        fill_color: Color::new(2.0, 3.0, 4.0, 1.0),
        outline_color: Color::new(3.0, 4.0, 2.0, 1.0),
        outline_width: 2.0,
        style: LabelStyle::FillAndOutline,
        show_background: true,
        background_color: Color::BLUE,
        background_padding: Cartesian2::new(11.0, 12.0),
        pixel_offset: Cartesian2::new(4.0, 5.0),
        eye_offset: Cartesian3::new(6.0, 7.0, 8.0),
        horizontal_origin: HorizontalOrigin::Left,
        vertical_origin: VerticalOrigin::Bottom,
        scale: 2.0,
        translucency_by_distance: Some(NearFarScalar::new(1.0e4, 1.0, 1.0e6, 0.0)),
        pixel_offset_scale_by_distance: Some(NearFarScalar::new(1.0e4, 1.0, 1.0e6, 0.0)),
        scale_by_distance: Some(NearFarScalar::new(1.0e4, 1.0, 1.0e6, 0.0)),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        disable_depth_test_distance: Some(10.0),
        ..Default::default()
    };
    custom.set_text("abc");

    let idx = labels.add(custom.clone());
    assert_eq!(idx, 0);
    assert_eq!(labels.len(), 1);

    let retrieved = labels.get(0).unwrap();
    assert_eq!(retrieved, &custom);
}

#[test]
fn collection_add_remove_and_contains_behavior() {
    let mut labels = LabelCollection::new();
    assert_eq!(labels.len(), 0);
    assert!(labels.is_empty());
    assert!(!labels.is_destroyed());

    let mut l1 = Label::default();
    l1.set_text("one");
    let mut l2 = Label::default();
    l2.set_text("two");

    labels.add(l1.clone());
    labels.add(l2.clone());

    assert!(labels.contains(&l1));
    assert!(labels.contains(&l2));
    assert_eq!(labels.len(), 2);

    // Remove first
    assert!(labels.remove_label(&l1));
    assert!(!labels.contains(&l1));
    assert!(labels.contains(&l2));
    assert_eq!(labels.len(), 1);

    // Remove already removed returns false
    assert!(!labels.remove_label(&l1));

    // Remove remaining
    assert!(labels.remove_label(&l2));
    assert_eq!(labels.len(), 0);
    assert!(labels.is_empty());
}

#[test]
fn can_remove_all_labels_and_destroy() {
    let mut labels = LabelCollection::new();

    let l1 = Label {
        position: Cartesian3::new(1.0, 2.0, 3.0),
        ..Default::default()
    };
    let l2 = Label {
        position: Cartesian3::new(4.0, 5.0, 6.0),
        ..Default::default()
    };

    labels.add(l1);
    labels.add(l2);
    assert_eq!(labels.len(), 2);

    labels.remove_all();
    assert_eq!(labels.len(), 0);
    assert!(!labels.is_destroyed());

    labels.destroy();
    assert!(labels.is_destroyed());
}
