use cesium_renderer::{Particle, ParticleEmitter, SphereEmitter};

#[test]
fn test_default_constructor() {
    let emitter = SphereEmitter::default();
    assert_eq!(emitter.radius(), 1.0);
}

#[test]
fn test_constructor() {
    let emitter = SphereEmitter::new(Some(5.0)).unwrap();
    assert_eq!(emitter.radius(), 5.0);
}

#[test]
fn test_constructor_throws_with_invalid_radius() {
    assert!(SphereEmitter::new(Some(0.0)).is_err());
    assert!(SphereEmitter::new(Some(-1.0)).is_err());
}

#[test]
fn test_radius_setter() {
    let mut emitter = SphereEmitter::default();
    emitter.set_radius(5.0).unwrap();
    assert_eq!(emitter.radius(), 5.0);
}

#[test]
fn test_radius_setter_throws_with_invalid_value() {
    let mut emitter = SphereEmitter::default();
    assert!(emitter.set_radius(0.0).is_err());
    assert!(emitter.set_radius(-1.0).is_err());
}

#[test]
fn test_emits() {
    let emitter = SphereEmitter::new(Some(5.0)).unwrap();
    let mut particle = Particle::default();

    for _ in 0..1000 {
        emitter.emit(&mut particle);
        assert!(particle.position.magnitude() <= emitter.radius());
        assert_eq!(particle.velocity, particle.position.normalize());
    }
}
