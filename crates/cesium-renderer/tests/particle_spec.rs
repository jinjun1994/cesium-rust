use cesium_math::{Cartesian2, Cartesian3, Color};
use cesium_renderer::{Particle, ParticleOptions};

#[test]
fn test_default_constructor() {
    let p = Particle::default();
    assert_eq!(p.mass, 1.0);
    assert_eq!(p.position, Cartesian3::ZERO);
    assert_eq!(p.velocity, Cartesian3::ZERO);
    assert_eq!(p.life, f64::MAX);
    assert_eq!(p.image, None);
    assert_eq!(p.start_color, Color::WHITE);
    assert_eq!(p.end_color, Color::WHITE);
    assert_eq!(p.start_scale, 1.0);
    assert_eq!(p.end_scale, 1.0);
    assert_eq!(p.image_size, Cartesian2::new(1.0, 1.0));
}

#[test]
fn test_constructor_with_options() {
    let options = ParticleOptions {
        mass: Some(10.0),
        position: Some(Cartesian3::new(1.0, 2.0, 3.0)),
        velocity: Some(Cartesian3::new(4.0, 5.0, 6.0)),
        life: Some(15.0),
        image: Some("url/to/image".to_string()),
        start_color: Some(Color::MAGENTA),
        end_color: Some(Color::LIME),
        start_scale: Some(0.5),
        end_scale: Some(20.0),
        image_size: Some(Cartesian2::new(7.0, 8.0)),
    };
    let p = Particle::new(Some(options.clone()));
    assert_eq!(p.mass, options.mass.unwrap());
    assert_eq!(p.position, options.position.unwrap());
    assert_eq!(p.velocity, options.velocity.unwrap());
    assert_eq!(p.life, options.life.unwrap());
    assert_eq!(p.image, options.image);
    assert_eq!(p.start_color, options.start_color.unwrap());
    assert_eq!(p.end_color, options.end_color.unwrap());
    assert_eq!(p.start_scale, options.start_scale.unwrap());
    assert_eq!(p.end_scale, options.end_scale.unwrap());
    assert_eq!(p.image_size, options.image_size.unwrap());
}

#[test]
fn test_update_without_update_function() {
    let position = Cartesian3::new(1.0, 2.0, 3.0);
    let velocity = Cartesian3::new(-1.0, 1.0, 1.0).normalize();
    let mut p = Particle::new(Some(ParticleOptions {
        life: Some(15.0),
        position: Some(position),
        velocity: Some(velocity),
        ..Default::default()
    }));

    let dt = 10.0;
    let expected_position = p.position + p.velocity * dt;

    assert!(p.update::<fn(&mut Particle, f64)>(dt, None));
    assert_eq!(p.position, expected_position);
    assert_eq!(p.velocity, velocity);
    assert_eq!(p.age(), dt);
    assert_eq!(p.normalized_age(), dt / p.life);
    // Second update takes total age to 20.0 > life (15.0) -> false
    assert!(!p.update::<fn(&mut Particle, f64)>(dt, None));
}

#[test]
fn test_update_with_update_function() {
    let increase_mass = |particle: &mut Particle, _dt: f64| {
        particle.mass += 1.0;
    };

    let position = Cartesian3::new(1.0, 2.0, 3.0);
    let velocity = Cartesian3::new(-1.0, 1.0, 1.0).normalize();
    let mut p = Particle::new(Some(ParticleOptions {
        life: Some(15.0),
        position: Some(position),
        velocity: Some(velocity),
        ..Default::default()
    }));

    let dt = 10.0;
    let expected_mass = p.mass + 1.0;
    let expected_position = p.position + p.velocity * dt;

    assert!(p.update(dt, Some(increase_mass)));
    assert_eq!(p.position, expected_position);
    assert_eq!(p.velocity, velocity);
    assert_eq!(p.age(), dt);
    assert_eq!(p.normalized_age(), dt / p.life);
    assert_eq!(p.mass, expected_mass);
    assert!(!p.update(dt, Some(increase_mass)));
}
