//! Direct port and verification against CesiumJS `createMaterialPropertyDescriptorSpec.js` (Apache-2.0).

use cesium_math::{
    Color, ColorMaterialProperty, ImageMaterialProperty, JulianDate, MaterialPropertyDescriptor,
    MaterialValueInput,
};

struct MockGraphics {
    material_property: MaterialPropertyDescriptor,
}

impl MockGraphics {
    fn new() -> Self {
        Self {
            material_property: MaterialPropertyDescriptor::new("materialProperty"),
        }
    }
}

#[test]
fn test_defaults_to_undefined() {
    let instance = MockGraphics::new();
    assert!(instance.material_property.get().is_none());
}

#[test]
fn test_creates_image_material_property_from_string() {
    let mut instance = MockGraphics::new();
    assert!(instance.material_property.get().is_none());

    let value = "test.invalid";
    instance.material_property.set_image(value).unwrap();

    let mat = instance.material_property.get().expect("material should be defined");
    assert_eq!(mat.get_type(&JulianDate::now()), "Image");

    let image_mat = mat
        .as_any()
        .downcast_ref::<ImageMaterialProperty>()
        .expect("should be ImageMaterialProperty");
    assert_eq!(
        image_mat.image.as_ref().unwrap().get_value(None).as_deref(),
        Some(value)
    );
}

#[test]
fn test_creates_color_material_property_from_color() {
    let mut instance = MockGraphics::new();
    assert!(instance.material_property.get().is_none());

    let value = Color::RED;
    instance.material_property.set_color(value).unwrap();

    let mat = instance.material_property.get().expect("material should be defined");
    assert_eq!(mat.get_type(&JulianDate::now()), "Color");

    let color_mat = mat
        .as_any()
        .downcast_ref::<ColorMaterialProperty>()
        .expect("should be ColorMaterialProperty");
    assert_eq!(
        color_mat.color.as_ref().unwrap().get_value(None),
        Some(value)
    );
}

#[test]
fn test_throws_if_type_cannot_be_inferred() {
    let mut instance = MockGraphics::new();
    let err = instance.material_property.set(Some(MaterialValueInput::Invalid));
    assert!(err.is_err());
}
