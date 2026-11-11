//! Direct port and verification against CesiumJS `RenderStateSpec.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{BoundingRectangle, Color, WindingOrder};
use cesium_renderer::{
    BlendEquation, BlendFunction, BlendingState, ColorMaskState, CullFace, CullState, DepthFunction,
    DepthRangeState, DepthTestState, PolygonOffsetState, RenderState, SampleCoverageState,
    ScissorTestState, StencilFunction, StencilOperation, StencilOperationState, StencilTestState,
};

#[test]
fn test_creates_with_defaults() {
    let rs = RenderState::from_cache(None);

    assert_eq!(rs.front_face, WindingOrder::CounterClockwise);
    assert!(!rs.cull.enabled);
    assert_eq!(rs.cull.face, CullFace::Back);
    assert_eq!(rs.line_width, 1);

    assert!(!rs.polygon_offset.enabled);
    assert_eq!(rs.polygon_offset.factor, 0.0);
    assert_eq!(rs.polygon_offset.units, 0.0);

    assert!(!rs.scissor_test.enabled);
    assert_eq!(rs.scissor_test.rectangle.x, 0.0);
    assert_eq!(rs.scissor_test.rectangle.y, 0.0);
    assert_eq!(rs.scissor_test.rectangle.width, 0.0);
    assert_eq!(rs.scissor_test.rectangle.height, 0.0);

    assert_eq!(rs.depth_range.near, 0.0);
    assert_eq!(rs.depth_range.far, 1.0);

    assert!(!rs.depth_test.enabled);
    assert_eq!(rs.depth_test.func, DepthFunction::Less);

    assert!(rs.color_mask.red);
    assert!(rs.color_mask.green);
    assert!(rs.color_mask.blue);
    assert!(rs.color_mask.alpha);

    assert!(rs.depth_mask);
    assert_eq!(rs.stencil_mask, !0);

    assert!(!rs.blending.enabled);
    assert_eq!(rs.blending.color, Color::new(0.0, 0.0, 0.0, 0.0));
    assert_eq!(rs.blending.equation_rgb, BlendEquation::Add);
    assert_eq!(rs.blending.equation_alpha, BlendEquation::Add);
    assert_eq!(rs.blending.function_source_rgb, BlendFunction::One);
    assert_eq!(rs.blending.function_source_alpha, BlendFunction::One);
    assert_eq!(rs.blending.function_destination_rgb, BlendFunction::Zero);
    assert_eq!(
        rs.blending.function_destination_alpha,
        BlendFunction::Zero
    );

    assert!(!rs.stencil_test.enabled);
    assert_eq!(rs.stencil_test.front_function, StencilFunction::Always);
    assert_eq!(rs.stencil_test.back_function, StencilFunction::Always);
    assert_eq!(rs.stencil_test.reference, 0);
    assert_eq!(rs.stencil_test.mask, !0);
    assert_eq!(
        rs.stencil_test.front_operation.fail,
        StencilOperation::Keep
    );
    assert_eq!(
        rs.stencil_test.front_operation.z_fail,
        StencilOperation::Keep
    );
    assert_eq!(
        rs.stencil_test.front_operation.z_pass,
        StencilOperation::Keep
    );
    assert_eq!(rs.stencil_test.back_operation.fail, StencilOperation::Keep);
    assert_eq!(
        rs.stencil_test.back_operation.z_fail,
        StencilOperation::Keep
    );
    assert_eq!(
        rs.stencil_test.back_operation.z_pass,
        StencilOperation::Keep
    );

    assert!(!rs.sample_coverage.enabled);
    assert_eq!(rs.sample_coverage.value, 1.0);
    assert!(!rs.sample_coverage.invert);
}

#[test]
fn test_creates_with_custom_values() {
    let custom = RenderState {
        front_face: WindingOrder::Clockwise,
        cull: CullState {
            enabled: true,
            face: CullFace::Front,
        },
        line_width: 2,
        polygon_offset: PolygonOffsetState {
            enabled: true,
            factor: 1.0,
            units: 2.0,
        },
        scissor_test: ScissorTestState {
            enabled: true,
            rectangle: BoundingRectangle::new(1.0, 2.0, 3.0, 4.0),
        },
        depth_range: DepthRangeState {
            near: 0.1,
            far: 0.9,
        },
        depth_test: DepthTestState {
            enabled: true,
            func: DepthFunction::Greater,
        },
        color_mask: ColorMaskState {
            red: false,
            green: true,
            blue: false,
            alpha: true,
        },
        depth_mask: false,
        stencil_mask: 0x1234,
        blending: BlendingState {
            enabled: true,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            equation_rgb: BlendEquation::Subtract,
            equation_alpha: BlendEquation::Subtract,
            function_source_rgb: BlendFunction::Zero,
            function_source_alpha: BlendFunction::Zero,
            function_destination_rgb: BlendFunction::One,
            function_destination_alpha: BlendFunction::One,
        },
        stencil_test: StencilTestState {
            enabled: true,
            front_function: StencilFunction::Never,
            back_function: StencilFunction::Never,
            reference: 1,
            mask: 0,
            front_operation: StencilOperationState {
                fail: StencilOperation::Replace,
                z_fail: StencilOperation::Replace,
                z_pass: StencilOperation::Replace,
            },
            back_operation: StencilOperationState {
                fail: StencilOperation::Replace,
                z_fail: StencilOperation::Replace,
                z_pass: StencilOperation::Replace,
            },
        },
        sample_coverage: SampleCoverageState {
            enabled: true,
            value: 0.5,
            invert: true,
        },
        viewport: Some(BoundingRectangle::new(10.0, 20.0, 30.0, 40.0)),
    };

    let rs = RenderState::from_cache(Some(custom.clone()));

    assert_eq!(rs.front_face, custom.front_face);
    assert_eq!(rs.cull, custom.cull);
    assert_eq!(rs.line_width, custom.line_width);
    assert_eq!(rs.polygon_offset, custom.polygon_offset);
    assert_eq!(rs.scissor_test, custom.scissor_test);
    assert_eq!(rs.depth_range, custom.depth_range);
    assert_eq!(rs.depth_test, custom.depth_test);
    assert_eq!(rs.color_mask, custom.color_mask);
    assert_eq!(rs.depth_mask, custom.depth_mask);
    assert_eq!(rs.stencil_mask, custom.stencil_mask);
    assert_eq!(rs.blending, custom.blending);
    assert_eq!(rs.stencil_test, custom.stencil_test);
    assert_eq!(rs.sample_coverage, custom.sample_coverage);
    assert_eq!(rs.viewport, custom.viewport);
}

#[test]
fn test_from_cache_deduplication() {
    let rs1 = RenderState::from_cache(None);
    let rs2 = RenderState::from_cache(None);
    assert!(Arc::ptr_eq(&rs1, &rs2));

    let custom = RenderState {
        front_face: WindingOrder::Clockwise,
        ..Default::default()
    };
    let rs3 = RenderState::from_cache(Some(custom.clone()));
    let rs4 = RenderState::from_cache(Some(custom));
    assert!(Arc::ptr_eq(&rs3, &rs4));
    assert!(!Arc::ptr_eq(&rs1, &rs3));
}
