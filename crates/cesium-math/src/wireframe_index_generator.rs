use crate::geometry::PrimitiveType;
use crate::index_datatype::{IndexBuffer, IndexDatatype};

/// Functions for generating indices for model wireframes.
/// The indices are outputted as index buffers for rendering.
pub struct WireframeIndexGenerator;

impl WireframeIndexGenerator {
    /// Generates a wireframe index buffer for a primitive, either by reindexing the existing indices
    /// or creating them from scratch if the model had none.
    ///
    /// Returns `None` if the primitive type does not use triangles.
    pub fn create_wireframe_indices(
        primitive_type: PrimitiveType,
        vertex_count: usize,
        original_indices: Option<&[u32]>,
    ) -> Option<IndexBuffer> {
        match primitive_type {
            PrimitiveType::Triangles => Some(match original_indices {
                Some(indices) => Self::create_from_triangle_indices(vertex_count, indices),
                None => Self::create_from_triangles(vertex_count),
            }),
            PrimitiveType::TriangleStrip => Some(match original_indices {
                Some(indices) => Self::create_from_triangle_strip_indices(vertex_count, indices),
                None => Self::create_from_triangle_strip(vertex_count),
            }),
            PrimitiveType::TriangleFan => Some(match original_indices {
                Some(indices) => Self::create_from_triangle_fan_indices(vertex_count, indices),
                None => Self::create_from_triangle_fan(vertex_count),
            }),
            _ => None,
        }
    }

    /// Gets the number of indices in the wireframe index buffer of a primitive type.
    pub fn get_wireframe_indices_count(
        primitive_type: PrimitiveType,
        original_count: usize,
    ) -> usize {
        if primitive_type == PrimitiveType::Triangles {
            return original_count * 2;
        }

        if primitive_type == PrimitiveType::TriangleStrip
            || primitive_type == PrimitiveType::TriangleFan
        {
            let number_of_triangles = original_count.saturating_sub(2);
            return 2 + number_of_triangles * 4;
        }

        original_count
    }

    fn create_from_triangles(vertex_count: usize) -> IndexBuffer {
        let mut wireframe = IndexDatatype::create_typed_array(vertex_count, vertex_count * 2);
        let mut index = 0;
        let mut i = 0;
        while i < vertex_count {
            wireframe.set(index, i as u32);
            index += 1;
            wireframe.set(index, (i + 1) as u32);
            index += 1;
            wireframe.set(index, (i + 1) as u32);
            index += 1;
            wireframe.set(index, (i + 2) as u32);
            index += 1;
            wireframe.set(index, (i + 2) as u32);
            index += 1;
            wireframe.set(index, i as u32);
            index += 1;
            i += 3;
        }
        wireframe
    }

    fn create_from_triangle_indices(vertex_count: usize, original_indices: &[u32]) -> IndexBuffer {
        let original_indices_count = original_indices.len();
        let mut wireframe =
            IndexDatatype::create_typed_array(vertex_count, original_indices_count * 2);
        let mut index = 0;
        let mut i = 0;
        while i < original_indices_count {
            let point0 = original_indices[i];
            let point1 = original_indices[i + 1];
            let point2 = original_indices[i + 2];

            wireframe.set(index, point0);
            index += 1;
            wireframe.set(index, point1);
            index += 1;
            wireframe.set(index, point1);
            index += 1;
            wireframe.set(index, point2);
            index += 1;
            wireframe.set(index, point2);
            index += 1;
            wireframe.set(index, point0);
            index += 1;

            i += 3;
        }
        wireframe
    }

    fn create_from_triangle_strip(vertex_count: usize) -> IndexBuffer {
        let number_of_triangles = vertex_count.saturating_sub(2);
        let wireframe_indices_count = 2 + number_of_triangles * 4;
        let mut wireframe =
            IndexDatatype::create_typed_array(vertex_count, wireframe_indices_count);
        let mut index = 0;

        wireframe.set(index, 0);
        index += 1;
        wireframe.set(index, 1);
        index += 1;

        for i in 0..number_of_triangles {
            wireframe.set(index, (i + 1) as u32);
            index += 1;
            wireframe.set(index, (i + 2) as u32);
            index += 1;
            wireframe.set(index, (i + 2) as u32);
            index += 1;
            wireframe.set(index, i as u32);
            index += 1;
        }

        wireframe
    }

    fn create_from_triangle_strip_indices(
        vertex_count: usize,
        original_indices: &[u32],
    ) -> IndexBuffer {
        let original_indices_count = original_indices.len();
        let number_of_triangles = original_indices_count.saturating_sub(2);
        let wireframe_indices_count = 2 + number_of_triangles * 4;
        let mut wireframe =
            IndexDatatype::create_typed_array(vertex_count, wireframe_indices_count);
        let mut index = 0;

        wireframe.set(index, original_indices[0]);
        index += 1;
        wireframe.set(index, original_indices[1]);
        index += 1;

        for i in 0..number_of_triangles {
            let point0 = original_indices[i];
            let point1 = original_indices[i + 1];
            let point2 = original_indices[i + 2];

            wireframe.set(index, point1);
            index += 1;
            wireframe.set(index, point2);
            index += 1;
            wireframe.set(index, point2);
            index += 1;
            wireframe.set(index, point0);
            index += 1;
        }

        wireframe
    }

    fn create_from_triangle_fan(vertex_count: usize) -> IndexBuffer {
        let number_of_triangles = vertex_count.saturating_sub(2);
        let wireframe_indices_count = 2 + number_of_triangles * 4;
        let mut wireframe =
            IndexDatatype::create_typed_array(vertex_count, wireframe_indices_count);
        let mut index = 0;

        wireframe.set(index, 0);
        index += 1;
        wireframe.set(index, 1);
        index += 1;

        for i in 0..number_of_triangles {
            wireframe.set(index, (i + 1) as u32);
            index += 1;
            wireframe.set(index, (i + 2) as u32);
            index += 1;
            wireframe.set(index, (i + 2) as u32);
            index += 1;
            wireframe.set(index, 0);
            index += 1;
        }

        wireframe
    }

    fn create_from_triangle_fan_indices(
        vertex_count: usize,
        original_indices: &[u32],
    ) -> IndexBuffer {
        let original_indices_count = original_indices.len();
        let number_of_triangles = original_indices_count.saturating_sub(2);
        let wireframe_indices_count = 2 + number_of_triangles * 4;
        let mut wireframe =
            IndexDatatype::create_typed_array(vertex_count, wireframe_indices_count);
        let mut index = 0;

        let first_point = original_indices[0];
        wireframe.set(index, first_point);
        index += 1;
        wireframe.set(index, original_indices[1]);
        index += 1;

        for i in 0..number_of_triangles {
            let point1 = original_indices[i + 1];
            let point2 = original_indices[i + 2];

            wireframe.set(index, point1);
            index += 1;
            wireframe.set(index, point2);
            index += 1;
            wireframe.set(index, point2);
            index += 1;
            wireframe.set(index, first_point);
            index += 1;
        }

        wireframe
    }
}
