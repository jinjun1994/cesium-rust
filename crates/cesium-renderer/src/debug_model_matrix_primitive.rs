use cesium_math::{Cartesian3, Color, DeveloperError, Matrix4};

/// Options for creating a `DebugModelMatrixPrimitive`.
#[derive(Debug, Clone, Default)]
pub struct DebugModelMatrixPrimitiveOptions {
    /// The length of the axes in meters. Defaults to 10000000.0.
    pub length: Option<f64>,
    /// The width of the axes in pixels. Defaults to 2.0.
    pub width: Option<f64>,
    /// The 4x4 matrix that defines the reference frame. Defaults to Matrix4::IDENTITY.
    pub model_matrix: Option<Matrix4>,
    /// Determines if this primitive will be shown. Defaults to true.
    pub show: Option<bool>,
    /// User-defined ID returned when picked.
    pub id: Option<String>,
}

/// A line segment representation of a single coordinate axis in the debug primitive.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugAxis {
    /// Origin of axis (in model space).
    pub origin: Cartesian3,
    /// Unit direction of axis.
    pub direction: Cartesian3,
    /// Color of axis (Red for X, Green for Y, Blue for Z).
    pub color: Color,
    /// Model matrix scaled by length.
    pub scaled_matrix: Matrix4,
}

/// Draws the axes of a reference frame defined by a model matrix.
/// The X axis is red; Y is green; and Z is blue.
#[derive(Debug, Clone)]
pub struct DebugModelMatrixPrimitive {
    /// The length of the axes in meters.
    pub length: f64,
    /// The width of the axes in pixels.
    pub width: f64,
    /// The 4x4 matrix that defines the reference frame.
    pub model_matrix: Matrix4,
    /// Determines if this primitive will be shown.
    pub show: bool,
    /// User-defined ID.
    pub id: Option<String>,
    is_destroyed: bool,
}

impl Default for DebugModelMatrixPrimitive {
    fn default() -> Self {
        Self::new(DebugModelMatrixPrimitiveOptions::default())
    }
}

impl DebugModelMatrixPrimitive {
    /// Creates a new `DebugModelMatrixPrimitive`.
    pub fn new(options: DebugModelMatrixPrimitiveOptions) -> Self {
        Self {
            length: options.length.unwrap_or(10000000.0),
            width: options.width.unwrap_or(2.0),
            model_matrix: options.model_matrix.unwrap_or(Matrix4::IDENTITY),
            show: options.show.unwrap_or(true),
            id: options.id,
            is_destroyed: false,
        }
    }

    /// Computes the 3 debug axes (X: Red, Y: Green, Z: Blue) transformed by model_matrix and scaled by length.
    pub fn compute_axes(&self) -> Result<[DebugAxis; 3], DeveloperError> {
        if self.is_destroyed {
            return Err(DeveloperError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let scaled = self.model_matrix.multiply_by_uniform_scale(self.length);

        Ok([
            DebugAxis {
                origin: Cartesian3::ZERO,
                direction: Cartesian3::UNIT_X,
                color: Color::RED,
                scaled_matrix: scaled,
            },
            DebugAxis {
                origin: Cartesian3::ZERO,
                direction: Cartesian3::UNIT_Y,
                color: Color::GREEN,
                scaled_matrix: scaled,
            },
            DebugAxis {
                origin: Cartesian3::ZERO,
                direction: Cartesian3::UNIT_Z,
                color: Color::BLUE,
                scaled_matrix: scaled,
            },
        ])
    }

    /// Returns true if destroyed.
    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    /// Destroys resources.
    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}
