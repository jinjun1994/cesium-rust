//! Tests for ModelArticulation matching CesiumJS `Scene/Model/ModelArticulationSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    ArticulationStageType, Cartesian3, Matrix3, Matrix4, EPSILON14,
};
use cesium_renderer::model_animation_channel::ModelRuntimeNode;
use cesium_renderer::model_articulation::{
    ArticulationConfig, ModelArticulation, ModelArticulationSceneGraphContext,
};
use cesium_renderer::model_articulation_stage::ArticulationStageConfig;

fn create_translate_stage(name: &str, value: f64) -> ArticulationStageConfig {
    ArticulationStageConfig::new(
        name,
        ArticulationStageType::XTranslate,
        -100.0,
        100.0,
        value,
    )
}

fn create_rotate_stage(name: &str, value: f64) -> ArticulationStageConfig {
    ArticulationStageConfig::new(
        name,
        ArticulationStageType::ZRotate,
        -360.0,
        360.0,
        value,
    )
}

fn create_scale_stage(name: &str, value: f64) -> ArticulationStageConfig {
    ArticulationStageConfig::new(
        name,
        ArticulationStageType::UniformScale,
        0.0,
        1.0,
        value,
    )
}

fn create_sample_articulation() -> ArticulationConfig {
    ArticulationConfig::new(
        "SampleArticulation",
        vec![
            create_translate_stage("MoveX", 50.0),
            create_rotate_stage("RotateZ", 180.0),
            create_scale_stage("Size", 0.5),
        ],
    )
}

#[test]
fn test_throws_for_undefined_articulation() {
    assert!(ModelArticulation::new(None, Some(ModelArticulationSceneGraphContext)).is_err());
}

#[test]
fn test_throws_for_undefined_scene_graph() {
    let articulation = create_sample_articulation();
    assert!(ModelArticulation::new(Some(articulation), None).is_err());
}

#[test]
fn test_constructs() {
    let articulation = create_sample_articulation();
    let runtime_articulation =
        ModelArticulation::new(Some(articulation.clone()), Some(ModelArticulationSceneGraphContext))
            .unwrap();

    assert_eq!(runtime_articulation.articulation, articulation);
    assert_eq!(runtime_articulation.name(), "SampleArticulation");

    let stages = &runtime_articulation.runtime_stages;
    assert_eq!(stages.len(), 3);

    assert_eq!(stages[0].name(), "MoveX");
    assert_eq!(stages[0].current_value(), 50.0);

    assert_eq!(stages[1].name(), "RotateZ");
    assert_eq!(stages[1].current_value(), 180.0);

    assert_eq!(stages[2].name(), "Size");
    assert_eq!(stages[2].current_value(), 0.5);

    assert_eq!(runtime_articulation.runtime_nodes.len(), 0);
    assert!(runtime_articulation.is_dirty());
}

#[test]
fn test_set_articulation_stage_does_not_throw_if_stage_name_doesnt_exist() {
    let articulation = create_sample_articulation();
    let mut runtime_articulation =
        ModelArticulation::new(Some(articulation), Some(ModelArticulationSceneGraphContext))
            .unwrap();

    assert_eq!(runtime_articulation.runtime_stages[0].current_value(), 50.0);
    assert_eq!(runtime_articulation.runtime_stages[1].current_value(), 180.0);
    assert_eq!(runtime_articulation.runtime_stages[2].current_value(), 0.5);

    runtime_articulation.set_dirty(false);
    runtime_articulation.set_articulation_stage("NonexistentStage", 100.0);

    assert_eq!(runtime_articulation.runtime_stages[0].current_value(), 50.0);
    assert_eq!(runtime_articulation.runtime_stages[1].current_value(), 180.0);
    assert_eq!(runtime_articulation.runtime_stages[2].current_value(), 0.5);
    assert!(!runtime_articulation.is_dirty());
}

#[test]
fn test_set_articulation_stage_works() {
    let articulation = create_sample_articulation();
    let mut runtime_articulation =
        ModelArticulation::new(Some(articulation), Some(ModelArticulationSceneGraphContext))
            .unwrap();

    runtime_articulation.set_dirty(false);

    assert_eq!(runtime_articulation.runtime_stages[0].current_value(), 50.0);
    assert_eq!(runtime_articulation.runtime_stages[1].current_value(), 180.0);
    assert_eq!(runtime_articulation.runtime_stages[2].current_value(), 0.5);

    runtime_articulation.set_articulation_stage("MoveX", 0.0);

    assert_eq!(runtime_articulation.runtime_stages[0].current_value(), 0.0);
    assert_eq!(runtime_articulation.runtime_stages[1].current_value(), 180.0);
    assert_eq!(runtime_articulation.runtime_stages[2].current_value(), 0.5);
    assert!(runtime_articulation.is_dirty());

    runtime_articulation.set_dirty(false);

    // This value should be clamped by the stage's minimum value
    runtime_articulation.set_articulation_stage("RotateZ", -480.0);
    assert_eq!(runtime_articulation.runtime_stages[0].current_value(), 0.0);
    assert_eq!(runtime_articulation.runtime_stages[1].current_value(), -360.0);
    assert_eq!(runtime_articulation.runtime_stages[2].current_value(), 0.5);
    assert!(runtime_articulation.is_dirty());

    runtime_articulation.set_dirty(false);

    // This value should be clamped by the stage's maximum value
    runtime_articulation.set_articulation_stage("Size", 2.0);
    assert_eq!(runtime_articulation.runtime_stages[0].current_value(), 0.0);
    assert_eq!(runtime_articulation.runtime_stages[1].current_value(), -360.0);
    assert_eq!(runtime_articulation.runtime_stages[2].current_value(), 1.0);
    assert!(runtime_articulation.is_dirty());
}

#[test]
fn test_apply_does_nothing_if_articulation_is_not_dirty() {
    let articulation = create_sample_articulation();
    let mut runtime_articulation =
        ModelArticulation::new(Some(articulation), Some(ModelArticulationSceneGraphContext))
            .unwrap();
    runtime_articulation.set_dirty(false);

    let transform = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    let node = Rc::new(RefCell::new(ModelRuntimeNode::with_transform(transform)));
    runtime_articulation.runtime_nodes.push(node.clone());

    assert_eq!(node.borrow().transform, transform);
    assert!(!runtime_articulation.is_dirty());

    runtime_articulation.apply();
    assert_eq!(node.borrow().transform, transform);
}

#[test]
fn test_apply_works() {
    let articulation = create_sample_articulation();
    let mut runtime_articulation =
        ModelArticulation::new(Some(articulation), Some(ModelArticulationSceneGraphContext))
            .unwrap();

    let transform = Matrix4::from_translation(Cartesian3::new(1.0, 2.0, 3.0));
    let node = Rc::new(RefCell::new(ModelRuntimeNode::with_transform(transform)));
    runtime_articulation.runtime_nodes.push(node.clone());

    assert!(runtime_articulation.is_dirty());

    let mut expected_matrix = Matrix4::from_translation(Cartesian3::new(50.0, 0.0, 0.0));
    let rotation = 180.0_f64.to_radians();
    let expected_rotation = Matrix3::from_rotation_z(rotation);

    expected_matrix = expected_matrix.multiply_by_matrix3(&expected_rotation);
    expected_matrix = expected_matrix.multiply_by_uniform_scale(0.5);
    expected_matrix = transform.multiply_transformation(&expected_matrix);

    runtime_articulation.apply();

    assert!(!runtime_articulation.is_dirty());
    assert!(node.borrow().transform.equals_epsilon(&expected_matrix, EPSILON14));
    assert_eq!(node.borrow().original_transform, transform);
}
