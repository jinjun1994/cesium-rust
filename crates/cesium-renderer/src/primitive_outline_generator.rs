//! PrimitiveOutlineGenerator matching CesiumJS `Scene/Model/PrimitiveOutlineGenerator.js` (Apache-2.0).

use std::collections::{HashMap, HashSet};
use cesium_math::DeveloperError;

use crate::framebuffer::PixelFormat;
use crate::sampler::{
    Sampler, TextureMagnificationFilter, TextureMinificationFilter, TextureWrap,
};
use crate::texture::{Texture, TextureOptions};

/// Options for constructing a [`PrimitiveOutlineGenerator`].
#[derive(Clone, Debug)]
pub struct PrimitiveOutlineGeneratorOptions {
    pub triangle_indices: Option<Vec<u32>>,
    pub outline_indices: Option<Vec<u32>>,
    pub original_vertex_count: Option<usize>,
}

/// Fast texture-based wireframe outline generator.
#[derive(Clone, Debug)]
pub struct PrimitiveOutlineGenerator {
    pub updated_triangle_indices: Vec<u32>,
    pub outline_coordinates: Vec<f32>,
    pub extra_vertices: Vec<usize>,
    pub original_vertex_count: usize,
}

impl PrimitiveOutlineGenerator {
    /// Constructs a new `PrimitiveOutlineGenerator`, updating indices and generating outline coordinates.
    pub fn try_new(options: PrimitiveOutlineGeneratorOptions) -> Result<Self, DeveloperError> {
        let triangle_indices = options
            .triangle_indices
            .ok_or_else(|| DeveloperError::new("options.triangleIndices is required."))?;
        let outline_indices = options
            .outline_indices
            .ok_or_else(|| DeveloperError::new("options.outlineIndices is required."))?;
        let original_vertex_count = options
            .original_vertex_count
            .ok_or_else(|| DeveloperError::new("options.originalVertexCount is required."))?;

        let edges = EdgeSet::new(&outline_indices);
        let mut generator = Self {
            updated_triangle_indices: triangle_indices,
            outline_coordinates: Vec::new(),
            extra_vertices: Vec::new(),
            original_vertex_count,
        };

        generator.initialize(&edges);
        Ok(generator)
    }

    /// Constructs a new `PrimitiveOutlineGenerator` or panics on invalid options.
    pub fn new(options: PrimitiveOutlineGeneratorOptions) -> Self {
        Self::try_new(options).expect("Invalid options for PrimitiveOutlineGenerator")
    }

    fn initialize(&mut self, edges: &EdgeSet) {
        let mut triangle_indices = std::mem::take(&mut self.updated_triangle_indices);
        let mut outline_coords_map: Vec<Option<[f32; 3]>> =
            vec![None; self.original_vertex_count];
        let mut vertex_copies: HashMap<usize, usize> = HashMap::new();

        let mut i = 0;
        while i < triangle_indices.len() {
            let mut i0 = triangle_indices[i] as usize;
            let mut i1 = triangle_indices[i + 1] as usize;
            let mut i2 = triangle_indices[i + 2] as usize;

            let has_edge01 = edges.has_edge(i0 as u32, i1 as u32);
            let has_edge12 = edges.has_edge(i1 as u32, i2 as u32);
            let has_edge20 = edges.has_edge(i2 as u32, i0 as u32);

            let mut unmatchable = match_and_store_coordinates(
                &mut outline_coords_map,
                i0,
                i1,
                i2,
                has_edge01,
                has_edge12,
                has_edge20,
            );

            while let Some(unmatch) = unmatchable {
                let copy = if let Some(&existing_copy) = vertex_copies.get(&unmatch) {
                    existing_copy
                } else {
                    let new_copy = self.original_vertex_count + self.extra_vertices.len();
                    let mut original = unmatch;
                    while original >= self.original_vertex_count {
                        original = self.extra_vertices[original - self.original_vertex_count];
                    }
                    self.extra_vertices.push(original);
                    vertex_copies.insert(unmatch, new_copy);
                    if outline_coords_map.len() <= new_copy {
                        outline_coords_map.resize(new_copy + 1, None);
                    }
                    new_copy
                };

                if unmatch == i0 {
                    i0 = copy;
                    triangle_indices[i] = copy as u32;
                } else if unmatch == i1 {
                    i1 = copy;
                    triangle_indices[i + 1] = copy as u32;
                } else {
                    i2 = copy;
                    triangle_indices[i + 2] = copy as u32;
                }

                unmatchable = match_and_store_coordinates(
                    &mut outline_coords_map,
                    i0,
                    i1,
                    i2,
                    has_edge01,
                    has_edge12,
                    has_edge20,
                );
            }

            i += 3;
        }

        let total_vertices = self.original_vertex_count + self.extra_vertices.len();
        let mut final_coords = Vec::with_capacity(total_vertices * 3);
        for v in outline_coords_map.iter().take(total_vertices) {
            let [a, b, c] = v.unwrap_or([0.0, 0.0, 0.0]);
            final_coords.push(a);
            final_coords.push(b);
            final_coords.push(c);
        }

        self.updated_triangle_indices = triangle_indices;
        self.outline_coordinates = final_coords;
    }

    /// Updates an attribute slice of elements, appending copies for duplicated vertices.
    pub fn update_attribute<T: Copy>(&self, original_data: &[T]) -> Vec<T> {
        if self.original_vertex_count == 0 {
            return original_data.to_vec();
        }

        let stride = original_data.len() / self.original_vertex_count;
        let mut result =
            Vec::with_capacity(original_data.len() + self.extra_vertices.len() * stride);
        result.extend_from_slice(original_data);

        for &source_idx in &self.extra_vertices {
            let start = source_idx * stride;
            let end = start + stride;
            result.extend_from_slice(&original_data[start..end]);
        }

        result
    }

    /// Creates an outline lookup texture with mipmap levels.
    pub fn create_texture() -> Texture {
        let max_size: u32 = 4096;
        let sampler = Sampler {
            wrap_s: TextureWrap::ClampToEdge,
            wrap_t: TextureWrap::ClampToEdge,
            wrap_r: TextureWrap::ClampToEdge,
            minification_filter: TextureMinificationFilter::LinearMipmapLinear,
            magnification_filter: TextureMagnificationFilter::Linear,
            maximum_anisotropy: 1.0,
        };

        Texture::new(TextureOptions {
            width: max_size,
            height: 1,
            pixel_format: Some(PixelFormat::Alpha),
            sampler: Some(sampler),
            ..Default::default()
        })
    }

    /// Creates a single mip level array for the outline lookup texture.
    pub fn create_mip_level(size: usize) -> Vec<u8> {
        let mut texture = vec![0u8; size];
        if size > 0 {
            let value = match size {
                8 => 96,
                4 => 48,
                2 => 24,
                1 => 12,
                _ => 192,
            };
            texture[size - 1] = value;
        }
        texture
    }
}

fn compute_order_mask(
    coords: &[[f32; 3]],
    index: usize,
    has_coord: bool,
    a: f32,
    b: f32,
    c: f32,
) -> u8 {
    if !has_coord {
        return 0b111111;
    }

    let [first, second, third] = coords[index];
    let eq = |x: f32, y: f32| (x - y).abs() < 1e-4;

    let mut mask = 0;
    if eq(first, a) && eq(second, b) && eq(third, c) {
        mask |= 1 << 0;
    }
    if eq(first, a) && eq(second, c) && eq(third, b) {
        mask |= 1 << 1;
    }
    if eq(first, b) && eq(second, a) && eq(third, c) {
        mask |= 1 << 2;
    }
    if eq(first, b) && eq(second, c) && eq(third, a) {
        mask |= 1 << 3;
    }
    if eq(first, c) && eq(second, a) && eq(third, b) {
        mask |= 1 << 4;
    }
    if eq(first, c) && eq(second, b) && eq(third, a) {
        mask |= 1 << 5;
    }
    mask
}

fn match_and_store_coordinates(
    coords: &mut [Option<[f32; 3]>],
    i0: usize,
    i1: usize,
    i2: usize,
    has_edge01: bool,
    has_edge12: bool,
    has_edge20: bool,
) -> Option<usize> {
    let a0 = if has_edge20 { 1.0 } else { 0.0 };
    let b0 = if has_edge01 { 1.0 } else { 0.0 };
    let c0 = 0.0;

    let i0_mask = compute_order_mask(
        &coords.iter().map(|o| o.unwrap_or([0.0, 0.0, 0.0])).collect::<Vec<_>>(),
        i0,
        coords[i0].is_some(),
        a0,
        b0,
        c0,
    );
    if i0_mask == 0 {
        return Some(i0);
    }

    let a1 = 0.0;
    let b1 = if has_edge01 { 1.0 } else { 0.0 };
    let c1 = if has_edge12 { 1.0 } else { 0.0 };

    let i1_mask = compute_order_mask(
        &coords.iter().map(|o| o.unwrap_or([0.0, 0.0, 0.0])).collect::<Vec<_>>(),
        i1,
        coords[i1].is_some(),
        a1,
        b1,
        c1,
    );
    if i1_mask == 0 {
        return Some(i1);
    }

    let a2 = if has_edge20 { 1.0 } else { 0.0 };
    let b2 = 0.0;
    let c2 = if has_edge12 { 1.0 } else { 0.0 };

    let i2_mask = compute_order_mask(
        &coords.iter().map(|o| o.unwrap_or([0.0, 0.0, 0.0])).collect::<Vec<_>>(),
        i2,
        coords[i2].is_some(),
        a2,
        b2,
        c2,
    );
    if i2_mask == 0 {
        return Some(i2);
    }

    let working_orders = i0_mask & i1_mask & i2_mask;
    let (a, b, c) = if (working_orders & (1 << 0)) != 0 {
        (0, 1, 2)
    } else if (working_orders & (1 << 1)) != 0 {
        (0, 2, 1)
    } else if (working_orders & (1 << 2)) != 0 {
        (1, 0, 2)
    } else if (working_orders & (1 << 3)) != 0 {
        (2, 0, 1)
    } else if (working_orders & (1 << 4)) != 0 {
        (1, 2, 0)
    } else if (working_orders & (1 << 5)) != 0 {
        (2, 1, 0)
    } else {
        let i0_count = i0_mask.count_ones();
        let i1_count = i1_mask.count_ones();
        let i2_count = i2_mask.count_ones();
        if i0_count < i1_count && i0_count < i2_count {
            return Some(i0);
        } else if i1_count < i2_count {
            return Some(i1);
        }
        return Some(i2);
    };

    let mut set_coord = |idx: usize, val_a: f32, val_b: f32, val_c: f32| {
        let mut entry = [0.0; 3];
        entry[a] = val_a;
        entry[b] = val_b;
        entry[c] = val_c;
        coords[idx] = Some(entry);
    };

    set_coord(i0, a0, b0, c0);
    set_coord(i1, a1, b1, c1);
    set_coord(i2, a2, b2, c2);

    None
}

/// Helper edge set structure.
struct EdgeSet {
    edges: HashSet<(u32, u32)>,
}

impl EdgeSet {
    fn new(edge_indices: &[u32]) -> Self {
        let mut edges = HashSet::new();
        for &[a, b] in edge_indices.as_chunks::<2>().0 {
            edges.insert((a.min(b), a.max(b)));
        }
        Self { edges }
    }

    fn has_edge(&self, a: u32, b: u32) -> bool {
        self.edges.contains(&(a.min(b), a.max(b)))
    }
}
