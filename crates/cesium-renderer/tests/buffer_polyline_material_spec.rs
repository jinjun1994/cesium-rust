use cesium_math::Color;
use cesium_renderer::{BufferPolylineMaterial, BufferPolylineMaterialOptions};

#[test]
fn test_constructor() {
    let material = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::RED),
        outline_color: Some(Color::GREEN),
        outline_width: Some(4),
        width: Some(5.0),
    }));

    assert_eq!(material.color, Color::RED);
    assert_eq!(material.outline_color, Color::GREEN);
    assert_eq!(material.outline_width, 4);
    assert_eq!(material.width, 5.0);
}

#[test]
fn test_pack_and_unpack() {
    let material1 = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::RED),
        outline_color: Some(Color::GREEN),
        outline_width: Some(1),
        width: Some(2.0),
    }));

    let material2 = BufferPolylineMaterial::new(Some(BufferPolylineMaterialOptions {
        color: Some(Color::BLUE),
        outline_color: Some(Color::BLACK),
        outline_width: Some(3),
        width: Some(4.0),
    }));

    let mut result = BufferPolylineMaterial::default();
    let mut view = vec![0u8; BufferPolylineMaterial::PACKED_LENGTH * 3];

    material1.pack(&mut view, 0);
    material2.pack(&mut view, BufferPolylineMaterial::PACKED_LENGTH);

    BufferPolylineMaterial::unpack(&view, 0, &mut result);
    assert_eq!(result.color, Color::RED);
    assert_eq!(result.outline_color, Color::GREEN);
    assert_eq!(result.outline_width, 1);
    assert_eq!(result.width, 2.0);

    BufferPolylineMaterial::unpack(&view, BufferPolylineMaterial::PACKED_LENGTH, &mut result);
    assert_eq!(result.color, Color::BLUE);
    assert_eq!(result.outline_color, Color::BLACK);
    assert_eq!(result.outline_width, 3);
    assert_eq!(result.width, 4.0);
}
