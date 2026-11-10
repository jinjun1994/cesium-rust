//! ImageryPipelineStage spec tests matching CesiumJS `Scene/Model/ImageryPipelineStageSpec.js`.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use cesium_math::{
    Cartesian4, Cartographic, ComponentDatatype, Ellipsoid, GeoRectangle, NativeRectangle,
};
use cesium_renderer::imagery_layer::{ImageryLayer, ImageryLayerOptions};
use cesium_renderer::imagery_layer_collection::ImageryLayerCollection;
use cesium_renderer::imagery_pipeline_stage::{
    Imagery, ImageryCoverage, ImageryFrameState, ImageryModelOptions, ImageryPipelineStage,
    ImageryRenderResources, ImageryRuntimePrimitive, ImageryStageVertexAttribute, ImageryState,
    ModelPrimitiveImagery,
};
use cesium_renderer::mapped_positions::MappedPositions;
use cesium_renderer::shader_builder::ShaderBuilder;

fn create_mock_primitive_imagery(layer_id: u64) -> (ModelPrimitiveImagery, Arc<RwLock<Imagery>>) {
    let mut mpi = ModelPrimitiveImagery::new();
    mpi.ready = true;
    mpi.projections = vec!["WebMercator".to_string()];

    let positions = vec![
        Cartographic::from_degrees(-120.0, 40.0, Some(0.0)),
        Cartographic::from_degrees(-119.0, 40.0, Some(0.0)),
        Cartographic::from_degrees(-120.0, 41.0, Some(0.0)),
        Cartographic::from_degrees(-119.0, 41.0, Some(0.0)),
    ];
    let rect = GeoRectangle::from_degrees(-120.0, 40.0, -119.0, 41.0);
    let mapped = MappedPositions::new(
        Some(positions),
        Some(4),
        Some(rect),
        Some(Ellipsoid::default()),
    )
    .unwrap();
    mpi.mapped_positions_by_layer_id.insert(layer_id, mapped);

    let mut imagery = Imagery::new(rect);
    imagery.state = ImageryState::Ready;
    imagery.texture = Some("mock_texture_0".to_string());
    let imagery_arc = Arc::new(RwLock::new(imagery));

    let coverage = ImageryCoverage {
        x: 0,
        y: 0,
        level: 0,
        imagery: Arc::clone(&imagery_arc),
        texture_coordinate_rectangle: Cartesian4::new(0.0, 0.0, 1.0, 1.0),
    };
    mpi.coverages_by_layer_id.insert(layer_id, vec![coverage]);

    let attr = ImageryStageVertexAttribute {
        index: 0,
        value: None,
        vertex_buffer: None,
        count: 4,
        components_per_attribute: 2,
        component_datatype: ComponentDatatype::Float,
        offset_in_bytes: 0,
        stride_in_bytes: Some(8),
        normalize: false,
    };
    mpi.imagery_tex_coord_attributes.push(attr);

    (mpi, imagery_arc)
}

#[test]
fn test_does_not_create_imagery_inputs_for_imagery_layers_that_are_not_shown() {
    let mut collection = ImageryLayerCollection::new();
    let layer = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            show: Some(true),
            ..Default::default()
        }),
    )));
    let layer_id = layer.read().unwrap().id();
    collection.add(Arc::clone(&layer), None).unwrap();

    let (mpi, _) = create_mock_primitive_imagery(layer_id);
    let indices = vec![0];

    // Initially show == true -> inputs generated
    let inputs_a = ImageryPipelineStage::create_imagery_inputs(&collection, &mpi, &indices);
    assert_eq!(inputs_a.len(), 1);
    assert_eq!(inputs_a[0].texture, "mock_texture_0");

    // Hide layer -> show == false -> no inputs generated
    layer.write().unwrap().set_show(false);
    let inputs_b = ImageryPipelineStage::create_imagery_inputs(&collection, &mpi, &indices);
    assert_eq!(inputs_b.len(), 0);
}

#[test]
fn test_process_builds_complete_imagery_shader_and_uniforms() {
    let mut collection = ImageryLayerCollection::new();
    let layer = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            show: Some(true),
            alpha: Some(0.8),
            brightness: Some(1.2),
            ..Default::default()
        }),
    )));
    let layer_id = layer.read().unwrap().id();
    collection.add(Arc::clone(&layer), None).unwrap();

    let (mpi, _) = create_mock_primitive_imagery(layer_id);
    let layers_arc = Arc::new(RwLock::new(collection));
    let model_options = ImageryModelOptions {
        imagery_layers: Some(layers_arc),
    };

    let primitive = ImageryRuntimePrimitive {
        model_primitive_imagery: Some(mpi),
        indices_count: Some(6),
        attribute_count: 4,
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 1;

    let mut rr = ImageryRenderResources {
        model: &model_options,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        attributes: &mut attributes,
        attribute_index: &mut attribute_index,
        count: 4,
    };

    let frame_state = ImageryFrameState::default();
    ImageryPipelineStage::process(&mut rr, &primitive, &frame_state);

    let fs = sb.create_fragment_shader_source();
    let vs = sb.create_vertex_shader_source();

    // Verify defines
    assert!(fs.defines.iter().any(|d| d == "HAS_IMAGERY"));
    assert!(fs.defines.iter().any(|d| d == "IMAGERY_TEXTURE_UNITS 1"));

    // Verify attributes
    assert_eq!(attributes.len(), 1);
    assert_eq!(attributes[0].index, 1);
    assert_eq!(attribute_index, 2);

    // Verify uniform map
    assert!(uniform_map.contains_key("u_imageryTextures"));
    assert!(uniform_map.contains_key("u_imageryTextureCoordinateRectangle"));
    assert!(uniform_map.contains_key("u_imageryTextureTranslationAndScale"));
    assert!(uniform_map.contains_key("u_imageryTextureAlpha"));
    assert!(uniform_map.contains_key("u_imageryTextureBrightness"));

    let tex_val = uniform_map["u_imageryTextures"]();
    assert_eq!(tex_val, serde_json::json!(["mock_texture_0"]));

    let alpha_val = uniform_map["u_imageryTextureAlpha"]();
    let alpha_num = alpha_val.as_array().unwrap()[0].as_f64().unwrap();
    assert!((alpha_num - 0.8).abs() < 1e-5);

    // Verify shader functions
    let fs_text = fs.sources.join("\n");
    assert!(fs_text.contains("blendBaseColorWithImagery"));
    assert!(fs_text.contains("sampleAndBlend"));

    let vs_text = vs.sources.join("\n");
    assert!(vs_text.contains("initializeImageryAttributes"));
    assert!(vs_text.contains("v_imagery_texCoord_0 = a_imagery_texCoord_0;"));
}

#[test]
fn test_process_returns_early_when_primitive_imagery_not_ready() {
    let layers_arc = Arc::new(RwLock::new(ImageryLayerCollection::new()));
    let model_options = ImageryModelOptions {
        imagery_layers: Some(layers_arc),
    };

    let mut mpi = ModelPrimitiveImagery::new();
    mpi.ready = false;

    let primitive = ImageryRuntimePrimitive {
        model_primitive_imagery: Some(mpi),
        indices_count: Some(6),
        attribute_count: 4,
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 0;

    let mut rr = ImageryRenderResources {
        model: &model_options,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        attributes: &mut attributes,
        attribute_index: &mut attribute_index,
        count: 4,
    };

    ImageryPipelineStage::process(&mut rr, &primitive, &ImageryFrameState::default());
    let fs = sb.create_fragment_shader_source();
    assert!(fs.defines.is_empty());
    assert!(attributes.is_empty());
}

#[test]
fn test_compute_index_mapping() {
    let source = vec!["Geographic", "WebMercator", "Geographic", "Stereographic"];
    let target = vec!["Geographic", "WebMercator", "Stereographic"];
    let mapping = ImageryPipelineStage::compute_index_mapping(&source, &target);
    assert_eq!(mapping, vec![0, 1, 0, 2]);
}

#[test]
fn test_compute_texture_translation_and_scale_from_native() {
    let native_bounding = NativeRectangle {
        west: 10.0,
        south: 20.0,
        east: 30.0,
        north: 40.0,
    };
    let native_imagery = NativeRectangle {
        west: 0.0,
        south: 10.0,
        east: 40.0,
        north: 50.0,
    };

    let trans_scale = ImageryPipelineStage::compute_texture_translation_and_scale_from_native(
        &native_bounding,
        &native_imagery,
    );

    // width = 40, height = 40
    // delta_west = 10, delta_south = 10
    // offset_x = 10 / 40 = 0.25, offset_y = 10 / 40 = 0.25
    // scale_x = 20 / 40 = 0.5, scale_y = 20 / 40 = 0.5
    assert!((trans_scale.x - 0.25).abs() < 1e-6);
    assert!((trans_scale.y - 0.25).abs() < 1e-6);
    assert!((trans_scale.z - 0.5).abs() < 1e-6);
    assert!((trans_scale.w - 0.5).abs() < 1e-6);
}

#[test]
fn test_skips_failed_or_invalid_imagery() {
    let mut collection = ImageryLayerCollection::new();
    let layer = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            show: Some(true),
            ..Default::default()
        }),
    )));
    let layer_id = layer.read().unwrap().id();
    collection.add(Arc::clone(&layer), None).unwrap();

    let (mpi, imagery_arc) = create_mock_primitive_imagery(layer_id);
    let indices = vec![0];

    // Mark as failed
    imagery_arc.write().unwrap().state = ImageryState::Failed;
    let inputs = ImageryPipelineStage::create_imagery_inputs(&collection, &mpi, &indices);
    assert!(inputs.is_empty());

    // Mark as invalid
    imagery_arc.write().unwrap().state = ImageryState::Invalid;
    let inputs_inv = ImageryPipelineStage::create_imagery_inputs(&collection, &mpi, &indices);
    assert!(inputs_inv.is_empty());
}

#[test]
fn test_all_imagery_layer_flags_and_color_to_alpha() {
    let mut collection = ImageryLayerCollection::new();
    let layer = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            show: Some(true),
            alpha: Some(0.5),
            brightness: Some(1.5),
            contrast: Some(1.2),
            hue: Some(0.3),
            saturation: Some(1.4),
            gamma: Some(2.2),
            color_to_alpha: Some(cesium_math::Color::new(1.0, 0.0, 0.0, 1.0)),
            color_to_alpha_threshold: Some(0.1),
            ..Default::default()
        }),
    )));
    let layer_id = layer.read().unwrap().id();
    collection.add(Arc::clone(&layer), None).unwrap();

    let (mpi, _) = create_mock_primitive_imagery(layer_id);
    let layers_arc = Arc::new(RwLock::new(collection));
    let model_options = ImageryModelOptions {
        imagery_layers: Some(layers_arc),
    };

    let primitive = ImageryRuntimePrimitive {
        model_primitive_imagery: Some(mpi),
        indices_count: Some(6),
        attribute_count: 4,
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 0;

    let mut rr = ImageryRenderResources {
        model: &model_options,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        attributes: &mut attributes,
        attribute_index: &mut attribute_index,
        count: 4,
    };

    ImageryPipelineStage::process(&mut rr, &primitive, &ImageryFrameState::default());

    assert!(uniform_map.contains_key("u_imageryTextureAlpha"));
    assert!(uniform_map.contains_key("u_imageryTextureBrightness"));
    assert!(uniform_map.contains_key("u_imageryTextureContrast"));
    assert!(uniform_map.contains_key("u_imageryTextureHue"));
    assert!(uniform_map.contains_key("u_imageryTextureSaturation"));
    assert!(uniform_map.contains_key("u_imageryTextureOneOverGamma"));
    assert!(uniform_map.contains_key("u_imageryTextureColorToAlpha"));

    let fs = sb.create_fragment_shader_source();
    let fs_text = fs.sources.join("\n");
    assert!(fs_text.contains("u_imageryTextureColorToAlpha"));
    assert!(fs_text.contains("czm_branchFreeTernary"));
}

#[test]
fn test_process_returns_early_when_all_layers_hidden() {
    let mut collection = ImageryLayerCollection::new();
    let layer = Arc::new(RwLock::new(ImageryLayer::new(
        None,
        Some(ImageryLayerOptions {
            show: Some(false),
            ..Default::default()
        }),
    )));
    let layer_id = layer.read().unwrap().id();
    collection.add(Arc::clone(&layer), None).unwrap();

    let (mpi, _) = create_mock_primitive_imagery(layer_id);
    let layers_arc = Arc::new(RwLock::new(collection));
    let model_options = ImageryModelOptions {
        imagery_layers: Some(layers_arc),
    };

    let primitive = ImageryRuntimePrimitive {
        model_primitive_imagery: Some(mpi),
        indices_count: Some(6),
        attribute_count: 4,
    };

    let mut sb = ShaderBuilder::default();
    let mut uniform_map = HashMap::new();
    let mut attributes = Vec::new();
    let mut attribute_index = 0;

    let mut rr = ImageryRenderResources {
        model: &model_options,
        shader_builder: &mut sb,
        uniform_map: &mut uniform_map,
        attributes: &mut attributes,
        attribute_index: &mut attribute_index,
        count: 4,
    };

    ImageryPipelineStage::process(&mut rr, &primitive, &ImageryFrameState::default());
    let fs = sb.create_fragment_shader_source();
    assert!(fs.defines.is_empty());
    assert!(attributes.is_empty());
}

