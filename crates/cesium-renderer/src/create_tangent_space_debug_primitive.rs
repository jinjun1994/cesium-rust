//! Creates a Primitive to visualize normal, tangent, and bitangent vertex attributes.
//! Adapted from CesiumJS `Scene/createTangentSpaceDebugPrimitive.js` (Apache-2.0).

use std::collections::HashMap;

use cesium_math::{
    ColorGeometryInstanceAttribute, DeveloperError, Geometry, GeometryInstance, GeometryPipeline,
    Matrix4,
};

use crate::per_instance_color_appearance::{
    PerInstanceColorAppearance, PerInstanceColorAppearanceOptions,
};
use crate::scene_primitive::{Primitive, PrimitiveOptions};

/// Options for configuring a tangent space debug primitive.
#[derive(Clone, Debug, Default)]
pub struct CreateTangentSpaceDebugPrimitiveOptions {
    pub geometry: Option<Geometry>,
    pub length: Option<f64>,
    pub model_matrix: Option<Matrix4>,
}

/// Creates a [`Primitive`] to visualize well-known vector vertex attributes:
/// `normal`, `tangent`, and `bitangent`.
///
/// - Normal is red (1, 0, 0, 1)
/// - Tangent is green (0, 1, 0, 1)
/// - Bitangent is blue (0, 0, 1, 1)
///
/// If an attribute is not present, it is not drawn.
pub fn create_tangent_space_debug_primitive(
    options: CreateTangentSpaceDebugPrimitiveOptions,
) -> Result<Option<Primitive>, DeveloperError> {
    let Some(geometry) = options.geometry else {
        return Err(DeveloperError::new("options.geometry is required."));
    };

    let model_matrix = options.model_matrix.unwrap_or(Matrix4::IDENTITY);
    let length = options.length.unwrap_or(10000.0);

    let mut instances = Vec::new();

    // Normal attribute
    if geometry.attributes.normal.is_some() {
        let lines_geom = GeometryPipeline::create_line_segments_for_vectors(
            &geometry,
            Some("normal"),
            Some(length),
        )?;
        let mut attributes = HashMap::new();
        attributes.insert(
            "color".to_string(),
            ColorGeometryInstanceAttribute::new(1.0, 0.0, 0.0, 1.0).into(),
        );
        instances.push(GeometryInstance::new(
            lines_geom,
            Some(model_matrix),
            None,
            Some(attributes),
        ));
    }

    // Tangent attribute
    if geometry.attributes.tangent.is_some() {
        let lines_geom = GeometryPipeline::create_line_segments_for_vectors(
            &geometry,
            Some("tangent"),
            Some(length),
        )?;
        let mut attributes = HashMap::new();
        attributes.insert(
            "color".to_string(),
            ColorGeometryInstanceAttribute::new(0.0, 1.0, 0.0, 1.0).into(),
        );
        instances.push(GeometryInstance::new(
            lines_geom,
            Some(model_matrix),
            None,
            Some(attributes),
        ));
    }

    // Bitangent attribute
    if geometry.attributes.bitangent.is_some() {
        let lines_geom = GeometryPipeline::create_line_segments_for_vectors(
            &geometry,
            Some("bitangent"),
            Some(length),
        )?;
        let mut attributes = HashMap::new();
        attributes.insert(
            "color".to_string(),
            ColorGeometryInstanceAttribute::new(0.0, 0.0, 1.0, 1.0).into(),
        );
        instances.push(GeometryInstance::new(
            lines_geom,
            Some(model_matrix),
            None,
            Some(attributes),
        ));
    }

    if instances.is_empty() {
        return Ok(None);
    }

    let appearance = PerInstanceColorAppearance::new(PerInstanceColorAppearanceOptions {
        flat: Some(true),
        translucent: Some(false),
        ..Default::default()
    });

    let primitive = Primitive::new(PrimitiveOptions {
        asynchronous: Some(false),
        geometry_instances: Some(instances),
        appearance: Some(appearance.into()),
        ..Default::default()
    });

    Ok(Some(primitive))
}
