//! ParticleSystemSpec matching CesiumJS `Specs/Scene/ParticleSystemSpec.js` (Apache-2.0).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use cesium_math::{Cartesian2, Color, Matrix4};
use cesium_renderer::particle_emitter::CircleEmitter;
use cesium_renderer::particle_system::{
    ParticleBurst, ParticleSystem, ParticleSystemOptions,
};

#[test]
fn test_default_constructor() {
    let p = ParticleSystem::default();
    assert!(p.show);
    assert!(p.forces.is_none());
    assert!(p.update_callback.is_none());
    assert_eq!(*p.model_matrix(), Matrix4::IDENTITY);
    assert_eq!(*p.emitter_model_matrix(), Matrix4::IDENTITY);
    assert_eq!(p.start_color(), Color::WHITE);
    assert_eq!(p.end_color(), Color::WHITE);
    assert_eq!(p.start_scale(), 1.0);
    assert_eq!(p.end_scale(), 1.0);
    assert_eq!(p.emission_rate(), 5.0);
    assert!(p.bursts.is_none());
    assert!(p.loop_system);
    assert_eq!(p.minimum_speed(), 1.0);
    assert_eq!(p.maximum_speed(), 1.0);
    assert_eq!(p.minimum_particle_life(), 5.0);
    assert_eq!(p.maximum_particle_life(), 5.0);
    assert_eq!(p.minimum_mass(), 1.0);
    assert_eq!(p.maximum_mass(), 1.0);
    assert!(p.image.is_none());
    assert_eq!(p.minimum_image_size(), Cartesian2::new(1.0, 1.0));
    assert_eq!(p.maximum_image_size(), Cartesian2::new(1.0, 1.0));
    assert_eq!(p.lifetime(), f64::MAX);
    assert!(!p.is_complete());
    assert!(!p.size_in_meters);
}

#[test]
fn test_constructor_with_options() {
    let update_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = update_counter.clone();

    let options = ParticleSystemOptions {
        show: Some(false),
        update_callback: Some(Arc::new(move |part, _dt| {
            part.mass += 1.0;
            counter_clone.fetch_add(1, Ordering::SeqCst);
        })),
        emitter: Some(Box::new(CircleEmitter::new(Some(10.0)).unwrap())),
        model_matrix: Some(Matrix4::new(
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        )),
        emitter_model_matrix: Some(Matrix4::new(
            10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            1.0,
        )),
        start_color: Some(Color::MAGENTA),
        end_color: Some(Color::new(1.0, 0.94, 0.96, 1.0)),
        start_scale: Some(19.0),
        end_scale: Some(20.0),
        emission_rate: Some(21.0),
        bursts: Some(vec![ParticleBurst::default()]),
        loop_system: Some(false),
        minimum_speed: Some(22.0),
        maximum_speed: Some(23.0),
        minimum_particle_life: Some(24.0),
        maximum_particle_life: Some(25.0),
        minimum_mass: Some(26.0),
        maximum_mass: Some(27.0),
        image: Some("url/to/image".to_string()),
        minimum_image_size: Some(Cartesian2::new(28.0, 30.0)),
        maximum_image_size: Some(Cartesian2::new(29.0, 31.0)),
        lifetime: Some(32.0),
        size_in_meters: Some(true),
        ..Default::default()
    };

    let p = ParticleSystem::new(options);
    assert!(!p.show);
    assert!(p.update_callback.is_some());
    assert_eq!(p.start_color(), Color::MAGENTA);
    assert_eq!(p.end_color(), Color::new(1.0, 0.94, 0.96, 1.0));
    assert_eq!(p.start_scale(), 19.0);
    assert_eq!(p.end_scale(), 20.0);
    assert_eq!(p.emission_rate(), 21.0);
    assert_eq!(p.bursts.as_ref().unwrap().len(), 1);
    assert!(!p.loop_system);
    assert_eq!(p.minimum_speed(), 22.0);
    assert_eq!(p.maximum_speed(), 23.0);
    assert_eq!(p.minimum_particle_life(), 24.0);
    assert_eq!(p.maximum_particle_life(), 25.0);
    assert_eq!(p.minimum_mass(), 26.0);
    assert_eq!(p.maximum_mass(), 27.0);
    assert_eq!(p.image.as_deref(), Some("url/to/image"));
    assert_eq!(p.minimum_image_size(), Cartesian2::new(28.0, 30.0));
    assert_eq!(p.maximum_image_size(), Cartesian2::new(29.0, 31.0));
    assert_eq!(p.lifetime(), 32.0);
    assert!(!p.is_complete());
    assert!(p.size_in_meters);
}

#[test]
fn test_getters_setters() {
    let mut p = ParticleSystem::default();
    p.show = false;
    p.forces = Some(vec![Arc::new(|part, _dt| {
        part.mass += 1.0;
    })]);
    p.set_emitter(Box::new(CircleEmitter::new(Some(10.0)).unwrap()));
    let mm = Matrix4::new(
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    p.set_model_matrix(mm);
    let emm = Matrix4::new(
        10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    p.set_emitter_model_matrix(emm);
    p.set_start_color(Color::MAGENTA);
    p.set_end_color(Color::CYAN);
    p.set_start_scale(19.0).unwrap();
    p.set_end_scale(20.0).unwrap();
    p.set_emission_rate(21.0).unwrap();
    p.bursts = Some(vec![ParticleBurst::default()]);
    p.loop_system = false;
    p.set_minimum_speed(22.0).unwrap();
    p.set_maximum_speed(23.0).unwrap();
    p.set_minimum_particle_life(24.0).unwrap();
    p.set_maximum_particle_life(25.0).unwrap();
    p.set_minimum_mass(26.0).unwrap();
    p.set_maximum_mass(27.0).unwrap();
    p.image = Some("url/to/image".to_string());
    p.set_minimum_image_size(Cartesian2::new(28.0, 30.0))
        .unwrap();
    p.set_maximum_image_size(Cartesian2::new(29.0, 31.0))
        .unwrap();
    p.set_lifetime(32.0).unwrap();
    p.size_in_meters = true;

    assert!(!p.show);
    assert_eq!(*p.model_matrix(), mm);
    assert_eq!(*p.emitter_model_matrix(), emm);
    assert_eq!(p.start_color(), Color::MAGENTA);
    assert_eq!(p.end_color(), Color::CYAN);
    assert_eq!(p.start_scale(), 19.0);
    assert_eq!(p.end_scale(), 20.0);
    assert_eq!(p.emission_rate(), 21.0);
    assert_eq!(p.minimum_speed(), 22.0);
    assert_eq!(p.maximum_speed(), 23.0);
    assert_eq!(p.minimum_particle_life(), 24.0);
    assert_eq!(p.maximum_particle_life(), 25.0);
    assert_eq!(p.minimum_mass(), 26.0);
    assert_eq!(p.maximum_mass(), 27.0);
    assert_eq!(p.image.as_deref(), Some("url/to/image"));
    assert_eq!(p.minimum_image_size(), Cartesian2::new(28.0, 30.0));
    assert_eq!(p.maximum_image_size(), Cartesian2::new(29.0, 31.0));
    assert_eq!(p.lifetime(), 32.0);
    assert!(!p.is_complete());
    assert!(p.size_in_meters);
}

#[test]
fn test_validation_errors() {
    let mut p = ParticleSystem::default();
    assert!(p.set_start_scale(-1.0).is_err());
    assert!(p.set_end_scale(-1.0).is_err());
    assert!(p.set_emission_rate(-1.0).is_err());
    assert!(p.set_minimum_speed(-1.0).is_err());
    assert!(p.set_maximum_speed(-1.0).is_err());
    assert!(p.set_minimum_particle_life(-1.0).is_err());
    assert!(p.set_maximum_particle_life(-1.0).is_err());
    assert!(p.set_minimum_mass(-1.0).is_err());
    assert!(p.set_maximum_mass(-1.0).is_err());
    assert!(p.set_minimum_image_size(Cartesian2::new(-1.0, 2.0)).is_err());
    assert!(p.set_minimum_image_size(Cartesian2::new(2.0, -1.0)).is_err());
    assert!(p.set_maximum_image_size(Cartesian2::new(-1.0, 2.0)).is_err());
    assert!(p.set_maximum_image_size(Cartesian2::new(2.0, -1.0)).is_err());
    assert!(p.set_lifetime(-1.0).is_err());
}

#[test]
fn test_clones_default_image_size() {
    let mut p = ParticleSystem::default();
    let min_size = p.minimum_image_size();
    p.set_minimum_image_size(Cartesian2::new(10.0, 20.0)).unwrap();
    assert_ne!(p.maximum_image_size(), p.minimum_image_size());
    assert_eq!(min_size, Cartesian2::new(1.0, 1.0));
}

#[test]
fn test_updates_and_renders() {
    let mut system = ParticleSystem::new(ParticleSystemOptions {
        start_color: Some(Color::GREEN),
        end_color: Some(Color::GREEN),
        emitter: Some(Box::new(CircleEmitter::new(Some(1.0)).unwrap())),
        emission_rate: Some(10000.0),
        image_size: Some(Cartesian2::new(100.0, 100.0)),
        ..Default::default()
    });

    // No particles emitted at time 0
    system.update(0.0);
    assert_eq!(system.particles().len(), 0);

    // Emits and updates after delta time
    system.update(0.1);
    assert!(!system.particles().is_empty());

    let rgba = system.simulate_rendered_rgba();
    assert_eq!(rgba, [0, 255, 0, 255]);
}

#[test]
fn test_destroys() {
    let mut p = ParticleSystem::default();
    assert!(!p.is_destroyed());
    p.destroy();
    assert!(p.is_destroyed());
}
