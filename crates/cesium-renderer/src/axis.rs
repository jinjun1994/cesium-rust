use cesium_math::{Cartesian3, DeveloperError, Matrix3, Matrix4};

/// An enum describing the x, y, and z axes and helper conversion matrices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum Axis {
    /// Denotes the x-axis.
    X = 0,
    /// Denotes the y-axis.
    Y = 1,
    /// Denotes the z-axis.
    Z = 2,
}

impl Axis {
    /// Matrix used to convert from y-up to z-up.
    /// Rotation about PI/2 around the X-axis.
    pub const Y_UP_TO_Z_UP: Matrix4 = Matrix4::from_column_major_array([
        1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Matrix used to convert from z-up to y-up.
    /// Rotation about -PI/2 around the X-axis.
    pub const Z_UP_TO_Y_UP: Matrix4 = Matrix4::from_column_major_array([
        1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, -1.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Matrix used to convert from x-up to z-up.
    /// Rotation about -PI/2 around the Y-axis.
    pub const X_UP_TO_Z_UP: Matrix4 = Matrix4::from_column_major_array([
        0.0, 0.0, 1.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Matrix used to convert from z-up to x-up.
    /// Rotation about PI/2 around the Y-axis.
    pub const Z_UP_TO_X_UP: Matrix4 = Matrix4::from_column_major_array([
        0.0, 0.0, -1.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Matrix used to convert from x-up to y-up.
    /// Rotation about PI/2 around the Z-axis.
    pub const X_UP_TO_Y_UP: Matrix4 = Matrix4::from_column_major_array([
        0.0, 1.0, 0.0, 0.0,
        -1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Matrix used to convert from y-up to x-up.
    /// Rotation about -PI/2 around the Z-axis.
    pub const Y_UP_TO_X_UP: Matrix4 = Matrix4::from_column_major_array([
        0.0, -1.0, 0.0, 0.0,
        1.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);

    /// Helper that constructs a conversion matrix from rotation matrix array (column-major [f64; 9]).
    pub fn from_rotation_matrix3_array(elements: [f64; 9]) -> Matrix4 {
        let rot = Matrix3::from_column_major_array(elements);
        Matrix4::from_rotation_translation(&rot, Cartesian3::ZERO)
    }

    /// Gets the axis by name ("X", "Y", "Z").
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "X" => Some(Self::X),
            "Y" => Some(Self::Y),
            "Z" => Some(Self::Z),
            _ => None,
        }
    }

    /// Returns the name string representation of the axis.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
        }
    }
}

impl From<Axis> for i32 {
    fn from(axis: Axis) -> Self {
        axis as i32
    }
}

impl TryFrom<i32> for Axis {
    type Error = DeveloperError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::X),
            1 => Ok(Self::Y),
            2 => Ok(Self::Z),
            _ => Err(DeveloperError::new(format!("Invalid axis value: {value}"))),
        }
    }
}
