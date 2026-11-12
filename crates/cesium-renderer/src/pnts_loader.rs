//! PntsLoader matching CesiumJS `Scene/Model/PntsLoader.js`.

use cesium_formats::{decode_point_cloud, parse_pnts, DecodeLimits, PointCloud, PositionEncoding};
use cesium_math::{Cartesian3, Matrix4, RuntimeError};
use serde_json::Value;

use crate::vertex_attribute_semantic::VertexAttributeSemantic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PntsAttributeType {
    Scalar,
    Vec3,
    Vec4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PntsComponentDatatype {
    Float,
    UnsignedByte,
    UnsignedShort,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PntsQuantization {
    pub component_datatype: PntsComponentDatatype,
    pub volume_offset: Cartesian3,
    pub volume_dimensions: Cartesian3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PntsAttribute {
    pub name: String,
    pub semantic: String,
    pub component_datatype: PntsComponentDatatype,
    pub attribute_type: PntsAttributeType,
    pub normalized: bool,
    pub count: usize,
    pub min: Option<Cartesian3>,
    pub max: Option<Cartesian3>,
    pub quantization: Option<PntsQuantization>,
}

#[derive(Clone, Debug)]
pub struct PntsPrimitive {
    pub attributes: Vec<PntsAttribute>,
}

#[derive(Clone, Debug)]
pub struct PntsNode {
    pub transform: Matrix4,
    pub primitives: Vec<PntsPrimitive>,
}

#[derive(Clone, Debug)]
pub struct PntsComponents {
    pub nodes: Vec<PntsNode>,
    pub metadata: Value,
    pub batch_length: Option<usize>,
    pub has_colors: bool,
    pub has_normals: bool,
    pub points_length: usize,
}

#[derive(Clone, Debug, Default)]
pub struct PntsLoaderOptions {
    pub array_buffer: Vec<u8>,
    pub load_attributes_for_2d: bool,
}

/// Loads a `.pnts` binary buffer into `PntsComponents` and decoded `PointCloud`.
#[derive(Debug)]
pub struct PntsLoader {
    array_buffer: Vec<u8>,
    load_attributes_for_2d: bool,
    components: Option<PntsComponents>,
    point_cloud: Option<PointCloud>,
    is_destroyed: bool,
}

impl PntsLoader {
    pub fn new(options: PntsLoaderOptions) -> Self {
        Self {
            array_buffer: options.array_buffer,
            load_attributes_for_2d: options.load_attributes_for_2d,
            components: None,
            point_cloud: None,
            is_destroyed: false,
        }
    }

    pub fn load(&mut self) -> Result<(), RuntimeError> {
        if self.is_destroyed {
            return Err(RuntimeError::new(
                "This object was destroyed, i.e., destroy() was called.",
            ));
        }

        let limits = DecodeLimits::default();
        let parsed = parse_pnts(&self.array_buffer, &limits)
            .map_err(|e| RuntimeError::new(e.to_string()))?;
        let cloud = decode_point_cloud(&self.array_buffer, &limits)
            .map_err(|e| RuntimeError::new(e.to_string()))?;

        let count = cloud.points.len();
        let (min_pos, max_pos) = if count > 0 {
            let mut min_p = [f64::INFINITY; 3];
            let mut max_p = [f64::NEG_INFINITY; 3];
            for pt in &cloud.points {
                for i in 0..3 {
                    let v = pt.position[i] as f64;
                    if v < min_p[i] {
                        min_p[i] = v;
                    }
                    if v > max_p[i] {
                        max_p[i] = v;
                    }
                }
            }
            (
                Some(Cartesian3::new(min_p[0], min_p[1], min_p[2])),
                Some(Cartesian3::new(max_p[0], max_p[1], max_p[2])),
            )
        } else {
            (None, None)
        };

        let quantization = match parsed.position_encoding {
            PositionEncoding::QuantizedU16 => Some(PntsQuantization {
                component_datatype: PntsComponentDatatype::UnsignedShort,
                volume_offset: parsed
                    .quantized_volume_offset
                    .map(|v| Cartesian3::new(v.x, v.y, v.z))
                    .unwrap_or(Cartesian3::ZERO),
                volume_dimensions: parsed
                    .quantized_volume_scale
                    .map(|v| Cartesian3::new(v.x, v.y, v.z))
                    .unwrap_or(Cartesian3::ZERO),
            }),
            PositionEncoding::Float32 => None,
        };

        let mut attributes = vec![PntsAttribute {
            name: "POSITION".to_string(),
            semantic: VertexAttributeSemantic::POSITION.to_string(),
            component_datatype: PntsComponentDatatype::Float,
            attribute_type: PntsAttributeType::Vec3,
            normalized: false,
            count,
            min: min_pos,
            max: max_pos,
            quantization,
        }];

        attributes.push(PntsAttribute {
            name: "COLOR_0".to_string(),
            semantic: VertexAttributeSemantic::COLOR.to_string(),
            component_datatype: PntsComponentDatatype::Float,
            attribute_type: PntsAttributeType::Vec4,
            normalized: false,
            count,
            min: None,
            max: None,
            quantization: None,
        });

        if cloud.has_normals {
            attributes.push(PntsAttribute {
                name: "NORMAL".to_string(),
                semantic: VertexAttributeSemantic::NORMAL.to_string(),
                component_datatype: PntsComponentDatatype::Float,
                attribute_type: PntsAttributeType::Vec3,
                normalized: false,
                count,
                min: None,
                max: None,
                quantization: None,
            });
        }

        if parsed.batch_length.is_some() {
            attributes.push(PntsAttribute {
                name: "_FEATURE_ID_0".to_string(),
                semantic: VertexAttributeSemantic::FEATURE_ID.to_string(),
                component_datatype: PntsComponentDatatype::UnsignedShort,
                attribute_type: PntsAttributeType::Scalar,
                normalized: false,
                count,
                min: None,
                max: None,
                quantization: None,
            });
        }

        let rtc = parsed.rtc_center;
        let transform = Matrix4::from_translation(Cartesian3::new(rtc.x, rtc.y, rtc.z));

        self.components = Some(PntsComponents {
            nodes: vec![PntsNode {
                transform,
                primitives: vec![PntsPrimitive { attributes }],
            }],
            metadata: parsed.metadata,
            batch_length: parsed.batch_length,
            has_colors: cloud.has_colors,
            has_normals: cloud.has_normals,
            points_length: count,
        });
        self.point_cloud = Some(cloud);

        Ok(())
    }

    pub fn components(&self) -> Option<&PntsComponents> {
        self.components.as_ref()
    }

    pub fn point_cloud(&self) -> Option<&PointCloud> {
        self.point_cloud.as_ref()
    }

    pub fn load_attributes_for_2d(&self) -> bool {
        self.load_attributes_for_2d
    }

    pub fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    pub fn destroy(&mut self) {
        self.array_buffer.clear();
        self.components = None;
        self.point_cloud = None;
        self.is_destroyed = true;
    }
}
