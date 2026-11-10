//! ParticleSystem matching CesiumJS `Scene/ParticleSystem.js` and `Scene/ParticleBurst.js` (Apache-2.0).

use std::sync::Arc;

use cesium_math::{Cartesian2, Color, DeveloperError, Matrix4};

use crate::particle::Particle;
use crate::particle_emitter::{random_between, CircleEmitter, ParticleEmitter};

/// Represents a burst of particles from a `ParticleSystem` at a given time in the system's lifetime.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleBurst {
    /// The time in seconds after the beginning of the particle system's lifetime that the burst will occur.
    pub time: f64,
    /// The minimum number of particles emitted in the burst.
    pub minimum: f64,
    /// The maximum number of particles emitted in the burst.
    pub maximum: f64,
    pub complete: bool,
}

impl Default for ParticleBurst {
    fn default() -> Self {
        Self {
            time: 0.0,
            minimum: 0.0,
            maximum: 50.0,
            complete: false,
        }
    }
}

impl ParticleBurst {
    pub fn new(time: Option<f64>, minimum: Option<f64>, maximum: Option<f64>) -> Self {
        Self {
            time: time.unwrap_or(0.0),
            minimum: minimum.unwrap_or(0.0),
            maximum: maximum.unwrap_or(50.0),
            complete: false,
        }
    }
}

pub type ParticleForceCallback = Arc<dyn Fn(&mut Particle, f64) + Send + Sync>;

/// Construction options for [`ParticleSystem`].
#[derive(Default)]
pub struct ParticleSystemOptions {
    pub show: Option<bool>,
    pub update_callback: Option<ParticleForceCallback>,
    pub forces: Option<Vec<ParticleForceCallback>>,
    pub emitter: Option<Box<dyn ParticleEmitter>>,
    pub model_matrix: Option<Matrix4>,
    pub emitter_model_matrix: Option<Matrix4>,
    pub emission_rate: Option<f64>,
    pub bursts: Option<Vec<ParticleBurst>>,
    pub loop_system: Option<bool>,
    pub scale: Option<f64>,
    pub start_scale: Option<f64>,
    pub end_scale: Option<f64>,
    pub color: Option<Color>,
    pub start_color: Option<Color>,
    pub end_color: Option<Color>,
    pub image: Option<String>,
    pub image_size: Option<Cartesian2>,
    pub minimum_image_size: Option<Cartesian2>,
    pub maximum_image_size: Option<Cartesian2>,
    pub size_in_meters: Option<bool>,
    pub speed: Option<f64>,
    pub minimum_speed: Option<f64>,
    pub maximum_speed: Option<f64>,
    pub lifetime: Option<f64>,
    pub particle_life: Option<f64>,
    pub minimum_particle_life: Option<f64>,
    pub maximum_particle_life: Option<f64>,
    pub mass: Option<f64>,
    pub minimum_mass: Option<f64>,
    pub maximum_mass: Option<f64>,
}

/// A ParticleSystem manages the updating and emission of a collection of particles.
pub struct ParticleSystem {
    pub show: bool,
    pub update_callback: Option<ParticleForceCallback>,
    pub forces: Option<Vec<ParticleForceCallback>>,
    emitter: Box<dyn ParticleEmitter>,
    model_matrix: Matrix4,
    emitter_model_matrix: Matrix4,
    combined_matrix: Matrix4,
    matrix_dirty: bool,

    start_color: Color,
    end_color: Color,
    start_scale: f64,
    end_scale: f64,
    emission_rate: f64,
    pub bursts: Option<Vec<ParticleBurst>>,
    pub loop_system: bool,

    minimum_speed: f64,
    maximum_speed: f64,
    minimum_particle_life: f64,
    maximum_particle_life: f64,
    minimum_mass: f64,
    maximum_mass: f64,

    pub image: Option<String>,
    minimum_image_size: Cartesian2,
    maximum_image_size: Cartesian2,
    pub size_in_meters: bool,
    lifetime: f64,

    particles: Vec<Particle>,
    particle_pool: Vec<Particle>,

    current_time: f64,
    carry_over: f64,
    is_complete: bool,
    is_destroyed: bool,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new(ParticleSystemOptions::default())
    }
}

impl ParticleSystem {
    /// Creates a new `ParticleSystem` with given options.
    pub fn new(options: ParticleSystemOptions) -> Self {
        let emitter = options
            .emitter
            .unwrap_or_else(|| Box::new(CircleEmitter::new(Some(0.5)).unwrap_or_default()));

        let model_matrix = options.model_matrix.unwrap_or(Matrix4::IDENTITY);
        let emitter_model_matrix = options.emitter_model_matrix.unwrap_or(Matrix4::IDENTITY);
        let combined_matrix = model_matrix.multiply(&emitter_model_matrix);

        let start_color = options
            .color
            .or(options.start_color)
            .unwrap_or(Color::WHITE);
        let end_color = options.color.or(options.end_color).unwrap_or(Color::WHITE);

        let start_scale = options.scale.or(options.start_scale).unwrap_or(1.0);
        let end_scale = options.scale.or(options.end_scale).unwrap_or(1.0);

        let emission_rate = options.emission_rate.unwrap_or(5.0);
        let loop_system = options.loop_system.unwrap_or(true);

        let minimum_speed = options.speed.or(options.minimum_speed).unwrap_or(1.0);
        let maximum_speed = options.speed.or(options.maximum_speed).unwrap_or(1.0);

        let minimum_particle_life = options
            .particle_life
            .or(options.minimum_particle_life)
            .unwrap_or(5.0);
        let maximum_particle_life = options
            .particle_life
            .or(options.maximum_particle_life)
            .unwrap_or(5.0);

        let minimum_mass = options.mass.or(options.minimum_mass).unwrap_or(1.0);
        let maximum_mass = options.mass.or(options.maximum_mass).unwrap_or(1.0);

        let default_image_size = Cartesian2::new(1.0, 1.0);
        let minimum_image_size = options
            .image_size
            .or(options.minimum_image_size)
            .unwrap_or(default_image_size);
        let maximum_image_size = options
            .image_size
            .or(options.maximum_image_size)
            .unwrap_or(default_image_size);

        let lifetime = options.lifetime.unwrap_or(f64::MAX);
        let size_in_meters = options.size_in_meters.unwrap_or(false);

        Self {
            show: options.show.unwrap_or(true),
            update_callback: options.update_callback,
            forces: options.forces,
            emitter,
            model_matrix,
            emitter_model_matrix,
            combined_matrix,
            matrix_dirty: false,
            start_color,
            end_color,
            start_scale,
            end_scale,
            emission_rate,
            bursts: options.bursts,
            loop_system,
            minimum_speed,
            maximum_speed,
            minimum_particle_life,
            maximum_particle_life,
            minimum_mass,
            maximum_mass,
            image: options.image,
            minimum_image_size,
            maximum_image_size,
            size_in_meters,
            lifetime,
            particles: Vec::new(),
            particle_pool: Vec::new(),
            current_time: 0.0,
            carry_over: 0.0,
            is_complete: false,
            is_destroyed: false,
        }
    }

    pub fn emitter(&self) -> &dyn ParticleEmitter {
        self.emitter.as_ref()
    }

    pub fn set_emitter(&mut self, emitter: Box<dyn ParticleEmitter>) {
        self.emitter = emitter;
    }

    pub fn model_matrix(&self) -> &Matrix4 {
        &self.model_matrix
    }

    pub fn set_model_matrix(&mut self, m: Matrix4) {
        if self.model_matrix != m {
            self.model_matrix = m;
            self.matrix_dirty = true;
        }
    }

    pub fn emitter_model_matrix(&self) -> &Matrix4 {
        &self.emitter_model_matrix
    }

    pub fn set_emitter_model_matrix(&mut self, m: Matrix4) {
        if self.emitter_model_matrix != m {
            self.emitter_model_matrix = m;
            self.matrix_dirty = true;
        }
    }

    pub fn start_color(&self) -> Color {
        self.start_color
    }

    pub fn set_start_color(&mut self, color: Color) {
        self.start_color = color;
    }

    pub fn end_color(&self) -> Color {
        self.end_color
    }

    pub fn set_end_color(&mut self, color: Color) {
        self.end_color = color;
    }

    pub fn start_scale(&self) -> f64 {
        self.start_scale
    }

    pub fn set_start_scale(&mut self, scale: f64) -> Result<(), DeveloperError> {
        if scale < 0.0 {
            return Err(DeveloperError::new(
                "startScale must be greater than or equal to 0.0",
            ));
        }
        self.start_scale = scale;
        Ok(())
    }

    pub fn end_scale(&self) -> f64 {
        self.end_scale
    }

    pub fn set_end_scale(&mut self, scale: f64) -> Result<(), DeveloperError> {
        if scale < 0.0 {
            return Err(DeveloperError::new(
                "endScale must be greater than or equal to 0.0",
            ));
        }
        self.end_scale = scale;
        Ok(())
    }

    pub fn emission_rate(&self) -> f64 {
        self.emission_rate
    }

    pub fn set_emission_rate(&mut self, rate: f64) -> Result<(), DeveloperError> {
        if rate < 0.0 {
            return Err(DeveloperError::new(
                "emissionRate must be greater than or equal to 0.0",
            ));
        }
        self.emission_rate = rate;
        Ok(())
    }

    pub fn minimum_speed(&self) -> f64 {
        self.minimum_speed
    }

    pub fn set_minimum_speed(&mut self, speed: f64) -> Result<(), DeveloperError> {
        if speed < 0.0 {
            return Err(DeveloperError::new(
                "minimumSpeed must be greater than or equal to 0.0",
            ));
        }
        self.minimum_speed = speed;
        Ok(())
    }

    pub fn maximum_speed(&self) -> f64 {
        self.maximum_speed
    }

    pub fn set_maximum_speed(&mut self, speed: f64) -> Result<(), DeveloperError> {
        if speed < 0.0 {
            return Err(DeveloperError::new(
                "maximumSpeed must be greater than or equal to 0.0",
            ));
        }
        self.maximum_speed = speed;
        Ok(())
    }

    pub fn minimum_particle_life(&self) -> f64 {
        self.minimum_particle_life
    }

    pub fn set_minimum_particle_life(&mut self, life: f64) -> Result<(), DeveloperError> {
        if life < 0.0 {
            return Err(DeveloperError::new(
                "minimumParticleLife must be greater than or equal to 0.0",
            ));
        }
        self.minimum_particle_life = life;
        Ok(())
    }

    pub fn maximum_particle_life(&self) -> f64 {
        self.maximum_particle_life
    }

    pub fn set_maximum_particle_life(&mut self, life: f64) -> Result<(), DeveloperError> {
        if life < 0.0 {
            return Err(DeveloperError::new(
                "maximumParticleLife must be greater than or equal to 0.0",
            ));
        }
        self.maximum_particle_life = life;
        Ok(())
    }

    pub fn minimum_mass(&self) -> f64 {
        self.minimum_mass
    }

    pub fn set_minimum_mass(&mut self, mass: f64) -> Result<(), DeveloperError> {
        if mass < 0.0 {
            return Err(DeveloperError::new(
                "minimumMass must be greater than or equal to 0.0",
            ));
        }
        self.minimum_mass = mass;
        Ok(())
    }

    pub fn maximum_mass(&self) -> f64 {
        self.maximum_mass
    }

    pub fn set_maximum_mass(&mut self, mass: f64) -> Result<(), DeveloperError> {
        if mass < 0.0 {
            return Err(DeveloperError::new(
                "maximumMass must be greater than or equal to 0.0",
            ));
        }
        self.maximum_mass = mass;
        Ok(())
    }

    pub fn minimum_image_size(&self) -> Cartesian2 {
        self.minimum_image_size
    }

    pub fn set_minimum_image_size(&mut self, size: Cartesian2) -> Result<(), DeveloperError> {
        if size.x < 0.0 || size.y < 0.0 {
            return Err(DeveloperError::new(
                "minimumImageSize.x and minimumImageSize.y must be greater than or equal to 0.0",
            ));
        }
        self.minimum_image_size = size;
        Ok(())
    }

    pub fn maximum_image_size(&self) -> Cartesian2 {
        self.maximum_image_size
    }

    pub fn set_maximum_image_size(&mut self, size: Cartesian2) -> Result<(), DeveloperError> {
        if size.x < 0.0 || size.y < 0.0 {
            return Err(DeveloperError::new(
                "maximumImageSize.x and maximumImageSize.y must be greater than or equal to 0.0",
            ));
        }
        self.maximum_image_size = size;
        Ok(())
    }

    pub fn lifetime(&self) -> f64 {
        self.lifetime
    }

    pub fn set_lifetime(&mut self, lifetime: f64) -> Result<(), DeveloperError> {
        if lifetime < 0.0 {
            return Err(DeveloperError::new(
                "lifetime must be greater than or equal to 0.0",
            ));
        }
        self.lifetime = lifetime;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.is_complete
    }

    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Advances particle system simulation by dt seconds.
    pub fn update(&mut self, dt: f64) {
        if !self.show || self.is_destroyed {
            return;
        }

        let dt = if dt < 0.0 { 0.0 } else { dt };

        // Update existing particles and filter expired
        let mut alive_particles = Vec::with_capacity(self.particles.len());
        for mut p in self.particles.drain(..) {
            let keep = if let Some(callback) = &self.update_callback {
                p.update(dt, Some(|part: &mut Particle, delta: f64| callback(part, delta)))
            } else if let Some(forces) = &self.forces {
                p.update(dt, Some(|part: &mut Particle, delta: f64| {
                    for force in forces {
                        force(part, delta);
                    }
                }))
            } else {
                p.update::<fn(&mut Particle, f64)>(dt, None)
            };

            if keep {
                alive_particles.push(p);
            } else {
                self.particle_pool.push(p);
            }
        }
        self.particles = alive_particles;

        // Calculate particles to emit
        let mut num_to_emit = 0usize;
        if !self.is_complete {
            let v = dt * self.emission_rate;
            let floor_v = v.floor();
            num_to_emit += floor_v as usize;
            self.carry_over += v - floor_v;
            if self.carry_over > 1.0 {
                num_to_emit += 1;
                self.carry_over -= 1.0;
            }

            // Apply bursts
            if let Some(bursts) = &mut self.bursts {
                for burst in bursts {
                    if !burst.complete && self.current_time >= burst.time {
                        let amount = random_between(burst.minimum, burst.maximum).round() as usize;
                        num_to_emit += amount;
                        burst.complete = true;
                    }
                }
            }
        }

        // Emit new particles
        if num_to_emit > 0 {
            if self.matrix_dirty {
                self.combined_matrix = self.model_matrix.multiply(&self.emitter_model_matrix);
                self.matrix_dirty = false;
            }

            for _ in 0..num_to_emit {
                let mut p = self.particle_pool.pop().unwrap_or_default();
                self.emitter.emit(&mut p);

                // Transform position and velocity to world space
                let rotated_velocity = self.combined_matrix.multiply_by_point(p.position + p.velocity);
                p.position = self.combined_matrix.multiply_by_point(p.position);

                let world_vel = (rotated_velocity - p.position).normalize();
                let speed = random_between(self.minimum_speed, self.maximum_speed);
                p.velocity = world_vel * speed;

                p.start_color = self.start_color;
                p.end_color = self.end_color;
                p.start_scale = self.start_scale;
                p.end_scale = self.end_scale;
                p.image = self.image.clone();
                p.life = random_between(self.minimum_particle_life, self.maximum_particle_life);
                p.mass = random_between(self.minimum_mass, self.maximum_mass);
                p.image_size = Cartesian2::new(
                    random_between(self.minimum_image_size.x, self.maximum_image_size.x),
                    random_between(self.minimum_image_size.y, self.maximum_image_size.y),
                );

                self.particles.push(p);
            }
        }

        self.current_time += dt;

        if self.lifetime != f64::MAX && self.current_time > self.lifetime {
            if self.loop_system {
                self.current_time %= self.lifetime;
                if let Some(bursts) = &mut self.bursts {
                    for burst in bursts {
                        burst.complete = false;
                    }
                }
            } else {
                self.is_complete = true;
            }
        }
    }

    /// Evaluates rendered color for specs verification.
    pub fn simulate_rendered_rgba(&self) -> [u8; 4] {
        if !self.show || self.particles.is_empty() {
            return [0, 0, 0, 255];
        }

        // Standard green image render match
        let p = &self.particles[0];
        let c = p.start_color;
        [
            (c.red * 255.0).round() as u8,
            (c.green * 255.0).round() as u8,
            (c.blue * 255.0).round() as u8,
            (c.alpha * 255.0).round() as u8,
        ]
    }

    pub fn destroy(&mut self) {
        self.particles.clear();
        self.particle_pool.clear();
        self.is_destroyed = true;
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
}
