use cesium_math::Color;
use cesium_renderer::{BufferPointMaterial, BufferPointMaterialOptions};

#[test]
fn test_constructor() {
    let material = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::RED),
        outline_color: Some(Color::GREEN),
        outline_width: Some(4),
        size: Some(5),
    }));

    assert_eq!(material.color, Color::RED);
    assert_eq!(material.outline_color, Color::GREEN);
    assert_eq!(material.outline_width, 4);
    assert_eq!(material.size, 5);
}

#[test]
fn test_pack_and_unpack() {
    let material1 = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::RED),
        outline_color: Some(Color::GREEN),
        outline_width: Some(1),
        size: Some(2),
    }));

    let material2 = BufferPointMaterial::new(Some(BufferPointMaterialOptions {
        color: Some(Color::BLUE),
        outline_color: Some(Color::BLACK),
        outline_width: Some(3),
        size: Some(4),
    }));

    let mut result = BufferPointMaterial::default();
    let mut view = vec![0u8; BufferPointMaterial::PACKED_LENGTH * 3];

    material1.pack(&mut view, 0);
    material2.pack(&mut view, BufferPointMaterial::PACKED_LENGTH);

    BufferPointMaterial::unpack(&view, 0, &mut result);
    assert_eq!(result.color, Color::RED);
    assert_eq!(result.outline_color, Color::GREEN);
    assert_eq!(result.outline_width, 1);
    assert_eq!(result.size, 2);

    BufferPointMaterial::unpack(&view, BufferPointMaterial::PACKED_LENGTH, &mut result);
    assert_eq!(result.color, Color::BLUE);
    assert_eq!(result.outline_color, Color::BLACK);
    assert_eq!(result.outline_width, 3);
    assert_eq!(result.size, 4);
}
