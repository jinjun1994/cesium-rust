use cesium_math::Cartesian3;
use cesium_renderer::{CircleEmitter, Particle, ParticleEmitter};

#[test]
fn test_default_constructor() {
    let emitter = CircleEmitter::default();
    assert_eq!(emitter.radius(), 1.0);
}

#[test]
fn test_constructor() {
    let emitter = CircleEmitter::new(Some(5.0)).unwrap();
    assert_eq!(emitter.radius(), 5.0);
}

#[test]
fn test_constructor_throws_with_invalid_radius() {
    assert!(CircleEmitter::new(Some(0.0)).is_err());
    assert!(CircleEmitter::new(Some(-1.0)).is_err());
}

#[test]
fn test_radius_setter() {
    let mut emitter = CircleEmitter::default();
    emitter.set_radius(5.0).unwrap();
    assert_eq!(emitter.radius(), 5.0);
}

#[test]
fn test_radius_setter_throws_with_invalid_value() {
    let mut emitter = CircleEmitter::default();
    assert!(emitter.set_radius(0.0).is_err());
    assert!(emitter.set_radius(-1.0).is_err());
}

#[test]
fn test_emits() {
    let emitter = CircleEmitter::new(Some(5.0)).unwrap();
    let mut particle = Particle::default();

    for _ in 0..1000 {
        emitter.emit(&mut particle);
        assert!(particle.position.magnitude() <= emitter.radius());
        assert_eq!(particle.position.z, 0.0);
        assert_eq!(particle.velocity, Cartesian3::UNIT_Z);
    }
}
