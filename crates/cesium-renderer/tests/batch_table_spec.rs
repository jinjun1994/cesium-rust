use cesium_math::{Cartesian4, ComponentDatatype};
use cesium_renderer::{BatchTable, BatchTableAttribute, PixelDatatype};

fn unsigned_byte_attributes() -> Vec<BatchTableAttribute> {
    vec![
        BatchTableAttribute::new("batchTable_getShow", ComponentDatatype::UnsignedByte, 1),
        BatchTableAttribute::new("batchTable_getPickColor", ComponentDatatype::UnsignedByte, 4)
            .with_normalize(true),
    ]
}

fn float_attributes() -> Vec<BatchTableAttribute> {
    vec![
        BatchTableAttribute::new("batchTable_getShow", ComponentDatatype::UnsignedByte, 1),
        BatchTableAttribute::new("batchTable_getCenter", ComponentDatatype::Float, 4),
    ]
}

#[test]
fn test_constructor_validation() {
    assert!(BatchTable::new(vec![], 5, false).is_err());
    assert!(BatchTable::new(unsigned_byte_attributes(), 0, false).is_err());

    let bt = BatchTable::new(unsigned_byte_attributes(), 2, false).unwrap();
    assert_eq!(bt.attributes().len(), 2);
    assert_eq!(bt.number_of_instances(), 2);
    assert!(!bt.is_destroyed());
}

#[test]
fn test_set_and_get_unsigned_byte_and_float_and_forced_packing() {
    // UnsignedByte
    let mut bt = BatchTable::new(unsigned_byte_attributes(), 5, false).unwrap();
    let color = Cartesian4::new(0.0, 1.0, 2.0, 3.0);
    for i in 0..5 {
        bt.set_batched_scalar(i, 0, 1.0).unwrap();
        bt.set_batched_attribute(i, 1, color).unwrap();
    }
    assert_eq!(bt.get_batched_attribute(3, 0).unwrap().x, 1.0);
    assert_eq!(bt.get_batched_attribute(3, 1).unwrap(), color);

    let color2 = Cartesian4::new(4.0, 5.0, 6.0, 7.0);
    bt.set_batched_scalar(3, 0, 0.0).unwrap();
    bt.set_batched_attribute(3, 1, color2).unwrap();
    assert_eq!(bt.get_batched_attribute(3, 0).unwrap().x, 0.0);
    assert_eq!(bt.get_batched_attribute(3, 1).unwrap(), color2);

    // Float texture
    let mut bt_float = BatchTable::new(float_attributes(), 5, true).unwrap();
    assert_eq!(bt_float.pixel_datatype(), PixelDatatype::Float);
    let big = Cartesian4::new(1.25e6, -2.5e12, 3.5, -4.5);
    bt_float.set_batched_attribute(2, 1, big).unwrap();
    let got_float = bt_float.get_batched_attribute(2, 1).unwrap();
    assert!((got_float.x - big.x).abs() / big.x.abs() < 1e-6);
    assert!((got_float.y - big.y).abs() / big.y.abs() < 1e-6);

    // Forced packing (float attribute on UnsignedByte texture)
    let mut bt_packed = BatchTable::new(float_attributes(), 5, false).unwrap();
    assert_eq!(bt_packed.pixel_datatype(), PixelDatatype::UnsignedByte);
    bt_packed.set_batched_attribute(1, 1, big).unwrap();
    let got_packed = bt_packed.get_batched_attribute(1, 1).unwrap();
    assert!((got_packed.x - big.x).abs() / big.x.abs() < 1e-6);
    assert!((got_packed.y - big.y).abs() / big.y.abs() < 1e-6);
}

#[test]
fn test_out_of_range_and_uniforms_and_shader_and_destroy() {
    let mut bt = BatchTable::new(unsigned_byte_attributes(), 5, false).unwrap();
    assert!(bt.get_batched_attribute(5, 0).is_err());
    assert!(bt.get_batched_attribute(0, 2).is_err());
    assert!(bt.set_batched_scalar(5, 0, 1.0).is_err());
    assert!(bt.set_batched_scalar(0, 2, 1.0).is_err());

    let tex = bt.update();
    assert_eq!(tex.pixel_datatype, PixelDatatype::UnsignedByte);
    assert!(tex.dimensions.x > 0.0);
    assert!(tex.dimensions.y > 0.0);
    assert!(tex.step.x > 0.0 && tex.step.y > 0.0 && tex.step.z > 0.0 && tex.step.w > 0.0);

    let shader = "void main() { gl_Position = vec4(0.0); }";
    let modified = bt.get_vertex_shader_callback(shader);
    assert!(modified.contains("batchTable_getShow"));
    assert!(modified.contains("batchTable_getPickColor"));

    bt.destroy();
    assert!(bt.is_destroyed());
}
