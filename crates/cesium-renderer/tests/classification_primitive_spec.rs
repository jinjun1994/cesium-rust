use std::collections::HashMap;

use cesium_math::{
    BoxGeometry, Cartesian3, ClassificationType, ComponentDatatype, GeometryInstance,
    GeometryInstanceAttribute, Matrix4,
};
use cesium_renderer::classification_primitive::{
    ClassificationPrimitive, ClassificationPrimitiveOptions,
};
use cesium_renderer::commands::Pass;
use cesium_renderer::scene_primitive::{PrimitivePasses, PrimitiveRenderFrameState};

fn make_box_instance(id: &str) -> GeometryInstance {
    let box_geom = BoxGeometry::from_dimensions(Cartesian3::new(100.0, 100.0, 100.0), None)
        .create_geometry()
        .expect("box geometry");

    let mut attributes = HashMap::new();
    attributes.insert(
        "color".to_string(),
        GeometryInstanceAttribute {
            component_datatype: ComponentDatatype::Float,
            components_per_attribute: 4,
            normalize: false,
            value: vec![1.0, 1.0, 0.0, 1.0],
        },
    );

    GeometryInstance {
        geometry: box_geom,
        model_matrix: Matrix4::IDENTITY,
        id: Some(id.to_string()),
        attributes,
    }
}

#[test]
fn default_constructs_and_constructs_with_options() {
    let default_prim = ClassificationPrimitive::default();
    assert!(default_prim.geometry_instances.is_none());
    assert!(default_prim.show);
    assert!(!default_prim.vertex_cache_optimize);
    assert!(!default_prim.interleave);
    assert!(default_prim.compress_vertices);
    assert!(default_prim.release_geometry_instances);
    assert!(default_prim.allow_picking);
    assert!(default_prim.asynchronous);
    assert_eq!(default_prim.classification_type, ClassificationType::Both);
    assert!(!default_prim.debug_show_bounding_volume);
    assert!(!default_prim.debug_show_shadow_volume);
    assert!(!default_prim.ready);

    let custom_prim = ClassificationPrimitive::new(ClassificationPrimitiveOptions {
        geometry_instances: Some(vec![]),
        show: Some(false),
        vertex_cache_optimize: Some(true),
        interleave: Some(true),
        compress_vertices: Some(false),
        release_geometry_instances: Some(false),
        allow_picking: Some(false),
        asynchronous: Some(false),
        classification_type: Some(ClassificationType::Terrain),
        debug_show_bounding_volume: Some(true),
        debug_show_shadow_volume: Some(true),
        ..Default::default()
    });

    assert!(custom_prim.geometry_instances.is_some());
    assert!(!custom_prim.show);
    assert!(custom_prim.vertex_cache_optimize);
    assert!(custom_prim.interleave);
    assert!(!custom_prim.compress_vertices);
    assert!(!custom_prim.release_geometry_instances);
    assert!(!custom_prim.allow_picking);
    assert!(!custom_prim.asynchronous);
    assert_eq!(custom_prim.classification_type, ClassificationType::Terrain);
    assert!(custom_prim.debug_show_bounding_volume);
    assert!(custom_prim.debug_show_shadow_volume);
    assert!(custom_prim.ready);
}

#[test]
fn releases_geometry_instances_when_configured() {
    let mut prim_release = ClassificationPrimitive::new(ClassificationPrimitiveOptions {
        geometry_instances: Some(vec![make_box_instance("b1")]),
        release_geometry_instances: Some(true),
        asynchronous: Some(false),
        ..Default::default()
    });
    assert!(prim_release.geometry_instances.is_some());
    let mut frame_state = PrimitiveRenderFrameState::default();
    prim_release.update(&mut frame_state).unwrap();
    assert!(prim_release.geometry_instances.is_none());
    assert_eq!(frame_state.command_list.len(), 2);

    let mut prim_keep = ClassificationPrimitive::new(ClassificationPrimitiveOptions {
        geometry_instances: Some(vec![make_box_instance("b2")]),
        release_geometry_instances: Some(false),
        asynchronous: Some(false),
        ..Default::default()
    });
    let mut frame_state2 = PrimitiveRenderFrameState::default();
    prim_keep.update(&mut frame_state2).unwrap();
    assert!(prim_keep.geometry_instances.is_some());
}

#[test]
fn does_not_render_when_show_is_false_or_no_instances_but_becomes_ready() {
    let mut empty_prim = ClassificationPrimitive::new(ClassificationPrimitiveOptions {
        geometry_instances: None,
        asynchronous: Some(false),
        ..Default::default()
    });
    let mut frame_state = PrimitiveRenderFrameState::default();
    empty_prim.update(&mut frame_state).unwrap();
    assert_eq!(frame_state.command_list.len(), 0);

    let mut hidden_prim = ClassificationPrimitive::new(ClassificationPrimitiveOptions {
        geometry_instances: Some(vec![make_box_instance("b3")]),
        show: Some(false),
        asynchronous: Some(true),
        ..Default::default()
    });
    assert!(!hidden_prim.ready);
    let mut frame_state2 = PrimitiveRenderFrameState::default();
    hidden_prim.update(&mut frame_state2).unwrap();
    assert!(hidden_prim.ready);
    assert_eq!(frame_state2.command_list.len(), 0);

    // No render or pick pass active suppresses commands
    let mut active_prim = ClassificationPrimitive::new(ClassificationPrimitiveOptions {
        geometry_instances: Some(vec![make_box_instance("b4")]),
        asynchronous: Some(false),
        ..Default::default()
    });
    let mut inactive_passes = PrimitiveRenderFrameState {
        passes: PrimitivePasses {
            render: false,
            pick: false,
        },
        ..Default::default()
    };
    active_prim.update(&mut inactive_passes).unwrap();
    assert_eq!(inactive_passes.command_list.len(), 0);
}

#[test]
fn routes_passes_by_classification_type_and_supports_destroy() {
    assert!(ClassificationPrimitive::is_supported(true));
    assert!(!ClassificationPrimitive::is_supported(false));

    let mut prim = ClassificationPrimitive::new(ClassificationPrimitiveOptions {
        geometry_instances: Some(vec![make_box_instance("box")]),
        release_geometry_instances: Some(false),
        asynchronous: Some(false),
        classification_type: Some(ClassificationType::Both),
        ..Default::default()
    });

    let attr = prim
        .get_geometry_instance_attribute("box", "color")
        .expect("attribute found");
    assert_eq!(attr.value, vec![1.0, 1.0, 0.0, 1.0]);

    // Both -> TerrainClassification + Cesium3dTileClassification
    let mut fs1 = PrimitiveRenderFrameState::default();
    prim.update(&mut fs1).unwrap();
    assert_eq!(fs1.command_list.len(), 2);
    assert_eq!(fs1.command_list[0].pass, Some(Pass::TerrainClassification));
    assert_eq!(fs1.command_list[1].pass, Some(Pass::Cesium3dTileClassification));

    // Terrain only
    prim.classification_type = ClassificationType::Terrain;
    let mut fs2 = PrimitiveRenderFrameState::default();
    prim.update(&mut fs2).unwrap();
    assert_eq!(fs2.command_list.len(), 1);
    assert_eq!(fs2.command_list[0].pass, Some(Pass::TerrainClassification));

    // Cesium3DTile only
    prim.classification_type = ClassificationType::Cesium3DTile;
    let mut fs3 = PrimitiveRenderFrameState::default();
    prim.update(&mut fs3).unwrap();
    assert_eq!(fs3.command_list.len(), 1);
    assert_eq!(fs3.command_list[0].pass, Some(Pass::Cesium3dTileClassification));

    prim.destroy();
    assert!(prim.is_destroyed());
    let mut fs4 = PrimitiveRenderFrameState::default();
    prim.update(&mut fs4).unwrap();
    assert!(fs4.command_list.is_empty());
}
