use crate::{
    Cartesian3, DeveloperError, Matrix4, RuntimeError,
};

/// Snapping modes for `IonSnapService`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IonSnapMode {
    Nearest = 1,
    NearestKeypoint = 2,
    Center = 8,
}

/// Snapping proximity heat states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IonSnapHeat {
    None = 0,
    NotInRange = 1,
    InRange = 2,
}

/// Geometry types returned from snapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IonSnapGeometryType {
    None = 0,
    Point = 1,
    Segment = 2,
    Curve = 3,
    Arc = 4,
    Surface = 5,
}

/// Parent geometry types returned from snapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IonSnapParentGeometryType {
    None = 0,
    Wire = 1,
    Sheet = 2,
    Solid = 3,
    Mesh = 4,
    Text = 5,
}

/// Result from an `IonSnapService::snap` request.
#[derive(Debug, Clone, PartialEq)]
pub struct IonSnapResult {
    pub snap_point: Option<Cartesian3>,
    pub hit_point: Option<Cartesian3>,
    pub heat: IonSnapHeat,
    pub geometry_type: IonSnapGeometryType,
    pub parent_geometry_type: IonSnapParentGeometryType,
}

/// Options for calling `IonSnapService::snap`.
#[derive(Debug, Clone)]
pub struct IonSnapOptions {
    pub element_id: String,
    pub test_point: Cartesian3,
    pub view_matrix: Matrix4,
    pub projection_matrix: Matrix4,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub snap_mode: Option<IonSnapMode>,
    pub snap_aperture: Option<f64>,
}

/// Provides snapping to geometry hosted on Cesium ion.
#[derive(Debug, Clone)]
pub struct IonSnapService {
    pub asset_id: u64,
    pub endpoint_url: String,
    pub ecef_transform: Matrix4,
}

impl IonSnapService {
    pub const DEFAULT_SNAP_APERTURE: f64 = 10.0;

    pub fn new(
        asset_id: u64,
        endpoint_url: impl Into<String>,
        ecef_transform: Option<Matrix4>,
    ) -> Self {
        Self {
            asset_id,
            endpoint_url: endpoint_url.into(),
            ecef_transform: ecef_transform.unwrap_or(Matrix4::IDENTITY),
        }
    }

    /// Computes the transform matrix from world space coordinates to viewport CSS pixel coordinates.
    pub fn compute_world_to_view(
        view_matrix: &Matrix4,
        projection_matrix: &Matrix4,
        canvas_width: f64,
        canvas_height: f64,
    ) -> Result<Matrix4, DeveloperError> {
        if canvas_width <= 0.0 || canvas_height <= 0.0 {
            return Err(DeveloperError::new(
                "canvasWidth and canvasHeight must be greater than 0.",
            ));
        }

        let view_projection = projection_matrix.multiply(view_matrix);

        // Viewport mapping from NDC [-1, 1] to screen [0, width] x [0, height]:
        // x_screen = (x_ndc + 1) * width * 0.5
        // y_screen = (1 - y_ndc) * height * 0.5
        let half_w = canvas_width * 0.5;
        let half_h = canvas_height * 0.5;

        #[rustfmt::skip]
        let viewport = Matrix4::new(
            half_w,  0.0,     0.0, half_w,
            0.0,    -half_h,  0.0, half_h,
            0.0,     0.0,     0.5, 0.5,
            0.0,     0.0,     0.0, 1.0,
        );

        Ok(viewport.multiply(&view_projection))
    }

    /// Parses ECEF 4x4 matrix from row-major rows array.
    pub fn parse_ecef_transform(rows: &[[f64; 4]; 4]) -> Result<Matrix4, RuntimeError> {
        Ok(Matrix4::new(
            rows[0][0], rows[0][1], rows[0][2], rows[0][3],
            rows[1][0], rows[1][1], rows[1][2], rows[1][3],
            rows[2][0], rows[2][1], rows[2][2], rows[2][3],
            rows[3][0], rows[3][1], rows[3][2], rows[3][3],
        ))
    }
}
