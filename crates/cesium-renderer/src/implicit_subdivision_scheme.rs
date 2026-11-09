use std::str::FromStr;
use cesium_math::DeveloperError;

/// The subdivision scheme for an implicit tileset: Quadtree or Octree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImplicitSubdivisionScheme {
    #[default]
    Quadtree,
    Octree,
}

impl FromStr for ImplicitSubdivisionScheme {
    type Err = DeveloperError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "QUADTREE" => Ok(Self::Quadtree),
            "OCTREE" => Ok(Self::Octree),
            _ => Err(DeveloperError::new(format!(
                "Invalid implicit subdivision scheme: {s}"
            ))),
        }
    }
}

impl ImplicitSubdivisionScheme {
    /// Parses scheme string ("QUADTREE" or "OCTREE").
    pub fn from_scheme_str(s: &str) -> Result<Self, DeveloperError> {
        s.parse()
    }

    /// The string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quadtree => "QUADTREE",
            Self::Octree => "OCTREE",
        }
    }

    /// Number of children per tile.
    pub fn branching_factor(&self) -> usize {
        match self {
            Self::Quadtree => 4,
            Self::Octree => 8,
        }
    }

    /// Gets the branching factor for the given subdivision scheme name string.
    pub fn get_branching_factor(s: &str) -> Result<usize, DeveloperError> {
        match s {
            "QUADTREE" => Ok(4),
            "OCTREE" => Ok(8),
            _ => Err(DeveloperError::new(
                "subdivisionScheme is not a valid value.",
            )),
        }
    }
}
