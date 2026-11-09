use cesium_math::Color;
use cesium_renderer::{BufferPolygonMaterial, BufferPolygonMaterialOptions};

#[test]
fn test_constructor() {
    let material = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::RED),
        outline_color: Some(Color::GREEN),
        outline_width: Some(4),
    }));

    assert_eq!(material.color, Color::RED);
    assert_eq!(material.outline_color, Color::GREEN);
    assert_eq!(material.outline_width, 4);
}

#[test]
fn test_pack_and_unpack() {
    let material1 = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::RED),
        outline_color: Some(Color::GREEN),
        outline_width: Some(1),
    }));

    let material2 = BufferPolygonMaterial::new(Some(BufferPolygonMaterialOptions {
        color: Some(Color::BLUE),
        outline_color: Some(Color::BLACK),
        outline_width: Some(3),
    }));

    let mut result = BufferPolygonMaterial::default();
    let mut view = vec![0u8; BufferPolygonMaterial::PACKED_LENGTH * 3];

    material1.pack(&mut view, 0);
    material2.pack(&mut view, BufferPolygonMaterial::PACKED_LENGTH);

    BufferPolygonMaterial::unpack(&view, 0, &mut result);
    assert_eq!(result.color, Color::RED);
    assert_eq!(result.outline_color, Color::GREEN);
    assert_eq!(result.outline_width, 1);

    BufferPolygonMaterial::unpack(&view, BufferPolygonMaterial::PACKED_LENGTH, &mut result);
    assert_eq!(result.color, Color::BLUE);
    assert_eq!(result.outline_color, Color::BLACK);
    assert_eq!(result.outline_width, 3);
}
