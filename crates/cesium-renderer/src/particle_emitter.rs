use crate::particle::Particle;
use cesium_math::{Cartesian3, DeveloperError};
use std::cell::Cell;
use std::f64::consts::{PI, TAU};

thread_local! {
    static RNG_STATE: Cell<u64> = const { Cell::new(0x853c_49e6_748f_ea9b) };
}

/// Generates a pseudo-random 64-bit float in the range [0.0, 1.0).
pub fn next_random_f64() -> f64 {
    RNG_STATE.with(|state| {
        let mut x = state.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        state.set(x);
        // Map to [0, 1) by masking to 53 bits (f64 mantissa)
        let mantissa = x >> 11;
        (mantissa as f64) / ((1u64 << 53) as f64)
    })
}

/// Generates a pseudo-random 64-bit float between `min` and `max`.
pub fn random_between(min: f64, max: f64) -> f64 {
    min + next_random_f64() * (max - min)
}

/// A particle emitter interface that initializes a [`Particle`].
pub trait ParticleEmitter {
    /// Initializes the given [`Particle`] by setting its position and velocity.
    fn emit(&self, particle: &mut Particle);
}

/// A ParticleEmitter that emits particles within a box.
/// Particles will be positioned randomly within the box and have initial velocities emanating from the center of the box.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxEmitter {
    dimensions: Cartesian3,
}

impl Default for BoxEmitter {
    fn default() -> Self {
        Self {
            dimensions: Cartesian3::new(1.0, 1.0, 1.0),
        }
    }
}

impl BoxEmitter {
    /// Creates a new BoxEmitter with the specified dimensions.
    pub fn new(dimensions: Option<Cartesian3>) -> Result<Self, DeveloperError> {
        let dims = dimensions.unwrap_or_else(|| Cartesian3::new(1.0, 1.0, 1.0));
        Self::validate_dimensions(dims)?;
        Ok(Self { dimensions: dims })
    }

    /// The width, height and depth dimensions of the box in meters.
    #[inline]
    pub fn dimensions(&self) -> Cartesian3 {
        self.dimensions
    }

    /// Sets the width, height and depth dimensions of the box in meters.
    pub fn set_dimensions(&mut self, dimensions: Cartesian3) -> Result<(), DeveloperError> {
        Self::validate_dimensions(dimensions)?;
        self.dimensions = dimensions;
        Ok(())
    }

    fn validate_dimensions(dims: Cartesian3) -> Result<(), DeveloperError> {
        if dims.x < 0.0 || dims.y < 0.0 || dims.z < 0.0 {
            return Err(DeveloperError::new(
                "dimensions.x, dimensions.y, and dimensions.z must be greater than or equal to zero.",
            ));
        }
        Ok(())
    }
}

impl ParticleEmitter for BoxEmitter {
    fn emit(&self, particle: &mut Particle) {
        let half_dim = self.dimensions * 0.5;

        let x = random_between(-half_dim.x, half_dim.x);
        let y = random_between(-half_dim.y, half_dim.y);
        let z = random_between(-half_dim.z, half_dim.z);

        particle.position = Cartesian3::new(x, y, z);
        particle.velocity = particle.position.normalize();
    }
}

/// A ParticleEmitter that emits particles from a circle.
/// Particles will be positioned within a circle and have initial velocities going along the z vector.
#[derive(Debug, Clone, PartialEq)]
pub struct CircleEmitter {
    radius: f64,
}

impl Default for CircleEmitter {
    fn default() -> Self {
        Self { radius: 1.0 }
    }
}

impl CircleEmitter {
    /// Creates a new CircleEmitter with the specified radius in meters.
    pub fn new(radius: Option<f64>) -> Result<Self, DeveloperError> {
        let r = radius.unwrap_or(1.0);
        Self::validate_radius(r)?;
        Ok(Self { radius: r })
    }

    /// The radius of the circle in meters.
    #[inline]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Sets the radius of the circle in meters.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), DeveloperError> {
        Self::validate_radius(radius)?;
        self.radius = radius;
        Ok(())
    }

    fn validate_radius(radius: f64) -> Result<(), DeveloperError> {
        if radius <= 0.0 {
            return Err(DeveloperError::new(
                "radius must be greater than zero.",
            ));
        }
        Ok(())
    }
}

impl ParticleEmitter for CircleEmitter {
    fn emit(&self, particle: &mut Particle) {
        let theta = random_between(0.0, TAU);
        let rad = random_between(0.0, self.radius);

        let x = rad * theta.cos();
        let y = rad * theta.sin();
        let z = 0.0;

        particle.position = Cartesian3::new(x, y, z);
        particle.velocity = Cartesian3::UNIT_Z;
    }
}

/// A ParticleEmitter that emits particles within a cone.
/// Particles will be positioned at the tip of the cone and have initial velocities going towards the base.
#[derive(Debug, Clone, PartialEq)]
pub struct ConeEmitter {
    angle: f64,
}

impl Default for ConeEmitter {
    fn default() -> Self {
        Self {
            angle: 30.0_f64.to_radians(),
        }
    }
}

impl ConeEmitter {
    /// Creates a new ConeEmitter with the specified angle in radians.
    pub fn new(angle: Option<f64>) -> Self {
        Self {
            angle: angle.unwrap_or_else(|| 30.0_f64.to_radians()),
        }
    }

    /// The angle of the cone in radians.
    #[inline]
    pub fn angle(&self) -> f64 {
        self.angle
    }

    /// Sets the angle of the cone in radians.
    #[inline]
    pub fn set_angle(&mut self, angle: f64) {
        self.angle = angle;
    }
}

impl ParticleEmitter for ConeEmitter {
    fn emit(&self, particle: &mut Particle) {
        let radius = self.angle.tan();

        let theta = random_between(0.0, TAU);
        let rad = random_between(0.0, radius);

        let x = rad * theta.cos();
        let y = rad * theta.sin();
        let z = 1.0;

        particle.velocity = Cartesian3::new(x, y, z).normalize();
        particle.position = Cartesian3::ZERO;
    }
}

/// A ParticleEmitter that emits particles within a sphere.
/// Particles will be positioned randomly within the sphere and have initial velocities emanating from the center of the sphere.
#[derive(Debug, Clone, PartialEq)]
pub struct SphereEmitter {
    radius: f64,
}

impl Default for SphereEmitter {
    fn default() -> Self {
        Self { radius: 1.0 }
    }
}

impl SphereEmitter {
    /// Creates a new SphereEmitter with the specified radius in meters.
    pub fn new(radius: Option<f64>) -> Result<Self, DeveloperError> {
        let r = radius.unwrap_or(1.0);
        Self::validate_radius(r)?;
        Ok(Self { radius: r })
    }

    /// The radius of the sphere in meters.
    #[inline]
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// Sets the radius of the sphere in meters.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), DeveloperError> {
        Self::validate_radius(radius)?;
        self.radius = radius;
        Ok(())
    }

    fn validate_radius(radius: f64) -> Result<(), DeveloperError> {
        if radius <= 0.0 {
            return Err(DeveloperError::new(
                "radius must be greater than zero.",
            ));
        }
        Ok(())
    }
}

impl ParticleEmitter for SphereEmitter {
    fn emit(&self, particle: &mut Particle) {
        let theta = random_between(0.0, TAU);
        let phi = random_between(0.0, PI);
        let rad = random_between(0.0, self.radius);

        let x = rad * theta.cos() * phi.sin();
        let y = rad * theta.sin() * phi.sin();
        let z = rad * phi.cos();

        particle.position = Cartesian3::new(x, y, z);
        particle.velocity = particle.position.normalize();
    }
}
