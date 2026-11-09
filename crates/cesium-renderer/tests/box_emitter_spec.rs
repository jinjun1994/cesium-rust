use cesium_math::Cartesian3;
use cesium_renderer::{BoxEmitter, Particle, ParticleEmitter};

#[test]
fn test_default_constructor() {
    let emitter = BoxEmitter::default();
    assert_eq!(emitter.dimensions(), Cartesian3::new(1.0, 1.0, 1.0));
}

#[test]
fn test_constructor() {
    let dimensions = Cartesian3::new(2.0, 3.0, 4.0);
    let emitter = BoxEmitter::new(Some(dimensions)).unwrap();
    assert_eq!(emitter.dimensions(), dimensions);
}

#[test]
fn test_constructor_throws_with_invalid_dimensions() {
    assert!(BoxEmitter::new(Some(Cartesian3::new(-1.0, 1.0, 1.0))).is_err());
    assert!(BoxEmitter::new(Some(Cartesian3::new(1.0, -1.0, 1.0))).is_err());
    assert!(BoxEmitter::new(Some(Cartesian3::new(1.0, 1.0, -1.0))).is_err());
}

#[test]
fn test_dimensions_setter() {
    let mut emitter = BoxEmitter::default();
    let dimensions = Cartesian3::new(2.0, 3.0, 4.0);
    emitter.set_dimensions(dimensions).unwrap();
    assert_eq!(emitter.dimensions(), dimensions);
}

#[test]
fn test_dimensions_setter_throws_with_invalid_value() {
    let mut emitter = BoxEmitter::default();
    assert!(emitter.set_dimensions(Cartesian3::new(-1.0, 1.0, 1.0)).is_err());
    assert!(emitter.set_dimensions(Cartesian3::new(1.0, -1.0, 1.0)).is_err());
    assert!(emitter.set_dimensions(Cartesian3::new(1.0, 1.0, -1.0)).is_err());
}

#[test]
fn test_emits() {
    let dimensions = Cartesian3::new(2.0, 3.0, 4.0);
    let emitter = BoxEmitter::new(Some(dimensions)).unwrap();
    let mut particle = Particle::default();

    for _ in 0..1000 {
        emitter.emit(&mut particle);
        assert!(particle.position.x <= emitter.dimensions().x);
        assert!(particle.position.y <= emitter.dimensions().y);
        assert!(particle.position.z <= emitter.dimensions().z);
        assert_eq!(particle.velocity, particle.position.normalize());
    }
}
