use cesium_math::{Cartesian3, EPSILON14};
use cesium_renderer::{ConeEmitter, Particle, ParticleEmitter};
use std::f64::consts::FRAC_PI_6;

#[test]
fn test_default_constructor() {
    let emitter = ConeEmitter::default();
    assert_eq!(emitter.angle(), 30.0_f64.to_radians());
}

#[test]
fn test_constructor() {
    let emitter = ConeEmitter::new(Some(FRAC_PI_6));
    assert_eq!(emitter.angle(), FRAC_PI_6);
}

#[test]
fn test_angle_setter() {
    let mut emitter = ConeEmitter::default();
    emitter.set_angle(FRAC_PI_6);
    assert_eq!(emitter.angle(), FRAC_PI_6);
}

#[test]
fn test_emits() {
    let emitter = ConeEmitter::new(Some(FRAC_PI_6));
    let mut particle = Particle::default();

    for _ in 0..1000 {
        emitter.emit(&mut particle);
        assert_eq!(particle.position, Cartesian3::ZERO);
        let mag = particle.velocity.magnitude();
        assert!(
            (mag - 1.0).abs() < EPSILON14,
            "Velocity magnitude {} not close to 1.0",
            mag
        );

        // acos(dot(unit v, unit z)) = acos(v.z) <= angle
        let angle_from_z = particle.velocity.z.clamp(-1.0, 1.0).acos();
        assert!(
            angle_from_z <= emitter.angle() + 1e-12,
            "angle_from_z {} exceeds emitter angle {}",
            angle_from_z,
            emitter.angle()
        );
    }
}
