//! Direct port and verification against CesiumJS `DataSources/ModelGraphicsSpec.js` (Apache-2.0).

use cesium_math::{
    Cartesian2, Cartesian3, ClippingPlaneCollection, Color, ColorBlendMode, ConstantProperty,
    CustomShader, DistanceDisplayCondition, HeightReference, ModelGraphics, ModelGraphicsOptions,
    ModelGraphicsValues, NodeTransformationProperty, NodeTransformationPropertyOptions,
    PropertyBag, Quaternion, ShadowMode,
};
use std::rc::Rc;

#[test]
fn creates_expected_instance_from_raw_assignment_and_construction() {
    let values = ModelGraphicsValues {
        uri: Some("0".to_string()),
        scale: Some(1.0),
        show: Some(false),
        minimum_pixel_size: Some(2.0),
        maximum_scale: Some(200.0),
        incrementally_load_textures: Some(false),
        run_animations: Some(false),
        clamp_animations: Some(false),
        shadows: Some(ShadowMode::Disabled),
        height_reference: Some(HeightReference::ClampToGround),
        distance_display_condition: Some(DistanceDisplayCondition::new(10.0, 100.0)),
        silhouette_color: Some(Color::new(1.0, 0.0, 0.0, 1.0)),
        silhouette_size: Some(3.0),
        color: Some(Color::new(0.0, 1.0, 0.0, 0.2)),
        color_blend_mode: Some(ColorBlendMode::Highlight),
        color_blend_amount: Some(0.5),
        clipping_planes: Some(ClippingPlaneCollection::new()),
        custom_shader: Some(CustomShader::new()),
        image_based_lighting_factor: Some(Cartesian2::new(0.5, 0.5)),
        light_color: Some(Color::new(1.0, 1.0, 0.0, 1.0)),
    };

    let model = ModelGraphics::from_values(values);

    assert_eq!(
        model.uri.as_ref().unwrap().get_value(None).unwrap(),
        "0".to_string()
    );
    assert_eq!(
        model.scale.as_ref().unwrap().get_value(None).unwrap(),
        1.0
    );
    assert!(!model.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        model
            .minimum_pixel_size
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        2.0
    );
    assert_eq!(
        model
            .maximum_scale
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        200.0
    );
    assert!(!model
        .incrementally_load_textures
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert_eq!(
        model.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::Disabled
    );
    assert_eq!(
        model
            .height_reference
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        HeightReference::ClampToGround
    );
    assert_eq!(
        model
            .distance_display_condition
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        DistanceDisplayCondition::new(10.0, 100.0)
    );
    assert_eq!(
        model
            .silhouette_color
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Color::new(1.0, 0.0, 0.0, 1.0)
    );
    assert_eq!(
        model
            .silhouette_size
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        3.0
    );
    assert_eq!(
        model.color.as_ref().unwrap().get_value(None).unwrap(),
        Color::new(0.0, 1.0, 0.0, 0.2)
    );
    assert_eq!(
        model
            .color_blend_mode
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ColorBlendMode::Highlight
    );
    assert_eq!(
        model
            .color_blend_amount
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        0.5
    );
    assert_eq!(
        model
            .clipping_planes
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ClippingPlaneCollection::new()
    );
    assert_eq!(
        model
            .custom_shader
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        CustomShader::new()
    );
    assert_eq!(
        model
            .image_based_lighting_factor
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        Cartesian2::new(0.5, 0.5)
    );
    assert_eq!(
        model.light_color.as_ref().unwrap().get_value(None).unwrap(),
        Color::new(1.0, 1.0, 0.0, 1.0)
    );
    assert!(!model
        .run_animations
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
    assert!(!model
        .clamp_animations
        .as_ref()
        .unwrap()
        .get_value(None)
        .unwrap());
}

#[test]
fn merge_assigns_unassigned_properties() {
    let mut node_bag = PropertyBag::new();
    let node_prop = Rc::new(NodeTransformationProperty::from_options(
        NodeTransformationPropertyOptions {
            translation: Some(Rc::new(ConstantProperty::from_value(Cartesian3::UNIT_Y))),
            rotation: Some(Rc::new(ConstantProperty::from_value(Quaternion::new(
                0.5, 0.5, 0.5, 0.5,
            )))),
            scale: Some(Rc::new(ConstantProperty::from_value(Cartesian3::UNIT_X))),
        },
    ));
    node_bag.add_property("node1", Some(node_prop));

    let mut articulations_bag = PropertyBag::new();
    articulations_bag.add_property(
        "a1 s1",
        Some(Rc::new(ConstantProperty::from_value(10.0))),
    );

    let source = ModelGraphics::from_options(ModelGraphicsOptions {
        uri: Some(Rc::new(ConstantProperty::from_value("uri".to_string()))),
        scale: Some(Rc::new(ConstantProperty::from_value(1.0))),
        show: Some(Rc::new(ConstantProperty::from_value(true))),
        minimum_pixel_size: Some(Rc::new(ConstantProperty::from_value(2.0))),
        maximum_scale: Some(Rc::new(ConstantProperty::from_value(200.0))),
        incrementally_load_textures: Some(Rc::new(ConstantProperty::from_value(true))),
        run_animations: Some(Rc::new(ConstantProperty::from_value(true))),
        clamp_animations: Some(Rc::new(ConstantProperty::from_value(true))),
        shadows: Some(Rc::new(ConstantProperty::from_value(ShadowMode::Enabled))),
        height_reference: Some(Rc::new(ConstantProperty::from_value(
            HeightReference::ClampToGround,
        ))),
        distance_display_condition: Some(Rc::new(ConstantProperty::from_value(
            DistanceDisplayCondition::new(10.0, 100.0),
        ))),
        silhouette_color: Some(Rc::new(ConstantProperty::from_value(Color::RED))),
        silhouette_size: Some(Rc::new(ConstantProperty::from_value(3.0))),
        color: Some(Rc::new(ConstantProperty::from_value(Color::BLUE))),
        color_blend_mode: Some(Rc::new(ConstantProperty::from_value(
            ColorBlendMode::Highlight,
        ))),
        color_blend_amount: Some(Rc::new(ConstantProperty::from_value(0.5))),
        clipping_planes: Some(Rc::new(ConstantProperty::from_value(
            ClippingPlaneCollection::new(),
        ))),
        custom_shader: Some(Rc::new(ConstantProperty::from_value(CustomShader::new()))),
        image_based_lighting_factor: Some(Rc::new(ConstantProperty::from_value(
            Cartesian2::new(0.5, 0.5),
        ))),
        light_color: Some(Rc::new(ConstantProperty::from_value(Color::YELLOW))),
        node_transformations: Some(node_bag),
        articulations: Some(articulations_bag),
    });

    let mut target = ModelGraphics::new();
    target.merge(&source);

    assert!(target.uri.is_some());
    assert!(target.scale.is_some());
    assert!(target.show.is_some());
    assert!(target.minimum_pixel_size.is_some());
    assert!(target.maximum_scale.is_some());
    assert!(target.incrementally_load_textures.is_some());
    assert!(target.run_animations.is_some());
    assert!(target.clamp_animations.is_some());
    assert!(target.shadows.is_some());
    assert!(target.height_reference.is_some());
    assert!(target.distance_display_condition.is_some());
    assert!(target.silhouette_color.is_some());
    assert!(target.silhouette_size.is_some());
    assert!(target.color.is_some());
    assert!(target.color_blend_mode.is_some());
    assert!(target.color_blend_amount.is_some());
    assert!(target.clipping_planes.is_some());
    assert!(target.custom_shader.is_some());
    assert!(target.image_based_lighting_factor.is_some());
    assert!(target.light_color.is_some());
    assert!(target.node_transformations.has_property("node1"));
    assert!(target.articulations.has_property("a1 s1"));
}

#[test]
fn merge_does_not_assign_assigned_properties() {
    let mut source = ModelGraphics::new();
    source.show = Some(Rc::new(ConstantProperty::from_value(false)));

    let mut target = ModelGraphics::new();
    target.show = Some(Rc::new(ConstantProperty::from_value(true)));
    target.merge(&source);

    assert!(target.show.as_ref().unwrap().get_value(None).unwrap());
}

#[test]
fn clone_works() {
    let mut source = ModelGraphics::new();
    source.uri = Some(Rc::new(ConstantProperty::from_value("uri".to_string())));
    source.scale = Some(Rc::new(ConstantProperty::from_value(2.0)));
    source.show = Some(Rc::new(ConstantProperty::from_value(true)));
    source.minimum_pixel_size = Some(Rc::new(ConstantProperty::from_value(5.0)));
    source.maximum_scale = Some(Rc::new(ConstantProperty::from_value(100.0)));
    source.shadows = Some(Rc::new(ConstantProperty::from_value(ShadowMode::CastOnly)));
    source.color_blend_mode = Some(Rc::new(ConstantProperty::from_value(ColorBlendMode::Mix)));

    let cloned = source.clone();
    assert_eq!(
        cloned.uri.as_ref().unwrap().get_value(None).unwrap(),
        "uri".to_string()
    );
    assert_eq!(
        cloned.scale.as_ref().unwrap().get_value(None).unwrap(),
        2.0
    );
    assert!(cloned.show.as_ref().unwrap().get_value(None).unwrap());
    assert_eq!(
        cloned
            .minimum_pixel_size
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        5.0
    );
    assert_eq!(
        cloned
            .maximum_scale
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        100.0
    );
    assert_eq!(
        cloned.shadows.as_ref().unwrap().get_value(None).unwrap(),
        ShadowMode::CastOnly
    );
    assert_eq!(
        cloned
            .color_blend_mode
            .as_ref()
            .unwrap()
            .get_value(None)
            .unwrap(),
        ColorBlendMode::Mix
    );
}
