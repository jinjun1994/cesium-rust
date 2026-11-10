//! ArticulationStageType matching CesiumJS `Core/ArticulationStageType.js` (Apache-2.0).

/// An enum describing the type of motion defined by an articulation stage
/// in the AGI_articulations extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArticulationStageType {
    XTranslate,
    YTranslate,
    ZTranslate,
    XRotate,
    YRotate,
    ZRotate,
    XScale,
    YScale,
    ZScale,
    UniformScale,
}

impl ArticulationStageType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::XTranslate => "xTranslate",
            Self::YTranslate => "yTranslate",
            Self::ZTranslate => "zTranslate",
            Self::XRotate => "xRotate",
            Self::YRotate => "yRotate",
            Self::ZRotate => "zRotate",
            Self::XScale => "xScale",
            Self::YScale => "yScale",
            Self::ZScale => "zScale",
            Self::UniformScale => "uniformScale",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "xTranslate" => Some(Self::XTranslate),
            "yTranslate" => Some(Self::YTranslate),
            "zTranslate" => Some(Self::ZTranslate),
            "xRotate" => Some(Self::XRotate),
            "yRotate" => Some(Self::YRotate),
            "zRotate" => Some(Self::ZRotate),
            "xScale" => Some(Self::XScale),
            "yScale" => Some(Self::YScale),
            "zScale" => Some(Self::ZScale),
            "uniformScale" => Some(Self::UniformScale),
            _ => None,
        }
    }
}
