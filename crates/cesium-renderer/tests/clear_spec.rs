use std::rc::Rc;
use std::sync::Arc;

use cesium_math::{BoundingRectangle, Color};
use cesium_renderer::{
    ClearCommand, ClearCommandOptions, ColorMaskState, Context, ContextOptions, Framebuffer,
    FramebufferOptions, ReadPixelsOptions, RenderState, ScissorTestState, Texture, TextureOptions,
};

#[test]
fn test_default_clear() {
    let mut context = Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap();

    let cmd = ClearCommand {
        color: Some(Color::new(0.0, 0.0, 0.0, 1.0)),
        ..Default::default()
    };
    cmd.execute(&mut context).unwrap();

    let pixels = context.read_pixels(None).unwrap();
    assert_eq!(pixels, vec![0, 0, 0, 255]);
}

#[test]
fn test_clears_to_white() {
    let mut context = Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap();

    let cmd = ClearCommand::new(ClearCommandOptions {
        color: Some(Color::WHITE),
        ..Default::default()
    });
    cmd.execute(&mut context).unwrap();

    let pixels = context.read_pixels(None).unwrap();
    assert_eq!(pixels, vec![255, 255, 255, 255]);
}

#[test]
fn test_clears_with_color_mask() {
    let mut context = Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap();

    // First clear to black
    ClearCommand::new(ClearCommandOptions {
        color: Some(Color::BLACK),
        ..Default::default()
    })
    .execute(&mut context)
    .unwrap();

    // Now clear to white, but mask out green
    let rs = RenderState {
        color_mask: ColorMaskState {
            red: true,
            green: false,
            blue: true,
            alpha: true,
        },
        ..Default::default()
    };

    let cmd = ClearCommand::new(ClearCommandOptions {
        color: Some(Color::WHITE),
        render_state: Some(Arc::new(rs)),
        ..Default::default()
    });
    cmd.execute(&mut context).unwrap();

    let pixels = context.read_pixels(None).unwrap();
    // Red=255, Green=0 (retained), Blue=255, Alpha=255
    assert_eq!(pixels, vec![255, 0, 255, 255]);
}

#[test]
fn test_clears_with_scissor_test() {
    let mut context = Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap();

    // Clear to white
    ClearCommand::new(ClearCommandOptions {
        color: Some(Color::WHITE),
        ..Default::default()
    })
    .execute(&mut context)
    .unwrap();

    assert_eq!(context.read_pixels(None).unwrap(), vec![255, 255, 255, 255]);

    // Scissor test enabled with empty rectangle (nothing inside)
    let rs_empty = RenderState {
        scissor_test: ScissorTestState {
            enabled: true,
            rectangle: BoundingRectangle::default(),
        },
        ..Default::default()
    };

    ClearCommand::new(ClearCommandOptions {
        color: Some(Color::BLACK),
        render_state: Some(Arc::new(rs_empty)),
        ..Default::default()
    })
    .execute(&mut context)
    .unwrap();

    // Still white
    assert_eq!(context.read_pixels(None).unwrap(), vec![255, 255, 255, 255]);

    // Scissor test matching viewport (0, 0, 1, 1)
    let rs_full = RenderState {
        scissor_test: ScissorTestState {
            enabled: true,
            rectangle: BoundingRectangle::new(0.0, 0.0, 1.0, 1.0),
        },
        ..Default::default()
    };

    ClearCommand::new(ClearCommandOptions {
        color: Some(Color::BLACK),
        render_state: Some(Arc::new(rs_full)),
        ..Default::default()
    })
    .execute(&mut context)
    .unwrap();

    // Now black
    assert_eq!(context.read_pixels(None).unwrap(), vec![0, 0, 0, 255]);
}

#[test]
fn test_clears_framebuffer_color_attachment() {
    let mut context = Context::new(ContextOptions {
        width: Some(1),
        height: Some(1),
        ..Default::default()
    })
    .unwrap();

    let color_texture = Texture::new(TextureOptions {
        width: 1,
        height: 1,
        ..Default::default()
    });

    let fb = Rc::new(
        Framebuffer::new(FramebufferOptions {
            color_textures: vec![Some(color_texture)],
            ..Default::default()
        })
        .unwrap(),
    );

    let cmd = ClearCommand::new(ClearCommandOptions {
        color: Some(Color::new(0.0, 1.0, 0.0, 1.0)),
        framebuffer: Some(fb.clone()),
        ..Default::default()
    });
    cmd.execute(&mut context).unwrap();

    let pixels = context
        .read_pixels(Some(ReadPixelsOptions {
            framebuffer: Some(fb),
            ..Default::default()
        }))
        .unwrap();

    assert_eq!(pixels, vec![0, 255, 0, 255]);
}

#[test]
fn test_fails_to_read_pixels_invalid_bounds() {
    let context = Context::default();

    assert!(context
        .read_pixels(Some(ReadPixelsOptions {
            width: -1,
            ..Default::default()
        }))
        .is_err());

    assert!(context
        .read_pixels(Some(ReadPixelsOptions {
            height: -1,
            ..Default::default()
        }))
        .is_err());
}
