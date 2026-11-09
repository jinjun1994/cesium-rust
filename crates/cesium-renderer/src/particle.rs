use cesium_math::{Cartesian2, Cartesian3, Color};

/// Options for constructing a [`Particle`].
#[derive(Debug, Clone, Default)]
pub struct ParticleOptions {
    /// The mass of the particle in kilograms.
    pub mass: Option<f64>,
    /// The initial position of the particle in world coordinates.
    pub position: Option<Cartesian3>,
    /// The velocity vector of the particle in world coordinates.
    pub velocity: Option<Cartesian3>,
    /// The life of the particle in seconds.
    pub life: Option<f64>,
    /// The URI or resource to use for the billboard.
    pub image: Option<String>,
    /// The color of a particle when it is born.
    pub start_color: Option<Color>,
    /// The color of a particle when it dies.
    pub end_color: Option<Color>,
    /// The scale of the particle when it is born.
    pub start_scale: Option<f64>,
    /// The scale of the particle when it dies.
    pub end_scale: Option<f64>,
    /// The dimensions, width by height, to scale the particle image in pixels.
    pub image_size: Option<Cartesian2>,
}

/// A particle emitted by a particle system.
#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    /// The mass of the particle in kilograms.
    pub mass: f64,
    /// The position of the particle in world coordinates.
    pub position: Cartesian3,
    /// The velocity of the particle in world coordinates.
    pub velocity: Cartesian3,
    /// The life of the particle in seconds.
    pub life: f64,
    /// The image to use for the particle.
    pub image: Option<String>,
    /// The color of the particle when it is born.
    pub start_color: Color,
    /// The color of the particle when it dies.
    pub end_color: Color,
    /// The scale of the particle when it is born.
    pub start_scale: f64,
    /// The scale of the particle when it dies.
    pub end_scale: f64,
    /// The dimensions, width by height, to scale the particle image in pixels.
    pub image_size: Cartesian2,

    age: f64,
    normalized_age: f64,
}

impl Default for Particle {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Particle {
    /// Creates a new particle with the given options.
    pub fn new(options: Option<ParticleOptions>) -> Self {
        let opts = options.unwrap_or_default();
        Self {
            mass: opts.mass.unwrap_or(1.0),
            position: opts.position.unwrap_or(Cartesian3::ZERO),
            velocity: opts.velocity.unwrap_or(Cartesian3::ZERO),
            life: opts.life.unwrap_or(f64::MAX),
            image: opts.image,
            start_color: opts.start_color.unwrap_or(Color::WHITE),
            end_color: opts.end_color.unwrap_or(Color::WHITE),
            start_scale: opts.start_scale.unwrap_or(1.0),
            end_scale: opts.end_scale.unwrap_or(1.0),
            image_size: opts.image_size.unwrap_or_else(|| Cartesian2::new(1.0, 1.0)),
            age: 0.0,
            normalized_age: 0.0,
        }
    }

    /// Gets the age of the particle in seconds.
    #[inline]
    pub fn age(&self) -> f64 {
        self.age
    }

    /// Gets the age normalized to a value in the range [0.0, 1.0].
    #[inline]
    pub fn normalized_age(&self) -> f64 {
        self.normalized_age
    }

    /// Updates the particle with delta time in seconds, and an optional particle update function.
    /// Returns true if the particle is still alive, false if it has expired.
    pub fn update<F>(&mut self, dt: f64, mut particle_update_function: Option<F>) -> bool
    where
        F: FnMut(&mut Particle, f64),
    {
        // Apply velocity: position += velocity * dt
        self.position = self.position + self.velocity * dt;

        // Update any forces / callbacks
        if let Some(ref mut f) = particle_update_function {
            f(self, dt);
        }

        // Age the particle
        self.age += dt;

        // Compute normalized age
        if self.life == f64::MAX {
            self.normalized_age = 0.0;
        } else {
            self.normalized_age = self.age / self.life;
        }

        self.age <= self.life
    }
}
