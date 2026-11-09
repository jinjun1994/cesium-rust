/// An enum describing the different types of `Cesium3DTileContent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cesium3DTileContentType {
    Batched3DModel,
    Instanced3DModel,
    Composite,
    PointCloud,
    Vector,
    Geometry,
    Gltf,
    GltfBinary,
    ImplicitSubtree,
    ImplicitSubtreeJson,
    ImplicitSubtreeGltf,
    ImplicitSubtreeGlb,
    ExternalTileset,
    MultipleContent,
    GeoJson,
    VoxelBinary,
    VoxelJson,
    Other,
}

impl Cesium3DTileContentType {
    /// A Batched 3D Model ("b3dm").
    pub const BATCHED_3D_MODEL: &'static str = "b3dm";
    /// An Instanced 3D Model ("i3dm").
    pub const INSTANCED_3D_MODEL: &'static str = "i3dm";
    /// A Composite model ("cmpt").
    pub const COMPOSITE: &'static str = "cmpt";
    /// A Point Cloud model ("pnts").
    pub const POINT_CLOUD: &'static str = "pnts";
    /// Vector tiles ("vctr").
    pub const VECTOR: &'static str = "vctr";
    /// Geometry tiles ("geom").
    pub const GEOMETRY: &'static str = "geom";
    /// A glTF model in JSON + external BIN form ("gltf").
    pub const GLTF: &'static str = "gltf";
    /// The binary form of a glTF file ("glb").
    pub const GLTF_BINARY: &'static str = "glb";
    /// Binary subtree file for implicit tiling ("subt").
    pub const IMPLICIT_SUBTREE: &'static str = "subt";
    /// Subtrees represented as JSON files ("subtreeJson").
    pub const IMPLICIT_SUBTREE_JSON: &'static str = "subtreeJson";
    /// Subtrees represented as glTF JSON files ("subtreeGltf").
    pub const IMPLICIT_SUBTREE_GLTF: &'static str = "subtreeGltf";
    /// Subtrees represented as binary glTF files ("subtreeGlb").
    pub const IMPLICIT_SUBTREE_GLB: &'static str = "subtreeGlb";
    /// External tileset JSON reference ("externalTileset").
    pub const EXTERNAL_TILESET: &'static str = "externalTileset";
    /// Multiple contents ("multipleContent").
    pub const MULTIPLE_CONTENT: &'static str = "multipleContent";
    /// GeoJSON content ("geoJson").
    pub const GEOJSON: &'static str = "geoJson";
    /// Binary voxel content ("voxl").
    pub const VOXEL_BINARY: &'static str = "voxl";
    /// JSON voxel content ("voxelJson").
    pub const VOXEL_JSON: &'static str = "voxelJson";

    /// Check if a content type is one of the supported binary formats.
    pub fn is_binary_format(content_type: &str) -> bool {
        matches!(
            content_type,
            Self::BATCHED_3D_MODEL
                | Self::INSTANCED_3D_MODEL
                | Self::COMPOSITE
                | Self::POINT_CLOUD
                | Self::VECTOR
                | Self::GEOMETRY
                | Self::IMPLICIT_SUBTREE
                | Self::VOXEL_BINARY
                | Self::GLTF_BINARY
                | Self::IMPLICIT_SUBTREE_GLB
        )
    }

    /// Check if a content type is one of the subtree formats used by implicit tiling.
    pub fn is_subtree(content_type: &str) -> bool {
        matches!(
            content_type,
            Self::IMPLICIT_SUBTREE
                | Self::IMPLICIT_SUBTREE_JSON
                | Self::IMPLICIT_SUBTREE_GLTF
                | Self::IMPLICIT_SUBTREE_GLB
        )
    }

    /// Parses a string slice into a `Cesium3DTileContentType`.
    pub fn from_str_name(name: &str) -> Self {
        match name {
            Self::BATCHED_3D_MODEL => Self::Batched3DModel,
            Self::INSTANCED_3D_MODEL => Self::Instanced3DModel,
            Self::COMPOSITE => Self::Composite,
            Self::POINT_CLOUD => Self::PointCloud,
            Self::VECTOR => Self::Vector,
            Self::GEOMETRY => Self::Geometry,
            Self::GLTF => Self::Gltf,
            Self::GLTF_BINARY => Self::GltfBinary,
            Self::IMPLICIT_SUBTREE => Self::ImplicitSubtree,
            Self::IMPLICIT_SUBTREE_JSON => Self::ImplicitSubtreeJson,
            Self::IMPLICIT_SUBTREE_GLTF => Self::ImplicitSubtreeGltf,
            Self::IMPLICIT_SUBTREE_GLB => Self::ImplicitSubtreeGlb,
            Self::EXTERNAL_TILESET => Self::ExternalTileset,
            Self::MULTIPLE_CONTENT => Self::MultipleContent,
            Self::GEOJSON => Self::GeoJson,
            Self::VOXEL_BINARY => Self::VoxelBinary,
            Self::VOXEL_JSON => Self::VoxelJson,
            _ => Self::Other,
        }
    }
}
