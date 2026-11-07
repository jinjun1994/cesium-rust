use cesium_math::{Cartesian3, Cartesian4, Matrix3, Matrix4};
use cesium_renderer::{
    AutomaticUniform, AutomaticUniforms, UniformState, UniformType, UniformValue,
};
use std::sync::{Arc, RwLock};

#[test]
fn test_get_declaration() {
    let u_vec4 = AutomaticUniform::new(1, UniformType::FloatVec4);
    assert_eq!(u_vec4.get_declaration("czm_viewport"), "uniform vec4 czm_viewport;");

    let u_mat4 = AutomaticUniform::new(1, UniformType::FloatMat4);
    assert_eq!(u_mat4.get_declaration("czm_modelView"), "uniform mat4 czm_modelView;");

    let u_float = AutomaticUniform::new(1, UniformType::Float);
    assert_eq!(u_float.get_declaration("czm_frameNumber"), "uniform float czm_frameNumber;");

    // Array syntax
    let u_arr = AutomaticUniform::new(4, UniformType::FloatVec3);
    assert_eq!(u_arr.get_declaration("czm_lights"), "uniform vec3 czm_lights[4];");
}

#[test]
fn test_evaluates_viewports_and_transforms() {
    let state = UniformState {
        viewport: Cartesian4::new(10.0, 20.0, 800.0, 600.0),
        viewport_orthographic: Matrix4::new(
            2.0, 0.0, 0.0, -1.0,
            0.0, 2.0, 0.0, -1.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ),
        ..Default::default()
    };

    let val = AutomaticUniforms::get_value("czm_viewport", &state).unwrap();
    assert_eq!(val, UniformValue::Vec4([10.0, 20.0, 800.0, 600.0]));

    let val_ortho = AutomaticUniforms::get_value("czm_viewportOrthographic", &state).unwrap();
    assert_eq!(val_ortho, UniformValue::from(state.viewport_orthographic));
}

#[test]
fn test_evaluates_matrices() {
    let state = UniformState {
        model: Matrix4::new(
            1.0, 0.0, 0.0, 10.0,
            0.0, 1.0, 0.0, 20.0,
            0.0, 0.0, 1.0, 30.0,
            0.0, 0.0, 0.0, 1.0,
        ),
        view: Matrix4::IDENTITY,
        projection: Matrix4::new(
            1.5, 0.0, 0.0, 0.0,
            0.0, 2.0, 0.0, 0.0,
            0.0, 0.0, -1.0, -2.0,
            0.0, 0.0, -1.0, 0.0,
        ),
        normal: Matrix3::IDENTITY,
        ..Default::default()
    };

    assert_eq!(
        AutomaticUniforms::get_value("czm_model", &state),
        Some(UniformValue::from(state.model))
    );
    assert_eq!(
        AutomaticUniforms::get_value("czm_view", &state),
        Some(UniformValue::from(state.view))
    );
    assert_eq!(
        AutomaticUniforms::get_value("czm_projection", &state),
        Some(UniformValue::from(state.projection))
    );
    assert_eq!(
        AutomaticUniforms::get_value("czm_normal", &state),
        Some(UniformValue::from(state.normal))
    );
}

#[test]
fn test_evaluates_sun_and_frame_number() {
    let state = UniformState {
        sun_direction_ec: Cartesian3::new(0.577, 0.577, 0.577),
        sun_position_wc: Cartesian3::new(149597870700.0, 0.0, 0.0),
        frame_number: 42.0,
        ..Default::default()
    };

    assert_eq!(
        AutomaticUniforms::get_value("czm_sunDirectionEC", &state),
        Some(UniformValue::from(state.sun_direction_ec))
    );
    assert_eq!(
        AutomaticUniforms::get_value("czm_sunPositionWC", &state),
        Some(UniformValue::from(state.sun_position_wc))
    );
    assert_eq!(
        AutomaticUniforms::get_value("czm_frameNumber", &state),
        Some(UniformValue::Float(42.0))
    );
}

#[test]
fn test_create_uniform_map_dynamic_binding() {
    let state = Arc::new(RwLock::new(UniformState::default()));
    let map = AutomaticUniforms::create_uniform_map(Arc::clone(&state));

    // Initial frame number is 0
    assert_eq!(map.evaluate("czm_frameNumber"), Some(UniformValue::Float(0.0)));

    // Update state to frame 100
    {
        let mut st = state.write().unwrap();
        st.frame_number = 100.0;
        st.viewport = Cartesian4::new(0.0, 0.0, 1920.0, 1080.0);
    }

    // Dynamic map reflects updated values
    assert_eq!(map.evaluate("czm_frameNumber"), Some(UniformValue::Float(100.0)));
    assert_eq!(
        map.evaluate("czm_viewport"),
        Some(UniformValue::Vec4([0.0, 0.0, 1920.0, 1080.0]))
    );
}
