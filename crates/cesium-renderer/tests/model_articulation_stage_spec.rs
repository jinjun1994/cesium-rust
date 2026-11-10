//! Tests for ModelArticulationStage matching CesiumJS `Scene/Model/ModelArticulationStageSpec.js` (Apache-2.0).

use std::cell::RefCell;
use std::rc::Rc;

use cesium_math::{
    ArticulationStageType, Cartesian3, Matrix3, Matrix4, EPSILON14,
};
use cesium_renderer::model_articulation_stage::{
    ArticulationStageConfig, ModelArticulationStage,
};

fn mock_dirty_flag() -> Rc<RefCell<bool>> {
    Rc::new(RefCell::new(false))
}

#[test]
fn test_throws_for_undefined_stage() {
    assert!(ModelArticulationStage::new(None, Some(mock_dirty_flag())).is_err());
}

#[test]
fn test_throws_for_undefined_runtime_articulation() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XTranslate,
        -100.0,
        100.0,
        0.0,
    );
    assert!(ModelArticulationStage::new(Some(stage), None).is_err());
}

#[test]
fn test_constructs() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XTranslate,
        -100.0,
        100.0,
        0.0,
    );
    let dirty_flag = mock_dirty_flag();
    let runtime_stage = ModelArticulationStage::new(Some(stage.clone()), Some(dirty_flag)).unwrap();

    assert_eq!(runtime_stage.stage, stage);
    assert_eq!(runtime_stage.name(), "SampleStage");
    assert_eq!(runtime_stage.stage_type(), ArticulationStageType::XTranslate);
    assert_eq!(runtime_stage.minimum_value(), -100.0);
    assert_eq!(runtime_stage.maximum_value(), 100.0);
    assert_eq!(runtime_stage.current_value(), 0.0);
}

#[test]
fn test_current_value_sets_new_value() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XTranslate,
        -100.0,
        100.0,
        0.0,
    );
    let dirty_flag = mock_dirty_flag();
    let mut runtime_stage =
        ModelArticulationStage::new(Some(stage), Some(dirty_flag.clone())).unwrap();

    assert_eq!(runtime_stage.current_value(), 0.0);

    runtime_stage.set_current_value(50.0);
    assert_eq!(runtime_stage.current_value(), 50.0);
    assert!(*dirty_flag.borrow());
}

#[test]
fn test_current_value_clamps_new_value() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XTranslate,
        -100.0,
        100.0,
        0.0,
    );
    let dirty_flag = mock_dirty_flag();
    let mut runtime_stage =
        ModelArticulationStage::new(Some(stage), Some(dirty_flag.clone())).unwrap();

    assert_eq!(runtime_stage.current_value(), 0.0);

    runtime_stage.set_current_value(150.0);
    assert_eq!(runtime_stage.current_value(), 100.0);
    assert!(*dirty_flag.borrow());

    *dirty_flag.borrow_mut() = false;
    runtime_stage.set_current_value(-150.0);
    assert_eq!(runtime_stage.current_value(), -100.0);
    assert!(*dirty_flag.borrow());
}

#[test]
fn test_current_value_has_no_effect_if_given_value_is_the_same() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XTranslate,
        -100.0,
        100.0,
        0.0,
    );
    let dirty_flag = mock_dirty_flag();
    let mut runtime_stage =
        ModelArticulationStage::new(Some(stage), Some(dirty_flag.clone())).unwrap();

    assert_eq!(runtime_stage.current_value(), 0.0);

    runtime_stage.set_current_value(0.0);
    assert_eq!(runtime_stage.current_value(), 0.0);
    assert!(!*dirty_flag.borrow());
}

#[test]
fn test_apply_stage_to_matrix_x_translate() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XTranslate,
        -100.0,
        100.0,
        50.0,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected = Matrix4::from_translation(Cartesian3::new(50.0, 0.0, 0.0));
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert_eq!(result, expected);
}

#[test]
fn test_apply_stage_to_matrix_y_translate() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::YTranslate,
        -100.0,
        100.0,
        50.0,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected = Matrix4::from_translation(Cartesian3::new(0.0, 50.0, 0.0));
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert_eq!(result, expected);
}

#[test]
fn test_apply_stage_to_matrix_z_translate() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::ZTranslate,
        -100.0,
        100.0,
        50.0,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected = Matrix4::from_translation(Cartesian3::new(0.0, 0.0, 50.0));
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert_eq!(result, expected);
}

#[test]
fn test_apply_stage_to_matrix_x_rotate() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XRotate,
        -360.0,
        360.0,
        180.0,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected_rot = Matrix3::from_rotation_x(180.0_f64.to_radians());
    let expected = Matrix4::IDENTITY.multiply_by_matrix3(&expected_rot);
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert!(result.equals_epsilon(&expected, EPSILON14));
}

#[test]
fn test_apply_stage_to_matrix_y_rotate() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::YRotate,
        -360.0,
        360.0,
        180.0,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected_rot = Matrix3::from_rotation_y(180.0_f64.to_radians());
    let expected = Matrix4::IDENTITY.multiply_by_matrix3(&expected_rot);
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert!(result.equals_epsilon(&expected, EPSILON14));
}

#[test]
fn test_apply_stage_to_matrix_z_rotate() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::ZRotate,
        -360.0,
        360.0,
        180.0,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected_rot = Matrix3::from_rotation_z(180.0_f64.to_radians());
    let expected = Matrix4::IDENTITY.multiply_by_matrix3(&expected_rot);
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert!(result.equals_epsilon(&expected, EPSILON14));
}

#[test]
fn test_apply_stage_to_matrix_x_scale() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::XScale,
        0.0,
        1.0,
        0.5,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected = Matrix4::from_scale(Cartesian3::new(0.5, 1.0, 1.0));
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert_eq!(result, expected);
}

#[test]
fn test_apply_stage_to_matrix_y_scale() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::YScale,
        0.0,
        1.0,
        0.5,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected = Matrix4::from_scale(Cartesian3::new(1.0, 0.5, 1.0));
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert_eq!(result, expected);
}

#[test]
fn test_apply_stage_to_matrix_z_scale() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::ZScale,
        0.0,
        1.0,
        0.5,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected = Matrix4::from_scale(Cartesian3::new(1.0, 1.0, 0.5));
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert_eq!(result, expected);
}

#[test]
fn test_apply_stage_to_matrix_uniform_scale() {
    let stage = ArticulationStageConfig::new(
        "SampleStage",
        ArticulationStageType::UniformScale,
        0.0,
        1.0,
        0.5,
    );
    let runtime_stage = ModelArticulationStage::new(Some(stage), Some(mock_dirty_flag())).unwrap();
    let expected = Matrix4::from_uniform_scale(0.5);
    let result = runtime_stage.apply_stage_to_matrix(&Matrix4::IDENTITY);
    assert_eq!(result, expected);
}
