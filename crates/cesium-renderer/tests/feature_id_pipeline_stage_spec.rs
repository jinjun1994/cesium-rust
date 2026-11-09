use std::collections::HashMap;

use cesium_math::Matrix3;
use cesium_renderer::feature_id_pipeline_stage::{
    FeatureIdPipelineStage, FeatureIdRenderResources, FeatureIdVertexAttribute,
};
use cesium_renderer::model_components::{
    FeatureId, FeatureIdAttribute, FeatureIdImplicitRange, FeatureIdTexture, Instances,
    ModelTextureReader, Primitive,
};
use cesium_renderer::shader_builder::ShaderBuilder;

struct MockSetup {
    shader_builder: ShaderBuilder,
    uniform_map: HashMap<String, Box<dyn Fn() -> serde_json::Value + Send + Sync>>,
    attributes: Vec<FeatureIdVertexAttribute>,
    geometry_bytes: usize,
    textures_bytes: usize,
}

impl MockSetup {
    fn new() -> Self {
        Self {
            shader_builder: ShaderBuilder::default(),
            uniform_map: HashMap::new(),
            attributes: vec![FeatureIdVertexAttribute {
                index: 0,
                instance_divisor: None,
                components_per_attribute: 3,
                stride_in_bytes: 12,
                offset_in_bytes: 0,
                size_in_bytes: 48,
            }],
            geometry_bytes: 0,
            textures_bytes: 0,
        }
    }

    fn as_render_resources(&mut self) -> FeatureIdRenderResources<'_> {
        FeatureIdRenderResources {
            shader_builder: &mut self.shader_builder,
            uniform_map: &mut self.uniform_map,
            attributes: &mut self.attributes,
            statistics_geometry_bytes: &mut self.geometry_bytes,
            statistics_textures_bytes: &mut self.textures_bytes,
        }
    }
}

#[test]
fn test_handles_primitives_without_feature_ids_gracefully() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();

    {
        let mut rr = setup.as_render_resources();
        FeatureIdPipelineStage::process(&mut rr, &primitive, None, &[]);
    }

    let vs = setup.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs.contains("struct FeatureIds"));
    assert!(fs.contains("struct FeatureIds"));
    assert!(vs.contains("featureIdStageVS"));
    assert!(fs.contains("featureIdStageFS"));
    assert_eq!(setup.attributes.len(), 1);
    assert!(setup.uniform_map.is_empty());
    assert_eq!(setup.geometry_bytes, 0);
    assert_eq!(setup.textures_bytes, 0);
}

#[test]
fn test_processes_feature_id_attributes() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();

    let feature_ids = vec![
        FeatureId::ImplicitRange(FeatureIdImplicitRange {
            feature_count: 10,
            label: Some("perPoint".to_string()),
            ..Default::default()
        }),
        FeatureId::Attribute(FeatureIdAttribute {
            feature_count: 10,
            set_index: Some(0),
            label: Some("town".to_string()),
            ..Default::default()
        }),
    ];

    {
        let mut rr = setup.as_render_resources();
        FeatureIdPipelineStage::process(&mut rr, &primitive, None, &feature_ids);
    }

    let vs = setup.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    // Check struct fields
    assert!(vs.contains("int featureId_0;"));
    assert!(vs.contains("int featureId_1;"));
    assert!(vs.contains("int perPoint;"));
    assert!(vs.contains("int town;"));

    // Check initialize functions
    assert!(vs.contains("featureIds.featureId_0 = int(czm_round(a_implicit_featureId_0));"));
    assert!(vs.contains("featureIds.featureId_1 = int(czm_round(attributes.featureId_0));"));
    assert!(fs.contains("featureIds.featureId_0 = int(czm_round(v_implicit_featureId_0));"));
    assert!(fs.contains("featureIds.featureId_1 = int(czm_round(attributes.featureId_0));"));

    // Check aliases
    assert!(vs.contains("featureIds.perPoint = featureIds.featureId_0;"));
    assert!(vs.contains("featureIds.town = featureIds.featureId_1;"));
    assert!(fs.contains("featureIds.perPoint = featureIds.featureId_0;"));
    assert!(fs.contains("featureIds.town = featureIds.featureId_1;"));

    // Check varyings and attributes
    assert!(vs.contains("v_implicit_featureId_0 = a_implicit_featureId_0;"));
    assert!(vs.contains("a_implicit_featureId_0"));
    assert_eq!(setup.attributes.len(), 2);
    assert_eq!(setup.attributes[1].index, 1);
    assert_eq!(setup.geometry_bytes, 16);
}

#[test]
fn test_processes_implicit_feature_id_attribute_with_default_feature_ids() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();

    let feature_ids = vec![
        FeatureId::Attribute(FeatureIdAttribute {
            feature_count: 5,
            set_index: Some(0),
            label: Some("buildings".to_string()),
            ..Default::default()
        }),
        FeatureId::ImplicitRange(FeatureIdImplicitRange {
            feature_count: 5,
            label: Some("defaultIdsTest".to_string()),
            ..Default::default()
        }),
    ];

    {
        let mut rr = setup.as_render_resources();
        FeatureIdPipelineStage::process(&mut rr, &primitive, None, &feature_ids);
    }

    let vs = setup.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs.contains("int buildings;"));
    assert!(vs.contains("int defaultIdsTest;"));
    assert!(vs.contains("featureIds.featureId_0 = int(czm_round(attributes.featureId_0));"));
    assert!(vs.contains("featureIds.featureId_1 = int(czm_round(a_implicit_featureId_1));"));
    assert!(fs.contains("featureIds.featureId_1 = int(czm_round(v_implicit_featureId_1));"));
}

#[test]
fn test_processes_feature_id_texture() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();

    let feature_ids = vec![FeatureId::Texture(FeatureIdTexture {
        feature_count: 4,
        texture_reader: Some(ModelTextureReader {
            index: 0,
            texture: None,
            tex_coord: 0,
            scale: None,
            transform: None,
            channels: Some("r".to_string()),
        }),
        label: Some("landCover".to_string()),
        ..Default::default()
    })];

    {
        let mut rr = setup.as_render_resources();
        FeatureIdPipelineStage::process(&mut rr, &primitive, None, &feature_ids);
    }

    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(fs.contains("int featureId_0;"));
    assert!(fs.contains("int landCover;"));
    assert!(fs.contains("featureIds.featureId_0 = czm_unpackUint(texture(u_featureIdTexture_0, v_texCoord_0).r);"));
    assert!(fs.contains("featureIds.landCover = featureIds.featureId_0;"));
    assert!(setup.uniform_map.contains_key("u_featureIdTexture_0"));
}

#[test]
fn test_adds_feature_id_texture_transforms_to_the_shader() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();

    let feature_ids = vec![FeatureId::Texture(FeatureIdTexture {
        feature_count: 4,
        texture_reader: Some(ModelTextureReader {
            index: 0,
            texture: None,
            tex_coord: 0,
            scale: None,
            transform: Some(Matrix3::IDENTITY),
            channels: Some("r".to_string()),
        }),
        label: None,
        ..Default::default()
    })];

    {
        let mut rr = setup.as_render_resources();
        FeatureIdPipelineStage::process(&mut rr, &primitive, None, &feature_ids);
    }

    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(fs.contains("u_featureIdTexture_0Transform"));
    assert!(fs.contains("vec2(u_featureIdTexture_0Transform * vec3(v_texCoord_0, 1.0))"));
}

#[test]
fn test_processes_feature_id_textures_with_multiple_channels() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();

    let channels_list = ["rgba", "rgb", "g", "ba", "gr", "agbb", "g"];
    let labels_list = [
        "idsRGBA",
        "idsRGB",
        "idsG",
        "idsBA",
        "idsGR",
        "idsAGBB",
        "idsGWithNull",
    ];

    let mut feature_ids = Vec::new();
    for (i, (&chan, &lbl)) in channels_list.iter().zip(labels_list.iter()).enumerate() {
        feature_ids.push(FeatureId::Texture(FeatureIdTexture {
            feature_count: 10,
            texture_reader: Some(ModelTextureReader {
                index: i,
                texture: None,
                tex_coord: 0,
                scale: None,
                transform: None,
                channels: Some(chan.to_string()),
            }),
            label: Some(lbl.to_string()),
            ..Default::default()
        }));
    }

    {
        let mut rr = setup.as_render_resources();
        FeatureIdPipelineStage::process(&mut rr, &primitive, None, &feature_ids);
    }

    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    for i in 0..7 {
        assert!(fs.contains(&format!("featureId_{}", i)));
        assert!(fs.contains(&format!("u_featureIdTexture_{}", i)));
    }
    for &lbl in &labels_list {
        assert!(fs.contains(lbl));
    }
    assert!(fs.contains(".rgba);"));
    assert!(fs.contains(".rgb);"));
    assert!(fs.contains(".ba);"));
    assert!(fs.contains(".gr);"));
    assert!(fs.contains(".agbb);"));
}

#[test]
fn test_processes_instance_feature_ids() {
    let mut setup = MockSetup::new();
    let primitive = Primitive::default();

    let instance_fids = vec![
        FeatureId::ImplicitRange(FeatureIdImplicitRange {
            feature_count: 5,
            label: Some("perInstance".to_string()),
            ..Default::default()
        }),
        FeatureId::Attribute(FeatureIdAttribute {
            feature_count: 5,
            set_index: Some(0),
            label: Some("section".to_string()),
            ..Default::default()
        }),
    ];

    let instances = Instances {
        feature_ids: instance_fids,
        ..Default::default()
    };

    {
        let mut rr = setup.as_render_resources();
        FeatureIdPipelineStage::process(&mut rr, &primitive, Some(&instances), &[]);
    }

    let vs = setup.shader_builder.create_vertex_shader_source().sources.join("\n");
    let fs = setup.shader_builder.create_fragment_shader_source().sources.join("\n");

    assert!(vs.contains("int instanceFeatureId_0;"));
    assert!(vs.contains("int instanceFeatureId_1;"));
    assert!(vs.contains("int perInstance;"));
    assert!(vs.contains("int section;"));

    assert!(vs.contains("featureIds.instanceFeatureId_0 = int(czm_round(a_implicit_instanceFeatureId_0));"));
    assert!(vs.contains("featureIds.instanceFeatureId_1 = int(czm_round(a_instanceFeatureId_0));"));
    assert!(fs.contains("featureIds.instanceFeatureId_0 = int(czm_round(v_implicit_instanceFeatureId_0));"));
    assert!(fs.contains("featureIds.instanceFeatureId_1 = int(czm_round(v_instanceFeatureId_0));"));

    assert!(vs.contains("featureIds.perInstance = featureIds.instanceFeatureId_0;"));
    assert!(vs.contains("featureIds.section = featureIds.instanceFeatureId_1;"));

    assert_eq!(setup.attributes.len(), 2);
    let inst_attr = &setup.attributes[1];
    assert_eq!(inst_attr.instance_divisor, Some(1));
    assert_eq!(setup.geometry_bytes, 64);
}
