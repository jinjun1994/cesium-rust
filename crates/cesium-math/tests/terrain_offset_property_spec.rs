//! Direct port and verification against CesiumJS `TerrainOffsetPropertySpec.js` (Apache-2.0).

use std::rc::Rc;

use cesium_math::{
    CallbackProperty, Cartesian3, ConstantProperty, DynProperty, JulianDate, TerrainOffsetProperty,
};

#[test]
fn test_can_construct_and_destroy() {
    let time = JulianDate::now();
    let position: Rc<dyn DynProperty<Cartesian3>> = Rc::new(CallbackProperty::new(
        Rc::new(|_time: &JulianDate| Some(Cartesian3::new(100.0, 200.0, 300.0))),
        false,
    ));
    let height: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(30.0));
    let extruded_height: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(0.0));

    let mut property = TerrainOffsetProperty::new(
        Some(()),
        Some(position),
        Some(height),
        Some(extruded_height),
    )
    .unwrap();

    assert!(!property.is_constant());
    assert_eq!(property.get_value(Some(&time)), Some(Cartesian3::ZERO));
    property.destroy();
    assert!(property.is_destroyed());
}

#[test]
fn test_throws_without_scene() {
    let position: Rc<dyn DynProperty<Cartesian3>> = Rc::new(CallbackProperty::new(
        Rc::new(|_time: &JulianDate| Some(Cartesian3::new(100.0, 200.0, 300.0))),
        false,
    ));
    let height: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(30.0));
    let extruded_height: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(0.0));

    let res = TerrainOffsetProperty::new(
        None,
        Some(position),
        Some(height),
        Some(extruded_height),
    );
    assert!(res.is_err());
}

#[test]
fn test_throws_without_position() {
    let height: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(30.0));
    let extruded_height: Rc<dyn DynProperty<f64>> = Rc::new(ConstantProperty::from_value(0.0));

    let res = TerrainOffsetProperty::new(
        Some(()),
        None,
        Some(height),
        Some(extruded_height),
    );
    assert!(res.is_err());
}
